use indexmap::IndexMap;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};

use cynic_parser::type_system::{readers::UnionDefinition, TypeDefinition};

use crate::{
    file::{EntityOutput, EntityRef},
    format_code,
    idents::IdIdent,
};

pub fn union_output(
    object: UnionDefinition<'_>,
    model_index: &IndexMap<&str, TypeDefinition<'_>>,
) -> anyhow::Result<EntityOutput> {
    let record_name = Ident::new(&format!("{}Record", object.name()), Span::call_site());
    let reader_name = Ident::new(object.name(), Span::call_site());
    let id_name = IdIdent(object.name());

    let edges = object
        .members()
        .map(|ty| -> anyhow::Result<TypeEdge> {
            Ok(TypeEdge {
                container: object,
                target: *model_index
                    .get(ty)
                    .ok_or_else(|| anyhow::anyhow!("Could not find type {ty}"))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let record_variants = edges.iter().copied().map(RecordVariant);
    let reader_variants = edges.iter().copied().map(ReaderVariant);
    let from_branches = edges.iter().copied().map(FromBranch);

    let record = format_code(quote! {
        pub enum #record_name {
            #(#record_variants),*
        }
    })?;

    let reader = format_code(quote! {
        #[derive(Clone, Copy)]
        pub enum #reader_name<'a> {
            #(#reader_variants),*
        }
    })?;

    let executable_id = format_code(quote! {
        impl ExecutableId for #id_name {
            type Reader<'a> = #reader_name<'a>;
        }
    })?;

    let from_impl = format_code(quote! {
        impl <'a> From<ReadContext<'a, #id_name>> for #reader_name<'a> {
            fn from(value: ReadContext<'a, #id_name>) -> Self {
                match value.document.lookup(value.id) {
                    #(#from_branches),*
                }
            }
        }
    })?;

    let contents = indoc::formatdoc!(
        r#"
        {record}

        {reader}

        {executable_id}

        {from_impl}
    "#
    );

    Ok(EntityOutput {
        requires: edges
            .iter()
            .copied()
            .filter_map(|edge| EntityRef::new(edge.target))
            .collect(),
        id: EntityRef::new(TypeDefinition::Union(object)).unwrap(),
        contents,
    })
}

#[derive(Clone, Copy)]
pub struct TypeEdge<'a> {
    container: UnionDefinition<'a>,
    target: TypeDefinition<'a>,
}

pub struct RecordVariant<'a>(TypeEdge<'a>);

impl quote::ToTokens for RecordVariant<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant_name = Ident::new(self.0.target.name(), Span::call_site());
        let id = IdIdent(self.0.target.name());

        tokens.append_all(quote! {
            #variant_name(#id)
        });
    }
}

pub struct ReaderVariant<'a>(TypeEdge<'a>);

impl quote::ToTokens for ReaderVariant<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant_name = Ident::new(self.0.target.name(), Span::call_site());
        let reader = Ident::new(self.0.target.name(), Span::call_site());

        tokens.append_all(quote! {
            #variant_name(#reader<'a>)
        });
    }
}

pub struct FromBranch<'a>(TypeEdge<'a>);

impl quote::ToTokens for FromBranch<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let this_record = Ident::new(
            &format!("{}Record", self.0.container.name()),
            Span::call_site(),
        );
        let this_reader = Ident::new(self.0.container.name(), Span::call_site());
        let variant_name = Ident::new(self.0.target.name(), Span::call_site());

        tokens.append_all(quote! {
            #this_record::#variant_name(id) => #this_reader::#variant_name(value.document.read(*id))
        });
    }
}
