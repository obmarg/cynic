use quote::{quote, ToTokens, TokenStreamExt};

use crate::schema::types::InterfaceType;

use super::fields::FieldOutput;

pub struct InterfaceOutput<'a> {
    iface: InterfaceType<'a>,
}

impl<'a> InterfaceOutput<'a> {
    pub fn new(iface: InterfaceType<'a>) -> Self {
        InterfaceOutput { iface }
    }
}

impl ToTokens for InterfaceOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let object_marker = proc_macro2::Ident::from(self.iface.marker_ident());
        tokens.append_all(quote! {
            pub struct #object_marker;
        });

        if !self.iface.fields.is_empty() {
            let field_module = self.iface.field_module().ident();
            let fields = self.iface.fields.iter().map(|f| FieldOutput {
                field: f,
                parent_marker: &object_marker,
            });
            tokens.append_all(quote! {
                pub mod #field_module {
                    #(#fields)*
                }
            });
        }
    }
}
