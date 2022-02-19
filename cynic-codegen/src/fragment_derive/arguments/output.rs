use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::Token;

use crate::idents::to_pascal_case;

use super::analyse::{AnalysedArguments, ArgumentValue, VariantDetails};

pub struct Output<'a> {
    pub(super) analysed: AnalysedArguments<'a>,
    pub(super) schema_module: syn::Path,
}

impl ToTokens for Output<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.analysed.arguments.is_empty() {
            return;
        }

        let schema_module = &self.schema_module;

        let argument_module = &self
            .analysed
            .schema_field
            .argument_module()
            .to_path(schema_module);

        let variant_structs = self
            .analysed
            .variants
            .iter()
            .map(|details| VariantDetailsTokens {
                details,
                schema_module,
            });

        let arg_markers = self
            .analysed
            .arguments
            .iter()
            .map(|arg| proc_macro2::Ident::from(arg.schema_field.marker_ident()));

        let arg_values = self
            .analysed
            .arguments
            .iter()
            .map(|arg| ArgumentValueTokens {
                value: &arg.value,
                schema_module,
            });

        tokens.append_all(quote! {
            {
                #(#variant_structs)*
                #(
                    field_builder.argument::<#argument_module::#arg_markers>()
                    #arg_values;
                )*
            }
        })
    }
}

struct ArgumentValueTokens<'a> {
    value: &'a ArgumentValue<'a>,
    schema_module: &'a syn::Path,
}

impl<'a> ArgumentValueTokens<'a> {
    fn wrap_value(&self, value: &'a ArgumentValue<'a>) -> Self {
        ArgumentValueTokens {
            value,
            schema_module: self.schema_module,
        }
    }
}

impl ToTokens for ArgumentValueTokens<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self.value {
            ArgumentValue::Object(obj) => {
                tokens.append_all(quote! { .object() });
                for field in &obj.fields {
                    let inner = self.wrap_value(&field.value);
                    let field_module = obj.schema_obj.field_module().to_path(self.schema_module);
                    let field_marker = field.schema_field.marker_ident().to_path(&field_module);

                    tokens.append_all(quote! {
                        .field::<#field_marker, _>(|builder| {
                            builder #inner;
                        })
                    })
                }
            }
            ArgumentValue::List(items) => {
                tokens.append_all(quote! { .list() });
                for item in items {
                    let inner = self.wrap_value(item);
                    tokens.append_all(quote! {
                        .item(|builder| { builder #inner })
                    })
                }
            }
            ArgumentValue::Literal(lit) => tokens.append_all(quote! {
                .literal(#lit)
            }),
            ArgumentValue::Expression(e) => tokens.append_all(quote! {
                .literal(#e)
            }),
            ArgumentValue::Variable(var) => {
                let var_ident = &var.ident;
                let argument_struct = &var.argument_struct;

                // TODO: Can I do a static_assertions::assert_fields!(xyz) here?
                // Gives a slightly better error on failure if nothing else...
                // Though might be tricky because it's in the middle
                // of a big chain...
                tokens.append_all(quote! {
                    .variable(<#argument_struct as ::cynic::QueryVariables>::Fields::#var_ident())
                });
            }
            ArgumentValue::Some(inner) => {
                tokens.append_all(quote! {
                    .value()
                });
                self.wrap_value(inner).to_tokens(tokens);
            }
            ArgumentValue::Null => tokens.append_all(quote! {
                .null()
            }),
            ArgumentValue::Variant(var) => {
                let variant_struct = var.ident();
                tokens.append_all(quote! {
                    .literal(#variant_struct)
                })
            }
        }
    }
}

impl<'a> VariantDetails<'a> {
    fn ident(&self) -> syn::Ident {
        format_ident!("{}{}", self.en.name, to_pascal_case(&self.variant))
    }
}

struct VariantDetailsTokens<'a> {
    details: &'a VariantDetails<'a>,
    schema_module: &'a syn::Path,
}

impl<'a> quote::ToTokens for VariantDetailsTokens<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.details.ident();
        let variant_str = proc_macro2::Literal::string(&self.details.variant);
        let enum_marker_ident = self.details.en.marker_ident().to_path(&self.schema_module);
        tokens.append_all(quote! {
            struct #ident;

            impl ::cynic::serde::Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::cynic::serde::Serializer
                {
                    serializer.serialize_unit_variant("", 0, #variant_str)
                }
            }

            impl ::cynic::coercions::CoercesTo<#enum_marker_ident> for #ident {};
        })
    }
}
