use proc_macro2::TokenStream;

mod enum_marker;
mod field_selector;
mod input_object_marker;
mod interface_struct;
mod interfaces_implementations;
mod schema_roots;
mod selection_builder;
mod selector_struct;
mod union_struct;

use super::module::Module;
use crate::{load_schema, schema, SchemaLoadError, TypeIndex};
use enum_marker::EnumMarker;
pub use field_selector::FieldSelector;
use input_object_marker::InputObjectMarker;
use interface_struct::InterfaceStruct;
use interfaces_implementations::InterfacesImplementations;
use schema_roots::{RootTypes, SchemaRoot};
use selection_builder::FieldSelectionBuilder;
pub use selector_struct::SelectorStruct;
use union_struct::UnionStruct;

#[derive(Debug)]
pub struct QueryDslParams {
    pub schema_filename: String,
}

impl QueryDslParams {
    fn new(schema_filename: String) -> Self {
        QueryDslParams { schema_filename }
    }
}

impl syn::parse::Parse for QueryDslParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input
            .parse::<syn::LitStr>()
            .map(|lit_str| QueryDslParams::new(lit_str.value()))
    }
}

pub fn query_dsl_from_schema(input: QueryDslParams) -> Result<TokenStream, SchemaLoadError> {
    use quote::quote;

    let schema_data: QueryDsl = load_schema(input.schema_filename)?.into();

    Ok(quote! {
        #schema_data
    })
}

#[derive(Debug)]
pub struct QueryDsl {
    pub selectors: Vec<SelectorStruct>,
    pub argument_struct_modules: Vec<Module<FieldSelectionBuilder>>,
    pub unions: Vec<UnionStruct>,
    pub interfaces: Vec<InterfaceStruct>,
    pub enums: Vec<EnumMarker>,
    pub input_objects: Vec<InputObjectMarker>,
    pub schema_roots: Vec<SchemaRoot>,
    pub interfaces_implementations: Vec<InterfacesImplementations>,
}

impl From<schema::Document> for QueryDsl {
    fn from(document: schema::Document) -> Self {
        use schema::{Definition, TypeDefinition};

        let type_index = TypeIndex::for_schema(&document);

        let mut selectors = vec![];
        let mut argument_struct_modules = vec![];
        let mut input_objects = vec![];
        let mut unions = vec![];
        let mut interfaces = vec![];
        let mut enums = vec![];
        let mut schema_roots = vec![];
        let mut interfaces_implementations = vec![];

        let root_types = RootTypes::from_definitions(&document.definitions);

        for definition in &document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    if let Some(impls) = InterfacesImplementations::from_object(object) {
                        interfaces_implementations.push(impls);
                    }

                    // Would be nice to restructure this so that the argument structs
                    // are visible at the point we're generating the field_selectors...
                    let selector = SelectorStruct::from_object(&object, &type_index);
                    if !selector.selection_builders.is_empty() {
                        argument_struct_modules.push(Module::new(
                            &object.name,
                            selector.selection_builders.clone(),
                        ));
                    }

                    schema_roots.extend(root_types.root_from_selector_struct(&selector));

                    selectors.push(selector);
                }
                Definition::TypeDefinition(TypeDefinition::InputObject(input_type)) => {
                    input_objects.push(InputObjectMarker::from_input_object(&input_type));
                }
                Definition::TypeDefinition(TypeDefinition::Union(union)) => {
                    unions.push(UnionStruct::from_union(&union));
                }
                Definition::TypeDefinition(TypeDefinition::Interface(interface)) => {
                    interfaces.push(InterfaceStruct::from_interface(&interface));
                }
                Definition::TypeDefinition(TypeDefinition::Enum(en)) => {
                    enums.push(EnumMarker::from_enum(&en));
                }
                _ => {}
            }
        }

        QueryDsl {
            selectors,
            argument_struct_modules,
            input_objects,
            unions,
            interfaces,
            enums,
            schema_roots,
            interfaces_implementations,
        }
    }
}

impl quote::ToTokens for QueryDsl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let selectors = &self.selectors;
        let argument_struct_modules = &self.argument_struct_modules;
        let input_objects = &self.input_objects;
        let unions = &self.unions;
        let interfaces = &self.interfaces;
        let enums = &self.enums;
        let schema_roots = &self.schema_roots;
        let interfaces_implementations = &self.interfaces_implementations;

        tokens.append_all(quote! {
            #(
                #unions
            )*
            #(
                #interfaces
            )*
            #(
                #selectors
            )*
            #(
                #argument_struct_modules
            )*
            #(
                #input_objects
            )*
            #(
                #enums
            )*
            #(
                #schema_roots
            )*
            #(
                #interfaces_implementations
            )*
        })
    }
}
