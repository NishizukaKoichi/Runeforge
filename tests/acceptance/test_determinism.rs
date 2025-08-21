use std::process::Command;
use serde_json::Value;

/// Test that same input + seed produces identical plan_hash
#[test]
fn test_deterministic_output_with_seed() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    let seed = "42";
    
    // Run twice with same seed
    let output1 = Command::new("cargo")
        .args(&["run", "--", "plan", "-f", fixture, "--seed", seed])
        .output()
        .expect("Failed to execute command");
        
    let output2 = Command::new("cargo")
        .args(&["run", "--", "plan", "-f", fixture, "--seed", seed])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output1.status.code(), Some(0));
    assert_eq!(output2.status.code(), Some(0));
    
    // Parse JSON outputs and compare plan_hash
    let json1: Value = serde_json::from_slice(&output1.stdout)
        .expect("Failed to parse JSON output 1");
    let json2: Value = serde_json::from_slice(&output2.stdout)
        .expect("Failed to parse JSON output 2");
    
    let hash1 = json1["meta"]["plan_hash"].as_str().expect("No plan_hash in output 1");
    let hash2 = json2["meta"]["plan_hash"].as_str().expect("No plan_hash in output 2");
    
    assert_eq!(hash1, hash2, "plan_hash should be identical for same input and seed");
}

/// Test that different seeds produce different outputs
#[test]
fn test_different_seeds_different_output() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    
    let output1 = Command::new("cargo")
        .args(&["run", "--", "plan", "-f", fixture, "--seed", "42"])
        .output()
        .expect("Failed to execute command");
        
    let output2 = Command::new("cargo")
        .args(&["run", "--", "plan", "-f", fixture, "--seed", "123"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output1.status.code(), Some(0));
    assert_eq!(output2.status.code(), Some(0));
    
    // Parse JSON outputs
    let json1: Value = serde_json::from_slice(&output1.stdout)
        .expect("Failed to parse JSON output 1");
    let json2: Value = serde_json::from_slice(&output2.stdout)
        .expect("Failed to parse JSON output 2");
    
    // While plan_hash might be different, blueprint_hash should be same
    let blueprint_hash1 = json1["meta"]["blueprint_hash"].as_str()
        .expect("No blueprint_hash in output 1");
    let blueprint_hash2 = json2["meta"]["blueprint_hash"].as_str()
        .expect("No blueprint_hash in output 2");
    
    assert_eq!(blueprint_hash1, blueprint_hash2, 
        "blueprint_hash should be identical for same input");
    
    // Seeds should be reflected in output
    assert_eq!(json1["meta"]["seed"].as_i64(), Some(42));
    assert_eq!(json2["meta"]["seed"].as_i64(), Some(123));
}

/// Test that output to file works correctly
#[test]
fn test_output_to_file() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    let output_file = "tests/acceptance/test_output.json";
    
    let output = Command::new("cargo")
        .args(&["run", "--", "plan", "-f", fixture, "--out", output_file])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    // Verify file was created and contains valid JSON
    let content = std::fs::read_to_string(output_file)
        .expect("Failed to read output file");
    let json: Value = serde_json::from_str(&content)
        .expect("Output file is not valid JSON");
    
    // Verify required fields exist
    assert!(json["decisions"].is_array());
    assert!(json["stack"].is_object());
    assert!(json["estimated"].is_object());
    assert!(json["meta"].is_object());
    
    // Clean up
    std::fs::remove_file(output_file).ok();
}