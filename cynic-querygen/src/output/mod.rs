use crate::casings::CasingExt;
use crate::{
    query_parsing::{inputs::InputObjectSet, normalisation::NormalisedDocument},
    schema::EnumDetails,
};

mod argument_struct;
mod enums;
mod field;
mod indent;
mod inline_fragments;
mod input_object;
pub mod query_fragment;

pub use argument_struct::{ArgumentStruct, ArgumentStructField};
pub use indent::indented;
pub use inline_fragments::InlineFragments;
pub use input_object::InputObject;
use proc_macro2::{Ident, Span, TokenStream};
pub use query_fragment::QueryFragment;
use quote::{quote, ToTokens};

use field::Field;

pub struct Output<'query, 'schema> {
    pub query_fragments: Vec<QueryFragment<'query, 'schema>>,
    pub inline_fragments: Vec<InlineFragments>,
    pub input_objects: Vec<InputObject<'schema>>,
    pub enums: Vec<EnumDetails<'schema>>,
    pub scalars: Vec<Scalar<'schema>>,
    pub argument_structs: Vec<ArgumentStruct<'query, 'schema>>,
    pub normalised_document: NormalisedDocument<'query, 'schema>,
    pub input_objects_raw: InputObjectSet<'schema>,
}

#[derive(Debug, Clone)]
pub struct Scalar<'schema>(pub &'schema str);

impl std::fmt::Display for Scalar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(cynic::Scalar, Debug, Clone)]")?;
        writeln!(f, "pub struct {}(pub String);", self.0.to_pascal_case())
    }
}

impl ToTokens for Scalar<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pascal_case = Ident::new(&self.0.to_pascal_case(), Span::call_site());
        tokens.extend(quote! {
            #[derive(cynic::Scalar, Debug, Clone)]
            pub struct #pascal_case(pub String);
        })
    }
}
