use criterion::{black_box, criterion_group, criterion_main, Criterion};
use runeforge::{schema, selector::Selector};

fn bench_blueprint_validation(c: &mut Criterion) {
    let blueprint_yaml = r#"
project_name: "benchmark-project"
goals:
  - "Build a high-performance web application"
  - "Support global users"
constraints:
  monthly_cost_usd_max: 1000
  region_allow: ["us-east", "eu-west", "asia-pacific"]
traffic_profile:
  rps_peak: 50000
  global: true
  latency_sensitive: true
prefs:
  backend: ["Rust", "Go"]
  database: ["PostgreSQL", "Redis"]
"#;

    c.bench_function("blueprint_validation", |b| {
        b.iter(|| {
            let _ = schema::validate_blueprint(black_box(blueprint_yaml));
        })
    });
}

fn bench_selection_algorithm(c: &mut Criterion) {
    let blueprint_yaml = r#"
project_name: "benchmark-project"
goals:
  - "Build a web application"
constraints:
  monthly_cost_usd_max: 1000
traffic_profile:
  rps_peak: 10000
  global: true
  latency_sensitive: false
"#;

    let rules_yaml =
        std::fs::read_to_string("resources/rules.yaml").expect("Failed to read rules.yaml");

    let blueprint = schema::validate_blueprint(blueprint_yaml).expect("Failed to parse blueprint");

    c.bench_function("selection_algorithm", |b| {
        b.iter(|| {
            let selector = Selector::new(&rules_yaml, 42, 8).unwrap();
            let _ = selector.select(black_box(&blueprint));
        })
    });
}

fn bench_stack_validation(c: &mut Criterion) {
    let blueprint_yaml = r#"
project_name: "benchmark-project"
goals:
  - "Build a web application"
constraints:
  monthly_cost_usd_max: 1000
traffic_profile:
  rps_peak: 10000
  global: true
  latency_sensitive: false
"#;

    let rules_yaml =
        std::fs::read_to_string("resources/rules.yaml").expect("Failed to read rules.yaml");

    let blueprint = schema::validate_blueprint(blueprint_yaml).expect("Failed to parse blueprint");

    let selector = Selector::new(&rules_yaml, 42, 8).unwrap();
    let plan = selector.select(&blueprint).unwrap();

    c.bench_function("stack_validation", |b| {
        b.iter(|| {
            let _ = schema::validate_stack_plan(black_box(&plan));
        })
    });
}

fn bench_end_to_end(c: &mut Criterion) {
    let blueprint_yaml = r#"
project_name: "benchmark-project"
goals:
  - "Build a web application"
constraints:
  monthly_cost_usd_max: 1000
traffic_profile:
  rps_peak: 10000
  global: true
  latency_sensitive: false
"#;

    let rules_yaml =
        std::fs::read_to_string("resources/rules.yaml").expect("Failed to read rules.yaml");

    c.bench_function("end_to_end", |b| {
        b.iter(|| {
            let blueprint = schema::validate_blueprint(black_box(blueprint_yaml)).unwrap();
            let selector = Selector::new(&rules_yaml, 42, 8).unwrap();
            let plan = selector.select(&blueprint).unwrap();
            let _ = schema::validate_stack_plan(&plan);
        })
    });
}

criterion_group!(
    benches,
    bench_blueprint_validation,
    bench_selection_algorithm,
    bench_stack_validation,
    bench_end_to_end
);
criterion_main!(benches);
