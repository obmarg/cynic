// (Lines like the one below ignore selected Clippy rules
//  - it's useful when you want to check your code with `cargo make verify`
// but some rules are too "annoying" or are not applicable for your case.)
#![allow(clippy::wildcard_imports)]

use seed::{prelude::*, *};

mod entry_page;
mod view;

// ------ ------
//     Init
// ------ ------

// `init` describes what should happen when your app started.
fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model::default()
}

// ------ ------
//     Model
// ------ ------

// `Model` describes our app state.
struct Model {
    schema: String,
    query: String,
    opts: cynic_querygen::QueryGenOptions,
    generated_code: Result<String, cynic_querygen::Error>,
    entry_page: entry_page::Model,
}

impl Default for Model {
    fn default() -> Model {
        Model {
            schema: "".into(),
            query: "".into(),
            opts: Default::default(),
            generated_code: Ok("".into()),
            entry_page: Default::default(),
        }
    }
}

impl Model {
    fn generate_code(&mut self) {
        if self.query != "" && self.schema != "" {
            self.generated_code =
                cynic_querygen::document_to_fragment_structs(&self.query, &self.schema, &self.opts);
        }
    }
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
// `Msg` describes the different events you can modify state with.
enum Msg {
    SchemaChange(String),
    QueryChange(String),
    SchemaPathChange(String),
    QueryModuleChange(String),
    EntryPageMsg(entry_page::Msg),
}

// `update` describes how to handle each `Msg`.
fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SchemaChange(schema) => {
            model.schema = schema;
            model.generate_code();
        }
        Msg::QueryChange(query) => {
            model.query = query;
            model.generate_code();
        }
        Msg::SchemaPathChange(schema_path) => {
            model.opts.schema_path = schema_path;
            model.generate_code();
        }
        Msg::QueryModuleChange(query_module) => {
            model.opts.query_module = query_module;
            model.generate_code();
        }
        Msg::EntryPageMsg(msg) => {
            let action = entry_page::update(
                msg,
                &mut model.entry_page,
                &mut orders.proxy(Msg::EntryPageMsg),
            );
            match action {
                None => {}
                Some(entry_page::Action::BuildFromSchema(schema)) => {}
                Some(entry_page::Action::BuildFromUrl(url)) => {}
            }
        }
    }
}

// ------ ------
//     View
// ------ ------

// (Remove the line below once your `Model` become more complex.)
#[allow(clippy::trivially_copy_pass_by_ref)]
// `view` describes what to display.
fn view(model: &Model) -> Node<Msg> {
    let generated_code = match &model.generated_code {
        Ok(code) => code.clone(),
        Err(e) => e.to_string(),
    };

    // Consider going super minimal like: https://legacy.graphqlbin.com/new

    div![
        view::header(),
        entry_page::view(&model.entry_page).map_msg(Msg::EntryPageMsg),
        /*
        div![
            style![
                St::Display => St::Flex,
                St::FlexDirection => "column"
            ],
            codegen_options(),
            textareas(&generated_code)
        ]*/
    ]
}

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
}

// ------ ------
//     Start
// ------ ------

// (This function is invoked by `init` function in `index.html`.)
#[wasm_bindgen(start)]
pub fn start() {
    // Mount the `app` to the element with the `id` "app".
    App::start("app", init, update, view);
}
