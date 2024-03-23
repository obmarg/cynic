use indexmap::IndexMap;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};

use cynic_parser::type_system::{
    readers::{FieldDefinition, ObjectDefinition},
    TypeDefinition,
};

use crate::{format_code, idents::IdIdent};

pub fn object_output(
    object: ObjectDefinition<'_>,
    model_index: &IndexMap<&str, TypeDefinition<'_>>,
) -> anyhow::Result<String> {
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

    // TODO: Split these up and format individually
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

    let executable_id = format_code(quote! {
        impl ExecutableId for #id_name {
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

    Ok(indoc::formatdoc!(
        r#"
        {record}

        {reader}

        {reader_impl}

        {executable_id}

        {from_impl}
    "#
    ))
}

#[derive(Clone, Copy)]
pub struct FieldEdge<'a> {
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
            TypeDefinition::Scalar(scalar)
                if scalar
                    .directives()
                    .any(|directive| directive.name() == "inline") =>
            {
                let ident = Ident::new(self.0.target.name(), Span::call_site());
                if self.0.field.ty().is_non_null() {
                    quote! { #ident }
                } else {
                    quote! { Option<#ident> }
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_list() =>
            {
                quote! {
                    IdRange<#target_id>
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_non_null() =>
            {
                quote! { #target_id }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                quote! { Option<#target_id> }
            }
            _ => unimplemented!("No support for this target type"),
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

        let ty = match self.0.target {
            TypeDefinition::Scalar(scalar)
                if scalar
                    .directives()
                    .any(|directive| directive.name() == "inline") =>
            {
                // For now I'm not generating scalar accessors, will revisit
                return;
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_list() =>
            {
                quote! {
                    impl ExactSizeIterator<Item = #target_type<'a>>
                }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_)
                if self.0.field.ty().is_non_null() =>
            {
                quote! { #target_type<'a> }
            }
            TypeDefinition::Object(_) | TypeDefinition::Union(_) | TypeDefinition::Scalar(_) => {
                quote! { Option<#target_type<'a>> }
            }
            _ => unimplemented!("No support for this target type"),
        };

        let body = match self.0.target {
            TypeDefinition::Scalar(scalar)
                if scalar
                    .directives()
                    .any(|directive| directive.name() == "inline") =>
            {
                // For now I'm not generating scalar accessors, will revisit
                return;
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
            fn #field_name(&self) -> #ty {
                #body
            }
        });
    }
}
