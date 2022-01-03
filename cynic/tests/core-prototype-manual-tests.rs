#![allow(dead_code)]

use cynic::{core::QueryFragment, queries::SelectionSet};
use cynic_proc_macros::QueryFragment2;
use serde::Deserialize;

mod schema {
    use cynic::core;

    pub struct Query {}

    pub mod query_fields {
        use cynic::core;

        pub struct AllPosts {}

        pub struct Post {}

        pub struct FilteredPosts {}

        impl cynic::schema::Field for AllPosts {
            type SchemaType = Vec<super::BlogPost>;

            fn name() -> &'static str {
                "allPosts"
            }
        }

        impl cynic::schema::Field for Post {
            type SchemaType = Option<super::BlogPost>;

            fn name() -> &'static str {
                "post"
            }
        }

        impl cynic::schema::Field for FilteredPosts {
            type SchemaType = Vec<super::BlogPost>;

            fn name() -> &'static str {
                "filteredPosts"
            }
        }

        impl cynic::schema::HasField<AllPosts, Vec<super::BlogPost>> for super::Query {}
        impl cynic::schema::HasField<Post, Option<super::BlogPost>> for super::Query {}
        impl cynic::schema::HasField<FilteredPosts, Vec<super::BlogPost>> for super::Query {}

        pub mod post_arguments {
            use cynic::core;

            pub struct Id {}

            impl cynic::schema::HasArgument<Id> for super::Post {
                type ArgumentSchemaType = cynic::Id;

                fn name() -> &'static str {
                    "id"
                }
            }
        }

        pub mod filtered_post_arguments {
            use cynic::core;

            pub struct Filters {}

            impl cynic::schema::HasArgument<Filters> for super::FilteredPosts {
                type ArgumentSchemaType = Option<super::super::PostFilters>;

                fn name() -> &'static str {
                    "filters"
                }
            }
        }
    }

    pub struct PostFilters {}

    impl cynic::schema::InputObjectMarker for PostFilters {}

    pub mod post_filters_fields {
        use cynic::core;

        pub struct AuthorId;

        impl cynic::schema::Field for AuthorId {
            type SchemaType = Option<cynic::Id>;

            fn name() -> &'static str {
                "author"
            }
        }

        impl cynic::schema::HasField<AuthorId, Option<cynic::Id>> for super::PostFilters {}
    }

    pub struct BlogPost;
    pub mod blog_post_fields {
        use cynic::core;

        pub struct Author;

        pub struct HasMetadata;

        impl cynic::schema::Field for Author {
            type SchemaType = Author;
            fn name() -> &'static str {
                "author"
            }
        }

        impl cynic::schema::Field for HasMetadata {
            type SchemaType = Option<bool>;

            fn name() -> &'static str {
                "hasMetadata"
            }
        }

        // Note: The schema module could also (probably) output HasField impls for
        // the various option types that are also valid for the given type?
        impl cynic::schema::HasField<Author, super::Author> for super::BlogPost {}

        impl cynic::schema::HasField<HasMetadata, Option<bool>> for super::BlogPost {}
    }

    pub struct Author;
    pub mod author_fields {
        use cynic::core;

        pub struct Name;

        impl cynic::schema::Field for Name {
            type SchemaType = Option<String>;

            fn name() -> &'static str {
                "name"
            }
        }

        impl cynic::schema::HasField<Name, Option<String>> for super::Author {}
    }
}

struct MyArguments {
    author_id: Option<cynic::Id>,
}

trait ArgStruct {
    type Mirror;
}

impl ArgStruct for MyArguments {
    type Mirror = MyArgumentsMirror;
}

struct MyArgumentsMirror {
    author_id: AuthorIdMirror,
}

struct AuthorIdMirror;

impl cynic::core::Variable for AuthorIdMirror {
    type ArgumentStruct = MyArguments;

    // TODO: Figuring this out might require more traits.
    // Although I can't just do a straight trait lookup because
    // a given type can have > 1 schema type.
    // So this is surprisingly tricky.  (Glad i'm typing this out)
    // _could_ have this as a paramter to the derive, but then
    // users need to understand the schema module.  not ideal.
    // Maybe each schema module has a TypeLookup<X> trait?
    // And each derive generates an impl that this can then use?
    // needs some thought for sure...
    type SchemaType = ();

    fn name() -> &'static str {
        "authorId"
    }
}

#[derive(serde::Deserialize, Debug)]
struct MyQuery {
    post: Option<BlogPostOutput>,
    posts: Vec<BlogPostOutput>,
}

impl<'de> cynic::core::QueryFragment<'de> for MyQuery {
    type SchemaType = schema::Query;

    fn query(mut builder: cynic::queries::QueryBuilder<Self::SchemaType>) {
        let mut post_field_builder =
            builder.select_field::<schema::query_fields::Post, <Option<BlogPostOutput> as QueryFragment>::SchemaType>();

        let post_builder = post_field_builder.select_children();

        <Option<BlogPostOutput> as QueryFragment>::query(post_builder);

        let arg_builder = post_field_builder.argument::<schema::query_fields::post_arguments::Id>();
        arg_builder.literal(cynic::Id::new("abcd"));

        post_field_builder.done();

        let mut filtered_post_field_builder =
            builder.select_field::<
                schema::query_fields::FilteredPosts,
                <Vec<BlogPostOutput> as QueryFragment>::SchemaType,
            >();

        <Vec<BlogPostOutput> as QueryFragment>::query(
            filtered_post_field_builder.select_children(),
        );

        let arg_builder = filtered_post_field_builder
            .argument::<schema::query_fields::filtered_post_arguments::Filters>();
        arg_builder
            .value()
            .field::<schema::post_filters_fields::AuthorId>()
            .value()
            .literal(cynic::Id::new("abcd"));

        let mut post_list_field_builder =
            builder.select_field::<schema::query_fields::AllPosts, <Vec<BlogPostOutput> as QueryFragment>::SchemaType>();

        let post_list_builder = post_list_field_builder.select_children();

        <Vec<BlogPostOutput> as QueryFragment>::query(post_list_builder);

        post_list_field_builder.done()
    }
}

#[derive(cynic::QueryFragment2, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    schema_module = "schema",
    graphql_type = "BlogPost"
)]
struct BlogPostOutput {
    has_metadata: Option<bool>,
    author: AuthorOutput,
}

// impl<'de> cynic::core::QueryFragment<'de> for BlogPostOutput {
//     type SchemaType = schema::BlogPost;

//     fn query(mut builder: cynic::queries::QueryBuilder<Self::SchemaType>) {
//         builder
//             .select_field::<schema::blog_post_fields::HasMetadata, <Option<bool> as QueryFragment>::SchemaType>()
//             .done();

//         let mut author_field_builder = builder
//             .select_field::<schema::blog_post_fields::Author, <AuthorOutput as QueryFragment>::SchemaType>();

//         let author_builder = author_field_builder.select_children();

//         AuthorOutput::query(author_builder);

//         author_field_builder.done();
//         builder.done();
//     }
// }

#[derive(cynic::QueryFragment2, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    schema_module = "schema",
    graphql_type = "Author"
)]
struct AuthorOutput {
    name: Option<String>,
}

// impl<'de> cynic::core::QueryFragment<'de> for AuthorOutput {
//     type SchemaType = schema::Author;

//     fn query(mut builder: cynic::queries::QueryBuilder<Self::SchemaType>) {
//         builder
//             .select_field::<schema::author_fields::Name, Option<String>>()
//             .done();

//         builder.done();
//     }
// }

#[test]
fn test_using_this_shit() {
    use cynic::{core::QueryFragment, queries::QueryBuilder};

    let mut selections = SelectionSet::default();

    let builder: QueryBuilder<schema::Query> = QueryBuilder::temp_new(&mut selections);

    MyQuery::query(builder);

    insta::assert_debug_snapshot!(selections);
}

#[test]
fn test_deserialize() {
    use serde_json::json;

    // TODO: this needs a big update.
    let data = json!({
        "post": {"hasMetadata": false, "author": {"name": "Not Me"}},
        "allPosts": [{"hasMetadata": true, "author": {"name": "Me"}}]
    });

    insta::assert_debug_snapshot!(serde_json::from_value::<MyQuery>(data).unwrap());
}
