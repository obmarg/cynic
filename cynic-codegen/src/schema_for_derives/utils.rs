use syn::{Attribute, Item, Meta, NestedMeta};

#[derive(Debug, PartialEq)]
pub enum Derive {
    QueryFragment,
    InlineFragments,
    Enum,
    Scalar,
    InputObject,
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

    if attr.is_none() {
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
                "QueryFragment2" => return Some(Derive::QueryFragment),
                "InlineFragments" => return Some(Derive::InlineFragments),
                "InlineFragments2" => return Some(Derive::InlineFragments),
                "Enum" => return Some(Derive::Enum),
                "Enum2" => return Some(Derive::Enum),
                "Scalar" => return Some(Derive::Scalar),
                "Scalar2" => return Some(Derive::Scalar),
                "InputObject" => return Some(Derive::InputObject),
                "InputObject2" => return Some(Derive::InputObject),
                _ => (),
            }
        }
    }

    None
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
}
