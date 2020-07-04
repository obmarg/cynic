use seed::{prelude::*, *};

use super::Msg;

pub(crate) fn header() -> Node<Msg> {
    section![
        C!["hero", "is-dark"],
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
        a![
            C!["navbar-burger", "has-text-light"],
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
