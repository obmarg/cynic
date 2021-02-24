use proc_macro2::TokenStream;

mod argument_parameter;
mod field_selector;
mod input_object_marker;
mod interface_struct;
mod interfaces_implementations;
mod schema_roots;
mod selection_builder;
mod selector_struct;
mod type_lock_marker;
mod union_struct;

pub use field_selector::FieldSelector;
pub use selector_struct::SelectorStruct;

use super::module::Module;
use crate::{load_schema, schema, SchemaLoadError, TypeIndex};
use argument_parameter::{ArgumentParameter, ArgumentParameterType};
use input_object_marker::InputObjectMarker;
use interface_struct::InterfaceStruct;
use interfaces_implementations::InterfacesImplementations;
use schema_roots::{RootTypes, SchemaRoot};
use selection_builder::FieldSelectionBuilder;
use type_lock_marker::TypeLockMarker;
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
    pub type_lock_markers: Vec<TypeLockMarker>,
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
        let mut type_lock_markers = vec![];
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
                Definition::TypeDefinition(TypeDefinition::Interface(interface_def)) => {
                    interfaces_implementations
                        .push(InterfacesImplementations::from_interface(interface_def));

                    let interface = InterfaceStruct::from_interface(&interface_def, &type_index);

                    // Could be nice to restructure this so that the argument structs
                    // just live inside the selector_struct or similar?
                    if !interface.selector_struct.selection_builders.is_empty() {
                        argument_struct_modules.push(Module::new(
                            &interface_def.name,
                            interface.selector_struct.selection_builders.clone(),
                        ));
                    }

                    interfaces.push(interface);
                }
                Definition::TypeDefinition(TypeDefinition::Enum(en)) => {
                    type_lock_markers.push(TypeLockMarker::from_enum(&en));
                }
                Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => {
                    type_lock_markers.push(TypeLockMarker::from_scalar(&scalar))
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
            type_lock_markers,
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
        let type_lock_markers = &self.type_lock_markers;
        let schema_roots = &self.schema_roots;
        let interfaces_implementations = &self.interfaces_implementations;

        // TODO: consider putting `type_lock_markers` into a submodule
        // so they don't clash with user types...
        //
        // Either that or document the breaking change...
        //
        // But it seems like bad practice to clash if users _don't_ stick their
        // query-dsl in another module
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
                #type_lock_markers
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
