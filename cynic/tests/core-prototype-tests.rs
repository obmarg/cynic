#![allow(dead_code)]

use cynic::core::QueryFragment;
use cynic_proc_macros::QueryFragment2;
use serde::Deserialize;

mod schema {
    cynic::use_schema_2!("tests/test-schema.graphql");
}

mod manual_schema {
    use cynic::core;

    pub struct Query {}

    pub mod query_fields {
        use cynic::core;

        pub struct AllPosts {}

        pub struct Post {}

        impl core::FieldName for AllPosts {
            fn name() -> &'static str {
                "allPosts"
            }
        }

        impl core::FieldName for Post {
            fn name() -> &'static str {
                "post"
            }
        }

        impl core::HasField<AllPosts, Vec<super::BlogPost>> for super::Query {}
        impl core::HasField<Post, Option<super::BlogPost>> for super::Query {}

        pub mod post_arguments {
            use cynic::core;

            pub struct Id {}

            impl core::HasArgument<Id> for super::Post {
                type ArgumentSchemaType = cynic::Id;

                fn name() -> &'static str {
                    "id"
                }
            }
        }
    }

    pub struct BlogPost;
    pub mod blog_post_fields {
        use cynic::core;

        pub struct Author;

        pub struct HasMetadata;

        impl core::FieldName for Author {
            fn name() -> &'static str {
                "author"
            }
        }

        impl core::FieldName for HasMetadata {
            fn name() -> &'static str {
                "hasMetadata"
            }
        }

        // Note: The schema module could also (probably) output HasField impls for
        // the various option types that are also valid for the given type?
        impl core::HasField<Author, super::Author> for super::BlogPost {}

        impl core::HasField<HasMetadata, Option<bool>> for super::BlogPost {}
    }

    // Note: this might not be needed..
    impl core::CompositeFieldType for BlogPost {
        type InnerNamedType = BlogPost;
    }

    pub struct Author;
    pub mod author_fields {
        use cynic::core;

        pub struct Name;

        impl core::FieldName for Name {
            fn name() -> &'static str {
                "name"
            }
        }

        impl core::HasField<Name, Option<String>> for super::Author {}
    }

    // Note: this might not be needed..
    impl core::CompositeFieldType for Author {
        type InnerNamedType = Author;
    }
}

#[derive(cynic::QueryFragment2, Debug)]
#[cynic(
    schema_path = "tests/test-schema.graphql",
    schema_module = "schema",
    graphql_type = "Query"
)]
// #[derive(serde::Deserialize)]
struct MyQuery {
    #[arguments(id = "TODO")]
    post: Option<BlogPostOutput>,
    #[cynic(rename = "allPosts")]
    posts: Vec<BlogPostOutput>,
}

// impl<'de> cynic::core::QueryFragment<'de> for MyQuery {
//     type SchemaType = schema::Query;

//     fn query(mut builder: cynic::core::QueryBuilder<Self::SchemaType>) {
//         let mut post_field_builder =
//             builder.select_field::<schema::query_fields::Post, <Option<BlogPostOutput> as QueryFragment>::SchemaType>();

//         let post_builder = post_field_builder.select_children();

//         <Option<BlogPostOutput> as QueryFragment>::query(post_builder);

//         post_field_builder.done();

//         let mut post_list_field_builder =
//             builder.select_field::<schema::query_fields::AllPosts, <Vec<BlogPostOutput> as QueryFragment>::SchemaType>();

//         let post_list_builder = post_list_field_builder.select_children();

//         <Vec<BlogPostOutput> as QueryFragment>::query(post_list_builder);

//         post_list_field_builder.done()
//     }
// }

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

//     fn query(mut builder: cynic::core::QueryBuilder<Self::SchemaType>) {
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

//     fn query(mut builder: cynic::core::QueryBuilder<Self::SchemaType>) {
//         builder
//             .select_field::<schema::author_fields::Name, Option<String>>()
//             .done();

//         builder.done();
//     }
// }

#[test]
fn test_using_this_shit() {
    use cynic::core::{QueryBuilder, QueryFragment};

    let mut fields = Vec::new();

    let builder: QueryBuilder<schema::Query> = QueryBuilder::temp_new(&mut fields);

    MyQuery::query(builder);

    insta::assert_debug_snapshot!(fields);
}

#[test]
fn test_deserialize() {
    use serde_json::json;

    let data = json!({
        "post": {"hasMetadata": false, "author": {"name": "Not Me"}},
        "allPosts": [{"hasMetadata": true, "author": {"name": "Me"}}]
    });

    insta::assert_debug_snapshot!(serde_json::from_value::<MyQuery>(data).unwrap());
}
