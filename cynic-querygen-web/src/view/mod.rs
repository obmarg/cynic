use seed::{prelude::*, *};

use super::Msg;

pub(crate) fn header() -> Node<Msg> {
    section![
        C!["hero", "is-primary"],
        div![C!["hero-head"], navbar()],
        div![
            C!["hero-body"],
            div![
                C!["container", "has-text-centered"],
                h1![C!["title"], "Build GraphQL queries for Rust with Cynic"],
            ]
        ]
    ]
}

fn navbar() -> Node<Msg> {
    nav![
        C!["navbar"],
        attrs![
            "role" => "navigation",
            "aria-label" => "main navigation"
        ],
        div![
            C!["navbar-brand"],
            a![
                C!["navbar-item"],
                attrs![
                    At::Href => "/"
                ],
                "Cynic"
            ]
        ],
        a![
            C!["navbar-burger"],
            attrs![
                "role" => "button",
                "aria-label" => "menu",
            ],
            span![attrs!["arial-hidden" => true]],
            span![attrs!["arial-hidden" => true]],
            span![attrs!["arial-hidden" => true]]
        ], // TODO: Experiment with bottom navbar?
        div![
            C!["navbar-menu"],
            div![C!["navbar-start"]],
            div![
                C!["navbar-end"],
                a![C!["navbar-item"], "Guide"],
                a![C!["navbar-item"], "Reference"],
            ]
        ]
    ]
}

pub(crate) fn entry_page() -> Node<Msg> {
    // TODO: Customise the colours
    section![
        C!["section"],
        div![
            C!["container", "has-text-centered"],
            p![h4![
                C!["subtitle", "is-4"],
                "Where should we get your GraphQL schema?"
            ]],
            div![
                C!["is-centered", "mt-4"],
                button![C!["button", "mr-6"], "A URL"],
                button![C!["button", "ml-6"], "I'll Paste It"]
            ]
        ]
    ]
}
