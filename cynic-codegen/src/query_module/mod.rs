// TODO: docstring.
use darling::{util::SpannedValue, FromMeta};
use proc_macro2::{Span, TokenStream};

mod utils;

use utils::Derive;

use crate::fragment_derive;

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
    let (_, module_items) = query_module.content.as_ref().unwrap();

    let derives: Vec<TokenStream> = module_items
        .into_iter()
        .map(|i| derive_for_item(i, &args, &fragment_derive_schema))
        .collect();

    let (brace, module_items) = query_module.content.unwrap();
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
}

fn derive_for_item(
    item: &syn::Item,
    args: &TransformModuleArgs,
    fragment_derive_schema: &fragment_derive::Schema,
) -> TokenStream {
    match utils::find_derives(item).first() {
        None => TokenStream::new(),
        Some(Derive::QueryFragment) => fragment_derive(item, args, fragment_derive_schema),
        Some(Derive::InlineFragments) => inline_fragments_derive(item, args),
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
