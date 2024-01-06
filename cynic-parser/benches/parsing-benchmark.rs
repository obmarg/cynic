use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cynic_parser::Ast;

pub fn criterion_benchmark(c: &mut Criterion) {
    let input = "type MyType { field: Whatever, field: Whatever }";
    c.bench_function("cynic-parser parse object", |b| {
        b.iter(|| {
            let lexer = cynic_parser::Lexer::new(input);
            let object = cynic_parser::ObjectParser::new()
                .parse(input, &mut Ast::new(), lexer)
                .unwrap();
            black_box(object)
        })
    });

    c.bench_function("graphql_parser parse object", |b| {
        b.iter(|| {
            let parsed = graphql_parser::parse_schema::<String>(input).unwrap();
            black_box(parsed)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
