#![allow(dead_code, unused_variables, missing_docs)]
// TODO: Don't allow the above

use std::marker::PhantomData;

use crate::{
    core::{self, VariableDefinition},
    schema,
};

use super::{ast::*, IntoInputLiteral};

// TODO: QueryBuilder or SelectionBuilder?
pub struct QueryBuilder<'a, SchemaType> {
    phantom: PhantomData<fn() -> SchemaType>,
    selection_set: &'a mut SelectionSet,
    has_typename: bool,
}

impl<'a, T> QueryBuilder<'a, Vec<T>> {
    pub(crate) fn into_inner(self) -> QueryBuilder<'a, T> {
        QueryBuilder {
            selection_set: self.selection_set,
            has_typename: self.has_typename,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> QueryBuilder<'a, Option<T>> {
    pub(crate) fn into_inner(self) -> QueryBuilder<'a, T> {
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
    // TODO: also see if we can simplify the type sig similar to the argument
    // field type sig?
    pub fn select_field<FieldMarker, FieldType>(
        &'_ mut self,
    ) -> FieldSelectionBuilder<'_, FieldMarker, FieldType>
    where
        FieldMarker: schema::Field,
        SchemaType: schema::HasField<FieldMarker, FieldType>,
    {
        self.selection_set
            .selections
            .push(Selection::Field(FieldSelection::new(FieldMarker::name())));

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
pub struct FieldSelectionBuilder<'a, Field, SchemaType> {
    phantom: PhantomData<fn() -> (Field, SchemaType)>,
    field: &'a mut FieldSelection,
}

impl<'a, Field, FieldSchemaType> FieldSelectionBuilder<'a, Field, FieldSchemaType> {
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

    // TODO: re-instate and implement this.
    // Probably want a schema::ArgumentLiteral<TypeLock> trait?
    // or maybe schema::IntoLiteral or IntoArgumentLiteral
    // Argument literal can be the name of the enum it outputs?
    //
    // pub fn argument<ArgumentName, ValueType>(&mut self, value: ValueType)
    // where
    //     FieldSchemaType: schema::HasArgument<ArgumentName>,
    //     ValueType: schema::InputValue<FieldSchemaType::ArgumentSchemaType>,
    // {
    //     todo!()
    // }

    pub fn argument<ArgumentName, ArgumentStruct>(
        &'_ mut self,
    ) -> ArgumentBuilder<'_, Field::ArgumentSchemaType, Field::ArgumentKind, ArgumentStruct>
    where
        Field: schema::HasArgument<ArgumentName>,
    {
        ArgumentBuilder {
            arguments: &mut self.field.arguments,
            argument_name: Field::name(),
            phantom: PhantomData,
        }
    }

    // Ok, so we need two ways to pass in arguments.
    // 1. Something similar to the above, for passing in rust types.
    // 2. A builder approachfor building up values.

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

// TODO: maybe rename this InputBuilder?
pub struct ArgumentBuilder<'a, SchemaType, ArgumentKind, ArgumentStruct> {
    // TODO: Remove the &'a from this phantomdata once it's actually being used.
    argument_name: &'static str,
    arguments: &'a mut Vec<Argument>,
    phantom: PhantomData<fn() -> (SchemaType, ArgumentKind, ArgumentStruct)>,
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

impl<'a, SchemaType, ArgumentStruct>
    ArgumentBuilder<'a, SchemaType, schema::ScalarArgument, ArgumentStruct>
{
    pub fn variable<Type>(self, def: VariableDefinition<ArgumentStruct, Type>)
    where
        // TODO: Think we need to do the whole unwrapping dance to make this work nice...
        // with auto Option wrapping and what not :sigh:
        Type: schema::IsScalar<SchemaType>,
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

impl<'a, SchemaType, ArgKind, ArgumentStruct>
    ArgumentBuilder<'a, Option<SchemaType>, ArgKind, ArgumentStruct>
{
    pub fn null(self) {
        self.arguments.push(Argument {
            name: self.argument_name,
            value: InputLiteral::Null,
        });
    }

    // TODO: name this some maybe?
    pub fn value(self) -> ArgumentBuilder<'a, SchemaType, ArgKind, ArgumentStruct> {
        ArgumentBuilder {
            argument_name: self.argument_name,
            arguments: self.arguments,
            phantom: PhantomData,
        }
    }

    // TODO: would undefined also be useful?  Not sure.
}

impl<'a, T, K, ArgStruct> ArgumentBuilder<'a, T, K, ArgStruct> {
    pub fn literal(self, l: impl IntoInputLiteral<T>) {
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

impl<'a, SchemaType, Kind, ArgStruct> ArgumentBuilder<'a, SchemaType, Kind, ArgStruct>
where
    SchemaType: schema::InputObjectMarker,
{
    // TODO: is FieldType even neccesary here or can we look up via Field?
    //  I think so - I've certainly tried right here...
    pub fn field<FieldMarker>(
        &'_ mut self,
    ) -> ArgumentBuilder<'_, FieldMarker::SchemaType, SchemaType::ArgumentKind, ArgStruct>
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

impl<'a, SchemaType, ArgKind, ArgStruct> ArgumentBuilder<'a, Vec<SchemaType>, ArgKind, ArgStruct> {
    pub fn item<InnerType>(&'_ mut self) -> ArgumentBuilder<'_, InnerType, ArgKind, ArgStruct> {
        // TODO: Think we actually need to return a ListBuilder type for this to work...
        todo!()
    }
}

// Handle custom scalars somehow.  I can only assume they'll need to support all the literals,
// with no real type checking.

// TODO: Move this somewhere else?
