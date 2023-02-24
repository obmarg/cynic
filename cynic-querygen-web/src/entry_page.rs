use seed::{prelude::*, *};

pub enum Model {
    New,
    Url(String),
    Pasted(String),
}

impl Default for Model {
    fn default() -> Self {
        Model::New
    }
}

#[derive(Clone)]
pub enum Msg {
    UrlClicked,
    PasteClicked,
    TextUpdated(String),
    BuildQuery,
}

/// Actions we can tell the parent module to take.
pub enum Action {
    BuildFromUrl(String),
    BuildFromSchema(String),
}

pub fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) -> Option<Action> {
    match (msg, &model) {
        (Msg::UrlClicked, _) => {
            *model = Model::Url("".into());
            None
        }
        (Msg::PasteClicked, _) => {
            *model = Model::Pasted("".into());
            None
        }
        (_, Model::New) => {
            // We only care about the above events when we're new...
            None
        }
        (Msg::TextUpdated(s), Model::Url(_)) => {
            *model = Model::Url(s);
            None
        }
        (Msg::TextUpdated(s), Model::Pasted(_)) => {
            *model = Model::Pasted(s);
            None
        }
        (Msg::BuildQuery, Model::Url(data)) => Some(Action::BuildFromUrl(data.clone())),
        (Msg::BuildQuery, Model::Pasted(data)) => Some(Action::BuildFromSchema(data.clone())),
    }
}

pub fn view(model: &Model) -> Node<Msg> {
    div![crate::view::header(), entry_page_view(model)]
}

fn entry_page_view(model: &Model) -> Node<Msg> {
    let schema_selector = match model {
        Model::New => div![],
        Model::Url(_) => view_url_entry(),
        Model::Pasted(_) => view_paste_entry(),
    };

    section![
        C!["section"],
        div![
            C!["container", "has-text-centered"],
            div![
                C!["is-flex" "is-justify-content-center" "mb-6"],
                div![
                    C!["notification is-warning is-light"],
                    "⚠️ This is the generator for v3 of cynic.",
                    ul![
                        li![
                            "For v1 please visit ",
                            a![
                                attrs! {At::Href => "https://v1.generator.cynic-rs.dev/"},
                                "https://v1.generator.cynic-rs.dev/"
                            ]
                        ],
                        li![
                            "For v2 please visit ",
                            a![
                                attrs! {At::Href => "https://v2.generator.cynic-rs.dev/"},
                                "https://v2.generator.cynic-rs.dev/"
                            ]
                        ]
                    ]
                ]
            ],
            p![h4![
                C!["subtitle", "is-4"],
                "Where should we get your GraphQL schema?"
            ]],
            div![
                C!["is-centered", "mt-4"],
                button![
                    C!["button", "mr-6"],
                    if let Model::Url(_) = model {
                        Some(C!["is-dark"])
                    } else {
                        None
                    },
                    ev(Ev::Click, |_| Msg::UrlClicked),
                    "A URL"
                ],
                button![
                    C!["button", "ml-6"],
                    if let Model::Pasted(_) = model {
                        Some(C!["is-dark"])
                    } else {
                        None
                    },
                    ev(Ev::Click, |_| Msg::PasteClicked),
                    "I'll Paste It"
                ]
            ],
            schema_selector
        ]
    ]
}

fn view_url_entry() -> Node<Msg> {
    div![
        C!["columns", "mt-4", "is-centered", "has-text-left"],
        div![
            C!["column", "is-half"],
            div![
                C!["field"],
                label![C!["label"], "GraphQL Endpoint"],
                div![
                    C!["control"],
                    input![
                        C!["input"],
                        input_ev(Ev::Input, Msg::TextUpdated),
                        attrs! {At::Type => "text", At::Placeholder => "Enter an endpoint URL"}
                    ]
                ],
                p![
                    C!["help"],
                    "We'll build your query using the schema from this endpoint"
                ]
            ],
            view_build_link_button()
        ]
    ]
}

fn view_paste_entry() -> Node<Msg> {
    div![
        C!["columns", "mt-4", "is-centered", "has-text-left"],
        div![
            C![ "column", "is-half"],
            div![
                C!["field"],
                label![C!["label"], "GraphQL Schema"],
                div![
                    C!["control"],
                    textarea![
                        C!["textarea"],
                        input_ev(Ev::Input, Msg::TextUpdated),
                        attrs! {
                            At::Rows => 15,
                            At::Cols => 40,
                            At::Placeholder => "Paste your graphql schema"
                        }
                    ]
                ],
                p![
                    C!["help"],
                    "We'll generate a query using the schema you provide here.  You won't be able to test your queries as we don't have a URL to run them against"
                ]
            ],
            view_build_link_button()
        ]
    ]
}

fn view_build_link_button() -> Node<Msg> {
    div![
        C!["field"],
        div![
            C!["control"],
            button![
                C!["button", "is-link"],
                ev(Ev::Click, |_| Msg::BuildQuery),
                "Build Query"
            ]
        ],
    ]
}
