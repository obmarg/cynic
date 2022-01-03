#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::fmt::Write;

// TODO: Everything in here is actually typed.  Need an untyped core with this
// layered on top...

use std::marker::PhantomData;

use crate::indent::indented;
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

// TODO: Does this need a TypeLock on it?
pub trait Enum<'de>: serde::Deserialize<'de> + serde::Serialize {}

// TODO: Does this need a TypeLock on it?
pub trait Scalar<'de>: serde::Deserialize<'de> + serde::Serialize {}

// TODO: QueryBuilder or SelectionBuilder?
pub struct QueryBuilder<'a, SchemaType> {
    phantom: PhantomData<fn() -> SchemaType>,
    selection_set: &'a mut SelectionSet,
    has_typename: bool,
}

impl<'a, T> QueryBuilder<'a, Vec<T>> {
    fn into_inner(self) -> QueryBuilder<'a, T> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> QueryBuilder<'a, Option<T>> {
    fn into_inner(self) -> QueryBuilder<'a, T> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
        }
    }
}

// TODO: move this to selection set module.
#[derive(Debug, Default)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

#[derive(Debug)]
enum Selection {
    Field(FieldSelection),
    InlineFragment(InlineFragment),
    FragmentSpread(FragmentSpread),
}

#[derive(Debug)]
pub struct FieldSelection {
    name: &'static str,
    children: SelectionSet,
}

#[derive(Debug, Default)]
pub struct InlineFragment {
    on_clause: Option<&'static str>,
    children: SelectionSet,
}

#[derive(Debug)]
pub struct FragmentSpread {}

// TODO: Maybe FieldSelector/SelectionSetBuilder/SelectionSet/SelectionBuilder/Selector?
// Kinda like SelectionSet actually since we are literally just building up a set of fields?
// But who knows.  Lets leave naming for later.
impl<'a, SchemaType> QueryBuilder<'a, SchemaType> {
    pub(crate) fn new(selection_set: &'a mut SelectionSet) -> Self {
        QueryBuilder {
            phantom: PhantomData,
            has_typename: false,
            selection_set,
        }
    }

    // TODO: this is just for testing
    pub fn temp_new(selection_set: &'a mut SelectionSet) -> Self {
        QueryBuilder {
            phantom: PhantomData,
            has_typename: false,
            selection_set,
        }
    }

    // TODO: reckon add_field might be better for this?
    // particularly if we name the container selectionset or something.
    pub fn select_field<FieldMarker, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldType>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker, FieldType>,
    {
        self.selection_set
            .selections
            .push(Selection::Field(FieldSelection {
                name: FieldMarker::name(),
                children: SelectionSet::default(),
            }));

        let field_selection = match self.selection_set.selections.last_mut() {
            Some(Selection::Field(field_selection)) => field_selection,
            _ => panic!("This should not be possible"),
        };

        FieldSelectionBuilder {
            field: field_selection,
            phantom: PhantomData,
        }
    }

    pub fn inline_fragment(&'_ mut self) -> InlineFragmentBuilder<'_, SchemaType> {
        if !self.has_typename {
            self.selection_set
                .selections
                .push(Selection::Field(FieldSelection {
                    name: "__typename",
                    children: SelectionSet::default(),
                }));
            self.has_typename = true;
        }

        self.selection_set
            .selections
            .push(Selection::InlineFragment(InlineFragment::default()));

        let inline_fragment = match self.selection_set.selections.last_mut() {
            Some(Selection::InlineFragment(inline_fragment)) => inline_fragment,
            _ => panic!("This should not be possible"),
        };

        InlineFragmentBuilder {
            inline_fragment,
            phantom: PhantomData,
        }
    }

    // TODO: FragmentSpread & InlineFragment go here...

    // TODO: Could done be done via drop?  Maybe.
    pub fn done(self) {}
}

pub struct FieldSelectionBuilder<'a, SchemaType> {
    phantom: PhantomData<fn() -> SchemaType>,
    field: &'a mut FieldSelection,
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
        QueryBuilder::new(&mut self.field.children)
    }

    // TODO: probably need an alias function here that defines an alias.

    // TODO: Could done be done via drop?  Maybe.
    pub fn done(self) {}
}

pub struct InlineFragmentBuilder<'a, SchemaType> {
    phantom: PhantomData<fn() -> SchemaType>,
    inline_fragment: &'a mut InlineFragment,
}

impl<'a, SchemaType> InlineFragmentBuilder<'a, SchemaType> {
    pub fn on<Subtype>(self) -> InlineFragmentBuilder<'a, Subtype>
    where
        Subtype: crate::schema::NamedType,
        SchemaType: crate::schema::HasSubtype<Subtype>,
    {
        self.inline_fragment.on_clause = Some(Subtype::name());
        InlineFragmentBuilder {
            inline_fragment: self.inline_fragment,
            phantom: PhantomData,
        }
    }

    pub fn select_children(&'_ mut self) -> QueryBuilder<'_, SchemaType> {
        QueryBuilder::new(&mut self.inline_fragment.children)
    }
}

// TODO: Move this somewhere else?
impl std::fmt::Display for SelectionSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.selections.is_empty() {
            writeln!(f, " {{")?;
            for child in &self.selections {
                write!(indented(f, 2), "{}", child)?;
            }
            write!(f, "}}")?;
        }
        writeln!(f)
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Field(field_selection) => {
                write!(f, "{}", field_selection.name)?;
                write!(f, "{}", field_selection.children)
            }
            Selection::InlineFragment(inline_fragment) => {
                write!(f, "...")?;
                if let Some(on_type) = inline_fragment.on_clause {
                    write!(f, " on {}", on_type)?;
                }
                write!(f, "{}", inline_fragment.children)
            }
            Selection::FragmentSpread(_) => todo!(),
        }
    }
}
