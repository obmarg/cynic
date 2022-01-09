use proc_macro2::Span;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Token;

use super::analyse::{AnalysedArguments, ArgumentValue};

pub struct Output<'a> {
    pub(super) analysed: AnalysedArguments<'a>,
    pub(super) schema_module: syn::Path,
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
            ArgumentValue::Variable(_) => todo!("variable"),
            ArgumentValue::Some(_) => todo!("some"),
            ArgumentValue::Null => todo!("null"),
        }
    }
}
