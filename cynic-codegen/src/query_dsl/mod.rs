use proc_macro2::TokenStream;

mod argument_struct;
mod field_selector;
mod graphql_enum;
mod input_struct;
mod selector_struct;
mod union_struct;

use super::module::Module;
use crate::graphql_extensions::FieldExt;
use crate::{Error, TypeIndex};
pub use argument_struct::ArgumentStruct;
pub use field_selector::FieldSelector;
use graphql_enum::GraphQLEnum;
use input_struct::InputStruct;
pub use selector_struct::SelectorStruct;
use union_struct::UnionStruct;

#[derive(Debug)]
pub struct QueryDslParams {
    schema_filename: String,
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

    let schema = std::fs::read_to_string(&input.schema_filename)?;
    let schema_data: QueryDsl = graphql_parser::schema::parse_schema(&schema)?.into();

    Ok(quote! {
        #schema_data
    })
}

#[derive(Debug)]
pub struct QueryDsl {
    pub selectors: Vec<SelectorStruct>,
    pub enums: Vec<GraphQLEnum>,
    pub argument_struct_modules: Vec<Module<ArgumentStruct>>,
    pub inputs: Vec<InputStruct>,
    pub unions: Vec<UnionStruct>,
}

impl From<graphql_parser::schema::Document> for QueryDsl {
    fn from(document: graphql_parser::schema::Document) -> Self {
        use graphql_parser::schema::{Definition, TypeDefinition};

        let type_index = TypeIndex::for_schema(&document);

        let mut selectors = vec![];
        let mut enums = vec![];
        let mut argument_struct_modules = vec![];
        let mut inputs = vec![];
        let mut unions = vec![];

        let root_query_type = find_root_query(&document.definitions);

        for definition in document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    selectors.push(SelectorStruct::from_object(
                        &object,
                        &type_index,
                        object.name == root_query_type,
                    ));

                    let mut argument_structs = vec![];
                    for field in &object.fields {
                        let required_arguments = field.required_arguments();
                        if !required_arguments.is_empty() {
                            argument_structs.push(ArgumentStruct::from_field(
                                field,
                                &required_arguments,
                                true,
                                &type_index,
                            ));
                        }

                        let optional_arguments = field.optional_arguments();
                        if !optional_arguments.is_empty() {
                            argument_structs.push(ArgumentStruct::from_field(
                                field,
                                &optional_arguments,
                                false,
                                &type_index,
                            ));
                        }
                    }

                    if !argument_structs.is_empty() {
                        argument_struct_modules.push(Module::new(&object.name, argument_structs));
                    }
                }
                Definition::TypeDefinition(TypeDefinition::Enum(gql_enum)) => {
                    enums.push(gql_enum.into());
                }
                Definition::TypeDefinition(TypeDefinition::InputObject(obj)) => {
                    inputs.push(InputStruct::from_input_object(obj, &type_index));
                }
                Definition::TypeDefinition(TypeDefinition::Union(union)) => {
                    unions.push(UnionStruct::from_union(&union));
                }
                _ => {}
            }
        }

        QueryDsl {
            selectors,
            enums,
            argument_struct_modules,
            inputs,
            unions,
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

        let enums = &self.enums;
        let selectors = &self.selectors;
        let argument_struct_modules = &self.argument_struct_modules;
        let inputs = &self.inputs;
        let unions = &self.unions;

        tokens.append_all(quote! {
            #(
                #enums
            )*
            #(
                #unions
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
        })
    }
}
