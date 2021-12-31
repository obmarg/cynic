#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

// TODO: Everything in here is actually typed.  Need an untyped core with this
// layered on top...

use std::marker::PhantomData;

use crate::schema;

// Annoyingly this means people can't derive Deserialize _as well as_ use cynics derives.
// But whatever, don't do that people?  I _think_ it's an OK limitation.
pub trait QueryFragment<'de>: serde::Deserialize<'de> {
    type SchemaType;

    fn query(builder: QueryBuilder<Self::SchemaType>);
}

impl<'de, T> QueryFragment<'de> for Option<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Option<T::SchemaType>;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder.into_inner())
    }
}

impl<'de, T> QueryFragment<'de> for Vec<T>
where
    T: QueryFragment<'de>,
{
    type SchemaType = Vec<T::SchemaType>;

    fn query(builder: QueryBuilder<Self::SchemaType>) {
        T::query(builder.into_inner())
    }
}

impl<'de> QueryFragment<'de> for bool {
    type SchemaType = bool;

    fn query(builder: QueryBuilder<Self::SchemaType>) {}
}

// TODO: Can I also impl this for &'static str?
impl<'de> QueryFragment<'de> for String {
    type SchemaType = String;

    fn query(builder: QueryBuilder<Self::SchemaType>) {}
}

pub trait Enum<'de>: serde::Deserialize<'de> + serde::Serialize {}

// TODO: impl QueryFragment for Option, Box etc.

// TODO: QueryBuilder or SelectionBuilder?
pub struct QueryBuilder<'a, SchemaType> {
    phantom: PhantomData<fn() -> SchemaType>,
    fields: &'a mut Vec<Field>,
}

impl<'a, T> QueryBuilder<'a, Vec<T>> {
    fn into_inner(self) -> QueryBuilder<'a, T> {
        QueryBuilder {
            fields: self.fields,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> QueryBuilder<'a, Option<T>> {
    fn into_inner(self) -> QueryBuilder<'a, T> {
        QueryBuilder {
            fields: self.fields,
            phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Field {
    name: &'static str,
    children: Vec<Field>,
}

impl<'a, SchemaType> QueryBuilder<'a, SchemaType> {
    // TODO: this is just for testing
    pub fn temp_new(fields: &'a mut Vec<Field>) -> Self {
        QueryBuilder {
            phantom: PhantomData,
            fields,
        }
    }

    pub fn select_field<'b, FieldMarker, FieldType>(
        &'b mut self,
    ) -> FieldSelectionBuilder<'b, FieldType>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker, FieldType>,
    {
        self.fields.push(Field {
            name: FieldMarker::name(),
            children: Vec::new(),
        });

        FieldSelectionBuilder {
            field: self.fields.last_mut().unwrap(),
            phantom: PhantomData,
        }
    }

    // TODO: FragmentSpread & InlineFragment go here...

    // TODO: Could done be done via drop?  Maybe.
    pub fn done(self) {}
}

pub struct FieldSelectionBuilder<'a, SchemaType> {
    phantom: PhantomData<fn() -> SchemaType>,
    field: &'a mut Field,
}

impl<'a, FieldSchemaType> FieldSelectionBuilder<'a, FieldSchemaType> {
    // Note: I'm assuming that the DSL will use $whatever for variables
    // which provides a name and a way to do magic on that name to make
    // it work in all the contexts I need.
    //
    // Although the way I've ended up writing this - I may actually
    // keep the existing behaviour for arguments where each arg ends up in
    // a variable in the eventual query even if they _could_ be coded into the
    // document. Think it's easier, and not aware of any downsides.  Although
    // could possibly look into $whatever magic later...
    //
    // Though that's technically not possible is it.  Because we can't
    // serialize values at this point without knowing the destination format,
    // unless we do some serde transcoding type thing.
    // Maybe that's the only option?

    // TODO: Need to work on typesafety here...
    //pub fn argument(&mut self, name: &'static str, value: InputValue) {}

    // TODO: probably need all the reference nonsense here for value type
    pub fn argument<ArgumentName, ValueType>(&mut self, value: ValueType)
    where
        FieldSchemaType: schema::HasArgument<ArgumentName>,
        ValueType: schema::InputValue<FieldSchemaType::ArgumentSchemaType>,
    {
        todo!()
    }

    // Note: this is old code - actually I don't want field unwrapping because I care about
    // the options & vecs
    //
    // pub fn select_children<'b>(&'b mut self) -> QueryBuilder<'b, FieldSchemaType::InnerNamedType>
    // where
    //     FieldSchemaType: CompositeFieldType,

    pub fn select_children<'b>(&'b mut self) -> QueryBuilder<'b, FieldSchemaType> {
        QueryBuilder {
            phantom: PhantomData,
            fields: &mut self.field.children,
        }
    }

    // TODO: probably need an alias function here that defines an alias.

    // TODO: Could done be done via drop?  Maybe.
    pub fn done(self) {}
}

// TODO: etc.
