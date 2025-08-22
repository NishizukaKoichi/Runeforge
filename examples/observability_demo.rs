use runeforge::{observability, schema, selector::Selector};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set RUST_LOG environment variable for this demo
    std::env::set_var("RUST_LOG", "info,runeforge=debug");

    // Initialize observability with structured logging
    observability::init_observability()?;

    println!("=== Runeforge Observability Demo ===\n");
    println!("Structured logs will be output in JSON format below:\n");

    // Create a sample blueprint
    let blueprint_content = r#"
project_name: "observability-demo"
goals:
  - "Build a high-performance web application"
  - "Support global users with low latency"
constraints:
  monthly_cost_usd_max: 500
  quality_min: 0.8
  security_min: 0.85
  slo_min: 0.9
traffic_profile:
  rps_peak: 10000
  global: true
  latency_sensitive: true
prefs:
  backend:
    - "Actix Web"
    - "Axum"
"#;

    // Validate blueprint (this will log validation details)
    let blueprint = schema::validate_blueprint(blueprint_content)?;

    // Load rules
    let rules_content = fs::read_to_string("resources/rules.yaml")?;

    // Create selector and generate plan (this will log selection process)
    let selector = Selector::new(&rules_content, 42, 8)?;
    let plan = selector.select(&blueprint)?;

    // Validate output
    schema::validate_stack_plan(&plan)?;

    println!("\n=== Selection Complete ===");
    println!("Stack selected:");
    println!("  language: {}", plan.stack.language);
    println!("  frontend: {}", plan.stack.frontend);
    println!("  backend: {}", plan.stack.backend);
    println!("  database: {}", plan.stack.database);
    println!("  cache: {}", plan.stack.cache);
    println!("  queue: {}", plan.stack.queue);
    println!("  ai: {}", plan.stack.ai.join(", "));
    println!("  infra: {}", plan.stack.infra);
    println!("  ci_cd: {}", plan.stack.ci_cd);
    println!("Total monthly cost: ${}", plan.estimated.monthly_cost_usd);

    // Create a metrics instance and simulate some operations
    let mut metrics = observability::Metrics::default();

    // Simulate multiple selections
    for i in 0..5 {
        metrics.record_validation();

        let start = std::time::Instant::now();
        let result = selector.select(&blueprint);
        let duration = start.elapsed().as_millis();

        metrics.record_selection(result.is_ok(), duration);

        if i == 2 {
            // Simulate a constraint violation
            metrics.record_constraint_violation();
        }
    }

    // Log metrics summary
    println!("\n=== Metrics Summary ===");
    metrics.log_summary();

    Ok(())
}
