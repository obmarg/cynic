<!--

TODO: Fix up this stuff, it's all wrong/old now.

# Selection Sets

Selection sets are involved in almost everything cynic does.  A selection set
contains a set of fields to fetch as part of a query, along with decoder
functions that can decode the contents of those fields after they've been
fetched.

When you derive a `QueryFragment`, cynic automatically creates a function that
outputs a `SelectionSet` for the fields which you've put into your struct.
That `SelectionSet` can then be added to `SelectionSet` of another
`QueryFragment` if you nest it, or turned straight into a `Query` if you're at
the root of the schema.

### Selecting Scalars

The simplest selection sets in cynic are scalar selections - these simply tell
cynic how to decode a field, but don't give any details about what field to
decode.  There are `string`, `integer`, `float`, `boolean` and more.

For example:

```rust
use cynic::selection_set::{bool, string};

// Will decode as a bool
let select_bool = bool();

// Will decode as a String:
let select_string = string()
```

### Selecting Lists & Optionals

If you're wanting to fetch some optional or list fields, cynic provides the
`option` & `vec` combinators respectively:

```rust
use cynic::selection_set::{string, bool, vec, option};

// Will decode as an Option<String>
let select_optional_string = option(string());

// Will decode as a Vec<bool>
let select_vec_bool = vec(bool());
```

Again, these selection sets don't know what field they're decoding from, only how
to decode a field of a particular type.

### Selecting Fields

On their own, the selection sets above are not very useful - you need to be
able to apply them to a particular field.  That's where the `field` function
comes in:

```rust
use cynic::selection_set::{field, string};

// Selects a string from a "name" field
let select_name_field = field("name", vec![], string());

// Selects an optional integer from an "age" field
let select_age_field = field("age", vec![], option(integer());
```

### Providing Arguments

The `field` function is also how you provide arguments to a GraphQL field - the
second argument is a list of `Argument` structs to provide to the field in the
query.  To pass a parameter of `adults: true` to a field:

```rust
let select_people = field(
	"names",
	vec![Argument::new("adults", "Bool", true)],
	vec(string())
)
```

Note that you need to provide the GraphQL type name of the argument here.

### Selecting Multiple Fields

Selecting an individual field is great but we need to be able to combine these
individual field selections to build up an object.  That's where the `mapN`
functions come in - they allow you to combine a number of field selections into
one, and pass the results of those selections to a function.

For example, to query for the name & age of a `User`:

```rust
struct User {
    name: String,
    age: Option<i32>
}

let select_user = map2(
	|(name, age)| User{ name, age },
	field("name", vec![], string()),
	field("age", vec![], option(integer());
);
```

This will select a String from a "name" field, and an Option<i32> from the
`age` field and pass those into the closure we provided as the first argument.
Since Rust doesn't have variadic functions there's a lot of these mapN
functions, one for each possible number of arguments up to 50.

### Selecting Nested Objects.

The third argument to the `field` function is just a `SelectionSet`, so you can
combine the `field` & `mapN` functions to build up a nested query:

```rust
let select_query = field(
	"query", 
	vec![], 
	field("user", vec![], option(select_user()))
);
```

When this selection set is converted into a query, the GraphQL will look like:

```graphql
query {
  user {
    name
    age
  }
}
```
-->
