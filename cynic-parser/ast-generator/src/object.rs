use indexmap::IndexMap;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};

use cynic_parser::type_system::{
    readers::{FieldDefinition, ObjectDefinition},
    TypeDefinition,
};

use crate::{
    exts::ScalarExt,
    file::{EntityOutput, EntityRef},
    format_code,
    idents::IdIdent,
};

pub fn object_output(
    object: ObjectDefinition<'_>,
    model_index: &IndexMap<&str, TypeDefinition<'_>>,
    id_trait: &str,
) -> anyhow::Result<EntityOutput> {
    let record_name = Ident::new(&format!("{}Record", object.name()), Span::call_site());
    let reader_name = Ident::new(object.name(), Span::call_site());
    let id_name = IdIdent(object.name());

    let edges = object
        .fields()
        .map(|field| -> anyhow::Result<FieldEdge> {
            Ok(FieldEdge {
                container: object,
                field,
                target: *model_index
                    .get(field.ty().name())
                    .ok_or_else(|| anyhow::anyhow!("Could not find type {}", field.ty().name()))?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let record_fields = edges.iter().copied().map(ObjectField);
    let reader_functions = edges.iter().copied().map(ReaderFunction);

    let record = format_code(quote! {
        pub struct #record_name {
            #(#record_fields),*
        }
    })?;

    let reader = format_code(quote! {
        #[derive(Clone, Copy)]
        pub struct #reader_name<'a>(ReadContext<'a, #id_name>);
    })?;

    let reader_impl = format_code(quote! {
        impl <'a> #reader_name<'a> {
            #(#reader_functions)*
        }
    })?;

    let id_trait = Ident::new(id_trait, Span::call_site());

    let executable_id = format_code(quote! {
        impl #id_trait for #id_name {
            type Reader<'a> = #reader_name<'a>;
        }
    })?;

    let from_impl = format_code(quote! {
        impl <'a> From<ReadContext<'a, #id_name>> for #reader_name<'a> {
            fn from(value: ReadContext<'a, #id_name>) -> Self {
                Self(value)
            }
        }
    })?;

    let contents = indoc::formatdoc!(
        r#"
        {record}

        {reader}

        {reader_impl}

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
        id: EntityRef::new(TypeDefinition::Object(object)).unwrap(),
        contents,
    })
}

#[derive(Clone, Copy)]
pub struct FieldEdge<'a> {
    #[allow(dead_code)]
    container: ObjectDefinition<'a>,
    field: FieldDefinition<'a>,
    target: TypeDefinition<'a>,
}

pub struct ObjectField<'a>(FieldEdge<'a>);

impl quote::ToTokens for ObjectField<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_name = Ident::new(self.0.field.name(), Span::call_site());

        let target_id = IdIdent(self.0.target.name());

        let ty = match self.0.target {
            TypeDefinition::Scalar(scalar) if scalar.is_inline() => {
                // I'm assuming inline scalars are copy here.
                let ident = Ident::new(self.0.target.name(), Span::call_site());
                if self.0.field.ty().is_list() {
                    quote! { Vec<#ident> }
                } else if self.0.field.ty().is_non_null() {
                    quote! { #ident }
                } else {
                    quote! { Option<#ident> }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                if self.0.field.ty().is_list() {
                    quote! { IdRange<#target_id> }
                } else if self.0.field.ty().is_non_null() {
                    quote! { #target_id }
                } else {
                    quote! { Option<#target_id> }
                }
            }
            _ => unimplemented!(),
        };

        tokens.append_all(quote! {
            pub #field_name: #ty
        });
    }
}

pub struct ReaderFunction<'a>(FieldEdge<'a>);

impl quote::ToTokens for ReaderFunction<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field_name = Ident::new(self.0.field.name(), Span::call_site());
        let target_type = Ident::new(self.0.target.name(), Span::call_site());

        let inner_ty = match self.0.target {
            TypeDefinition::Scalar(scalar) if scalar.is_inline() => {
                // I'm assuming inline scalars are copy here.
                quote! { #target_type }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() => {
                scalar.reader_fn_override().unwrap()
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                quote! { #target_type<'a> }
            }
            _ => unimplemented!(),
        };

        let ty = if self.0.field.ty().is_list() {
            quote! { impl ExactSizeIterator<Item = #inner_ty> }
        } else if self.0.field.ty().is_non_null() {
            quote! { #inner_ty }
        } else {
            quote! { Option<#inner_ty> }
        };

        let body = match self.0.target {
            TypeDefinition::Scalar(scalar)
                if scalar
                    .directives()
                    .any(|directive| directive.name() == "inline") =>
            {
                // I'm assuming inline scalars are copy here.
                quote! {
                    let document = self.0.document;

                    document.lookup(self.0.id).#field_name
                }
            }
            TypeDefinition::Scalar(scalar)
                if scalar.reader_fn_override().is_some() && self.0.field.ty().is_non_null() =>
            {
                // Scalars with reader_fn_override return the scalar directly _not_ a reader
                quote! {
                    let ast = &self.0.document;
                    ast.lookup(ast.lookup(self.0.id).#field_name)
                }
            }
            TypeDefinition::Scalar(scalar) if scalar.reader_fn_override().is_some() => {
                // Scalars with reader_fn_override return the scalar directly _not_ a reader
                quote! {
                    let document = self.0.document;

                    document.lookup(self.0.id).#field_name.map(|id| document.lookup(id))
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_list() =>
            {
                quote! {
                    let document = self.0.document;

                    document.lookup(self.0.id).#field_name.iter().map(|id| document.read(id))
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_non_null() =>
            {
                quote! {
                    let document = self.0.document;

                    document.read(document.lookup(self.0.id).#field_name)
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                quote! {
                    let document = self.0.document;

                    document.lookup(self.0.id).#field_name.map(|id| document.read(id))
                }
            }
            _ => unimplemented!("No support for this target type"),
        };

        tokens.append_all(quote! {
            pub fn #field_name(&self) -> #ty {
                #body
            }
        });
    }
}
