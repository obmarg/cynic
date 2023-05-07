use std::{borrow::Cow, marker::PhantomData};

use crate::{coercions::CoercesTo, schema, variables::VariableDefinition};

use super::{ast::*, to_input_literal, FlattensInto, IsFieldType, Recursable};

/// Builds a SelectionSet for the given `SchemaType` and `VariablesFields`
pub struct SelectionBuilder<'a, SchemaType, VariablesFields> {
    phantom: PhantomData<fn() -> (SchemaType, VariablesFields)>,
    selection_set: &'a mut SelectionSet,
    has_typename: bool,
    recurse_depth: Option<u8>,
}

impl<'a, T, U> SelectionBuilder<'a, Vec<T>, U> {
    pub(crate) fn into_inner(self) -> SelectionBuilder<'a, T, U> {
        SelectionBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
            recurse_depth: self.recurse_depth,
        }
    }
}

impl<'a, T, U> SelectionBuilder<'a, Option<T>, U> {
    pub(crate) fn into_inner(self) -> SelectionBuilder<'a, T, U> {
        SelectionBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
            recurse_depth: self.recurse_depth,
        }
    }
}

impl<'a, SchemaType, VariablesFields> SelectionBuilder<'a, SchemaType, VariablesFields> {
    pub(crate) fn new(selection_set: &'a mut SelectionSet) -> Self {
        SelectionBuilder {
            phantom: PhantomData,
            has_typename: false,
            selection_set,
            recurse_depth: None,
        }
    }

    /// Selects a field applying the flattening rules to its type.
    ///
    /// Note that the deserialization for this query must also
    /// implement these rules if calling this function.
    pub fn select_flattened_field<FieldMarker, Flattened, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldMarker, Flattened, VariablesFields>
    where
        FieldMarker: schema::Field,
        FieldType: FlattensInto<Flattened>,
        SchemaType: schema::HasField<FieldMarker>,
        FieldType: IsFieldType<SchemaType::Type>,
    {
        FieldSelectionBuilder {
            recurse_depth: self.recurse_depth,
            field: self.push_selection(FieldMarker::NAME),
            phantom: PhantomData,
        }
    }

    /// Adds the `FieldMarker` field into this selection, with the given
    /// `FieldType`.  Will type error if the field is not applicable or is
    /// not of this type.
    ///
    /// This returns a `FieldSelectionBuilder` that can be used to apply
    /// arguments, aliases, and to build an inner selection.
    pub fn select_field<FieldMarker, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldMarker, FieldType, VariablesFields>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker>,
        FieldType: IsFieldType<SchemaType::Type>,
    {
        FieldSelectionBuilder {
            recurse_depth: self.recurse_depth,
            field: self.push_selection(FieldMarker::NAME),
            phantom: PhantomData,
        }
    }

    /// Recursively selects a field into this selection, with the given
    /// `FieldMarker` and `FieldType`, up to the given `max_depth`.
    ///
    /// This will return a `None` if we have reached the max_depth.
    pub fn recurse<FieldMarker, FieldType>(
        &'_ mut self,
        max_depth: u8,
    ) -> Option<FieldSelectionBuilder<'_, FieldMarker, FieldType, VariablesFields>>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker>,
        FieldType: Recursable<FieldMarker::Type>,
    {
        let new_depth = self.recurse_depth.map(|d| d + 1).unwrap_or(0);
        if new_depth >= max_depth {
            return None;
        }

        Some(FieldSelectionBuilder {
            recurse_depth: Some(new_depth),
            field: self.push_selection(FieldMarker::NAME),
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

    /// Adds an inline fragment to the SelectionSet
    pub fn inline_fragment(&'_ mut self) -> InlineFragmentBuilder<'_, SchemaType, VariablesFields> {
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
}

/// Builds the selection of a field
pub struct FieldSelectionBuilder<'a, Field, SchemaType, VariablesFields> {
    #[allow(clippy::type_complexity)]
    phantom: PhantomData<fn() -> (Field, SchemaType, VariablesFields)>,
    field: &'a mut FieldSelection,
    recurse_depth: Option<u8>,
}

impl<'a, Field, FieldSchemaType, VariablesFields>
    FieldSelectionBuilder<'a, Field, FieldSchemaType, VariablesFields>
{
    /// Adds an alias to this field.
    ///
    /// Should accept static strs or owned Stringsk
    pub fn alias(&mut self, alias: impl Into<Cow<'static, str>>) {
        self.field.alias = Some(alias.into())
    }

    /// Adds an argument to this field.
    ///
    /// Accepts `ArgumentName` - the schema marker struct for the argument you
    /// wish to add.
    pub fn argument<ArgumentName>(
        &'_ mut self,
    ) -> InputBuilder<'_, Field::ArgumentType, VariablesFields>
    where
        Field: schema::HasArgument<ArgumentName>,
    {
        InputBuilder {
            destination: InputLiteralContainer::object(Field::NAME, &mut self.field.arguments),
            phantom: PhantomData,
        }
    }

    /// Returns a SelectionBuilder that can be used to select fields
    /// within this field.
    pub fn select_children<InnerVariables>(
        &'_ mut self,
    ) -> SelectionBuilder<'_, FieldSchemaType, InnerVariables>
    where
        VariablesFields: VariableMatch<InnerVariables>,
    {
        // Ok, so theoretically this could be the point that does fragment
        // magic?
        //
        // Or we could be explicit about it and move it to the QueryFragment
        // HMM?
        //
        // Or a hybrid approach?  Add a way to name particular selection
        // sets to the builder, _and then_ add an optimisation phase
        // that detects identical selection sets and extracts them as
        // fragments?
        SelectionBuilder {
            recurse_depth: self.recurse_depth,
            ..SelectionBuilder::new(&mut self.field.children)
        }
    }
}

/// Builds an inline fragment in a selection
pub struct InlineFragmentBuilder<'a, SchemaType, VariablesFields> {
    phantom: PhantomData<fn() -> (SchemaType, VariablesFields)>,
    inline_fragment: &'a mut InlineFragment,
}

impl<'a, SchemaType, VariablesFields> InlineFragmentBuilder<'a, SchemaType, VariablesFields> {
    /// Adds an on clause for the given `Subtype` to the inline fragment.
    ///
    /// `Subtype` should be the schema marker type for the type you wish this
    /// fragment to match.
    pub fn on<Subtype>(self) -> InlineFragmentBuilder<'a, Subtype, VariablesFields>
    where
        Subtype: crate::schema::NamedType,
        SchemaType: crate::schema::HasSubtype<Subtype>,
    {
        self.inline_fragment.on_clause = Some(Subtype::NAME);
        InlineFragmentBuilder {
            inline_fragment: self.inline_fragment,
            phantom: PhantomData,
        }
    }

    /// Returns a SelectionBuilder that can be used to select the fields
    /// of this fragment.
    pub fn select_children<InnerVariablesFields>(
        &'_ mut self,
    ) -> SelectionBuilder<'_, SchemaType, InnerVariablesFields>
    where
        VariablesFields: VariableMatch<InnerVariablesFields>,
    {
        SelectionBuilder::new(&mut self.inline_fragment.children)
    }
}

pub struct InputBuilder<'a, SchemaType, VariablesFields> {
    destination: InputLiteralContainer<'a>,

    phantom: PhantomData<fn() -> (SchemaType, VariablesFields)>,
}

impl<'a, SchemaType, VariablesFields> InputBuilder<'a, SchemaType, VariablesFields> {
    /// Puts a variable into the input.
    pub fn variable<Type>(self, def: VariableDefinition<VariablesFields, Type>)
    where
        Type: CoercesTo<SchemaType>,
    {
        self.destination.push(InputLiteral::Variable(def.name));
    }
}

impl<'a, SchemaType, ArgumentStruct> InputBuilder<'a, Option<SchemaType>, ArgumentStruct> {
    /// Puts null into the input.
    pub fn null(self) {
        self.destination.push(InputLiteral::Null);
    }

    /// Returns a builder that can put input into a nullable input position.
    pub fn value(self) -> InputBuilder<'a, SchemaType, ArgumentStruct> {
        InputBuilder {
            destination: self.destination,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, ArgStruct> InputBuilder<'a, T, ArgStruct> {
    /// Puts a literal input type into the input.
    pub fn literal(self, l: impl serde::Serialize + CoercesTo<T>) {
        self.destination
            .push(to_input_literal(&l).expect("could not convert to InputLiteral"));
    }
}

impl<'a, SchemaType, VariablesFields> InputBuilder<'a, SchemaType, VariablesFields>
where
    SchemaType: schema::InputObjectMarker,
{
    /// Puts an object literal into the input
    pub fn object(self) -> ObjectArgumentBuilder<'a, SchemaType, VariablesFields> {
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

/// Builds an object literal into some input.
pub struct ObjectArgumentBuilder<'a, ItemType, VariablesFields> {
    fields: &'a mut Vec<Argument>,
    phantom: PhantomData<fn() -> (ItemType, VariablesFields)>,
}

impl<'a, SchemaType, ArgStruct> ObjectArgumentBuilder<'a, SchemaType, ArgStruct> {
    /// Adds a field to the object literal, using the field_fn to determine the
    /// contents of that field.
    pub fn field<FieldMarker, F>(self, field_fn: F) -> Self
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasInputField<FieldMarker, FieldMarker::Type>,
        F: FnOnce(InputBuilder<'_, FieldMarker::Type, ArgStruct>),
    {
        field_fn(InputBuilder {
            destination: InputLiteralContainer::object(FieldMarker::NAME, self.fields),
            phantom: PhantomData,
        });

        self
    }
}

impl<'a, SchemaType, VariablesFields> InputBuilder<'a, Vec<SchemaType>, VariablesFields> {
    /// Adds a list literal into some input
    pub fn list(self) -> ListArgumentBuilder<'a, SchemaType, VariablesFields> {
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

pub struct ListArgumentBuilder<'a, ItemType, VariablesFields> {
    items: &'a mut Vec<InputLiteral>,
    phantom: PhantomData<fn() -> (ItemType, VariablesFields)>,
}

impl<'a, ItemType, VariablesFields> ListArgumentBuilder<'a, ItemType, VariablesFields> {
    /// Adds an item to the list literal, using the item_fn to determine the
    /// contents of that item.
    pub fn item(self, item_fn: impl FnOnce(InputBuilder<'_, ItemType, VariablesFields>)) -> Self {
        item_fn(InputBuilder {
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

/// Enforces type equality on a VariablesFields struct.
///
/// Each `crate::QueryVariablesFields` implementation should also implement this
/// for `()` for compatibility with QueryFragments that don't need variables.
pub trait VariableMatch<T> {}

impl<T> VariableMatch<()> for T where T: crate::QueryVariablesFields {}
