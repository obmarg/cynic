// TODO: docstring.
use darling::{util::SpannedValue, FromMeta};
use proc_macro2::TokenStream;

mod utils;

use utils::Derive;

use crate::{fragment_derive, schema::Document};

#[derive(Debug, FromMeta)]
struct TransformModuleArgs {
    schema_path: SpannedValue<String>,
    // TODO: consider getting rid of query_module at some point (or at least making optional)
    query_module: SpannedValue<String>,
}

pub fn transform_query_module(
    args: syn::AttributeArgs,
    query_module: syn::ItemMod,
) -> Result<TokenStream, syn::Error> {
    match TransformModuleArgs::from_list(&args) {
        Ok(args) => transform_query_module_impl(args, query_module),
        Err(e) => Ok(e.write_errors()),
    }
}

fn transform_query_module_impl(
    args: TransformModuleArgs,
    query_module: syn::ItemMod,
) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    let schema = crate::load_schema(&*args.schema_path)
        .map_err(|e| e.to_syn_error(args.schema_path.span()))?;

    let fragment_derive_schema: fragment_derive::Schema = schema.clone().into();

    if let None = query_module.content {
        return Ok(quote! { #query_module });
    }

    if cfg!(feature = "optimised-query-modules") {
        let (_, module_items) = query_module.content.as_ref().unwrap();

        let derives: Vec<TokenStream> = module_items
            .into_iter()
            .map(|i| derive_for_item(i, &args, &schema, &fragment_derive_schema))
            .collect();

        let (_, module_items) = query_module.content.unwrap();
        let module_items: Vec<_> = module_items
            .into_iter()
            .map(utils::strip_cynic_attrs)
            .collect();

        let attrs = query_module.attrs;
        let visibility = query_module.vis;
        let module_name = query_module.ident;

        Ok(quote! {
            #(#attrs)*
            #visibility mod #module_name {
                #(#module_items)*
                #(#derives)*
            }
        })
    } else {
        let (_, module_items) = query_module.content.unwrap();

        let module_items = module_items
            .into_iter()
            .map(|item| insert_cynic_attrs(&args, item));

        let attrs = query_module.attrs;
        let visibility = query_module.vis;
        let module_name = query_module.ident;

        Ok(quote! {
            #(#attrs)*
            #visibility mod #module_name {
                #(#module_items)*
            }
        })
    }
}

fn derive_for_item(
    item: &syn::Item,
    args: &TransformModuleArgs,
    schema: &Document,
    fragment_derive_schema: &fragment_derive::Schema,
) -> TokenStream {
    match utils::find_derives(item).first() {
        None => TokenStream::new(),
        Some(Derive::QueryFragment) => fragment_derive(item, args, fragment_derive_schema),
        Some(Derive::InlineFragments) => inline_fragments_derive(item, args),
        Some(Derive::Enum) => enum_derive(item, args, schema),
        Some(Derive::Scalar) => scalar_derive(item),
    }
}

fn fragment_derive(
    item: &syn::Item,
    args: &TransformModuleArgs,
    schema: &fragment_derive::Schema,
) -> TokenStream {
    use crate::fragment_derive::{fragment_derive_impl, input::QueryModuleFragmentDeriveInput};
    use darling::FromDeriveInput;
    use syn::spanned::Spanned;

    let derive_input: syn::DeriveInput = match item {
        syn::Item::Struct(s) => s.clone().into(),
        _ => {
            return syn::Error::new(
                item.span(),
                format!("Can only derive QueryFragment on a struct"),
            )
            .to_compile_error()
        }
    };

    let input = match QueryModuleFragmentDeriveInput::from_derive_input(&derive_input) {
        Ok(input) => input,
        Err(e) => return e.write_errors(),
    };

    match fragment_derive_impl(
        input.to_fragment_derive_input(&args.schema_path, &args.query_module),
        schema,
    ) {
        Ok(res) => res,
        Err(e) => e.to_compile_error(),
    }
}

fn inline_fragments_derive(item: &syn::Item, args: &TransformModuleArgs) -> TokenStream {
    use crate::inline_fragments_derive::{
        inline_fragments_derive_impl, input::QueryModuleInlineFragmentsDeriveInput,
    };
    use darling::FromDeriveInput;
    use syn::spanned::Spanned;

    let derive_input: syn::DeriveInput = match item {
        syn::Item::Enum(e) => e.clone().into(),
        _ => {
            return syn::Error::new(
                item.span(),
                format!("Can only derive InlineFragments on an enum"),
            )
            .to_compile_error()
        }
    };

    let input = match QueryModuleInlineFragmentsDeriveInput::from_derive_input(&derive_input) {
        Ok(input) => input,
        Err(e) => return e.write_errors(),
    };

    match inline_fragments_derive_impl(
        input.to_inline_fragments_derive_input(&args.schema_path, &args.query_module),
    ) {
        Ok(res) => res,
        Err(e) => e.to_compile_error(),
    }
}

fn enum_derive(item: &syn::Item, args: &TransformModuleArgs, schema: &Document) -> TokenStream {
    use crate::enum_derive::{enum_derive_impl, input::QueryModuleEnumDeriveInput};
    use darling::FromDeriveInput;
    use syn::spanned::Spanned;

    let derive_input: syn::DeriveInput = match item {
        syn::Item::Enum(e) => e.clone().into(),
        _ => {
            return syn::Error::new(item.span(), format!("Can only derive Enum on an enum"))
                .to_compile_error()
        }
    };

    let input = match QueryModuleEnumDeriveInput::from_derive_input(&derive_input) {
        Ok(input) => input,
        Err(e) => return e.write_errors(),
    };

    match enum_derive_impl(
        input.to_enum_derive_input(&args.schema_path, &args.query_module),
        schema,
        item.span(),
    ) {
        Ok(res) => res,
        Err(e) => e.to_compile_error(),
    }
}

fn scalar_derive(item: &syn::Item) -> TokenStream {
    use crate::scalar_derive::{scalar_derive_impl, ScalarDeriveInput};
    use darling::FromDeriveInput;
    use syn::spanned::Spanned;

    let derive_input: syn::DeriveInput = match item {
        syn::Item::Struct(s) => s.clone().into(),
        _ => {
            return syn::Error::new(
                item.span(),
                format!("Can only derive Scalar on a newtype struct"),
            )
            .to_compile_error()
        }
    };

    let input = match ScalarDeriveInput::from_derive_input(&derive_input) {
        Ok(input) => input,
        Err(e) => return e.write_errors(),
    };

    match scalar_derive_impl(input) {
        Ok(res) => res,
        Err(e) => e.to_compile_error(),
    }
}

fn insert_cynic_attrs(args: &TransformModuleArgs, item: syn::Item) -> syn::Item {
    use darling::FromDeriveInput;
    use syn::Item;

    match utils::find_derives(&item).get(0) {
        None => item,
        Some(Derive::Scalar) => item,
        Some(Derive::InlineFragments) | Some(Derive::Enum) => {
            if let Item::Enum(mut en) = item {
                let attrs = PresentAttributes::from_attributes(&en.attrs);
                attrs.add_missing_attributes(&mut en.attrs, args);
                Item::Enum(en)
            } else {
                item
            }
        }
        Some(Derive::QueryFragment) => {
            if let Item::Struct(mut st) = item {
                let attrs = PresentAttributes::from_attributes(&st.attrs);
                attrs.add_missing_attributes(&mut st.attrs, args);
                Item::Struct(st)
            } else {
                item
            }
        }
    }
}

#[derive(Default, Debug)]
struct PresentAttributes {
    pub has_schema_path: bool,
    pub has_query_module: bool,
}

impl PresentAttributes {
    fn from_attributes(attrs: &[syn::Attribute]) -> Self {
        use syn::{Meta, NestedMeta};

        let mut rv = PresentAttributes::default();
        for attr in attrs {
            if attr.path.is_ident("cynic") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(name_val)) = nested {
                            if name_val.path.is_ident("schema_path") {
                                rv.has_schema_path = true;
                            } else if name_val.path.is_ident("query_module") {
                                rv.has_query_module = true;
                            }
                        }
                    }
                }
            }
        }
        rv
    }

    fn add_missing_attributes(self, attrs: &mut Vec<syn::Attribute>, args: &TransformModuleArgs) {
        if !self.has_schema_path {
            let schema_path = proc_macro2::Literal::string(&args.schema_path);
            attrs.push(syn::parse_quote! {
                #[cynic(schema_path = #schema_path)]
            })
        }

        if !self.has_query_module {
            let query_module = proc_macro2::Literal::string(&args.query_module);
            attrs.push(syn::parse_quote! {
                #[cynic(query_module = #query_module)]
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> TransformModuleArgs {
        TransformModuleArgs {
            schema_path: "test.graphql".to_string().into(),
            query_module: "query_dsl".to_string().into(),
        }
    }

    #[test]
    fn test_insert_cynic_attrs() {
        let item: syn::Item = syn::parse_quote! {
            #[derive(cynic::QueryFragment)]
            struct Test {
                a: String
            }
        };

        let result = insert_cynic_attrs(&args(), item);

        assert_eq!(
            result,
            syn::parse_quote! {
                #[derive(cynic::QueryFragment)]
                #[cynic(schema_path = "test.graphql")]
                #[cynic(query_module = "query_dsl")]
                struct Test {
                    a: String
                }
            }
        )
    }

    #[test]
    fn test_insert_cynic_attrs_when_already_inserted() {
        let item: syn::Item = syn::parse_quote! {
            #[derive(cynic::QueryFragment)]
            #[cynic(schema_path = "other.graphql", query_module = "something")]
            struct Test {
                a: String
            }
        };

        let result = insert_cynic_attrs(&args(), item);

        assert_eq!(
            result,
            syn::parse_quote! {
                #[derive(cynic::QueryFragment)]
                #[cynic(schema_path = "other.graphql", query_module = "something")]
                struct Test {
                    a: String
                }
            }
        )
    }
}
