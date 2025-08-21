use std::process::Command;

/// Test that valid input produces exit code 0
#[test]
fn test_valid_input_success() {
    let fixtures = [
        "valid_baseline.yaml",
        "valid_latency_sensitive.yaml", 
        "valid_compliance_heavy.yaml",
        "valid_minimal.yaml",
        "valid_cost_constraint.yaml",
        "valid_region_constraint.yaml",
    ];

    for fixture in &fixtures {
        let output = Command::new("cargo")
            .args(["run", "--", "plan", "-f", &format!("tests/acceptance/fixtures/{fixture}")])
            .output()
            .expect("Failed to execute command");

        assert_eq!(output.status.code(), Some(0), 
            "Expected exit code 0 for {}, got {:?}", fixture, output.status.code());
    }
}

/// Test that invalid input schema produces exit code 1
#[test]
fn test_invalid_input_schema_exit_1() {
    let fixtures = [
        "invalid_missing_required.yaml",
        "invalid_schema_type.yaml",
    ];

    for fixture in &fixtures {
        let output = Command::new("cargo")
            .args(["run", "--", "plan", "-f", &format!("tests/acceptance/fixtures/{fixture}")])
            .output()
            .expect("Failed to execute command");

        assert_eq!(output.status.code(), Some(1), 
            "Expected exit code 1 for {}, got {:?}", fixture, output.status.code());
    }
}

/// Test that non-existent file produces appropriate error
#[test]
fn test_nonexistent_file() {
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/nonexistent.yaml"])
        .output()
        .expect("Failed to execute command");

    assert_ne!(output.status.code(), Some(0), 
        "Expected non-zero exit code for nonexistent file");
}

/// Test that JSON input is also accepted
#[test]
fn test_json_input_format() {
    // First create a JSON fixture
    let json_content = r#"{
        "project_name": "json-project",
        "goals": ["Test JSON input"],
        "constraints": {},
        "traffic_profile": {
            "rps_peak": 100,
            "global": false,
            "latency_sensitive": false
        }
    }"#;
    
    std::fs::write("tests/acceptance/fixtures/valid_baseline.json", json_content)
        .expect("Failed to write JSON fixture");

    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/valid_baseline.json"])
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0), 
        "Expected exit code 0 for valid JSON input");
    
    // Clean up
    std::fs::remove_file("tests/acceptance/fixtures/valid_baseline.json").ok();
}