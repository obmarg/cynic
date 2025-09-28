use quote::{ToTokens, TokenStreamExt, quote};

use crate::schema::types::ObjectType;

use super::fields::FieldOutput;

pub struct ObjectOutput<'a> {
    object: ObjectType<'a>,
}

impl<'a> ObjectOutput<'a> {
    pub fn new(object: ObjectType<'a>) -> Self {
        ObjectOutput { object }
    }

    pub fn append_fields(&self, field_module: &mut proc_macro2::TokenStream) {
        if !self.object.fields.is_empty() {
            let object_marker = self.object.marker_ident().to_rust_ident();
            let field_module_ident = self.object.field_module().ident();
            let fields = self.object.fields.iter().map(|f| FieldOutput {
                field: f,
                parent_marker: &object_marker,
            });
            field_module.append_all(quote! {
                pub mod #field_module_ident {
                    #(#fields)*
                }
            });
        }
    }
}

impl ToTokens for ObjectOutput<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let object_marker = self.object.marker_ident().to_rust_ident();
        tokens.append_all(quote! {
            pub struct #object_marker;
        });
    }
}
