#![allow(clippy::panic)]

use std::collections::HashMap;

use async_graphql::dynamic::{FieldValue, ResolverContext};
use cynic_parser::{common::WrappingType, type_system as parser};
use serde::Deserialize;

use crate::{MockGraphQlServer, MockGraphQlServerBuilder};

use super::{resolvers::Resolver, DynamicSchema};

pub struct DynamicSchemaBuilder {
    sdl: String,
    field_resolvers: ResolverMap,
}

type ResolverMap = HashMap<(String, String), Box<dyn Resolver>>;

impl DynamicSchemaBuilder {
    pub fn new(sdl: &str) -> Self {
        DynamicSchemaBuilder {
            sdl: sdl.into(),
            field_resolvers: Default::default(),
        }
    }

    pub fn with_resolver(
        mut self,
        ty: &str,
        field: &str,
        resolver: impl Resolver + 'static,
    ) -> Self {
        self.field_resolvers
            .insert((ty.into(), field.into()), Box::new(resolver));
        self
    }

    pub fn into_server_builder(self) -> MockGraphQlServerBuilder {
        let Self {
            sdl,
            mut field_resolvers,
        } = self;

        let schema = cynic_parser::parse_type_system_document(&sdl)
            .map_err(|e| e.to_report(&sdl))
            .expect("a valid document");

        let mut builder = schema_builder(&schema);

        for definition in schema.definitions() {
            match definition {
                parser::Definition::Type(def) => {
                    builder = builder.register(convert_type(def, &mut field_resolvers));
                }
                parser::Definition::TypeExtension(_) => {
                    unimplemented!("this is just for tests, extensions aren't supported")
                }
                _ => {}
            }
        }

        let schema = builder.finish().unwrap();

        MockGraphQlServer::builder(DynamicSchema { schema, sdl })
    }
}

fn convert_type(
    def: parser::TypeDefinition<'_>,
    resolvers: &mut ResolverMap,
) -> async_graphql::dynamic::Type {
    match def {
        parser::TypeDefinition::Scalar(def) => {
            async_graphql::dynamic::Scalar::new(def.name()).into()
        }
        parser::TypeDefinition::Object(def) => convert_object(def, resolvers),
        parser::TypeDefinition::Interface(def) => convert_iface(def),
        parser::TypeDefinition::Union(def) => convert_union(def),
        parser::TypeDefinition::Enum(def) => convert_enum(def),
        parser::TypeDefinition::InputObject(def) => convert_input_object(def),
    }
}

fn convert_object(
    def: parser::ObjectDefinition<'_>,
    resolvers: &mut ResolverMap,
) -> async_graphql::dynamic::Type {
    use async_graphql::dynamic::*;

    let mut object = Object::new(def.name());

    if let Some(description) = def.description() {
        object = object.description(description.to_cow());
    }

    for name in def.implements_interfaces() {
        object = object.implement(name);
    }

    for field_def in def.fields() {
        let type_ref = convert_type_ref(field_def.ty());
        let resolver = std::sync::Mutex::new(
            resolvers
                .remove(&(def.name().into(), field_def.name().into()))
                .unwrap_or_else(|| Box::new(default_field_resolver(field_def.name()))),
        );

        let mut field = Field::new(field_def.name(), type_ref, move |context| {
            let mut resolver = resolver.lock().expect("mutex to be unpoisoned");
            FieldFuture::Value(resolver.resolve(context).map(|value| {
                let value = async_graphql::Value::deserialize(value).unwrap();
                transform_into_field_value(value)
            }))
        });

        if let Some(description) = field_def.description() {
            field = field.description(description.to_cow());
        }

        for argument in field_def.arguments() {
            field = field.argument(convert_input_value(argument));
        }

        object = object.field(field);
    }

    object.into()
}

fn transform_into_field_value(mut value: async_graphql::Value) -> FieldValue<'static> {
    match value {
        async_graphql::Value::Object(ref mut fields) => {
            if let Some(async_graphql::Value::String(ty)) = fields.swap_remove("__typename") {
                FieldValue::from(value).with_type(ty)
            } else {
                FieldValue::from(value)
            }
        }
        async_graphql::Value::List(values) => {
            FieldValue::list(values.into_iter().map(transform_into_field_value))
        }
        value => FieldValue::from(value),
    }
}

fn convert_iface(def: parser::InterfaceDefinition<'_>) -> async_graphql::dynamic::Type {
    use async_graphql::dynamic::*;
    let mut interface = Interface::new(def.name());

    if let Some(description) = def.description() {
        interface = interface.description(description.to_cow());
    }

    for field_def in def.fields() {
        let type_ref = convert_type_ref(field_def.ty());

        let mut field = InterfaceField::new(field_def.name(), type_ref);

        for argument in field_def.arguments() {
            field = field.argument(convert_input_value(argument));
        }

        interface = interface.field(field);
    }

    interface.into()
}

fn convert_union(def: parser::UnionDefinition<'_>) -> async_graphql::dynamic::Type {
    use async_graphql::dynamic::*;

    let mut output = Union::new(def.name());

    if let Some(description) = def.description() {
        output = output.description(description.to_cow());
    }

    for member in def.members() {
        output = output.possible_type(member.name());
    }

    output.into()
}

fn convert_enum(def: parser::EnumDefinition<'_>) -> async_graphql::dynamic::Type {
    use async_graphql::dynamic::*;

    let mut output = Enum::new(def.name()).items(def.values().map(|value| {
        let mut item = EnumItem::new(value.value());

        if let Some(desc) = value.description() {
            item = item.description(desc.to_cow());
        }

        item
    }));

    if let Some(description) = def.description() {
        output = output.description(description.to_cow());
    }

    output.into()
}

fn convert_input_object(def: parser::InputObjectDefinition<'_>) -> async_graphql::dynamic::Type {
    use async_graphql::dynamic::*;

    let mut object = InputObject::new(def.name());

    if let Some(description) = def.description() {
        object = object.description(description.to_cow());
    }

    for field_def in def.fields() {
        object = object.field(convert_input_value(field_def))
    }

    object.into()
}

fn convert_type_ref(ty: parser::Type<'_>) -> async_graphql::dynamic::TypeRef {
    use async_graphql::dynamic::TypeRef;

    let mut output = TypeRef::named(ty.name());

    for wrapper in ty.wrappers() {
        match wrapper {
            WrappingType::NonNull => {
                output = TypeRef::NonNull(Box::new(output));
            }
            WrappingType::List => {
                output = TypeRef::List(Box::new(output));
            }
        }
    }

    output
}

fn convert_input_value(
    value_def: parser::InputValueDefinition<'_>,
) -> async_graphql::dynamic::InputValue {
    use async_graphql::dynamic::InputValue;

    let mut value = InputValue::new(value_def.name(), convert_type_ref(value_def.ty()));

    if let Some(description) = value_def.description() {
        value = value.description(description.to_cow());
    }

    if let Some(default) = value_def.default_value() {
        value = value.default_value(convert_value(default))
    }

    value
}

fn convert_value(value: cynic_parser::ConstValue<'_>) -> async_graphql::Value {
    match value {
        cynic_parser::ConstValue::Int(inner) => async_graphql::Value::Number(inner.as_i64().into()),
        cynic_parser::ConstValue::Float(inner) => {
            async_graphql::Value::Number(serde_json::Number::from_f64(inner.as_f64()).unwrap())
        }
        cynic_parser::ConstValue::String(inner) => {
            async_graphql::Value::String(inner.as_str().into())
        }
        cynic_parser::ConstValue::Boolean(inner) => async_graphql::Value::Boolean(inner.as_bool()),
        cynic_parser::ConstValue::Null(_) => async_graphql::Value::Null,
        cynic_parser::ConstValue::Enum(inner) => {
            async_graphql::Value::Enum(async_graphql::Name::new(inner.name()))
        }
        cynic_parser::ConstValue::List(inner) => {
            async_graphql::Value::List(inner.items().map(convert_value).collect())
        }
        cynic_parser::ConstValue::Object(inner) => async_graphql::Value::Object(
            inner
                .fields()
                .map(|field| {
                    (
                        async_graphql::Name::new(field.name()),
                        convert_value(field.value()),
                    )
                })
                .collect(),
        ),
    }
}

fn schema_builder(
    schema: &cynic_parser::TypeSystemDocument,
) -> async_graphql::dynamic::SchemaBuilder {
    let (query_name, mutation_name, subscription_name) = root_types(schema);
    async_graphql::dynamic::Schema::build(query_name, mutation_name, subscription_name)
}

fn root_types(schema: &cynic_parser::TypeSystemDocument) -> (&str, Option<&str>, Option<&str>) {
    use parser::Definition;

    let mut query_name = "Query";
    let mut mutation_name = None;
    let mut subscription_name = None;
    let mut found_schema_def = false;
    let mut mutation_present = false;
    let mut subscription_present = false;
    for definition in schema.definitions() {
        if let Definition::Schema(_) = definition {
            found_schema_def = true;
        }
        match definition {
            Definition::Schema(schema) | Definition::SchemaExtension(schema) => {
                if let Some(def) = schema.query_type() {
                    query_name = def.named_type();
                }
                if let Some(def) = schema.mutation_type() {
                    mutation_name = Some(def.named_type());
                }
                if let Some(def) = schema.subscription_type() {
                    subscription_name = Some(def.named_type());
                }
            }
            Definition::Type(ty) | Definition::TypeExtension(ty) if ty.name() == "Mutation" => {
                mutation_present = true
            }
            Definition::Type(ty) | Definition::TypeExtension(ty) if ty.name() == "Subscription" => {
                subscription_present = true
            }
            _ => {}
        }
    }
    if !found_schema_def {
        if mutation_present {
            mutation_name = Some("Mutation");
        }
        if subscription_present {
            mutation_name = Some("Subscription");
        }
    }

    (query_name, mutation_name, subscription_name)
}

fn default_field_resolver(field_name: &str) -> impl Resolver {
    let field_name = async_graphql::Name::new(field_name);

    move |context: ResolverContext<'_>| {
        if let Some(value) = context.parent_value.as_value() {
            return match value {
                async_graphql::Value::Object(map) => map
                    .get(&field_name)
                    .map(|value| value.clone().into_json().unwrap()),
                _ => None,
            };
        }
        panic!("Unexpected parent value for tests",)
    }
}
