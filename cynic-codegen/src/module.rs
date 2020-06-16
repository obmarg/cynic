use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::ident::Ident;

#[derive(Debug)]
pub struct Module<T: ToTokens> {
    name: Ident,
    contents: Vec<T>,
}

impl<T> Module<T>
where
    T: ToTokens,
{
    pub fn new(name: &str, contents: Vec<T>) -> Self {
        Module {
            name: Ident::for_module(name),
            contents,
        }
    }
}

impl<T> quote::ToTokens for Module<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let contents = &self.contents;

        tokens.append_all(quote! {
            #[allow(dead_code)]
            pub mod #name {
                #(
                    #contents
                )*
            }
        })
    }
}
