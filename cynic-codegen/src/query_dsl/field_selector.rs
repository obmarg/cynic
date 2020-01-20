use proc_macro2::{Span, TokenStream};

use crate::{FieldType, Ident, TypePath};

/// A FieldSelector in our generated DSL.
///
/// Each field in the schema will have one of these associated with it.  
/// The generated function can be called to get a SelectionSet for that
/// field.
#[derive(Debug)]
pub struct FieldSelector {
    pub rust_field_name: Ident,
    pub query_field_name: String,
    pub field_type: FieldType,
    pub type_lock: Ident,
    pub required_args_struct_name: Option<TypePath>,
    pub optional_args_struct_name: Option<TypePath>,
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

        let selector = if self.field_type.contains_scalar() {
            // We call the scalar selector for scalars
            quote! { ::cynic::selection_set::scalar() }
        } else {
            // Otherwise we pass in the fields that the function
            // we generate accept as an argument.
            quote! { fields }
        };
        let selector = self.field_type.selection_set_call(selector);

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
        let argument_type_lock = self.field_type.as_type_lock();

        let argument_names = vec![
            self.required_args_struct_name
                .as_ref()
                .map(|_| Ident::new("required")),
            self.optional_args_struct_name
                .clone()
                .map(|_| Ident::new("optional")),
        ];
        let argument_names: Vec<_> = argument_names.iter().flatten().collect();
        let decodes_to = self.field_type.decodes_to(quote! { T });

        if self.field_type.contains_scalar() {
            tokens.append_all(quote! {
                pub fn #rust_field_name(#(#arguments, )*) ->
                ::cynic::selection_set::SelectionSet<'static, #field_type, #type_lock> {
                    use ::cynic::selection_set::{string, integer, float, boolean};

                    let mut args: Vec<::cynic::Argument> = vec![];
                    #(
                        args.extend(#argument_names.into_iter());
                    )*

                    ::cynic::selection_set::field(#query_field_name, args, #selector)
                }
            })
        } else {
            tokens.append_all(quote! {
                pub fn #rust_field_name<'a, T>(
                    #(#arguments, )*
                    fields: ::cynic::selection_set::SelectionSet<'a, T, #argument_type_lock>
                ) -> ::cynic::selection_set::SelectionSet<'a, #decodes_to, #type_lock>
                    where T: 'a + Send + Sync {
                        let mut args: Vec<::cynic::Argument> = vec![];
                        #(
                            args.extend(#argument_names.into_iter());
                        )*

                        ::cynic::selection_set::field(
                            #query_field_name,
                            args,
                            #selector
                        )
                    }
            })
        }
    }
}
