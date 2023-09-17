// TODO: docstring.
use darling::{util::SpannedValue, FromMeta};
use proc_macro2::TokenStream;

mod utils;

use utils::Derive;

#[derive(Debug, FromMeta)]
struct AddSchemaAttrParams {
    file: SpannedValue<String>,

    #[darling(default)]
    module: Option<String>,
}

pub fn add_schema_attrs_to_derives(
    args: Vec<darling::ast::NestedMeta>,
    query_module: syn::ItemMod,
) -> Result<TokenStream, syn::Error> {
    match AddSchemaAttrParams::from_list(&args) {
        Ok(args) => Ok(add_schema_attrs_to_derives_impl(args, query_module)),
        Err(e) => Ok(e.write_errors()),
    }
}

#[derive(Debug, FromMeta)]
struct QueryModuleParams {
    schema_path: SpannedValue<String>,
    // TODO: consider getting rid of query_module at some point (or at least making optional)
    query_module: Option<String>,
}

fn add_schema_attrs_to_derives_impl(
    args: AddSchemaAttrParams,
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

fn insert_cynic_attrs(args: &AddSchemaAttrParams, item: syn::Item) -> syn::Item {
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
        Derive::QueryFragment | Derive::QueryVariables | Derive::InputObject | Derive::Scalar => {
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
    needs_schema_path: bool,
    needs_schema_module: bool,
}

impl RequiredAttributes {
    fn for_derive(d: &Derive) -> RequiredAttributes {
        match d {
            Derive::QueryVariables | Derive::Scalar => RequiredAttributes {
                needs_schema_path: false,
                needs_schema_module: true,
            },
            _ => RequiredAttributes {
                needs_schema_path: true,
                needs_schema_module: true,
            },
        }
    }

    fn with_current_attrs(mut self, attrs: &[syn::Attribute]) -> Self {
        use darling::ast::NestedMeta;
        use syn::Meta;

        for attr in attrs {
            if attr.path().is_ident("cynic") {
                if let Meta::List(meta_list) = &attr.meta {
                    for nested in NestedMeta::parse_meta_list(meta_list.tokens.clone())
                        .expect("TODO: deal with this ")
                    {
                        if let NestedMeta::Meta(Meta::NameValue(name_val)) = nested {
                            if name_val.path.is_ident("schema_path") {
                                self.needs_schema_path = false;
                            } else if name_val.path.is_ident("schema_module") {
                                self.needs_schema_module = false;
                            }
                        }
                    }
                }
            }
        }

        self
    }

    fn add_missing_attributes(self, attrs: &mut Vec<syn::Attribute>, args: &AddSchemaAttrParams) {
        if self.needs_schema_path {
            let schema_path = proc_macro2::Literal::string(&args.file);
            attrs.push(syn::parse_quote! {
                #[cynic(schema_path = #schema_path)]
            })
        }

        if self.needs_schema_module {
            let query_module =
                proc_macro2::Literal::string(args.module.as_deref().unwrap_or("schema"));

            attrs.push(syn::parse_quote! {
                #[cynic(schema_module = #query_module)]
            })
        }
    }
}

impl From<QueryModuleParams> for AddSchemaAttrParams {
    fn from(params: QueryModuleParams) -> Self {
        AddSchemaAttrParams {
            file: params.schema_path,
            module: params.query_module,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args() -> AddSchemaAttrParams {
        AddSchemaAttrParams {
            file: "test.graphql".to_string().into(),
            module: "schema".to_string().into(),
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
                #[cynic(schema_module = "schema")]
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
            #[cynic(schema_path = "other.graphql", schema_module = "something")]
            struct Test {
                a: String
            }
        };

        let result = insert_cynic_attrs(&args(), item);

        assert_eq!(
            result,
            syn::parse_quote! {
                #[derive(cynic::QueryFragment)]
                #[cynic(schema_path = "other.graphql", schema_module = "something")]
                struct Test {
                    a: String
                }
            }
        )
    }
}
