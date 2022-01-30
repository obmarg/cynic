use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::schema::types::{Field, InputType, InputValue, ObjectType};

use super::fields::FieldOutput;

pub struct ObjectOutput<'a> {
    object: ObjectType<'a>,
}

impl<'a> ObjectOutput<'a> {
    pub fn new(object: ObjectType<'a>) -> Self {
        ObjectOutput { object }
    }
}

impl ToTokens for ObjectOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let object_marker = proc_macro2::Ident::from(self.object.marker_ident());
        tokens.append_all(quote! {
            pub struct #object_marker;
        });

        if !self.object.fields.is_empty() {
            let field_module = self.object.field_module().ident();
            let fields = self.object.fields.iter().map(|f| FieldOutput {
                field: f,
                parent_marker: &object_marker,
            });
            tokens.append_all(quote! {
                pub mod #field_module {
                    #(#fields)*
                }
            });
        }
    }
}
