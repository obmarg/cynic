use proc_macro2::TokenStream;

mod argument_struct;
mod enum_marker;
mod field_selector;
mod input_struct;
mod interface_struct;
mod selector_struct;
mod union_struct;

use super::module::Module;
use crate::graphql_extensions::FieldExt;
use crate::{load_schema, Error, TypeIndex};
pub use argument_struct::ArgumentStruct;
use enum_marker::EnumMarker;
pub use field_selector::FieldSelector;
use input_struct::InputStruct;
use interface_struct::InterfaceStruct;
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

pub fn query_dsl_from_schema(input: QueryDslParams) -> Result<TokenStream, Error> {
    use quote::quote;

    let schema_data: QueryDsl = load_schema(input.schema_filename)?.into();

    Ok(quote! {
        #schema_data
    })
}

#[derive(Debug)]
pub struct QueryDsl {
    pub selectors: Vec<SelectorStruct>,
    pub argument_struct_modules: Vec<Module<ArgumentStruct>>,
    pub inputs: Vec<InputStruct>,
    pub unions: Vec<UnionStruct>,
    pub interfaces: Vec<InterfaceStruct>,
    pub enums: Vec<EnumMarker>,
}

impl From<graphql_parser::schema::Document> for QueryDsl {
    fn from(document: graphql_parser::schema::Document) -> Self {
        use graphql_parser::schema::{Definition, TypeDefinition};

        let type_index = TypeIndex::for_schema(&document);

        let mut selectors = vec![];
        let mut argument_struct_modules = vec![];
        let mut inputs = vec![];
        let mut unions = vec![];
        let mut interfaces = vec![];
        let mut enums = vec![];

        let root_query_type = find_root_query(&document.definitions);

        for definition in document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    let selector = SelectorStruct::from_object(
                        &object,
                        &type_index,
                        object.name == root_query_type,
                    );
                    if !selector.argument_structs.is_empty() {
                        argument_struct_modules
                            .push(Module::new(&object.name, selector.argument_structs.clone()));
                    }

                    selectors.push(selector);
                }
                Definition::TypeDefinition(TypeDefinition::InputObject(obj)) => {
                    inputs.push(InputStruct::from_input_object(obj, &type_index));
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
            inputs,
            unions,
            interfaces,
            enums,
        }
    }
}

fn find_root_query(definitions: &Vec<graphql_parser::schema::Definition>) -> String {
    use graphql_parser::schema::Definition;
    for definition in definitions {
        match definition {
            Definition::SchemaDefinition(schema) => {
                if let Some(query_type) = &schema.query {
                    return query_type.clone();
                }
            }
            _ => {}
        }
    }

    // Default to a type named "Query"
    "Query".to_string()
}

impl quote::ToTokens for QueryDsl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let selectors = &self.selectors;
        let argument_struct_modules = &self.argument_struct_modules;
        let inputs = &self.inputs;
        let unions = &self.unions;
        let interfaces = &self.interfaces;
        let enums = &self.enums;

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
                #inputs
            )*
            #(
                #enums
            )*
        })
    }
}
