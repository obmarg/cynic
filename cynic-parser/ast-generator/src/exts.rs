use cynic_parser::type_system::{ScalarDefinition, TypeDefinition, UnionDefinition};
use quote::quote;

pub trait ScalarExt {
    fn is_inline(&self) -> bool;
    fn reader_fn_override(&self) -> Option<proc_macro2::TokenStream>;
}

impl ScalarExt for ScalarDefinition<'_> {
    fn is_inline(&self) -> bool {
        self.directives()
            .any(|directive| directive.name() == "inline")
    }

    fn reader_fn_override(&self) -> Option<proc_macro2::TokenStream> {
        if self.name() == "String" {
            return Some(quote! { &'a str });
        }
        None
    }
}

pub trait FileDirectiveExt<'a> {
    fn file_name(&self) -> &'a str;
}

impl<'a> FileDirectiveExt<'a> for TypeDefinition<'a> {
    fn file_name(&self) -> &'a str {
        self.directives()
            .find(|directive| directive.name() == "file")
            .and_then(|directive| directive.arguments().next()?.value().as_str())
            .unwrap_or(self.name())
    }
}

pub trait UnionExt<'a> {
    fn variant_name_override(&self, index: usize) -> Option<&'a str>;
}

impl<'a> UnionExt<'a> for UnionDefinition<'a> {
    fn variant_name_override(&self, index: usize) -> Option<&'a str> {
        self.directives()
            .find(|directive| directive.name() == "variant")?
            .arguments()
            .next()?
            .value()
            .as_items()?
            .nth(index)?
            .as_str()
    }
}
