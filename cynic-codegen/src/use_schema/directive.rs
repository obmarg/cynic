use quote::{ToTokens, TokenStreamExt, quote};

use crate::schema::types::Directive;

use super::argument::ArgumentOutput;

pub struct FieldDirectiveOutput<'a> {
    pub(super) directive: &'a Directive<'a>,
}

impl ToTokens for FieldDirectiveOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let directive_marker = self.directive.marker_ident().to_rust_ident();
        let directive_name_literal = proc_macro2::Literal::string(&self.directive.name);
        tokens.append_all(quote! {
            #[allow(non_camel_case_types)]
            pub struct #directive_marker;

            impl cynic::schema::FieldDirective for #directive_marker {
                const NAME: &'static str = #directive_name_literal;
            }
        });

        if !self.directive.arguments.is_empty() {
            let argument_module = self.directive.argument_module().ident();
            let arguments =
                self.directive.arguments.iter().map(|argument| {
                    ArgumentOutput::directive_argument(argument, &directive_marker)
                });

            tokens.append_all(quote! {
                #[allow(non_camel_case_types)]
                pub mod #argument_module {
                    #(#arguments)*
                }
            });
        }
    }
}
