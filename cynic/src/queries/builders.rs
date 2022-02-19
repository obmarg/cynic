#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::{borrow::Cow, marker::PhantomData};

use crate::{coercions::CoercesTo, schema, variables::VariableDefinition};

use super::{ast::*, to_input_literal, FlattensInto, Recursable};

// TODO: QueryBuilder or SelectionBuilder?
pub struct QueryBuilder<'a, SchemaType, Variables> {
    phantom: PhantomData<fn() -> (SchemaType, Variables)>,
    selection_set: &'a mut SelectionSet,
    has_typename: bool,
    recurse_depth: Option<u8>,
}

impl<'a, T, U> QueryBuilder<'a, Vec<T>, U> {
    pub(crate) fn into_inner(self) -> QueryBuilder<'a, T, U> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
            recurse_depth: self.recurse_depth,
        }
    }
}

impl<'a, T, U> QueryBuilder<'a, Option<T>, U> {
    pub(crate) fn into_inner(self) -> QueryBuilder<'a, T, U> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
            recurse_depth: self.recurse_depth,
        }
    }
}

// TODO: move this to selection set module.
// TODO: Maybe FieldSelector/SelectionSetBuilder/SelectionSet/SelectionBuilder/Selector?
// Kinda like SelectionSet actually since we are literally just building up a set of fields?
// But who knows.  Lets leave naming for later.
impl<'a, SchemaType, Variables> QueryBuilder<'a, SchemaType, Variables> {
    pub(crate) fn new(selection_set: &'a mut SelectionSet) -> Self {
        QueryBuilder {
            phantom: PhantomData,
            has_typename: false,
            selection_set,
            recurse_depth: None,
        }
    }

    // TODO: this is just for testing
    pub fn temp_new(selection_set: &'a mut SelectionSet) -> Self {
        QueryBuilder {
            phantom: PhantomData,
            has_typename: false,
            selection_set,
            recurse_depth: None,
        }
    }

    pub fn select_flattened_field<FieldMarker, Flattened, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldMarker, Flattened, Variables>
    where
        FieldMarker: schema::Field,
        FieldType: FlattensInto<Flattened>,
        SchemaType: schema::HasField<FieldMarker, FieldType>,
    {
        FieldSelectionBuilder {
            recurse_depth: self.recurse_depth,
            field: self.push_selection(FieldMarker::name()),
            phantom: PhantomData,
        }
    }

    // TODO: reckon add_field might be better for this?
    // particularly if we name the container selectionset or something.
    // TODO: also see if we can simplify the type sig similar to the argument
    // field type sig?
    pub fn select_field<FieldMarker, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldMarker, FieldType, Variables>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker, FieldType>,
    {
        FieldSelectionBuilder {
            recurse_depth: self.recurse_depth,
            field: self.push_selection(FieldMarker::name()),
            phantom: PhantomData,
        }
    }

    pub fn recurse<FieldMarker, FieldType>(
        &'_ mut self,
        max_depth: u8,
    ) -> Option<FieldSelectionBuilder<'_, FieldMarker, FieldType, Variables>>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker, FieldMarker::SchemaType>,
        FieldType: Recursable<FieldMarker::SchemaType>,
    {
        let new_depth = self.recurse_depth.map(|d| d + 1).unwrap_or(0);
        if new_depth >= max_depth {
            return None;
        }

        Some(FieldSelectionBuilder {
            recurse_depth: Some(new_depth),
            field: self.push_selection(FieldMarker::name()),
            phantom: PhantomData,
        })
    }

    fn push_selection(&'_ mut self, name: &'static str) -> &mut FieldSelection {
        self.selection_set
            .selections
            .push(Selection::Field(FieldSelection::new(name)));

        match self.selection_set.selections.last_mut() {
            Some(Selection::Field(field_selection)) => field_selection,
            _ => panic!("This should not be possible"),
        }
    }

    pub fn inline_fragment(&'_ mut self) -> InlineFragmentBuilder<'_, SchemaType, Variables> {
        if !self.has_typename {
            self.selection_set
                .selections
                .push(Selection::Field(FieldSelection::new("__typename")));
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

// TODO: do we even need SchemaType here or can we do a lookup through Field
pub struct FieldSelectionBuilder<'a, Field, SchemaType, Variables> {
    phantom: PhantomData<fn() -> (Field, SchemaType, Variables)>,
    field: &'a mut FieldSelection,
    recurse_depth: Option<u8>,
}

impl<'a, Field, FieldSchemaType, Variables>
    FieldSelectionBuilder<'a, Field, FieldSchemaType, Variables>
{
    // TODO: be sure to document (and test) that this supports owned _and_ static
    // strings.
    pub fn alias(&mut self, alias: impl Into<Cow<'static, str>>) {
        self.field.alias = Some(alias.into())
    }

    pub fn argument<ArgumentName>(
        &'_ mut self,
    ) -> ArgumentBuilder<'_, Field::ArgumentSchemaType, Variables>
    where
        Field: schema::HasArgument<ArgumentName>,
    {
        ArgumentBuilder {
            destination: InputLiteralContainer::object(Field::name(), &mut self.field.arguments),
            phantom: PhantomData,
        }
    }

    pub fn select_children<InnerVariables>(
        &'_ mut self,
    ) -> QueryBuilder<'_, FieldSchemaType, InnerVariables>
    where
        Variables: VariableMatch<InnerVariables>,
    {
        QueryBuilder {
            recurse_depth: self.recurse_depth,
            ..QueryBuilder::new(&mut self.field.children)
        }
    }

    // TODO: probably need an alias function here that defines an alias.

    // TODO: Could done be done via drop?  Maybe.
    pub fn done(self) {}
}

pub struct InlineFragmentBuilder<'a, SchemaType, Variables> {
    phantom: PhantomData<fn() -> (SchemaType, Variables)>,
    inline_fragment: &'a mut InlineFragment,
}

impl<'a, SchemaType, Variables> InlineFragmentBuilder<'a, SchemaType, Variables> {
    pub fn on<Subtype>(self) -> InlineFragmentBuilder<'a, Subtype, Variables>
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

    pub fn select_children<InnerVariables>(
        &'_ mut self,
    ) -> QueryBuilder<'_, SchemaType, InnerVariables>
    where
        Variables: VariableMatch<InnerVariables>,
    {
        // static_assertions::assert_impl_one!(InnerVariables: VariableMatch<SchemaType>, IsVoid);
        QueryBuilder::new(&mut self.inline_fragment.children)
    }
}

// TODO: maybe rename this to InputBuilder?
// TODO: Check if we can actually get rid of the ArgumentKind parameter
// here.  And if so see if we can get rid of IsScalar/IsInputObject etc.
pub struct ArgumentBuilder<'a, SchemaType, Variables> {
    destination: InputLiteralContainer<'a>,

    phantom: PhantomData<fn() -> (SchemaType, Variables)>,
}

impl<'a, SchemaType, Variables> ArgumentBuilder<'a, SchemaType, Variables> {
    pub fn variable<Type>(self, def: VariableDefinition<Variables, Type>)
    where
        Type: CoercesTo<SchemaType>,
    {
        self.destination.push(InputLiteral::Variable(def.name));
    }
}

impl<'a, SchemaType, ArgumentStruct> ArgumentBuilder<'a, Option<SchemaType>, ArgumentStruct> {
    pub fn null(self) {
        self.destination.push(InputLiteral::Null);
    }

    // TODO: name this some maybe?
    pub fn value(self) -> ArgumentBuilder<'a, SchemaType, ArgumentStruct> {
        ArgumentBuilder {
            destination: self.destination,
            phantom: PhantomData,
        }
    }

    // TODO: would undefined also be useful?  Not sure.
}

impl<'a, T, ArgStruct> ArgumentBuilder<'a, T, ArgStruct> {
    pub fn literal(self, l: impl serde::Serialize + CoercesTo<T>) {
        self.destination
            .push(to_input_literal(&l).expect("could not convert to InputLiteral"));
    }
}

impl<'a, SchemaType, Variables> ArgumentBuilder<'a, SchemaType, Variables>
where
    SchemaType: schema::InputObjectMarker,
{
    pub fn object(self) -> ObjectArgumentBuilder<'a, SchemaType, Variables> {
        let fields = match self.destination.push(InputLiteral::Object(Vec::new())) {
            InputLiteral::Object(fields) => fields,
            _ => panic!("This should be impossible"),
        };

        ObjectArgumentBuilder {
            fields,
            phantom: PhantomData,
        }
    }
}

pub struct ObjectArgumentBuilder<'a, ItemType, Variables> {
    fields: &'a mut Vec<Argument>,
    phantom: PhantomData<fn() -> (ItemType, Variables)>,
}

impl<'a, SchemaType, ArgStruct> ObjectArgumentBuilder<'a, SchemaType, ArgStruct> {
    pub fn field<FieldMarker, F>(self, field_fn: F) -> Self
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasInputField<FieldMarker, FieldMarker::SchemaType>,
        F: FnOnce(ArgumentBuilder<'_, FieldMarker::SchemaType, ArgStruct>),
    {
        field_fn(ArgumentBuilder {
            destination: InputLiteralContainer::object(FieldMarker::name(), self.fields),
            phantom: PhantomData,
        });

        self
    }
}

impl<'a, SchemaType, Variables> ArgumentBuilder<'a, Vec<SchemaType>, Variables> {
    pub fn list(self) -> ListArgumentBuilder<'a, SchemaType, Variables> {
        let items = match self.destination.push(InputLiteral::List(Vec::new())) {
            InputLiteral::List(items) => items,
            _ => panic!("This should be impossible"),
        };

        ListArgumentBuilder {
            items,
            phantom: PhantomData,
        }
    }
}

pub struct ListArgumentBuilder<'a, ItemType, Variables> {
    items: &'a mut Vec<InputLiteral>,
    phantom: PhantomData<fn() -> (ItemType, Variables)>,
}

impl<'a, ItemType, Variables> ListArgumentBuilder<'a, ItemType, Variables> {
    pub fn item(self, item_fn: impl FnOnce(ArgumentBuilder<'_, ItemType, Variables>)) -> Self {
        item_fn(ArgumentBuilder {
            destination: InputLiteralContainer::list(self.items),
            phantom: PhantomData,
        });

        self
    }
}

enum InputLiteralContainer<'a> {
    Object {
        // The name of the field we're inserting
        argument_name: &'static str,

        // The list to insert into once we're done
        arguments: &'a mut Vec<Argument>,
    },
    List(&'a mut Vec<InputLiteral>),
}

impl<'a> InputLiteralContainer<'a> {
    fn list(list: &'a mut Vec<InputLiteral>) -> Self {
        InputLiteralContainer::List(list)
    }

    fn object(argument_name: &'static str, arguments: &'a mut Vec<Argument>) -> Self {
        InputLiteralContainer::Object {
            argument_name,
            arguments,
        }
    }

    fn push(self, value: InputLiteral) -> &'a mut InputLiteral {
        match self {
            InputLiteralContainer::Object {
                argument_name: name,
                arguments,
            } => {
                arguments.push(Argument::new(name, value));

                &mut arguments.last_mut().unwrap().value
            }
            InputLiteralContainer::List(arguments) => {
                arguments.push(value);

                arguments.last_mut().unwrap()
            }
        }
    }
}

// TODO: Think about the name and location of this trait.
pub trait VariableMatch<T> {}
impl<T> VariableMatch<()> for T where T: crate::QueryVariables {}

// Handle custom scalars somehow.  I can only assume they'll need to support all the literals,
// with no real type checking.

// TODO: Move this somewhere else?
