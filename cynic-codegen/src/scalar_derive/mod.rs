use proc_macro2::TokenStream;

pub(crate) mod input;

pub use input::ScalarDeriveInput;

pub fn scalar_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match ScalarDeriveInput::from_derive_input(ast) {
        Ok(input) => scalar_derive_impl(input).or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn scalar_derive_impl(input: ScalarDeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    // We're assuming that Darling has already validated this as a newtype enum,
    // so we can get away with panicing here.
    let field = input
        .data
        .take_struct()
        .expect("Expected enum")
        .into_iter()
        .nth(0)
        .expect("Expected enum with one variant");

    let ident = input.ident;
    let inner_type = field.ty;

    Ok(quote! {
        impl ::cynic::Scalar for #ident {
            fn decode(value: &::cynic::serde_json::Value) -> Result<Self, ::cynic::DecodeError> {
                Ok(#ident(<#inner_type as ::cynic::Scalar>::decode(value)?))
            }
            fn encode(&self) -> Result<::cynic::serde_json::Value, ::cynic::SerializeError> {
                Ok(self.0.encode()?)
            }
        }

    })
}
