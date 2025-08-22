//! Schema definitions and validation for Blueprint input and Stack output.
//!
//! This module provides the core data structures and validation logic for:
//! - Blueprint: Input requirements specification
//! - StackPlan: Output technology stack recommendations

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Blueprint represents the input requirements for technology stack selection.
///
/// A blueprint describes the project requirements, constraints, and preferences
/// that guide the selection of an optimal technology stack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Blueprint {
    pub project_name: String,
    pub goals: Vec<String>,
    pub constraints: Constraints,
    pub traffic_profile: TrafficProfile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefs: Option<Preferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_language_mode: Option<LanguageMode>,
}

/// Constraints define the limitations and requirements for the technology stack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Constraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_cost_usd_max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<PersistenceType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compliance: Option<Vec<ComplianceType>>,
}

/// Type of data persistence required by the application.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceType {
    Kv,
    Sql,
    Both,
}

/// Compliance requirements that the technology stack must support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ComplianceType {
    AuditLog,
    Sbom,
    Pci,
    Sox,
    Hipaa,
}

/// Traffic characteristics that influence technology selection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrafficProfile {
    pub rps_peak: f64,
    pub global: bool,
    pub latency_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Preferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LanguageMode {
    Rust,
    Go,
    Ts,
}

// Stack output schema structures
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StackPlan {
    pub decisions: Vec<Decision>,
    pub stack: Stack,
    pub estimated: Estimated,
    pub meta: Meta,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Decision {
    pub topic: String,
    pub choice: String,
    pub reasons: Vec<String>,
    pub alternatives: Vec<String>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Stack {
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<Service>>,
    pub frontend: String,
    pub backend: String,
    pub database: String,
    pub cache: String,
    pub queue: String,
    pub ai: Vec<String>,
    pub infra: String,
    pub ci_cd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Service {
    pub name: String,
    pub kind: String,
    pub language: String,
    pub framework: String,
    pub runtime: String,
    pub build: String,
    pub tests: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Estimated {
    pub monthly_cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub egress_gb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Meta {
    pub seed: i64,
    pub blueprint_hash: String,
    pub plan_hash: String,
}

// Validation functions
pub fn validate_blueprint(data: &str) -> Result<Blueprint, String> {
    // Try to parse as YAML first, then JSON
    let blueprint: Blueprint = serde_yaml::from_str(data)
        .or_else(|_| serde_json::from_str(data))
        .map_err(|e| format!("Failed to parse blueprint: {e}"))?;

    // Validate against schema
    validate_against_schema(&blueprint)?;

    // Additional validation
    if blueprint.project_name.is_empty() {
        return Err("project_name cannot be empty".to_string());
    }

    if blueprint.goals.is_empty() {
        return Err("goals cannot be empty".to_string());
    }

    if blueprint.traffic_profile.rps_peak < 0.0 {
        return Err("rps_peak must be non-negative".to_string());
    }

    if let Some(cost) = blueprint.constraints.monthly_cost_usd_max {
        if cost < 0.0 {
            return Err("monthly_cost_usd_max must be non-negative".to_string());
        }
    }

    Ok(blueprint)
}

pub fn validate_stack_plan(plan: &StackPlan) -> Result<(), String> {
    // Validate against schema
    validate_against_schema(plan)?;

    // Additional validation
    if plan.estimated.monthly_cost_usd < 0.0 {
        return Err("monthly_cost_usd must be non-negative".to_string());
    }

    for decision in &plan.decisions {
        if decision.score < 0.0 || decision.score > 1.0 {
            return Err(format!(
                "Score for {} must be between 0 and 1",
                decision.topic
            ));
        }
    }

    Ok(())
}

fn validate_against_schema<T: JsonSchema + Serialize>(_data: &T) -> Result<(), String> {
    // For now, we'll rely on serde's deserialization validation
    // In a full implementation, we would use jsonschema crate for runtime validation
    // against the actual JSON schema files
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_blueprint() {
        let yaml = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: 500
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_blueprint_json() {
        let json = r#"{
            "project_name": "test-project",
            "goals": ["Build a web app"],
            "constraints": {
                "monthly_cost_usd_max": 500
            },
            "traffic_profile": {
                "rps_peak": 1000,
                "global": true,
                "latency_sensitive": false
            }
        }"#;

        let result = validate_blueprint(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_blueprint_with_all_fields() {
        let yaml = r#"
project_name: "full-project"
goals:
  - "Build a scalable API"
  - "Support real-time features"
constraints:
  monthly_cost_usd_max: 1000
  persistence: sql
  region_allow: ["us-east-1", "eu-west-1"]
  compliance: ["audit-log", "sbom", "hipaa"]
traffic_profile:
  rps_peak: 5000
  global: true
  latency_sensitive: true
prefs:
  frontend: ["SvelteKit", "Next.js"]
  backend: ["Actix Web", "Axum"]
  database: ["PostgreSQL"]
  ai: ["RuneSage"]
single_language_mode: rust
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_ok());
        let blueprint = result.unwrap();
        assert_eq!(blueprint.project_name, "full-project");
        assert_eq!(blueprint.goals.len(), 2);
        assert_eq!(blueprint.constraints.monthly_cost_usd_max, Some(1000.0));
        assert!(matches!(
            blueprint.constraints.persistence,
            Some(PersistenceType::Sql)
        ));
        assert!(matches!(
            blueprint.single_language_mode,
            Some(LanguageMode::Rust)
        ));
    }

    #[test]
    fn test_invalid_blueprint_empty_project_name() {
        let yaml = r#"
project_name: ""
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("project_name cannot be empty"));
    }

    #[test]
    fn test_invalid_blueprint_empty_goals() {
        let yaml = r#"
project_name: "test-project"
goals: []
constraints: {}
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("goals cannot be empty"));
    }

    #[test]
    fn test_invalid_blueprint_negative_rps() {
        let yaml = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints: {}
traffic_profile:
  rps_peak: -100
  global: true
  latency_sensitive: false
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("rps_peak must be non-negative"));
    }

    #[test]
    fn test_invalid_blueprint_negative_cost() {
        let yaml = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: -500
traffic_profile:
  rps_peak: 1000
  global: true
  latency_sensitive: false
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("monthly_cost_usd_max must be non-negative"));
    }

    #[test]
    fn test_invalid_blueprint_malformed_yaml() {
        let yaml = r#"
project_name: "test-project
goals:
  - "Build a web app"
  this is invalid yaml
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse blueprint"));
    }

    #[test]
    fn test_persistence_type_parsing() {
        let yaml_kv = r#"
project_name: "test"
goals: ["test"]
constraints:
  persistence: kv
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
"#;
        let result = validate_blueprint(yaml_kv).unwrap();
        assert!(matches!(
            result.constraints.persistence,
            Some(PersistenceType::Kv)
        ));

        let yaml_sql = r#"
project_name: "test"
goals: ["test"]
constraints:
  persistence: sql
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
"#;
        let result = validate_blueprint(yaml_sql).unwrap();
        assert!(matches!(
            result.constraints.persistence,
            Some(PersistenceType::Sql)
        ));

        let yaml_both = r#"
project_name: "test"
goals: ["test"]
constraints:
  persistence: both
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
"#;
        let result = validate_blueprint(yaml_both).unwrap();
        assert!(matches!(
            result.constraints.persistence,
            Some(PersistenceType::Both)
        ));
    }

    #[test]
    fn test_compliance_type_parsing() {
        let yaml = r#"
project_name: "test"
goals: ["test"]
constraints:
  compliance: ["audit-log", "sbom", "pci", "sox", "hipaa"]
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
"#;
        let result = validate_blueprint(yaml).unwrap();
        let compliance = result.constraints.compliance.unwrap();
        assert_eq!(compliance.len(), 5);
        assert!(compliance.contains(&ComplianceType::AuditLog));
        assert!(compliance.contains(&ComplianceType::Sbom));
        assert!(compliance.contains(&ComplianceType::Pci));
        assert!(compliance.contains(&ComplianceType::Sox));
        assert!(compliance.contains(&ComplianceType::Hipaa));
    }

    #[test]
    fn test_language_mode_parsing() {
        let yaml_rust = r#"
project_name: "test"
goals: ["test"]
constraints: {}
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
single_language_mode: rust
"#;
        let result = validate_blueprint(yaml_rust).unwrap();
        assert!(matches!(
            result.single_language_mode,
            Some(LanguageMode::Rust)
        ));

        let yaml_go = r#"
project_name: "test"
goals: ["test"]
constraints: {}
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
single_language_mode: go
"#;
        let result = validate_blueprint(yaml_go).unwrap();
        assert!(matches!(
            result.single_language_mode,
            Some(LanguageMode::Go)
        ));

        let yaml_ts = r#"
project_name: "test"
goals: ["test"]
constraints: {}
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
single_language_mode: ts
"#;
        let result = validate_blueprint(yaml_ts).unwrap();
        assert!(matches!(
            result.single_language_mode,
            Some(LanguageMode::Ts)
        ));
    }

    #[test]
    fn test_valid_stack_plan() {
        let plan = StackPlan {
            decisions: vec![Decision {
                topic: "language".to_string(),
                choice: "Rust".to_string(),
                reasons: vec!["High performance".to_string()],
                alternatives: vec!["Go".to_string()],
                score: 0.9,
            }],
            stack: Stack {
                language: "Rust".to_string(),
                services: None,
                frontend: "SvelteKit".to_string(),
                backend: "Actix Web".to_string(),
                database: "PostgreSQL".to_string(),
                cache: "Redis".to_string(),
                queue: "NATS".to_string(),
                ai: vec!["RuneSage".to_string()],
                infra: "Terraform".to_string(),
                ci_cd: "GitHub Actions".to_string(),
            },
            estimated: Estimated {
                monthly_cost_usd: 500.0,
                egress_gb: None,
                notes: None,
            },
            meta: Meta {
                seed: 42,
                blueprint_hash: "sha256:abc123".to_string(),
                plan_hash: "sha256:def456".to_string(),
            },
        };

        let result = validate_stack_plan(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_stack_plan_negative_cost() {
        let plan = StackPlan {
            decisions: vec![],
            stack: Stack {
                language: "Rust".to_string(),
                services: None,
                frontend: "SvelteKit".to_string(),
                backend: "Actix Web".to_string(),
                database: "PostgreSQL".to_string(),
                cache: "Redis".to_string(),
                queue: "NATS".to_string(),
                ai: vec!["RuneSage".to_string()],
                infra: "Terraform".to_string(),
                ci_cd: "GitHub Actions".to_string(),
            },
            estimated: Estimated {
                monthly_cost_usd: -100.0,
                egress_gb: None,
                notes: None,
            },
            meta: Meta {
                seed: 42,
                blueprint_hash: "sha256:abc123".to_string(),
                plan_hash: "sha256:def456".to_string(),
            },
        };

        let result = validate_stack_plan(&plan);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("monthly_cost_usd must be non-negative"));
    }

    #[test]
    fn test_invalid_stack_plan_invalid_score() {
        let plan = StackPlan {
            decisions: vec![Decision {
                topic: "language".to_string(),
                choice: "Rust".to_string(),
                reasons: vec!["High performance".to_string()],
                alternatives: vec!["Go".to_string()],
                score: 1.5, // Invalid: > 1.0
            }],
            stack: Stack {
                language: "Rust".to_string(),
                services: None,
                frontend: "SvelteKit".to_string(),
                backend: "Actix Web".to_string(),
                database: "PostgreSQL".to_string(),
                cache: "Redis".to_string(),
                queue: "NATS".to_string(),
                ai: vec!["RuneSage".to_string()],
                infra: "Terraform".to_string(),
                ci_cd: "GitHub Actions".to_string(),
            },
            estimated: Estimated {
                monthly_cost_usd: 500.0,
                egress_gb: None,
                notes: None,
            },
            meta: Meta {
                seed: 42,
                blueprint_hash: "sha256:abc123".to_string(),
                plan_hash: "sha256:def456".to_string(),
            },
        };

        let result = validate_stack_plan(&plan);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Score for language must be between 0 and 1"));

        // Test negative score
        let mut plan2 = plan;
        plan2.decisions[0].score = -0.1;
        let result2 = validate_stack_plan(&plan2);
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .contains("Score for language must be between 0 and 1"));
    }

    #[test]
    fn test_zero_cost_and_rps() {
        let yaml = r#"
project_name: "test-project"
goals:
  - "Build a web app"
constraints:
  monthly_cost_usd_max: 0
traffic_profile:
  rps_peak: 0
  global: true
  latency_sensitive: false
"#;

        let result = validate_blueprint(yaml);
        assert!(result.is_ok());
        let bp = result.unwrap();
        assert_eq!(bp.constraints.monthly_cost_usd_max, Some(0.0));
        assert_eq!(bp.traffic_profile.rps_peak, 0.0);
    }

    #[test]
    fn test_missing_required_fields() {
        // Missing project_name
        let yaml1 = r#"
goals: ["test"]
constraints: {}
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
"#;
        assert!(validate_blueprint(yaml1).is_err());

        // Missing goals
        let yaml2 = r#"
project_name: "test"
constraints: {}
traffic_profile: { rps_peak: 100, global: false, latency_sensitive: false }
"#;
        assert!(validate_blueprint(yaml2).is_err());

        // Missing traffic_profile
        let yaml3 = r#"
project_name: "test"
goals: ["test"]
constraints: {}
"#;
        assert!(validate_blueprint(yaml3).is_err());
    }
}
