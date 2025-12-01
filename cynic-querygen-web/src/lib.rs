// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports, clippy::enum_variant_names)]

use seed::prelude::*;

mod entry_page;
mod graphiql_page;
mod view;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model<'static> {
    Model::default()
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
enum Model<'a> {
    EntryPage(entry_page::Model),
    GraphiqlPage(graphiql_page::Model<'a>),
}

impl Default for Model<'_> {
    fn default() -> Model<'static> {
        Model::EntryPage(entry_page::Model::default())
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    EntryPageMsg(entry_page::Msg),
    GraphiqlPageMsg(graphiql_page::Msg),
    SwitchToGraphiqlWithSchema(String),
    SwitchToGraphiqlWithUrl(String),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match (&msg, model) {
        (Msg::GraphiqlPageMsg(msg), Model::GraphiqlPage(page_model)) => {
            graphiql_page::update(
                msg.clone(),
                page_model,
                &mut orders.proxy(Msg::GraphiqlPageMsg),
            );
        }
        (Msg::EntryPageMsg(msg), Model::EntryPage(page_model)) => {
            let action = entry_page::update(
                msg.clone(),
                page_model,
                &mut orders.proxy(Msg::EntryPageMsg),
            );

            match action {
                None => {}
                Some(entry_page::Action::BuildFromSchema(schema)) => {
                    orders.send_msg(Msg::SwitchToGraphiqlWithSchema(schema));
                }
                Some(entry_page::Action::BuildFromUrl(url)) => {
                    orders.send_msg(Msg::SwitchToGraphiqlWithUrl(url));
                }
            }
        }
        (Msg::SwitchToGraphiqlWithSchema(schema), ref mut model) => {
            **model = Model::GraphiqlPage(graphiql_page::Model::new_from_schema(schema.clone()))
        }
        (Msg::SwitchToGraphiqlWithUrl(url), ref mut model) => {
            **model = Model::GraphiqlPage(graphiql_page::Model::new_from_url(url.clone()))
        }
        (_, _) => panic!("Invalid state reached"),
    }
}

// ------ ------
//     View
// ------ ------

// (Remove the line below once your `Model` become more complex.)
#[allow(clippy::trivially_copy_pass_by_ref)]
// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    match model {
        Model::EntryPage(model) => entry_page::view(model).map_msg(Msg::EntryPageMsg),
        Model::GraphiqlPage(model) => graphiql_page::view(model).map_msg(Msg::GraphiqlPageMsg),
    }
}

/*
fn codegen_options() -> Node<Msg> {
    section![
        style! {
            St::Display => St::Flex,
            St::JustifyContent => "center",
            St::AlignContent => "center"
        },
        div![
            style! {
                St::Display => St::Flex,
                St::FlexDirection => "column"
            },
            h5!["Options"],
            div![
                label!("Schema Path"),
                input!(
                    attrs! {
                        At::Type => "text"
                    },
                    input_ev(Ev::Input, Msg::SchemaPathChange)
                ),
            ],
            div![
                label!("Query Module"),
                input!(
                    attrs! {
                        At::Type => "text"
                    },
                    input_ev(Ev::Input, Msg::QueryModuleChange)
                ),
            ]
        ]
    ]
}

fn textareas(generated_code: &str) -> Node<Msg> {
    section![
        style! {
            St::Display => St::Flex,
            St::JustifyContent => "center",
            St::AlignContent => "center"
        },
        input_textarea("Schema", Msg::SchemaChange),
        input_textarea("Query", Msg::QueryChange),
        output_textarea("Output", &generated_code),
    ]
}

fn output_textarea(label: &str, text: &str) -> Node<Msg> {
    div![
        style! {
            St::FlexGrow => "1",
            St::Margin => px(4)
            St::Display => St::Flex,
            St::FlexDirection => "column"
        },
        label![label],
        textarea![
            attrs! {
                At::Rows => 40
                At::Cols => 40
            },
            text
        ],
    ]
}

fn input_textarea<F>(label: &str, msg: F) -> Node<Msg>
where
    F: (Fn(String) -> Msg) + Clone + 'static,
{
    div![
        style! {
            St::FlexGrow => "1",
            St::Margin => px(4)
            St::Display => St::Flex,
            St::FlexDirection => "column"
        },
        label![label],
        textarea![
            attrs! {
                At::Rows => 40
                At::Cols => 40
            },
            input_ev(Ev::Input, msg)
        ],
    ]
}*/

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
