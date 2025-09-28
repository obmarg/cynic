use quote::{ToTokens, TokenStreamExt, quote};

use crate::schema::types::InterfaceType;

use super::fields::FieldOutput;

pub struct InterfaceOutput<'a> {
    iface: InterfaceType<'a>,
    marker_ident: proc_macro2::Ident,
}

impl<'a> InterfaceOutput<'a> {
    pub fn new(iface: InterfaceType<'a>) -> Self {
        InterfaceOutput {
            marker_ident: iface.marker_ident().to_rust_ident(),
            iface,
        }
    }

    pub fn append_fields(&self, field_module: &mut proc_macro2::TokenStream) {
        if !self.iface.fields.is_empty() {
            let field_module_ident = self.iface.field_module().ident();
            let fields = self.iface.fields.iter().map(|f| FieldOutput {
                field: f,
                parent_marker: &self.marker_ident,
            });
            field_module.append_all(quote! {
                pub mod #field_module_ident {
                    #(#fields)*
                }
            });
        }
    }
}

impl ToTokens for InterfaceOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let marker_ident = &self.marker_ident;
        tokens.append_all(quote! {
            pub struct #marker_ident;
        });
    }
}
