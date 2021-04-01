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
        if !self.query.is_empty() && self.schema_data.is_some() {
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
}

// `update` describes how to handle each `Msg`.
pub fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
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
            style! {
                "height" => "100vh",
                "display" => "flex",
                "flex-direction" => "column"
            },
            crate::view::header(),
            gql_editor(schema_url, None, &generated_code)
        ]
    } else {
        div![
            style! {
                "height" => "100vh",
                "display" => "flex",
                "flex-direction" => "column"
            },
            crate::view::header(),
            gql_editor("", model.schema_data.as_deref(), &generated_code)
        ]
    }
}

fn gql_editor(schema_url: &str, schema: Option<&str>, generated_code: &str) -> Node<Msg> {
    div![
        C!["columns"],
        style![
            St::Height => "100%",
            "flex-grow" => 1
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
                "schema" => schema.unwrap_or("").replace("\n", "&NL;")
                "generated-code" => generated_code.replace("\n", "&NL;")
            },
            Tag::from("gql-editor"),
        ]
    ]
}
