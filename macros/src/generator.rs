use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::Result;

use crate::args::Args;

fn expand_sync(mut func: syn::ItemFn, args: Args) -> Result<TokenStream> {
    let krate = match args.crate_ {
        Some(krate) => krate.value,
        None => syn::parse_quote!(genawaiter),
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

    let token = syn::Ident::new("__generator_token", Span::mixed_site());
    let macro_ident = syn::Ident::new_raw("yield", Span::call_site());

    let block = func.block;
    let yield_macro = quote::quote_spanned! { Span::mixed_site() =>
        #[allow(unused_macros)]
        macro_rules! #macro_ident { ($value:expr) => { #token.do_yield($value).await } }
    };

    func.sig.output = syn::parse_quote!(
        -> impl #krate::Generator<#arg_ty, Yield = #yield_ty, Return = #return_ty>
    );
    // func.sig.output = syn::parse_quote!(
    //     -> #krate::detail::SyncGeneratorWrapper<
    //         impl ::core::future::Future<Output = #arg_ty>,
    //         #yield_ty,
    //         #return_ty
    //     >
    // );
    func.block = syn::parse_quote!({
        let #token: #krate::detail::GeneratorToken<#yield_ty, #arg_ty> = #krate::detail::GeneratorToken::new();
        #yield_macro

        #krate::detail::SyncGeneratorWrapper::new(async move #block)
    });

    Ok(func.to_token_stream())
}

pub fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let func: syn::ItemFn = syn::parse2(item)?;
    let args: Args = syn::parse2(attr)?;

    if let Some(asynk) = &func.sig.asyncness {
        Err(syn::Error::new_spanned(
            asynk,
            "async generators are not supported yet",
        ))
    } else {
        expand_sync(func, args)
    }
}
