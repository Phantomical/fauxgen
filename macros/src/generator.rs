use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
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

    let block = func.block;

    let prelude = quote::quote! {
        let #token = ::core::pin::pin!(#krate::__private::token::<#yield_ty, #arg_ty>());
        let #token = #krate::__private::register(#token.as_ref()).await;

        // Most people won't see this but it will show up in rust-analyzer.
        /// Yield a value from this generator.
        #[allow(unused_macros)]
        macro_rules! #macro_ident {
            ($value:expr) => { #token.yield_($value).await }
        }
    };

    if func.sig.asyncness.is_some() {
        func.sig.asyncness = None;
        func.sig.output = syn::parse_quote!(
            -> #krate::export::AsyncGenerator<
                impl #krate::__private::Future<Output = #return_ty>,
                #yield_ty,
                #arg_ty
            >
        );

        func.block = syn::parse_quote!({
            #krate::__private::gen_async(async move {
                #prelude
                #block
            })
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
            #krate::__private::gen_sync(async move {
                #prelude
                #block
            })
        });
    }

    Ok(func.to_token_stream())
}
