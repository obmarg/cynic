use proc_macro2::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Token;

use super::analyse::{AnalysedArguments, ArgumentValue};

pub struct Output<'a> {
    pub(super) analysed: AnalysedArguments<'a>,
    pub(super) schema_module: syn::Path,
    pub(super) argument_struct: Option<syn::Ident>,
}

impl ToTokens for Output<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.analysed.arguments.is_empty() {
            return;
        }

        let argument_module = self
            .analysed
            .schema_field
            .argument_module()
            .to_path(&self.schema_module);

        for arg in &self.analysed.arguments {
            let arg_marker = proc_macro2::Ident::from(arg.schema_field.marker_ident());
            let value = &arg.value;

            tokens.append_all(quote! {
                field_builder.argument::<#argument_module::#arg_marker>()
                #value;
            });
        }
    }
}

impl ToTokens for ArgumentValue<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ArgumentValue::Object(_) => todo!("obj"),
            ArgumentValue::List(_) => todo!("list"),
            ArgumentValue::Literal(lit) => tokens.append_all(quote! {
                .literal(#lit)
            }),
            ArgumentValue::Bool(b) => {
                let lit = syn::LitBool::new(*b, Span::call_site());
                tokens.append_all(quote! {
                    .literal(#lit)
                });
            }
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
                    .variable(<#argument_struct as ::cynic::core::QueryVariables>::Fields::#var_ident())
                });
            }
            ArgumentValue::Some(inner) => {
                tokens.append_all(quote! {
                    .value()
                });
                inner.to_tokens(tokens);
            }
            ArgumentValue::Null => tokens.append_all(quote! {
                .null()
            }),
        }
    }
}
