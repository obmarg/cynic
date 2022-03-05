use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::schema::types::{InputObjectType, InputValue};

pub struct InputObjectOutput<'a> {
    object: InputObjectType<'a>,
}

struct FieldOutput<'a> {
    field: &'a InputValue<'a>,
    object_marker: &'a proc_macro2::Ident,
}

impl<'a> InputObjectOutput<'a> {
    pub fn new(object: InputObjectType<'a>) -> Self {
        InputObjectOutput { object }
    }
}

impl ToTokens for InputObjectOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let object_marker = proc_macro2::Ident::from(self.object.marker_ident());
        tokens.append_all(quote! {
            pub struct #object_marker;

            impl ::cynic::schema::InputObjectMarker for #object_marker {}
        });

        if !self.object.fields.is_empty() {
            let field_module = self.object.field_module().ident();
            let fields = self.object.fields.iter().map(|f| FieldOutput {
                field: f,
                object_marker: &object_marker,
            });
            tokens.append_all(quote! {
                pub mod #field_module {
                    #(#fields)*
                }
            });
        }
    }
}

impl ToTokens for FieldOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let object_marker = self.object_marker;
        let field_marker = &proc_macro2::Ident::from(self.field.marker_ident());
        let field_name_literal = proc_macro2::Literal::string(self.field.name.as_str());

        let field_type_marker = self
            .field
            .value_type
            .marker_type()
            .to_path(&parse_quote! { super });

        tokens.append_all(quote! {
            pub struct #field_marker;

            impl ::cynic::schema::Field for #field_marker{
                type Type = #field_type_marker;

                const NAME: &'static str = #field_name_literal;
            }

            impl ::cynic::schema::HasInputField<#field_marker, #field_type_marker> for super::#object_marker {
            }
        });
    }
}
