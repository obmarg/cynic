mod value;

// ValueDeserialize vs DeserializeValue
trait ValueDeserialize<'a>: Sized {
    fn deserialize(input: DeserValue) -> Result<Self, Error>;
}

pub enum DeserValue {
    // TODO: Replicate all the options for Value in here
    // Deal with pulling things out of variables later?
    // Deal with an inner ConstValue vs Value later as well...

    // Is this better than serde?  Not sure...
    // Thoughts:
    // - Both would be able to deal with arguments vs value, variables/no
    // - Serde can also borrow
    // - Serde is arguably less work.
    // - Serde might be able to ferry errors in?
    // - Serde data model is _technically_ different but in a way that barely matters...
    // - Serde is more complicated/slower to compile (this one seems fair)
}

pub enum Error {}

// impl ConstValue {
//     fn deserialize<T>(self) -> Result<T>
// }
//
// impl Directive {
//     fn deserialize_arguments<T>(self) -> Result<T>
// }
//
// impl Value<'a> {
//     // Internally this one can convert to whatever DeserInput we use?
//     fn deserizlize<T>(self, arguments: &'a dyn Arguments) -> Result<T>
// }

// Sources:
// Schema:
//  - ConstValue
//  - Directive (via Arguments)
// Executable:
//  - ConstValue (easy)
//  - Directive (via Arguments, involves variables)
//  - Value (involves variables)
//  - FieldSelection (via Arguments, involves variables)
