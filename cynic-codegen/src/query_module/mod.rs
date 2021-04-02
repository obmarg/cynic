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

    let derives = utils::find_derives(&item);
    let derive = derives.first();
    if derive.is_none() {
        return item;
    }

    let derive = derive.unwrap();

    let required_attrs = RequiredAttributes::for_derive(derive);

    match derive {
        Derive::InlineFragments | Derive::Enum => {
            if let Item::Enum(mut en) = item {
                let attrs = required_attrs.with_current_attrs(&en.attrs);
                attrs.add_missing_attributes(&mut en.attrs, args);
                Item::Enum(en)
            } else {
                item
            }
        }
        Derive::QueryFragment | Derive::InputObject | Derive::Scalar => {
            if let Item::Struct(mut st) = item {
                let attrs = required_attrs.with_current_attrs(&st.attrs);
                attrs.add_missing_attributes(&mut st.attrs, args);
                Item::Struct(st)
            } else {
                item
            }
        }
    }
}

#[derive(Debug)]
struct RequiredAttributes {
    pub needs_schema_path: bool,
    pub needs_query_module: bool,
}

impl RequiredAttributes {
    fn for_derive(d: &Derive) -> RequiredAttributes {
        match d {
            Derive::Scalar => RequiredAttributes {
                needs_query_module: true,
                needs_schema_path: false,
            },
            _ => RequiredAttributes {
                needs_query_module: true,
                needs_schema_path: true,
            },
        }
    }

    fn with_current_attrs(mut self, attrs: &[syn::Attribute]) -> Self {
        use syn::{Meta, NestedMeta};

        for attr in attrs {
            if attr.path.is_ident("cynic") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(name_val)) = nested {
                            if name_val.path.is_ident("schema_path") {
                                self.needs_schema_path = false;
                            } else if name_val.path.is_ident("query_module") {
                                self.needs_query_module = false;
                            }
                        }
                    }
                }
            }
        }

        self
    }

    fn add_missing_attributes(self, attrs: &mut Vec<syn::Attribute>, args: &TransformModuleArgs) {
        if self.needs_schema_path {
            let schema_path = proc_macro2::Literal::string(&args.schema_path);
            attrs.push(syn::parse_quote! {
                #[cynic(schema_path = #schema_path)]
            })
        }

        if self.needs_query_module {
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
