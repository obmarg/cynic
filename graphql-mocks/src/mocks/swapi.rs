use serde_json::{Value, json};

use crate::{DynamicSchema, MockGraphQlServer, ResolverContext};

pub async fn serve() -> MockGraphQlServer {
    DynamicSchema::builder(include_str!("../../../schemas/starwars.schema.graphql"))
        .with_resolver("Root", "film", film_resolver)
        .with_resolver("Root", "node", node_resolver)
        .into_server_builder()
        .await
}

fn film_resolver(context: ResolverContext) -> Option<Value> {
    let id = context.args.get("id")?.string().ok()?;
    match id {
        "ZmlsbXM6MQ==" => Some(json!({
            "id": "ZmlsbXM6MQ==",
            "title": "A New Hope",
            "director": "George Lucas",
            "releaseDate": "1977-05-25",
            "producers": ["Gary Kurtz", "Rick McCallum"]
        })),
        _ => None,
    }
}

fn node_resolver(context: ResolverContext) -> Option<Value> {
    let id = context.args.get("id")?.string().ok()?;
    match id {
        "ZmlsbXM6MQ==" => Some(json!({
            "__typename": "Film",
            "id": "ZmlsbXM6MQ==",
            "title": "A New Hope",
            "director": "George Lucas",
            "releaseDate": "1977-05-25",
            "producers": ["Gary Kurtz", "Rick McCallum"]
        })),
        "cGxhbmV0czo0OQ==" => Some(json!({
            "__typename": "Planet",
            "id": "cGxhbmV0czo0OQ==",
            "name": "Dorin"
        })),
        "c3RhcnNoaXBzOjY1" => Some(json!({
            "__typename": "Starship",
            "id": "c3RhcnNoaXBzOjY1"
        })),
        _ => None,
    }
}
