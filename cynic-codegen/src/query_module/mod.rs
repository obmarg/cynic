// TODO: docstring.
use darling::{util::SpannedValue, FromMeta};
use proc_macro2::TokenStream;

mod utils;

use utils::Derive;

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
        Ok(args) => Ok(transform_query_module_impl(args, query_module)),
        Err(e) => Ok(e.write_errors()),
    }
}

fn transform_query_module_impl(
    args: TransformModuleArgs,
    query_module: syn::ItemMod,
) -> TokenStream {
    use quote::quote;

    if query_module.content.is_none() {
        return quote! { #query_module };
    }

    let (_, module_items) = query_module.content.unwrap();

    let module_items = module_items
        .into_iter()
        .map(|item| insert_cynic_attrs(&args, item));

    let attrs = query_module.attrs;
    let visibility = query_module.vis;
    let module_name = query_module.ident;

    quote! {
        #(#attrs)*
        #visibility mod #module_name {
            #(#module_items)*
        }
    }
}

fn insert_cynic_attrs(args: &TransformModuleArgs, item: syn::Item) -> syn::Item {
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
        Some(Derive::QueryFragment) | Some(Derive::InputObject) => {
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
