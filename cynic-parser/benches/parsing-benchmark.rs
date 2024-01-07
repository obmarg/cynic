use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cynic_parser::AstBuilder;

pub fn criterion_benchmark(c: &mut Criterion) {
    let input = "type MyType { field: Whatever, field: Whatever }";
    c.bench_function("cynic-parser parse object", |b| {
        b.iter(|| {
            let object = cynic_parser::parse_type_system_document(input);
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
