#[cfg(not(feature = "std"))]
compile_error!("The CLI binary requires the 'std' feature");

use clap::{Parser, Subcommand};
#[cfg(feature = "std")]
use runeforge::{observability, schema, selector::Selector};
use std::fs;
use std::process;
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an optimal technology stack plan from a blueprint
    Plan {
        /// Input blueprint file (YAML or JSON)
        #[arg(short = 'f', long = "file", required = true)]
        file: String,

        /// Random seed for deterministic selection
        #[arg(long = "seed", default_value = "42")]
        seed: u64,

        /// Output file (default: stdout)
        #[arg(long = "out")]
        out: Option<String>,

        /// Enable strict schema validation
        #[arg(long = "strict")]
        strict: bool,
    },
}

fn main() {
    // Initialize observability
    if let Err(e) = observability::init_observability() {
        eprintln!("Failed to initialize observability: {e}");
    }

    let cli = Cli::parse();

    match &cli.command {
        Commands::Plan {
            file,
            seed,
            out,
            strict,
        } => {
            if let Err(e) = run_plan(file, *seed, out.as_deref(), *strict) {
                eprintln!("Error: {e}");
                // Determine exit code based on error type
                let exit_code = if e.contains("Failed to parse blueprint") || e.contains("schema") {
                    1 // Input schema error
                } else if e.contains("output schema") {
                    2 // Output schema error
                } else if e.contains("No suitable") || e.contains("No stack found") {
                    3 // No matching stack found
                } else {
                    1 // Default to input error
                };
                process::exit(exit_code);
            }
        }
    }
}

fn run_plan(file: &str, seed: u64, out: Option<&str>, _strict: bool) -> Result<(), String> {
    run_plan_with_rules(file, seed, out, _strict, "resources/rules.yaml")
}

fn run_plan_with_rules(
    file: &str,
    seed: u64,
    out: Option<&str>,
    _strict: bool,
    rules_path: &str,
) -> Result<(), String> {
    let _start_time = Instant::now();
    let _span = observability::DurationSpan::new("run_plan");

    // Read input file
    let input_content =
        fs::read_to_string(file).map_err(|e| format!("Failed to read input file: {e}"))?;

    // Validate and parse blueprint
    let format = if file.ends_with(".json") {
        "json"
    } else {
        "yaml"
    };
    observability::log_blueprint_validation(input_content.len(), format);

    let blueprint = match schema::validate_blueprint(&input_content) {
        Ok(bp) => bp,
        Err(e) => {
            observability::log_error("blueprint_validation", &e);
            return Err(format!("Failed to parse blueprint: {e}"));
        }
    };

    // Load rules
    let rules_content =
        fs::read_to_string(rules_path).map_err(|e| format!("Failed to read rules file: {e}"))?;

    // Create selector and generate plan
    observability::log_selection_start(&blueprint.project_name, seed);
    let selector = Selector::new(&rules_content, seed)?;
    let plan = match selector.select(&blueprint) {
        Ok(p) => p,
        Err(e) => {
            observability::log_error("selection", &e);
            return Err(e);
        }
    };

    // Validate output
    if let Err(e) = schema::validate_stack_plan(&plan) {
        return Err(format!("Output schema validation failed: {e}"));
    }

    // Serialize to JSON
    let output_json = serde_json::to_string_pretty(&plan)
        .map_err(|e| format!("Failed to serialize output: {e}"))?;

    // Write output
    if let Some(output_file) = out {
        fs::write(output_file, &output_json)
            .map_err(|e| format!("Failed to write output file: {e}"))?;
    } else {
        println!("{output_json}");
    }

    // Log final selection summary
    let stack_summary = vec![
        ("language".to_string(), plan.stack.language.clone()),
        ("frontend".to_string(), plan.stack.frontend.clone()),
        ("backend".to_string(), plan.stack.backend.clone()),
        ("database".to_string(), plan.stack.database.clone()),
        ("cache".to_string(), plan.stack.cache.clone()),
        ("queue".to_string(), plan.stack.queue.clone()),
        ("ai".to_string(), plan.stack.ai.join(", ")),
        ("infra".to_string(), plan.stack.infra.clone()),
        ("ci_cd".to_string(), plan.stack.ci_cd.clone()),
    ];
    observability::log_final_selection(&stack_summary, plan.estimated.monthly_cost_usd);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_blueprint(content: &str) -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test_blueprint.yaml");
        fs::write(&file_path, content).unwrap();
        (dir, file_path.to_str().unwrap().to_string())
    }

    fn create_test_rules() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        let rules_content = r#"
version: 1
weights:
  quality: 0.30
  slo: 0.25
  cost: 0.20
  security: 0.15
  ops: 0.10
candidates:
  language:
    - name: "Rust"
      metrics: { quality: 0.9, slo: 0.95, cost: 0.8, security: 0.95, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 0
  backend:
    - name: "Actix Web"
      requires: { language: "Rust" }
      metrics: { quality: 0.9, slo: 0.9, cost: 0.7, security: 0.8, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 100
  frontend:
    - name: "SvelteKit"
      metrics: { quality: 0.85, slo: 0.8, cost: 0.8, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 50
  database:
    - name: "PostgreSQL"
      persistence: "sql"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.7, security: 0.9, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 200
  cache:
    - name: "Redis"
      metrics: { quality: 0.9, slo: 0.95, cost: 0.6, security: 0.85, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 100
  queue:
    - name: "NATS"
      metrics: { quality: 0.85, slo: 0.9, cost: 0.5, security: 0.85, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 50
  ai:
    - name: "RuneSage"
      metrics: { quality: 0.8, slo: 0.8, cost: 0.7, security: 0.8, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 100
  infra:
    - name: "Terraform"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.8, security: 0.9, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 0
  ci_cd:
    - name: "GitHub Actions"
      metrics: { quality: 0.85, slo: 0.8, cost: 0.9, security: 0.85, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 20
"#;
        let rules_path = dir.path().join("rules.yaml");
        fs::write(&rules_path, rules_content).unwrap();
        (dir, rules_path.to_str().unwrap().to_string())
    }

    #[test]
    fn test_run_plan_success() {
        let blueprint_content = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: 1000
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);
        let (_rules_dir, rules_path) = create_test_rules();
        let output_dir = TempDir::new().unwrap();
        let output_path = output_dir.path().join("output.json");

        let result = run_plan_with_rules(
            &bp_path,
            42,
            Some(output_path.to_str().unwrap()),
            false,
            &rules_path,
        );

        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify output is valid JSON
        let output_content = fs::read_to_string(&output_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output_content).unwrap();
        assert!(parsed.get("stack").is_some());
        assert!(parsed.get("decisions").is_some());
        assert!(parsed.get("estimated").is_some());
        assert!(parsed.get("meta").is_some());
    }

    #[test]
    fn test_run_plan_invalid_blueprint() {
        let blueprint_content = r#"
project_name: ""
goals: []
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);
        let (_rules_dir, rules_path) = create_test_rules();

        let result = run_plan_with_rules(&bp_path, 42, None, false, &rules_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse blueprint"));
    }

    #[test]
    fn test_run_plan_file_not_found() {
        let (_rules_dir, rules_path) = create_test_rules();

        let result = run_plan_with_rules("/nonexistent/file.yaml", 42, None, false, &rules_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read input file"));
    }

    #[test]
    fn test_run_plan_rules_not_found() {
        let blueprint_content = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);

        let result = run_plan_with_rules(&bp_path, 42, None, false, "/nonexistent/rules.yaml");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read rules file"));
    }

    #[test]
    fn test_run_plan_deterministic_output() {
        let blueprint_content = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);
        let (_rules_dir, rules_path) = create_test_rules();
        let output_dir = TempDir::new().unwrap();

        let output_path1 = output_dir.path().join("output1.json");
        let output_path2 = output_dir.path().join("output2.json");

        // Run twice with same seed
        let result1 = run_plan_with_rules(
            &bp_path,
            42,
            Some(output_path1.to_str().unwrap()),
            false,
            &rules_path,
        );
        let result2 = run_plan_with_rules(
            &bp_path,
            42,
            Some(output_path2.to_str().unwrap()),
            false,
            &rules_path,
        );

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let content1 = fs::read_to_string(&output_path1).unwrap();
        let content2 = fs::read_to_string(&output_path2).unwrap();

        // Parse and compare stack selections (ignoring meta.plan_hash which includes timestamp)
        let json1: serde_json::Value = serde_json::from_str(&content1).unwrap();
        let json2: serde_json::Value = serde_json::from_str(&content2).unwrap();

        assert_eq!(json1.get("stack"), json2.get("stack"));
        assert_eq!(json1.get("decisions"), json2.get("decisions"));
    }

    #[test]
    fn test_run_plan_cost_constraint_exceeded() {
        let blueprint_content = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: 10
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);
        let (_rules_dir, rules_path) = create_test_rules();

        let result = run_plan_with_rules(&bp_path, 42, None, false, &rules_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("cost constraint") || err.contains("No suitable"),
            "Expected cost constraint error, got: {err}"
        );
    }

    #[test]
    fn test_run_plan_json_input() {
        let blueprint_content = r#"{
            "project_name": "test-project",
            "goals": ["Build a web app"],
            "constraints": {
                "monthly_cost_usd_max": 1000
            },
            "traffic_profile": {
                "rps_peak": 1000,
                "global": true,
                "latency_sensitive": false
            }
        }"#;

        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test_blueprint.json");
        fs::write(&file_path, blueprint_content).unwrap();

        let (_rules_dir, rules_path) = create_test_rules();

        let result = run_plan_with_rules(file_path.to_str().unwrap(), 42, None, false, &rules_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_output_to_stdout() {
        // This test verifies that when no output file is specified,
        // the result is printed to stdout (we can't easily capture stdout in unit tests,
        // so we just verify the function succeeds)
        let blueprint_content = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);
        let (_rules_dir, rules_path) = create_test_rules();

        let result = run_plan_with_rules(&bp_path, 42, None, false, &rules_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_malformed_yaml() {
        let blueprint_content = r#"
project_name: "test-project
goals:
  - "Build a web app"
  this is invalid yaml
"#;

        let (_bp_dir, bp_path) = create_test_blueprint(blueprint_content);
        let (_rules_dir, rules_path) = create_test_rules();

        let result = run_plan_with_rules(&bp_path, 42, None, false, &rules_path);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse blueprint"));
    }
}
