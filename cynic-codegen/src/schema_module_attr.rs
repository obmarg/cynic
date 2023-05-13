//! The implementation of the schema attribute for modules.

use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, parse_quote, Item, LitStr};

use crate::error::Errors;

pub fn attribute_impl(
    schema_literal: LitStr,
    mut module: syn::ItemMod,
) -> Result<TokenStream, Errors> {
    let schema_name = schema_literal.value();

    let filename_str = LitStr::new(
        &format!("/cynic-schemas/{schema_name}.rs"),
        schema_literal.span(),
    );

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let mut path = PathBuf::from(&out_dir);
        path.push("cynic-schemas");
        path.push(format!("{schema_name}.rs"));
        if !path.exists() {
            return Err(syn::Error::new(
                schema_literal.span(),
                format!("Couldn't find a schema named {schema_name} in {out_dir}\n\nHave you registered it in your build.rs?")
            ).into());
        }
    }

    // Silence a bunch of warnings inside this module
    module.attrs.push(parse_quote! {
        #[allow(clippy::all, clippy::pedantic, non_snake_case, non_camel_case_types, dead_code)]
    });

    let include: Item = parse_quote! {
        include!(concat!(env!("OUT_DIR"), #filename_str));
    };
    if let Some((_, items)) = &mut module.content {
        items.push(include);
    }

    Ok(quote! { #module })
}
