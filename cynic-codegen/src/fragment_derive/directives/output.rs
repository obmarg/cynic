use quote::{ToTokens, TokenStreamExt, quote};

use crate::fragment_derive::arguments::output::{ArgumentValueTokens, VariantDetailsTokens};

use super::AnalysedFieldDirective;

pub struct Output<'a> {
    pub analysed: &'a AnalysedFieldDirective<'a>,
    pub schema_module: &'a syn::Path,
}

impl ToTokens for Output<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let schema_module = &self.schema_module;

        let directive_marker = self
            .analysed
            .directive
            .marker_ident()
            .to_path(schema_module);

        let argument_module = &self
            .analysed
            .directive
            .argument_module()
            .to_path(schema_module);

        let variant_structs =
            self.analysed
                .arguments
                .variants
                .iter()
                .map(|details| VariantDetailsTokens {
                    details,
                    schema_module,
                });

        let arg_markers = self
            .analysed
            .arguments
            .arguments
            .iter()
            .map(|arg| arg.schema_field.marker_ident().to_rust_ident());

        let arg_values = self
            .analysed
            .arguments
            .arguments
            .iter()
            .map(|arg| ArgumentValueTokens {
                value: &arg.value,
                schema_module,
            });

        tokens.append_all(quote! {
            {
                #(#variant_structs)*
                let mut directive_builder = field_builder.directive::<#directive_marker>();
                #(
                    directive_builder.argument::<#argument_module::#arg_markers>()
                    #arg_values;
                )*
            }
        })
    }
}
