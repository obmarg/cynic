use proc_macro2::{Span, TokenStream};

use crate::field_type::FieldType;
use crate::ident::Ident;
use crate::type_path::TypePath;

/// A FieldSelector in our generated DSL.
///
/// Each field in the schema will have one of these associated with it.  
/// The generated function can be called to get a SelectionSet for that
/// field.
#[derive(Debug)]
pub struct FieldSelector {
    rust_field_name: Ident,
    query_field_name: String,
    field_type: FieldType,
    type_lock: Ident,
    required_args_struct_name: Option<TypePath>,
    optional_args_struct_name: Option<TypePath>,
}

impl FieldSelector {
    pub fn for_field(
        name: &str,
        field_type: FieldType,
        type_lock: Ident,
        required_args_struct_name: Option<TypePath>,
        optional_args_struct_name: Option<TypePath>,
    ) -> FieldSelector {
        FieldSelector {
            rust_field_name: Ident::for_field(name),
            query_field_name: name.to_string(),
            field_type,
            type_lock,
            required_args_struct_name,
            optional_args_struct_name,
        }
    }
}

impl quote::ToTokens for FieldSelector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let query_field_name = syn::LitStr::new(&self.query_field_name, Span::call_site());

        let scalar_call = if self.field_type.contains_scalar() {
            if self.field_type.is_nullable() {
                Some(quote! { ::cynic::selection_set::option(::cynic::selection_set::scalar()) })
            } else {
                Some(quote! { ::cynic::selection_set::scalar() })
            }
        } else {
            None
        };

        let arguments = vec![
            self.required_args_struct_name
                .as_ref()
                .map(|type_path| quote! { required: #type_path }),
            self.optional_args_struct_name
                .as_ref()
                .map(|type_path| quote! { optional: #type_path }),
        ];
        let arguments = arguments.iter().flatten().collect::<Vec<_>>();

        let field_type = &self.field_type;
        let type_lock = &self.type_lock;
        let rust_field_name = &self.rust_field_name;

        if let Some(scalar_call) = scalar_call {
            tokens.append_all(quote! {
                pub fn #rust_field_name(#(#arguments, )*) ->
                ::cynic::selection_set::SelectionSet<'static, #field_type, #type_lock> {
                    use ::cynic::selection_set::{string, integer, float, boolean};

                    ::cynic::selection_set::field(#query_field_name, vec![], #scalar_call)
                }
            })
        } else {
            tokens.append_all(quote! {
                pub fn #rust_field_name<'a, T>(
                    #(#arguments, )*
                    fields: ::cynic::selection_set::SelectionSet<'a, T, #field_type>
                ) -> ::cynic::selection_set::SelectionSet<T, #type_lock>
                    where T: 'a {
                        ::cynic::selection_set::field(#query_field_name, vec![], fields)
                    }
            })
        }
    }
}
