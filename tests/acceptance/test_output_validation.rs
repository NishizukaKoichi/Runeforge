use std::process::Command;
use serde_json::Value;
use jsonschema::{Draft, JSONSchema};

/// Test that output matches the stack.schema.json
#[test]
fn test_output_matches_schema() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    // Load the stack schema
    let schema_content = std::fs::read_to_string("schemas/stack.schema.json")
        .expect("Failed to read stack schema");
    let schema: Value = serde_json::from_str(&schema_content)
        .expect("Failed to parse stack schema");
    
    // Parse output
    let output_json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse output JSON");
    
    // Validate against schema
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("Failed to compile schema");
    
    let result = compiled.validate(&output_json);
    assert!(result.is_ok(), "Output does not match stack.schema.json");
}

/// Test all required fields are present in output
#[test]
fn test_required_fields_present() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // Check top-level required fields
    assert!(json["decisions"].is_array(), "Missing decisions array");
    assert!(json["stack"].is_object(), "Missing stack object");
    assert!(json["estimated"].is_object(), "Missing estimated object");
    assert!(json["meta"].is_object(), "Missing meta object");
    
    // Check decisions array structure
    let decisions = json["decisions"].as_array().expect("decisions not an array");
    for decision in decisions {
        assert!(decision["topic"].is_string(), "Missing topic in decision");
        assert!(decision["choice"].is_string(), "Missing choice in decision");
        assert!(decision["reasons"].is_array(), "Missing reasons in decision");
        assert!(decision["alternatives"].is_array(), "Missing alternatives in decision");
        assert!(decision["score"].is_number(), "Missing score in decision");
    }
    
    // Check stack required fields
    let stack = &json["stack"];
    assert!(stack["language"].is_string(), "Missing language in stack");
    assert!(stack["frontend"].is_string(), "Missing frontend in stack");
    assert!(stack["backend"].is_string(), "Missing backend in stack");
    assert!(stack["database"].is_string(), "Missing database in stack");
    assert!(stack["cache"].is_string(), "Missing cache in stack");
    assert!(stack["queue"].is_string(), "Missing queue in stack");
    assert!(stack["ai"].is_array(), "Missing ai array in stack");
    assert!(stack["infra"].is_string(), "Missing infra in stack");
    assert!(stack["ci_cd"].is_string(), "Missing ci_cd in stack");
    
    // Check estimated fields
    assert!(json["estimated"]["monthly_cost_usd"].is_number(), 
        "Missing monthly_cost_usd in estimated");
    
    // Check meta fields
    let meta = &json["meta"];
    assert!(meta["seed"].is_number(), "Missing seed in meta");
    assert!(meta["blueprint_hash"].is_string(), "Missing blueprint_hash in meta");
    assert!(meta["plan_hash"].is_string(), "Missing plan_hash in meta");
    
    // Verify hash format (should be hex string)
    let blueprint_hash = meta["blueprint_hash"].as_str().unwrap();
    let plan_hash = meta["plan_hash"].as_str().unwrap();
    assert!(blueprint_hash.starts_with("sha256:") || blueprint_hash.len() == 64,
        "Invalid blueprint_hash format");
    assert!(plan_hash.starts_with("sha256:") || plan_hash.len() == 64,
        "Invalid plan_hash format");
}

/// Test that decisions contain valid scoring information
#[test]
fn test_decision_scoring() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    let decisions = json["decisions"].as_array()
        .expect("decisions not an array");
    
    for decision in decisions {
        let score = decision["score"].as_f64()
            .expect("score not a number");
        
        // Score should be between 0 and 1 (normalized)
        assert!((0.0..=1.0).contains(&score), 
            "Score {score} is out of range [0, 1]");
        
        // Reasons should not be empty
        let reasons = decision["reasons"].as_array()
            .expect("reasons not an array");
        assert!(!reasons.is_empty(), "Decision reasons should not be empty");
        
        // Should have at least one alternative (or empty array)
        assert!(decision["alternatives"].is_array(), 
            "alternatives should be an array");
    }
}

/// Test --strict flag behavior
#[test]
fn test_strict_mode() {
    // Test with valid input and --strict
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/valid_baseline.yaml", "--strict"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0), 
        "Valid input with --strict should succeed");
    
    // Test with invalid input and --strict
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/invalid_schema_type.yaml", "--strict"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(1), 
        "Invalid input with --strict should exit with code 1");
}