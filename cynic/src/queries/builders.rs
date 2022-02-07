#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::{borrow::Cow, marker::PhantomData};

use crate::{
    coercions::CoercesTo,
    core::{self, VariableDefinition, VariableType},
    schema,
};

use super::{ast::*, FlattensInto, IntoInputLiteral};

// TODO: QueryBuilder or SelectionBuilder?
pub struct QueryBuilder<'a, SchemaType, Variables> {
    phantom: PhantomData<fn() -> (SchemaType, Variables)>,
    selection_set: &'a mut SelectionSet,
    has_typename: bool,
}

impl<'a, T, U> QueryBuilder<'a, Vec<T>, U> {
    pub(crate) fn into_inner(self) -> QueryBuilder<'a, T, U> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
        }
    }
}

impl<'a, T, U> QueryBuilder<'a, Option<T>, U> {
    pub(crate) fn into_inner(self) -> QueryBuilder<'a, T, U> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
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

    pub fn select_flattened_field<FieldMarker, Flattened, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldMarker, Flattened, Variables>
    where
        FieldMarker: schema::Field,
        FieldType: FlattensInto<Flattened>,
        SchemaType: schema::HasField<FieldMarker, FieldType>,
    {
        FieldSelectionBuilder {
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
            field: self.push_selection(FieldMarker::name()),
            phantom: PhantomData,
        }
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
            arguments: &mut self.field.arguments,
            argument_name: Field::name(),
            phantom: PhantomData,
        }
    }

    pub fn select_children<InnerVariables>(
        &'_ mut self,
    ) -> QueryBuilder<'_, FieldSchemaType, InnerVariables>
    where
        Variables: VariableMatch<InnerVariables>,
    {
        QueryBuilder::new(&mut self.field.children)
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

pub struct ArgumentBuilder<'a, SchemaType, Variables> {
    // TODO: Remove the &'a from this phantomdata once it's actually being used.
    argument_name: &'static str,
    arguments: &'a mut Vec<Argument>,
    phantom: PhantomData<fn() -> (SchemaType, Variables)>,
}

/*

TODO: Think about these two...

impl<'a, SchemaType> ArgumentBuilder<'a, SchemaType> {
    fn scalar_literal<T>(self, val: T)
    where
        T: schema::IsScalar<SchemaType>,
    {
        todo!()
    }

    fn enum_literal<T>(self, val: T)
    where
        T: schema::IsEnum<SchemaType>,
    {
        todo!()
    }
}
*/

impl<'a, SchemaType, Variables> ArgumentBuilder<'a, SchemaType, Variables> {
    pub fn variable<Type>(self, def: VariableDefinition<Variables, Type>)
    where
        Type: CoercesTo<SchemaType>,
    {
        self.arguments.push(Argument {
            name: self.argument_name,
            value: InputLiteral::Variable(def.name),
        });
    }
}

// TODO: reinstate & correct these two

// impl<'a, SchemaType> ArgumentBuilder<'a, SchemaType, schema::EnumArgument> {
//     pub fn variable<VariableDef>(self, def: VariableDef)
//     where
//         // TODO: presumably need to constrain on ArgumentStruct somehow.
//         VariableDef: core::Variable,
//         VariableDef::Type: schema::IsEnum<SchemaType>,
//     {
//         self.arguments.push(Argument {
//             name: self.argument_name,
//             value: InputLiteral::Variable(VariableDef::name()),
//         });
//     }
// }

// impl<'a, SchemaType> ArgumentBuilder<'a, SchemaType, schema::InputObjectArgument> {
//     pub fn variable<VariableDef>(self, def: VariableDef)
//     where
//         // TODO: presumably need to constrain on ArgumentStruct somehow.
//         VariableDef: core::Variable,
//         VariableDef::Type: schema::IsInputObject<SchemaType>,
//     {
//         self.arguments.push(Argument {
//             name: self.argument_name,
//             value: InputLiteral::Variable(VariableDef::name()),
//         });
//     }
// }

impl<'a, SchemaType, ArgumentStruct> ArgumentBuilder<'a, Option<SchemaType>, ArgumentStruct> {
    pub fn null(self) {
        self.arguments.push(Argument {
            name: self.argument_name,
            value: InputLiteral::Null,
        });
    }

    // TODO: name this some maybe?
    pub fn value(self) -> ArgumentBuilder<'a, SchemaType, ArgumentStruct> {
        ArgumentBuilder {
            argument_name: self.argument_name,
            arguments: self.arguments,
            phantom: PhantomData,
        }
    }

    // TODO: would undefined also be useful?  Not sure.
}

impl<'a, T, ArgStruct> ArgumentBuilder<'a, T, ArgStruct> {
    pub fn literal(self, l: impl IntoInputLiteral + CoercesTo<T>) {
        self.arguments.push(Argument {
            name: self.argument_name,
            value: l.into_literal(),
        })
    }
}

// TODO: ArgumentBuilder for options, enums, scalars...

// impl<'a> ArgumentBuilder<'a, i32> {
//     pub fn literal(self, i: i32) {
//         self.arguments.push(Argument {
//             name: self.argument_name,
//             value: InputLiteral::Int(i),
//         });
//     }
// }

// impl<'a> ArgumentBuilder<'a, bool> {
//     pub fn literal(self, i: bool) {
//         self.arguments.push(Argument {
//             name: self.argument_name,
//             value: InputLiteral::Bool(i),
//         });
//     }
// }

// impl<'a> ArgumentBuilder<'a, crate::Id> {
//     // TODO: Could this take an `impl Into` or similar?
//     // Or maybe the entire ArgumentBuilder just takes
//     // an `impl IntoLiteral<TypeMarker>`
//     pub fn literal(self, i: crate::Id) {
//         self.arguments.push(Argument {
//             name: self.argument_name,
//             value: InputLiteral::Id(i.into_inner()),
//         });
//     }
// }

impl<'a, SchemaType, ArgStruct> ArgumentBuilder<'a, SchemaType, ArgStruct>
where
    SchemaType: schema::InputObjectMarker,
{
    // TODO: is FieldType even neccesary here or can we look up via Field?
    //  I think so - I've certainly tried right here...
    pub fn field<FieldMarker>(
        &'_ mut self,
    ) -> ArgumentBuilder<'_, FieldMarker::SchemaType, ArgStruct>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasInputField<FieldMarker, FieldMarker::SchemaType>,
    {
        self.arguments.push(Argument {
            name: self.argument_name,
            value: InputLiteral::Object(Vec::new()),
        });

        let arguments = match &mut self.arguments.last_mut().unwrap().value {
            InputLiteral::Object(arguments) => arguments,
            _ => panic!("This should be impossible"),
        };

        ArgumentBuilder {
            argument_name: FieldMarker::name(),
            arguments,
            phantom: PhantomData,
        }
    }
}

impl<'a, SchemaType, ArgStruct> ArgumentBuilder<'a, Vec<SchemaType>, ArgStruct> {
    pub fn item<InnerType>(&'_ mut self) -> ArgumentBuilder<'_, InnerType, ArgStruct> {
        // TODO: Think we actually need to return a ListBuilder type for this to work...
        todo!()
    }
}

// TODO: Think about the name and location of this trait.
pub trait VariableMatch<T> {}
impl<T> VariableMatch<()> for T where T: crate::core::QueryVariables {}

// Handle custom scalars somehow.  I can only assume they'll need to support all the literals,
// with no real type checking.

// TODO: Move this somewhere else?
