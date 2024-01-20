#[test]
fn roundtrip_github() {
    const GITHUB_SCHEMA: &str = include_str!("../../schemas/github.graphql");

    let parsed = cynic_parser::parse_type_system_document(GITHUB_SCHEMA).unwrap();
    insta::assert_snapshot!(parsed.to_sdl());
}
