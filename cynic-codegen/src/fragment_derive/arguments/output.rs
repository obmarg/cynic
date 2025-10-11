use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::Path;

use crate::{fragment_derive::arguments::analyse::Field, idents::to_pascal_case};

use super::analyse::{AnalysedFieldArguments, ArgumentValue, VariantDetails};

pub struct Output<'a> {
    pub(super) analysed: AnalysedFieldArguments<'a>,
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

        let arguments = self.analysed.arguments.iter().map(|arg| ArgOutput {
            arg,
            schema_module,
            argument_module,
        });

        tokens.append_all(quote! {
            {
                #(#variant_structs)*
                #(#arguments)*
            }
        })
    }
}

struct ArgOutput<'a> {
    arg: &'a Field<'a>,
    schema_module: &'a Path,
    argument_module: &'a Path,
}

impl ToTokens for ArgOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let marker = self.arg.schema_field.marker_ident().to_rust_ident();
        let value = ArgumentValueTokens {
            value: &self.arg.value,
            schema_module: self.schema_module,
        };
        let argument_module = &self.argument_module;
        let argument_tokens = quote! {
            field_builder.argument::<#argument_module::#marker>()
            #value;
        };

        tokens.append_all(match &self.arg.requires_feature {
            Some(feature) => {
                quote! {
                    if field_builder.is_feature_enabled(#feature) {
                        #argument_tokens
                    }
                }
            }
            None => argument_tokens,
        })
    }
}

pub struct ArgumentValueTokens<'a> {
    pub value: &'a ArgumentValue<'a>,
    pub schema_module: &'a syn::Path,
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
                        .item(|builder| { builder #inner; })
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
                let variables_fields = &var.variables_fields_struct;

                tokens.append_all(quote! {
                    .variable(#variables_fields::#var_ident())
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

impl VariantDetails<'_> {
    fn ident(&self) -> syn::Ident {
        format_ident!("{}{}", self.en.name, to_pascal_case(&self.variant))
    }
}

/// Tokens for serializing an enum variant literal.
///
/// We can't rely on any types outside of our derive for these so we need to construct
/// individual structs for each variant that we need to serialize.
pub struct VariantDetailsTokens<'a> {
    pub details: &'a VariantDetails<'a>,
    pub schema_module: &'a syn::Path,
}

impl quote::ToTokens for VariantDetailsTokens<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.details.ident();
        let variant_str = proc_macro2::Literal::string(&self.details.variant);
        let enum_marker_ident = self.details.en.marker_ident().to_path(self.schema_module);
        tokens.append_all(quote! {
            struct #ident;

            impl cynic::serde::Serialize for #ident {
                fn serialize<__S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
                where
                    __S: cynic::serde::Serializer
                {
                    serializer.serialize_unit_variant("", 0, #variant_str)
                }
            }

            impl cynic::coercions::CoercesTo<#enum_marker_ident> for #ident {};
        })
    }
}
