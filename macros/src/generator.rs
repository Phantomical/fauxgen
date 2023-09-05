use std::borrow::Cow;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::Result;

use crate::args::Args;
use crate::lifetime::CollectLifetimes;

struct Warning {
    span: Span,
    message: Cow<'static, str>,
}

impl ToTokens for Warning {
    fn to_token_stream(&self) -> TokenStream {
        let message = syn::LitStr::new(&self.message, self.span);

        quote::quote_spanned! { self.span =>
            const _: () = {
                #[deprecated = #message]
                const fn warning() {}

                warning()
            };
        }
    }

    fn into_token_stream(self) -> TokenStream
    where
        Self: Sized,
    {
        self.to_token_stream()
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream())
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let mut func: syn::ItemFn = syn::parse2(item)?;
    let args: Args = syn::parse2(attr)?;

    let krate = match &args.crate_ {
        Some(krate) => krate.value.clone(),
        None => syn::parse_quote!(::fauxgen),
    };
    let mut yield_ty = match &args.yield_ {
        Some(ty) => ty.value.clone(),
        None => syn::parse_quote!(()),
    };
    let mut arg_ty = match &args.arg {
        Some(ty) => ty.value.clone(),
        None => syn::parse_quote!(()),
    };
    let mut return_ty = match std::mem::replace(&mut func.sig.output, syn::ReturnType::Default) {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => ty,
    };

    // By using mixed-site hygiene we ensure that user code within the function can
    // never actually use this token.
    //
    // It is still named using underscores so it doesn't show up as much within
    // rust-analzyer.
    let token = syn::Ident::new("__token", Span::mixed_site());
    let yield_ident = syn::Ident::new_raw("yield", Span::call_site());
    let argument_ident = syn::Ident::new("argument", Span::call_site());

    let mut warnings = Vec::new();
    expand_yield(&token, &mut warnings, &mut func.block);
    transform_sig(
        &mut func.sig,
        &mut yield_ty,
        &mut arg_ty,
        &mut return_ty,
        &krate,
    );

    let block = func.block;

    let yield_macro_span = args
        .yield_
        .as_ref()
        .map(|arg| arg.span())
        .unwrap_or(Span::call_site());
    let yield_macro_body = quote::quote_spanned! {
        yield_macro_span => #token.yield_($value).await
    };

    let argument_macro_span = args
        .arg
        .as_ref()
        .map(|arg| arg.span())
        .unwrap_or(Span::call_site());
    let argument_macro_body = quote::quote_spanned! {
        argument_macro_span => #token.argument().await
    };

    let prelude = quote::quote! {
        let #token = #krate::__private::token::<#yield_ty, #arg_ty>();
        let #token = #krate::__private::pin!(#token);
        let #token = #token.as_ref();
        #krate::__private::register(#token).await;

        // Most people won't see this but it will show up in rust-analyzer.
        /// Yield a value from this generator.
        #[allow(unused_macros)]
        macro_rules! #yield_ident {
            ($value:expr) => { #yield_macro_body }
        }

        /// Argument passed into the generator before the first yield.
        #[allow(unused_macros)]
        macro_rules! #argument_ident {
            () => { #argument_macro_body }
        }
    };

    if func.sig.asyncness.take().is_some() {
        func.block = syn::parse_quote!({
            #krate::__private::gen_async(
                #krate::__private::TokenMarker::new(),
                async move {
                    #prelude
                    #block
                }
            )
        });
    } else {
        func.block = syn::parse_quote!({
            #krate::__private::gen_sync(
                #krate::__private::TokenMarker::new(),
                async move {
                    #prelude
                    #block
                }
            )
        });
    }

    Ok(quote::quote! {
        #( #warnings )*

        #func
    })
}

struct ExpandYield<'w> {
    token: syn::Ident,
    warnings: &'w mut Vec<Warning>,
}

impl<'w> ExpandYield<'w> {
    fn new(name: syn::Ident, warnings: &'w mut Vec<Warning>) -> Self {
        Self {
            token: name,
            warnings,
        }
    }

    fn visit_token_stream(&mut self, stream: TokenStream) {
        for tree in stream.into_iter() {
            self.visit_token_tree(&tree);
        }
    }

    fn visit_token_tree(&mut self, t: &TokenTree) {
        match t {
            TokenTree::Group(group) => self.visit_token_stream(group.stream()),
            TokenTree::Ident(ident) => self.visit_ident(ident),
            _ => (),
        }
    }

    fn visit_ident(&mut self, ident: &syn::Ident) {
        if let Ok(token) = syn::parse2::<syn::token::Yield>(ident.to_token_stream()) {
            self.warnings.push(Warning {
                span: token.span,
                message: "`yield` expressions in macros are not expanded by fauxgen".into(),
            })
        }
    }
}

impl VisitMut for ExpandYield<'_> {
    fn visit_expr_mut(&mut self, i: &mut syn::Expr) {
        match i {
            syn::Expr::Yield(y) => {
                let token = &self.token;
                let attrs = &y.attrs;
                let expr = match &y.expr {
                    Some(expr) => expr.clone(),
                    None => syn::parse_quote_spanned! { y.yield_token.span => () },
                };
                *i = syn::parse_quote_spanned!( y.yield_token.span =>
                    #( #attrs )*
                    #token.yield_(#expr).await
                );
            }
            // Don't recurse into closures. They are a different function and may actually be a rust
            // generator.
            syn::Expr::Closure(_) => (),
            _ => visit_mut::visit_expr_mut(self, i),
        }
    }

    fn visit_expr_macro_mut(&mut self, i: &mut syn::ExprMacro) {
        self.visit_token_stream(i.mac.tokens.clone())        
    }

    fn visit_stmt_macro_mut(&mut self, i: &mut syn::StmtMacro) {
        self.visit_token_stream(i.mac.tokens.clone())
    }
}

/// Input:
/// ```ignore
/// #[generator(yield = A, arg = B)]
/// async? fn some_fn<'a, T>(self, x: &'a T, y: &u32) -> Ret;
/// ```
///
/// Output:
/// ```ignore
/// fn some_fn<'a, 'life2, 'gen>(self, x: &'a T, y: &'life2 u32) ->
///     (Sync|Async)Generator<impl Future<Output = Ret> + 'gen, A, B>
/// where
///     'a: 'gen,
///     'life2: 'gen,
///     T: 'gen,
///     Self: 'gen;
/// ```
fn transform_sig(
    sig: &mut syn::Signature,
    yield_ty: &mut syn::Type,
    arg_ty: &mut syn::Type,
    return_ty: &mut syn::Type,
    krate: &syn::Path,
) {
    use std::mem;

    let gen_lt: syn::Lifetime = syn::parse_quote_spanned! {
        sig.ident.span() => 'gen
    };
    let gen_lt = &gen_lt;
    let mut needs_gen = false;
    let mut has_receiver = false;

    let mut lifetimes = CollectLifetimes::default();
    for arg in sig.inputs.iter_mut() {
        has_receiver |= matches!(arg, syn::FnArg::Receiver(_));

        lifetimes.visit_fn_arg_mut(arg)
    }

    lifetimes.visit_type_mut(yield_ty);
    lifetimes.visit_type_mut(arg_ty);
    lifetimes.visit_type_mut(return_ty);

    for param in &mut sig.generics.params {
        match param {
            syn::GenericParam::Type(param) => {
                let span = param
                    .colon_token
                    .take()
                    .map(|token| token.span)
                    .unwrap_or_else(|| param.ident.span());
                let bounds = mem::take(&mut param.bounds);
                where_clause_or_default(&mut sig.generics.where_clause)
                    .predicates
                    .push(syn::parse_quote_spanned! { span => #param: #gen_lt + #bounds });
                needs_gen = true;
            }
            syn::GenericParam::Lifetime(param) => {
                let span = param
                    .colon_token
                    .take()
                    .map(|token| token.span)
                    .unwrap_or_else(|| param.lifetime.span());
                let bounds = mem::take(&mut param.bounds);
                where_clause_or_default(&mut sig.generics.where_clause)
                    .predicates
                    .push(syn::parse_quote_spanned! { span => #param: #gen_lt + #bounds });
                needs_gen = true;
            }
            syn::GenericParam::Const(_) => (),
        }
    }

    if sig.generics.lt_token.is_none() {
        sig.generics.lt_token = Some(syn::Token![<](sig.ident.span()));
    }
    if sig.generics.gt_token.is_none() {
        sig.generics.gt_token = Some(syn::Token![>](sig.paren_token.span.join()));
    }

    for elided in lifetimes.elided.iter() {
        sig.generics.params.push(syn::parse_quote!(#elided));
        where_clause_or_default(&mut sig.generics.where_clause)
            .predicates
            .push(syn::parse_quote_spanned!(elided.span()=> #elided: #gen_lt));
        needs_gen = true;
    }

    let gen_bound = if needs_gen {
        if has_receiver {
            where_clause_or_default(&mut sig.generics.where_clause)
                .predicates
                .push(syn::parse_quote!( Self: #gen_lt ));
        }

        sig.generics.params.push(syn::parse_quote!(#gen_lt));

        quote::quote!(+ #gen_lt)
    } else {
        TokenStream::new()
    };

    if sig.asyncness.is_none() {
        sig.output = syn::parse_quote!(
            -> #krate::__private::SyncGenerator<
                impl #krate::__private::Future<Output = #return_ty> #gen_bound,
                #yield_ty,
                #arg_ty,
            >
        );
    } else {
        sig.output = syn::parse_quote!(
            -> #krate::__private::AsyncGenerator<
                impl #krate::__private::Future<Output = #return_ty> #gen_bound,
                #yield_ty,
                #arg_ty,
            >
        );
    }
}

/// Replaces all instances of `yield $expr` in a block with `r#yield!($expr)`.
fn expand_yield(macro_token: &syn::Ident, warnings: &mut Vec<Warning>, block: &mut syn::Block) {
    ExpandYield::new(macro_token.clone(), warnings).visit_block_mut(block);
}

fn where_clause_or_default(clause: &mut Option<syn::WhereClause>) -> &mut syn::WhereClause {
    use syn::punctuated::Punctuated;

    clause.get_or_insert_with(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Punctuated::new(),
    })
}
