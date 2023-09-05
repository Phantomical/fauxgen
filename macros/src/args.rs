use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

pub struct MacroArg<K, V> {
    pub key: K,
    pub eq_token: syn::Token![=],
    pub value: V,
}

impl<K: Parse, V: Parse> Parse for MacroArg<K, V> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl<K: ToTokens, V: ToTokens> ToTokens for MacroArg<K, V> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.key.to_tokens(tokens);
        self.eq_token.to_tokens(tokens);
        self.value.to_tokens(tokens)
    }
}

pub enum ArgName {
    Ident(syn::Ident),
    Yield(syn::Token![yield]),
    Crate(syn::Token![crate]),
}

impl Parse for ArgName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        match () {
            _ if lookahead.peek(syn::Ident) => Ok(Self::Ident(input.parse()?)),
            _ if lookahead.peek(syn::Token![yield]) => Ok(Self::Yield(input.parse()?)),
            _ if lookahead.peek(syn::Token![crate]) => Ok(Self::Crate(input.parse()?)),
            _ => Err(lookahead.error()),
        }
    }
}

pub struct Args {
    pub crate_: Option<MacroArg<syn::Token![crate], syn::Path>>,
    pub yield_: Option<MacroArg<syn::Token![yield], Box<syn::Type>>>,
    pub arg: Option<MacroArg<syn::Ident, Box<syn::Type>>>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut crate_ = None;
        let mut yield_ = None;
        let mut arg_ = None;

        while !input.is_empty() {
            let name: ArgName = input.fork().parse()?;

            match name {
                ArgName::Crate(token) => {
                    if crate_.replace(input.parse()?).is_some() {
                        return Err(syn::Error::new_spanned(
                            token,
                            format!(
                                "argument `{}` specified multiple times",
                                token.to_token_stream()
                            ),
                        ));
                    }
                }
                ArgName::Yield(token) => {
                    if yield_.replace(input.parse()?).is_some() {
                        return Err(syn::Error::new_spanned(
                            token,
                            format!(
                                "argument `{}` specified multiple times",
                                token.to_token_stream()
                            ),
                        ));
                    }
                }
                ArgName::Ident(ident) if ident == "arg" => {
                    if arg_.replace(input.parse()?).is_some() {
                        return Err(syn::Error::new_spanned(
                            ident.clone(),
                            format!("argument `{ident}` specified multiple times",),
                        ));
                    }
                }
                ArgName::Ident(ident) => {
                    return Err(syn::Error::new_spanned(
                        ident.clone(),
                        format!("unknown argument `{ident}`"),
                    ))
                }
            }

            if input.peek(syn::Token![,]) {
                let _comma: syn::Token![,] = input.parse()?;
            }
        }

        Ok(Self {
            crate_,
            yield_,
            arg: arg_,
        })
    }
}
