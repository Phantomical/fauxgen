use proc_macro::TokenStream;

mod args;
mod generator;

#[proc_macro_attribute]
pub fn generator(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    match generator::expand(attr.into(), item.clone().into()) {
        Ok(tokens) => tokens.into(),
        Err(e) => {
            let tokens: TokenStream = e.into_compile_error().into();
            item.extend(tokens);
            item
        }
    }
}
