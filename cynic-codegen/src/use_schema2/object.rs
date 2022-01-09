use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse_quote;

use crate::schema::types::{Field, InputValue, ObjectType};

pub struct ObjectOutput<'a> {
    object: ObjectType<'a>,
}

struct FieldOutput<'a> {
    field: &'a Field<'a>,
    object_marker: &'a proc_macro2::Ident,
}

struct ArgumentOutput<'a> {
    argument: &'a InputValue<'a>,
    field_marker: &'a proc_macro2::Ident,
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
            .field_type
            .marker_type()
            .to_path(&parse_quote! { super });

        tokens.append_all(quote! {
            pub struct #field_marker;

            impl ::cynic::schema::Field for #field_marker{
                type SchemaType = #field_type_marker;

                fn name() -> &'static str {
                    #field_name_literal
                }
            }

            impl ::cynic::schema::HasField<#field_marker, #field_type_marker> for super::#object_marker {}
        });

        if !self.field.arguments.is_empty() {
            let argument_module = self.field.argument_module().ident();
            let arguments = self.field.arguments.iter().map(|argument| ArgumentOutput {
                argument,
                field_marker,
            });

            tokens.append_all(quote! {
                pub mod #argument_module {
                    #(#arguments)*
                }
            });
        }
    }
}

impl ToTokens for ArgumentOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = proc_macro2::Literal::string(self.argument.name.as_str());
        let argument_ident = proc_macro2::Ident::from(self.argument.marker_ident());
        let field_marker = self.field_marker;

        let schema_type = self
            .argument
            .value_type
            .marker_type()
            .to_path(&parse_quote! { super::super });

        tokens.append_all(quote! {
            pub struct #argument_ident;

            impl ::cynic::schema::HasArgument<#argument_ident> for super::#field_marker {
                type ArgumentSchemaType = #schema_type;

                fn name() -> &'static str {
                    #name
                }
            }
        })
    }
}
