use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::{fs::File, io::Read};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_schema", |b| {
        let github_schema = {
            let mut github_schema = String::new();
            File::open("../schemas/github.graphql")
                .unwrap()
                .read_to_string(&mut github_schema)
                .unwrap();
            github_schema
        };

        b.iter(|| {
            graphql_parser::schema::parse_schema::<String>(&github_schema).unwrap();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
