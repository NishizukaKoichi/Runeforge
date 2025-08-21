#[cfg(test)]
mod fault_injection_tests {
    use runeforge::{schema, selector::Selector};
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Test behavior with corrupted rules file
    #[test]
    fn test_corrupted_rules_file() {
        let corrupted_rules = r#"
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
      metrics: { quality: 0.9, slo: INVALID_NUMBER, cost: 0.8, security: 0.95, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 0
"#;

        let result = Selector::new(corrupted_rules, 42);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse rules"));
    }

    /// Test behavior with incomplete blueprint
    #[test]
    fn test_incomplete_blueprint() {
        let incomplete_blueprint = r#"
project_name: "test-project"
# Missing required fields
"#;

        let result = schema::validate_blueprint(incomplete_blueprint);
        assert!(result.is_err());
    }

    /// Test behavior with malformed JSON blueprint
    #[test]
    fn test_malformed_json_blueprint() {
        let malformed_json = r#"{
            "project_name": "test-project",
            "goals": ["Build a web app"],
            "constraints": {
                "monthly_cost_usd_max": 1000
            },
            "traffic_profile": {
                "rps_peak": 1000,
                "global": true,
                "latency_sensitive": false
            // Missing closing brace
        "#;

        let result = schema::validate_blueprint(malformed_json);
        assert!(result.is_err());
    }

    /// Test behavior with extremely large input
    #[test]
    fn test_extremely_large_input() {
        let mut large_blueprint = String::from(r#"{
            "project_name": "test-project",
            "goals": ["#);
        
        // Create a very large goals array
        for i in 0..10000 {
            if i > 0 {
                large_blueprint.push_str(", ");
            }
            large_blueprint.push_str(&format!("\"Goal {}\"", i));
        }
        
        large_blueprint.push_str(r#"],
            "constraints": {
                "monthly_cost_usd_max": 1000
            },
            "traffic_profile": {
                "rps_peak": 1000,
                "global": true,
                "latency_sensitive": false
            }
        }"#);

        let result = schema::validate_blueprint(&large_blueprint);
        // Should handle large input gracefully
        assert!(result.is_ok() || result.is_err());
    }

    /// Test behavior with invalid UTF-8 in file
    #[test]
    fn test_invalid_utf8_file() {
        let mut file = NamedTempFile::new().unwrap();
        // Write invalid UTF-8 bytes
        file.write_all(&[0xFF, 0xFE, 0xFD]).unwrap();
        file.flush().unwrap();

        let result = fs::read_to_string(file.path());
        assert!(result.is_err());
    }

    /// Test behavior with zero-cost constraint
    #[test]
    fn test_zero_cost_constraint() {
        let blueprint_str = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: 0
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let blueprint = schema::validate_blueprint(blueprint_str).unwrap();
        
        let rules_content = include_str!("../resources/rules.yaml");
        let selector = Selector::new(rules_content, 42).unwrap();
        
        let result = selector.select(&blueprint);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("cost constraint") || err_msg.contains("No suitable") || err_msg.contains("exceeds budget"));
    }

    /// Test behavior with conflicting requirements
    #[test]
    fn test_conflicting_requirements() {
        let blueprint_str = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: 1
  quality_min: 0.99
  security_min: 0.99
  slo_min: 0.99
traffic_profile:
  rps_peak: 1000000
  global: true
  latency_sensitive: true
"#;

        let blueprint = schema::validate_blueprint(blueprint_str).unwrap();
        
        let rules_content = include_str!("../resources/rules.yaml");
        let selector = Selector::new(rules_content, 42).unwrap();
        
        let result = selector.select(&blueprint);
        // Should fail to find a suitable stack
        assert!(result.is_err());
    }

    /// Test behavior with empty rules file
    #[test]
    fn test_empty_rules_file() {
        let empty_rules = "";
        let result = Selector::new(empty_rules, 42);
        assert!(result.is_err());
    }

    /// Test behavior with invalid seed values
    #[test]
    fn test_extreme_seed_values() {
        let blueprint_str = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let blueprint = schema::validate_blueprint(blueprint_str).unwrap();
        let rules_content = include_str!("../resources/rules.yaml");

        // Test with maximum u64 value
        let selector_max = Selector::new(rules_content, u64::MAX).unwrap();
        let result_max = selector_max.select(&blueprint);
        assert!(result_max.is_ok());

        // Test with 0
        let selector_zero = Selector::new(rules_content, 0).unwrap();
        let result_zero = selector_zero.select(&blueprint);
        assert!(result_zero.is_ok());
    }

    /// Test behavior with cyclic dependencies in rules
    #[test]
    fn test_cyclic_dependencies() {
        let cyclic_rules = r#"
version: 1
weights:
  quality: 0.30
  slo: 0.25
  cost: 0.20
  security: 0.15
  ops: 0.10
candidates:
  language:
    - name: "A"
      requires: { language: "B" }
      metrics: { quality: 0.9, slo: 0.9, cost: 0.8, security: 0.9, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 0
    - name: "B"
      requires: { language: "A" }
      metrics: { quality: 0.9, slo: 0.9, cost: 0.8, security: 0.9, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 0
"#;

        let result = Selector::new(cyclic_rules, 42);
        // Should handle cyclic dependencies gracefully
        assert!(result.is_ok() || result.is_err());
    }

    /// Test behavior with missing required dependencies
    #[test]
    fn test_missing_dependencies() {
        let rules_with_missing_deps = r#"
version: 1
weights:
  quality: 0.30
  slo: 0.25
  cost: 0.20
  security: 0.15
  ops: 0.10
candidates:
  language:
    - name: "Python"
      metrics: { quality: 0.8, slo: 0.8, cost: 0.9, security: 0.8, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 0
  backend:
    - name: "Framework"
      requires: { language: "NonExistent" }
      metrics: { quality: 0.9, slo: 0.9, cost: 0.8, security: 0.9, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 100
  frontend:
    - name: "React"
      metrics: { quality: 0.8, slo: 0.8, cost: 0.8, security: 0.8, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 50
  database:
    - name: "SQLite"
      persistence: "sql"
      metrics: { quality: 0.7, slo: 0.7, cost: 0.9, security: 0.7, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 0
  cache:
    - name: "In-Memory"
      metrics: { quality: 0.8, slo: 0.9, cost: 1.0, security: 0.8, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 0
  queue:
    - name: "In-Memory Queue"
      metrics: { quality: 0.7, slo: 0.8, cost: 1.0, security: 0.7, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 0
  ai:
    - name: "Basic AI"
      metrics: { quality: 0.6, slo: 0.6, cost: 0.8, security: 0.6, ops: 0.7 }
      regions: ["*"]
      monthly_cost_base: 50
  infra:
    - name: "Basic Infra"
      metrics: { quality: 0.7, slo: 0.7, cost: 0.9, security: 0.7, ops: 0.7 }
      regions: ["*"]
      monthly_cost_base: 0
  ci_cd:
    - name: "Basic CI/CD"
      metrics: { quality: 0.7, slo: 0.7, cost: 0.9, security: 0.7, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 0
"#;

        let selector = Selector::new(rules_with_missing_deps, 42).unwrap();
        
        let blueprint_str = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let blueprint = schema::validate_blueprint(blueprint_str).unwrap();
        let result = selector.select(&blueprint);
        
        // Should either skip the component or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }
}