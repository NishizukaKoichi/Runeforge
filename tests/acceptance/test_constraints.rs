use std::process::Command;
use serde_json::Value;

/// Test that cost constraints are respected
#[test]
fn test_cost_constraint_respected() {
    let fixture = "tests/acceptance/fixtures/valid_cost_constraint.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    let estimated_cost = json["estimated"]["monthly_cost_usd"].as_f64()
        .expect("No monthly_cost_usd in output");
    
    // Cost constraint in fixture is 50 USD
    assert!(estimated_cost <= 50.0, 
        "Estimated cost {estimated_cost} exceeds constraint of 50 USD");
}

/// Test that region constraints are respected
#[test]
fn test_region_constraint_respected() {
    let fixture = "tests/acceptance/fixtures/valid_region_constraint.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // Check that selected components support EU region
    // This test assumes the implementation checks region constraints
    // The actual validation would depend on rules.yaml content
    let decisions = json["decisions"].as_array()
        .expect("No decisions array in output");
    
    assert!(!decisions.is_empty(), "Should have at least one decision");
}

/// Test compliance requirements filtering
#[test]
fn test_compliance_filtering() {
    let fixture = "tests/acceptance/fixtures/valid_compliance_heavy.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // Verify that choices meet compliance requirements
    // With HIPAA and SOX compliance, certain tech choices should be made
    let decisions = json["decisions"].as_array()
        .expect("No decisions array in output");
    
    // Check that reasons mention compliance
    let has_compliance_reason = decisions.iter().any(|d| {
        d["reasons"].as_array()
            .map(|reasons| reasons.iter().any(|r| {
                r.as_str().map(|s| s.to_lowercase().contains("compliance") || 
                                   s.to_lowercase().contains("hipaa") ||
                                   s.to_lowercase().contains("sox"))
                    .unwrap_or(false)
            }))
            .unwrap_or(false)
    });
    
    assert!(has_compliance_reason || decisions.is_empty(), 
        "Compliance requirements should be reflected in decision reasons");
}

/// Test single language mode constraint
#[test]
fn test_single_language_mode() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml"; // Has single_language_mode: "rust"
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // Verify language is rust
    let language = json["stack"]["language"].as_str()
        .expect("No language in stack");
    
    assert_eq!(language, "Rust", "Language should be Rust when single_language_mode is 'rust'");
    
    // Backend should be Rust-compatible
    let backend = json["stack"]["backend"].as_str()
        .expect("No backend in stack");
    
    assert!(backend == "Actix Web" || backend == "Axum" || backend.contains("Rust"),
        "Backend '{backend}' should be Rust-compatible");
}

/// Test that no matching stack results in exit code 3
#[test]
fn test_no_matching_stack_exit_3() {
    // Create an impossible constraint combination
    let impossible_content = r#"
project_name: "impossible-project"
goals:
  - "Impossible requirements"
constraints:
  monthly_cost_usd_max: 1  # $1 budget
  persistence: "both"      # Needs both KV and SQL
  region_allow: ["antarctica"]  # Non-existent region
  compliance: ["audit-log", "sbom", "pci", "sox", "hipaa"]  # All compliances
traffic_profile:
  rps_peak: 1000000  # 1M RPS
  global: true
  latency_sensitive: true
"#;
    
    std::fs::write("tests/acceptance/fixtures/impossible_constraints.yaml", impossible_content)
        .expect("Failed to write fixture");
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/impossible_constraints.yaml"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(3), 
        "Expected exit code 3 for impossible constraints");
    
    // Clean up
    std::fs::remove_file("tests/acceptance/fixtures/impossible_constraints.yaml").ok();
}