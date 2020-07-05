use seed::{prelude::*, *};

pub struct Model {
    schema_url: Option<String>,
    schema_data: Option<String>,
    query: String,
    opts: cynic_querygen::QueryGenOptions,
    generated_code: Result<String, cynic_querygen::Error>,
}

impl Model {
    pub fn new_from_url(url: String) -> Self {
        Model {
            schema_url: Some(url),
            ..Model::default()
        }
    }

    pub fn new_from_schema(schema: String) -> Self {
        Model {
            schema_data: Some(schema),
            ..Model::default()
        }
    }

    fn generate_code(&mut self) {
        if self.query != "" && self.schema_data.is_some() {
            self.generated_code = cynic_querygen::document_to_fragment_structs(
                &self.query,
                self.schema_data.as_ref().unwrap(),
                &self.opts,
            );
        }
    }
}

impl Default for Model {
    fn default() -> Model {
        Model {
            schema_url: None,
            schema_data: None,
            query: "".into(),
            opts: Default::default(),
            generated_code: Ok("".into()),
        }
    }
}

#[derive(Clone)]
// `Msg` describes the different events you can modify state with.
pub enum Msg {
    QueryChange(String),
    SchemaLoaded(String),
    SchemaPathChange(String),
    QueryModuleChange(String),
}

// `update` describes how to handle each `Msg`.
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::QueryChange(query) => {
            seed::log!("Got query: {}", &query);
            model.query = query;
            model.generate_code();
        }
        Msg::SchemaLoaded(schema) => {
            seed::log!("Got schema: {}", &schema);
            model.schema_data = Some(schema);
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
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let generated_code = match &model.generated_code {
        Ok(code) => code.clone(),
        Err(e) => e.to_string(),
    };

    // Consider going super minimal like: https://legacy.graphqlbin.com/new

    if let Some(schema_url) = &model.schema_url {
        div![
            crate::view::header(),
            gql_editor(schema_url, &generated_code)
        ]
    } else {
        div!["TODO"]
    }
}

fn gql_editor(schema_url: &str, generated_code: &str) -> Node<Msg> {
    div![
        C!["columns"],
        style![
            St::Height => "80vh",
        ],
        custom![
            C!["column", "is-full"],
            ev(Ev::from("schema-loaded"), |event| {
                let custom_event: web_sys::CustomEvent = event.unchecked_into();
                custom_event.detail().as_string().map(Msg::SchemaLoaded)
            }),
            ev(Ev::Change, |event| {
                let custom_event: web_sys::CustomEvent = event.unchecked_into();
                custom_event.detail().as_string().map(Msg::QueryChange)
            }),
            attrs! {
                "schema-url" => schema_url
                "generated-code" => generated_code
            },
            Tag::from("gql-editor"),
        ]
    ]
}
