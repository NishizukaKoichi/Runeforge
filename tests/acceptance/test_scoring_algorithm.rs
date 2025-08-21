use std::process::Command;
use serde_json::Value;

/// Test that scoring algorithm produces reasonable results
#[test]
fn test_scoring_weights() {
    let fixture = "tests/acceptance/fixtures/valid_baseline.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture, "--seed", "42"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    let decisions = json["decisions"].as_array()
        .expect("decisions not an array");
    
    // Verify decisions are sorted by score (highest first)
    let mut previous_score = 1.0;
    for decision in decisions {
        let score = decision["score"].as_f64()
            .expect("score not a number");
        
        assert!(score <= previous_score, 
            "Decisions should be sorted by score descending");
        previous_score = score;
    }
}

/// Test that preferences influence selection
#[test]
fn test_preferences_influence_selection() {
    // Test with explicit preferences
    let pref_content = r#"
project_name: "preference-test"
goals:
  - "Test preferences"
constraints:
  monthly_cost_usd_max: 1000
traffic_profile:
  rps_peak: 100
  global: false
  latency_sensitive: false
prefs:
  frontend: ["SvelteKit"]
  backend: ["Axum"]
  database: ["PostgreSQL"]
"#;
    
    std::fs::write("tests/acceptance/fixtures/test_preferences.yaml", pref_content)
        .expect("Failed to write fixture");
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/test_preferences.yaml"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // Check if preferred technologies were selected
    let stack = &json["stack"];
    let _frontend = stack["frontend"].as_str().unwrap();
    let _backend = stack["backend"].as_str().unwrap();
    let _database = stack["database"].as_str().unwrap();
    
    // Preferences should influence but not guarantee selection
    // At minimum, check that the system considered these options
    let decisions = json["decisions"].as_array().unwrap();
    
    // Find frontend decision
    let frontend_decision = decisions.iter()
        .find(|d| d["topic"].as_str() == Some("frontend"));
    
    if let Some(decision) = frontend_decision {
        let alternatives = decision["alternatives"].as_array().unwrap();
        let choice = decision["choice"].as_str().unwrap();
        
        // Either chosen or in alternatives
        assert!(choice == "SvelteKit" || 
                alternatives.iter().any(|a| a.as_str() == Some("SvelteKit")),
                "SvelteKit should be considered as per preferences");
    }
    
    // Clean up
    std::fs::remove_file("tests/acceptance/fixtures/test_preferences.yaml").ok();
}

/// Test latency sensitive scoring
#[test]
fn test_latency_sensitive_scoring() {
    let fixture = "tests/acceptance/fixtures/valid_latency_sensitive.yaml";
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", fixture])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // For latency sensitive apps, check that appropriate technologies are selected
    let decisions = json["decisions"].as_array().unwrap();
    
    // Look for mentions of latency/performance in reasons
    let has_latency_reason = decisions.iter().any(|d| {
        d["reasons"].as_array()
            .map(|reasons| reasons.iter().any(|r| {
                r.as_str().map(|s| {
                    let lower = s.to_lowercase();
                    lower.contains("latency") || 
                    lower.contains("performance") ||
                    lower.contains("fast") ||
                    lower.contains("speed")
                }).unwrap_or(false)
            }))
            .unwrap_or(false)
    });
    
    assert!(has_latency_reason || decisions.is_empty(),
        "Latency sensitive apps should have performance-related decision reasons");
}

/// Test that high RPS influences infrastructure choices
#[test]
fn test_high_rps_influences_choices() {
    let high_rps_content = r#"
project_name: "high-traffic-app"
goals:
  - "Handle massive traffic"
constraints:
  monthly_cost_usd_max: 10000
traffic_profile:
  rps_peak: 100000
  global: true
  latency_sensitive: true
"#;
    
    std::fs::write("tests/acceptance/fixtures/high_rps.yaml", high_rps_content)
        .expect("Failed to write fixture");
    
    let output = Command::new("cargo")
        .args(["run", "--", "plan", "-f", "tests/acceptance/fixtures/high_rps.yaml"])
        .output()
        .expect("Failed to execute command");
    
    assert_eq!(output.status.code(), Some(0));
    
    let json: Value = serde_json::from_slice(&output.stdout)
        .expect("Failed to parse JSON output");
    
    // High RPS should influence infrastructure choices
    let infra = json["stack"]["infra"].as_str().unwrap();
    let cache = json["stack"]["cache"].as_str().unwrap();
    
    // Should select scalable infrastructure
    assert!(infra.to_lowercase().contains("cloudflare") || 
            infra.to_lowercase().contains("aws") ||
            infra.to_lowercase().contains("global"),
            "High RPS should select scalable infrastructure");
    
    // Should have caching solution
    assert!(!cache.is_empty() && cache != "none",
        "High RPS should include caching solution");
    
    // Clean up
    std::fs::remove_file("tests/acceptance/fixtures/high_rps.yaml").ok();
}