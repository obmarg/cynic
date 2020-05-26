use proc_macro2::{Span, TokenStream};

use crate::{query_dsl::ArgumentStruct, FieldType, Ident, TypePath};

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
    pub argument_structs_path: Ident,
    pub required_args_struct: Option<ArgumentStruct>,
    pub optional_args_struct: Option<ArgumentStruct>,
}

impl FieldSelector {
    pub fn for_field(
        name: &str,
        field_type: FieldType,
        type_lock: Ident,
        argument_structs_path: Ident,
        required_args_struct: Option<ArgumentStruct>,
        optional_args_struct: Option<ArgumentStruct>,
    ) -> FieldSelector {
        FieldSelector {
            rust_field_name: Ident::for_field(name),
            query_field_name: name.to_string(),
            field_type,
            type_lock,
            argument_structs_path,
            required_args_struct,
            optional_args_struct,
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
            self.required_args_struct.as_ref().map(|args| {
                let type_tokens = args.type_tokens(&self.argument_structs_path);
                quote! { required: #type_tokens }
            }),
            self.optional_args_struct.as_ref().map(|args| {
                let type_tokens = args.type_tokens(&self.argument_structs_path);
                quote! { optional: #type_tokens }
            }),
        ];
        let arguments = arguments.iter().flatten().collect::<Vec<_>>();

        let field_type = &self.field_type;
        let type_lock = &self.type_lock;
        let rust_field_name = &self.rust_field_name;
        let argument_type_lock = self.field_type.as_type_lock(TypePath::empty());

        let argument_names = vec![
            self.required_args_struct
                .as_ref()
                .map(|_| Ident::new("required")),
            self.optional_args_struct
                .as_ref()
                .map(|_| Ident::new("optional")),
        ];
        let argument_names: Vec<_> = argument_names.iter().flatten().collect();
        let decodes_to = self.field_type.decodes_to(quote! { T });

        let mut generic_params = Vec::new();
        if !self.field_type.contains_scalar() {
            generic_params.push(quote! { 'a });
            generic_params.push(quote! { T: 'a + Send + Sync});
        }
        for args in vec![&self.required_args_struct, &self.optional_args_struct] {
            if let Some(args) = args {
                generic_params.extend(
                    args.generic_parameters()
                        .into_iter()
                        .map(|p| p.to_tokens(TypePath::empty())),
                )
            }
        }

        if self.field_type.contains_scalar() {
            tokens.append_all(quote! {
                pub fn #rust_field_name<#(#generic_params)*>(#(#arguments, )*) ->
                ::cynic::selection_set::SelectionSet<'static, #field_type, #type_lock> {
                    #![allow(dead_code)]
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
                pub fn #rust_field_name<#(#generic_params, )*>(
                    #(#arguments, )*
                    fields: ::cynic::selection_set::SelectionSet<'a, T, #argument_type_lock>
                ) -> ::cynic::selection_set::SelectionSet<'a, #decodes_to, #type_lock>
                    {
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
