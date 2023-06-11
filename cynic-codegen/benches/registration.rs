use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let github_schema = include_str!("../../schemas/github.graphql");
    c.bench_function("schema registration", |b| {
        b.iter(|| {
            cynic_codegen::register_schema("github")
                .dry_run()
                .from_sdl(github_schema)
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
