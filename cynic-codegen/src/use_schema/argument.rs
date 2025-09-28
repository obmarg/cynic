use {
    quote::{ToTokens, TokenStreamExt, quote},
    syn::parse_quote,
};

use crate::schema::types::InputValue;

pub struct ArgumentOutput<'a> {
    argument: &'a InputValue<'a>,

    // Marker for the field or directive this is contained within
    container_marker: &'a proc_macro2::Ident,

    kind: ArgumentKind,
}

enum ArgumentKind {
    Directive,
    Field,
}

impl<'a> ArgumentOutput<'a> {
    pub fn field_argument(
        argument: &'a InputValue<'a>,
        container_marker: &'a proc_macro2::Ident,
    ) -> Self {
        ArgumentOutput {
            argument,
            container_marker,
            kind: ArgumentKind::Field,
        }
    }

    pub fn directive_argument(
        argument: &'a InputValue<'a>,
        container_marker: &'a proc_macro2::Ident,
    ) -> Self {
        ArgumentOutput {
            argument,
            container_marker,
            kind: ArgumentKind::Directive,
        }
    }
}

impl ToTokens for ArgumentOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = proc_macro2::Literal::string(self.argument.name.as_str());
        let argument_ident = self.argument.marker_ident().to_rust_ident();
        let field_marker = self.container_marker;

        let prefix = match self.kind {
            ArgumentKind::Directive => parse_quote! { super },
            ArgumentKind::Field => parse_quote! { super::super::super },
        };

        let schema_type = self.argument.value_type.marker_type().to_path(&prefix);

        tokens.append_all(quote! {
            pub struct #argument_ident;

            impl cynic::schema::HasArgument<#argument_ident> for super::#field_marker {
                type ArgumentType = #schema_type;

                const NAME: &'static ::core::primitive::str = #name;
            }
        })
    }
}
