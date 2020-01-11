extern crate proc_macro;

mod error;
mod ident;
mod module;
mod query_dsl;

use error::Error;

#[proc_macro]
pub fn query_dsl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as query_dsl::QueryDslParams);

    query_dsl::query_dsl_from_schema(input).unwrap().into()
}
