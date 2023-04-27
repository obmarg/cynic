use {
    quote::{quote, ToTokens, TokenStreamExt},
    syn::parse_quote,
};

use crate::schema::types::{Field, InputValue};

pub struct FieldOutput<'a> {
    pub(super) field: &'a Field<'a>,
    pub(super) parent_marker: &'a proc_macro2::Ident,
}

struct ArgumentOutput<'a> {
    argument: &'a InputValue<'a>,
    field_marker: &'a proc_macro2::Ident,
}

impl ToTokens for FieldOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let parent_marker = self.parent_marker;
        let field_marker = &proc_macro2::Ident::from(self.field.marker_ident());
        let field_name_literal = proc_macro2::Literal::string(self.field.name.as_str());

        let field_type_marker = self
            .field
            .field_type
            .marker_type()
            .to_path(&parse_quote! { super::super });

        tokens.append_all(quote! {
            pub struct #field_marker;

            impl cynic::schema::Field for #field_marker{
                type Type = #field_type_marker;

                const NAME: &'static str = #field_name_literal;
            }

            impl cynic::schema::HasField<#field_marker> for super::super::#parent_marker {
                type Type = #field_type_marker;
            }
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
            .to_path(&parse_quote! { super::super::super  });

        tokens.append_all(quote! {
            pub struct #argument_ident;

            impl cynic::schema::HasArgument<#argument_ident> for super::#field_marker {
                type ArgumentType = #schema_type;

                const NAME: &'static str = #name;
            }
        })
    }
}
