use syn::{Attribute, Item, Meta, MetaList, NestedMeta};

#[derive(Debug, PartialEq)]
pub enum Derive {
    QueryFragment,
    InlineFragments,
    Enum,
    Scalar,
}

pub fn find_derives(item: &Item) -> Vec<Derive> {
    match item {
        Item::Struct(s) => derive_from_attributes(&s.attrs),
        Item::Enum(e) => derive_from_attributes(&e.attrs),
        _ => vec![],
    }
}

fn derive_from_attributes(attrs: &[Attribute]) -> Vec<Derive> {
    let attr = attrs.iter().find(|attr| attr.path.is_ident("derive"));

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

fn derive_for_nested_meta(nested: &NestedMeta) -> Option<Derive> {
    if let NestedMeta::Meta(Meta::Path(path)) = nested {
        if let Some(last) = path.segments.last() {
            match last.ident.to_string().as_ref() {
                "QueryFragment" => return Some(Derive::QueryFragment),
                "InlineFragments" => return Some(Derive::InlineFragments),
                "Enum" => return Some(Derive::Enum),
                "Scalar" => return Some(Derive::Scalar),
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
            s.attrs = filter_cynic_attrs(s.attrs);

            for field in &mut s.fields {
                field.attrs = field
                    .attrs
                    .iter()
                    .filter(|attr| !is_cynic_attr(&attr.path))
                    .cloned()
                    .collect();
            }

            Item::Struct(s)
        }
        Item::Enum(mut e) => {
            e.attrs = filter_cynic_attrs(e.attrs);

            for variant in &mut e.variants {
                variant.attrs = variant
                    .attrs
                    .iter()
                    .filter(|attr| !is_cynic_attr(&attr.path))
                    .cloned()
                    .collect();
            }

            Item::Enum(e)
        }
        other => other,
    }
}

fn is_cynic_attr(path: &syn::Path) -> bool {
    path.is_ident("cynic_arguments") || path.is_ident("cynic")
}

fn filter_cynic_attrs(attrs: Vec<Attribute>) -> Vec<Attribute> {
    attrs
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

                syn::parse_quote! {
                    #[#meta_list]
                }
            } else {
                attr
            }
        })
        .collect()
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
    fn test_find_derives_on_enum() {
        let item: syn::Item = syn::parse2(quote! {
            #[derive(cynic::InlineFragments)]
            enum Something {}
        })
        .unwrap();

        assert_eq!(find_derives(&item), vec![Derive::InlineFragments]);
    }

    #[test]
    fn test_strip_cynic_attrs_struct() {
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

    #[test]
    fn test_strip_cynic_attrs_enum() {
        let input: syn::Item = syn::parse2(quote! {
            #[derive(Debug, cynic::InlineFragments, serde::Serialize)]
            #[cynic(query_path = "something")]
            #[serde(something)]
            enum Something {
                #[cynic(rename = "test")]
                AVariant
            }
        })
        .unwrap();

        let expected: syn::Item = syn::parse2(quote! {
            #[derive(Debug, serde::Serialize)]
            #[serde(something)]
            enum Something {
                AVariant
            }
        })
        .unwrap();

        assert_eq!(strip_cynic_attrs(input), expected)
    }
}
