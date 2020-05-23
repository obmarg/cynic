use syn::{Item, Meta, MetaList, NestedMeta};

#[derive(Debug, PartialEq)]
pub enum Derive {
    QueryFragment,
}

pub fn find_derives(item: &Item) -> Vec<Derive> {
    if let Item::Struct(s) = item {
        let attr = s.attrs.iter().find(|attr| attr.path.is_ident("derive"));

        if let None = attr {
            return vec![];
        }
        let attr = attr.unwrap();

        let meta_list = match attr.parse_meta() {
            Ok(Meta::List(list)) => list,
            _ => {
                return vec![];
            }
        };

        return meta_list
            .nested
            .iter()
            .map(derive_for_nested_meta)
            .flatten()
            .collect();
    }

    return vec![];
}

fn derive_for_nested_meta(nested: &NestedMeta) -> Option<Derive> {
    if let NestedMeta::Meta(Meta::Path(path)) = nested {
        if let Some(last) = path.segments.last() {
            match last.ident.to_string().as_ref() {
                "QueryFragment" => return Some(Derive::QueryFragment),
                _ => (),
            }
        }
    }
    return None;
}

pub fn strip_cynic_attrs(item: syn::Item) -> syn::Item {
    let mut item = item;
    match item {
        Item::Struct(mut s) => {
            s.attrs = s
                .attrs
                .into_iter()
                .filter(|attr| !attr.path.is_ident("cynic"))
                .map(|attr| {
                    if attr.path.is_ident("derive") {
                        let mut meta_list = match attr.parse_meta() {
                            Ok(Meta::List(list)) => list,
                            _ => return attr,
                        };
                        meta_list.nested = meta_list
                            .nested
                            .into_iter()
                            .filter(|nested| derive_for_nested_meta(nested).is_none())
                            .collect();

                        // TODO: test this fucker

                        syn::parse_quote! {
                            #[#meta_list]
                        }
                    } else {
                        attr
                    }
                })
                .collect();

            for field in &mut s.fields {
                field.attrs = field
                    .attrs
                    .iter()
                    .filter(|attr| !attr.path.is_ident("cynic_arguments"))
                    .cloned()
                    .collect();
            }

            Item::Struct(s)

            // TODO: Do a filter on field attributes as well.
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_find_derives_with_fragment() {
        let item: syn::Item = syn::parse2(quote! {
            #[derive(Debug, cynic::QueryFragment)]
            struct Something {}
        })
        .unwrap();

        assert_eq!(find_derives(&item), vec![Derive::QueryFragment]);
    }

    #[test]
    fn test_find_derives_when_no_cynic_derive() {
        let item: syn::Item = syn::parse2(quote! {
            #[derive(Debug)]
            struct Something {}
        })
        .unwrap();

        assert_eq!(find_derives(&item), vec![]);
    }

    #[test]
    fn test_find_derives_when_no_derive() {
        let item: syn::Item = syn::parse2(quote! {
            struct Something {}
        })
        .unwrap();

        assert_eq!(find_derives(&item), vec![]);
    }

    #[test]
    fn test_strip_cynic_attrs() {
        let input: syn::Item = syn::parse2(quote! {
            #[derive(Debug, cynic::QueryFragment, serde::Serialize)]
            #[cynic(query_path = "something")]
            #[serde(something)]
            struct Something {
                #[cynic_arguments(x = "1")]
                field: i32,
                other_field: f32
            }
        })
        .unwrap();

        let expected: syn::Item = syn::parse2(quote! {
            #[derive(Debug, serde::Serialize)]
            #[serde(something)]
            struct Something {
                field: i32,
                other_field: f32
            }
        })
        .unwrap();

        assert_eq!(strip_cynic_attrs(input), expected)
    }
}
