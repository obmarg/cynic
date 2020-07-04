use seed::{prelude::*, *};

enum Schema {
    Url(String),
    Schema(String),
}

pub struct Model {
    schema: Schema,
    query: String,
    opts: cynic_querygen::QueryGenOptions,
    generated_code: Result<String, cynic_querygen::Error>,
}

impl Model {
    pub fn new_from_url(url: String) -> Self {
        Model {
            schema: Schema::Url(url),
            ..Model::default()
        }
    }

    pub fn new_from_schema(schema: String) -> Self {
        Model {
            schema: Schema::Schema(schema),
            ..Model::default()
        }
    }

    fn generate_code(&mut self) {
        if self.query != "" {
            todo!();
            // TODO: re-implement this bit
            self.generated_code =
                cynic_querygen::document_to_fragment_structs(&self.query, "TODO", &self.opts);
        }
    }
}

impl Default for Model {
    fn default() -> Model {
        Model {
            schema: Schema::Schema("".into()),
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
    SchemaPathChange(String),
    QueryModuleChange(String),
}

// `update` describes how to handle each `Msg`.
pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
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
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    let generated_code = match &model.generated_code {
        Ok(code) => code.clone(),
        Err(e) => e.to_string(),
    };

    // Consider going super minimal like: https://legacy.graphqlbin.com/new

    div![crate::view::header(), gql_editor(&model.schema)]
}

fn gql_editor<Msg>(schema: &Schema) -> Node<Msg> {
    let schema_url = match schema {
        Schema::Url(url) => url,
        Schema::Schema(_) => todo!(),
    };

    div![
        C!["columns"],
        style![
            St::Height => "80vh",
        ],
        custom![
            C!["column", "is-full"],
            attrs! {
                "schema-url" => schema_url
            },
            Tag::from("gql-editor")
        ]
    ]
}
