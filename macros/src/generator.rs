use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::visit_mut::{self, VisitMut};
use syn::Result;

use crate::args::Args;

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut func: syn::ItemFn = syn::parse2(item)?;
    let args: Args = syn::parse2(attr)?;

    let krate = match args.crate_ {
        Some(krate) => krate.value,
        None => syn::parse_quote!(::fauxgen),
    };
    let yield_ty = match args.yield_ {
        Some(ty) => ty.value,
        None => syn::parse_quote!(()),
    };
    let arg_ty = match args.arg {
        Some(ty) => ty.value,
        None => syn::parse_quote!(()),
    };
    let return_ty = match func.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => ty,
    };

    // By using mixed-site hygiene we ensure that user code within the function can
    // never actually use this token.
    //
    // It is still named using underscores so it doesn't show up as much within
    // rust-analzyer.
    let token = syn::Ident::new("__token", Span::mixed_site());
    let macro_ident = syn::Ident::new_raw("yield", Span::call_site());

    ExpandYield::new(macro_ident.clone()).visit_block_mut(&mut func.block);
    let block = func.block;

    let token_decl = quote::quote! {
        let #token = #krate::__private::token();
    };
    let prelude = quote::quote! {
        let #token = #krate::__private::pin!(#token);
        let #token = #token.as_ref();
        #krate::__private::register(#token).await;

        // Most people won't see this but it will show up in rust-analyzer.
        /// Yield a value from this generator.
        #[allow(unused_macros)]
        macro_rules! #macro_ident {
            () => { #token.argument().await };
            ($value:expr) => { #token.yield_($value).await }
        }
    };

    if func.sig.asyncness.take().is_some() {
        func.sig.output = syn::parse_quote!(
            -> #krate::export::AsyncGenerator<
                impl #krate::__private::Future<Output = #return_ty>,
                #yield_ty,
                #arg_ty
            >
        );

        func.block = syn::parse_quote!({
            #token_decl
            #krate::__private::gen_async(
                #token.marker(),
                async move {
                    #prelude
                    #block
                }
            )
        });
    } else {
        func.sig.output = syn::parse_quote!(
            -> #krate::export::SyncGenerator<
                impl #krate::__private::Future<Output = #return_ty>,
                #yield_ty,
                #arg_ty
            >
        );

        func.block = syn::parse_quote!({
            #token_decl
            #krate::__private::gen_sync(
                #token.marker(),
                async move {
                    #prelude
                    #block
                }
            )
        });
    }

    Ok(func.to_token_stream())
}

struct ExpandYield {
    macro_name: syn::Ident,
}

impl ExpandYield {
    fn new(name: syn::Ident) -> Self {
        Self { macro_name: name }
    }
}

impl VisitMut for ExpandYield {
    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
        match i {
            syn::Expr::Yield(y) => {
                let tokens = if let Some(expr) = &mut y.expr {
                    self.visit_expr_mut(expr);
                    expr.to_token_stream()
                } else {
                    TokenStream::default()
                };

                let name = &self.macro_name;
                *i = syn::Expr::Macro(syn::ExprMacro {
                    attrs: std::mem::take(&mut y.attrs),
                    mac: syn::Macro {
                        path: syn::parse_quote!(#name),
                        bang_token: syn::Token![!](y.yield_token.span),
                        delimiter: syn::MacroDelimiter::Paren(syn::token::Paren(
                            y.yield_token.span,
                        )),
                        tokens,
                    },
                });
            }
            _ => visit_mut::visit_expr_mut(self, i),
        }
    }

    fn visit_expr_yield_mut(&mut self, i: &mut syn::ExprYield) {
        if let Some(expr) = &mut i.expr {
            self.visit_expr_mut(expr);
        }
    }
}
