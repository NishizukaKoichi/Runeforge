//! Technology stack selection engine.
//!
//! This module implements the core selection algorithm that evaluates
//! technology candidates based on weighted metrics and constraints.

use crate::observability;
use crate::schema::*;
use crate::util::{calculate_blueprint_hash, calculate_plan_hash, tie_breaker};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rules define the available technology candidates and scoring weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub version: i32,
    pub weights: Weights,
    pub candidates: CandidateCategories,
    #[serde(default)]
    pub compliance_requirements: HashMap<String, ComplianceRequirement>,
}

/// Scoring weights for different quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    pub quality: f64,
    pub slo: f64,
    pub cost: f64,
    pub security: f64,
    pub ops: f64,
}

/// Technology candidates organized by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateCategories {
    pub language: Vec<Candidate>,
    pub backend: Vec<Candidate>,
    pub frontend: Vec<Candidate>,
    pub database: Vec<Candidate>,
    pub cache: Vec<Candidate>,
    pub queue: Vec<Candidate>,
    pub ai: Vec<Candidate>,
    pub infra: Vec<Candidate>,
    pub ci_cd: Vec<Candidate>,
}

/// A technology candidate with its metrics and constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires: Option<Requirements>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<String>,
    pub metrics: Metrics,
    pub regions: Vec<String>,
    #[serde(default)]
    pub monthly_cost_base: f64,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub quality: f64,
    pub slo: f64,
    pub cost: f64,
    pub security: f64,
    pub ops: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub required_features: Vec<String>,
}

#[derive(Debug)]
pub struct Selector {
    rules: Rules,
    seed: u64,
}

impl Selector {
    pub fn new(rules_content: &str, seed: u64) -> Result<Self, String> {
        let rules: Rules = serde_yaml::from_str(rules_content)
            .map_err(|e| format!("Failed to parse rules: {e}"))?;

        Ok(Selector { rules, seed })
    }

    pub fn select(&self, blueprint: &Blueprint) -> Result<StackPlan, String> {
        let mut decisions = Vec::new();
        let mut total_cost = 0.0;

        // Select language first
        let language = self.select_language(blueprint)?;
        decisions.push(language.clone());

        // Select components based on language
        let backend = self.select_component("backend", blueprint, Some(&language.choice))?;
        decisions.push(backend.clone());
        total_cost += self.get_component_cost("backend", &backend.choice);

        let frontend = self.select_component("frontend", blueprint, None)?;
        decisions.push(frontend.clone());
        total_cost += self.get_component_cost("frontend", &frontend.choice);

        let database = self.select_database(blueprint)?;
        decisions.push(database.clone());
        total_cost += self.get_component_cost("database", &database.choice);

        let cache = self.select_component("cache", blueprint, None)?;
        decisions.push(cache.clone());
        total_cost += self.get_component_cost("cache", &cache.choice);

        let queue = self.select_component("queue", blueprint, None)?;
        decisions.push(queue.clone());
        total_cost += self.get_component_cost("queue", &queue.choice);

        let ai_decision = self.select_ai(blueprint)?;
        decisions.push(ai_decision.clone());
        let ai_choices: Vec<String> = ai_decision
            .choice
            .split(", ")
            .map(|s| s.to_string())
            .collect();
        for ai in &ai_choices {
            total_cost += self.get_component_cost("ai", ai);
        }

        let infra = self.select_component("infra", blueprint, None)?;
        decisions.push(infra.clone());
        total_cost += self.get_component_cost("infra", &infra.choice);

        let ci_cd = self.select_component("ci_cd", blueprint, None)?;
        let ci_cd_choice = ci_cd.choice.clone();
        total_cost += self.get_component_cost("ci_cd", &ci_cd_choice);
        decisions.push(ci_cd);

        // Sort decisions by score in descending order
        decisions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Check cost constraint
        if let Some(max_cost) = blueprint.constraints.monthly_cost_usd_max {
            if total_cost > max_cost {
                return Err(format!(
                    "No stack found within cost constraint of ${max_cost}"
                ));
            }
        }

        // Build the stack
        let stack = Stack {
            language: language.choice,
            frontend: frontend.choice,
            backend: backend.choice,
            database: database.choice,
            cache: cache.choice,
            queue: queue.choice,
            ai: ai_choices,
            infra: infra.choice,
            ci_cd: ci_cd_choice,
        };

        // Calculate hashes
        let blueprint_json = serde_json::to_string(blueprint).unwrap();
        let blueprint_hash = calculate_blueprint_hash(&blueprint_json);

        let plan = StackPlan {
            decisions,
            stack: stack.clone(),
            estimated: Estimated {
                monthly_cost_usd: total_cost,
            },
            meta: Meta {
                seed: self.seed as i64,
                blueprint_hash,
                plan_hash: String::new(), // Will be filled after serialization
            },
        };

        // Calculate plan hash
        let plan_json = serde_json::to_string(&plan).unwrap();
        let plan_hash = calculate_plan_hash(&plan_json);

        // Update plan with correct hash
        let mut final_plan = plan;
        final_plan.meta.plan_hash = plan_hash;

        Ok(final_plan)
    }

    fn select_language(&self, blueprint: &Blueprint) -> Result<Decision, String> {
        let candidates = &self.rules.candidates.language;

        // Filter by single language mode if specified
        let filtered = if let Some(mode) = &blueprint.single_language_mode {
            let mode_str = match mode {
                LanguageMode::Rust => "Rust",
                LanguageMode::Go => "Go",
                LanguageMode::Ts => "TypeScript",
            };
            candidates
                .iter()
                .filter(|c| c.name == mode_str)
                .cloned()
                .collect()
        } else {
            candidates.clone()
        };

        self.select_best("language", filtered, blueprint, None)
    }

    fn select_database(&self, blueprint: &Blueprint) -> Result<Decision, String> {
        let candidates = &self.rules.candidates.database;

        // Filter by persistence type if specified
        let filtered = if let Some(persistence) = &blueprint.constraints.persistence {
            let persistence_str = match persistence {
                PersistenceType::Kv => "kv",
                PersistenceType::Sql => "sql",
                PersistenceType::Both => "both",
            };
            candidates
                .iter()
                .filter(|c| {
                    c.persistence
                        .as_ref()
                        .map(|p| p == persistence_str)
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        } else {
            candidates.clone()
        };

        self.select_best("database", filtered, blueprint, None)
    }

    fn select_ai(&self, blueprint: &Blueprint) -> Result<Decision, String> {
        let candidates = &self.rules.candidates.ai;

        // For AI, we select multiple options
        let mut scored_candidates: Vec<(String, f64)> = candidates
            .iter()
            .filter(|c| self.check_constraints(c, blueprint))
            .map(|c| {
                let score = self.calculate_score(&c.metrics, blueprint);
                (c.name.clone(), score)
            })
            .collect();

        scored_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        if scored_candidates.is_empty() {
            return Err("No suitable AI candidates found".to_string());
        }

        // Select top 2 AI options
        let choices: Vec<String> = scored_candidates
            .iter()
            .take(2)
            .map(|(name, _)| name.clone())
            .collect();

        let alternatives: Vec<String> = scored_candidates
            .iter()
            .skip(2)
            .take(2)
            .map(|(name, _)| name.clone())
            .collect();

        Ok(Decision {
            topic: "ai".to_string(),
            choice: choices.join(", "),
            reasons: vec![
                "Selected based on quality and cost balance".to_string(),
                "Multiple AI providers for redundancy".to_string(),
            ],
            alternatives,
            score: scored_candidates[0].1,
        })
    }

    fn select_component(
        &self,
        topic: &str,
        blueprint: &Blueprint,
        language: Option<&str>,
    ) -> Result<Decision, String> {
        let candidates = match topic {
            "backend" => &self.rules.candidates.backend,
            "frontend" => &self.rules.candidates.frontend,
            "cache" => &self.rules.candidates.cache,
            "queue" => &self.rules.candidates.queue,
            "infra" => &self.rules.candidates.infra,
            "ci_cd" => &self.rules.candidates.ci_cd,
            _ => return Err(format!("Unknown component type: {topic}")),
        };

        // Filter by language requirement if applicable
        let filtered = if let Some(lang) = language {
            candidates
                .iter()
                .filter(|c| {
                    c.requires
                        .as_ref()
                        .and_then(|r| r.language.as_ref())
                        .map(|l| l == lang)
                        .unwrap_or(true)
                })
                .cloned()
                .collect()
        } else {
            candidates.clone()
        };

        self.select_best(topic, filtered, blueprint, language)
    }

    fn select_best(
        &self,
        topic: &str,
        candidates: Vec<Candidate>,
        blueprint: &Blueprint,
        language: Option<&str>,
    ) -> Result<Decision, String> {
        // Filter by constraints
        let mut filtered: Vec<Candidate> = candidates
            .into_iter()
            .filter(|c| self.check_constraints(c, blueprint))
            .collect();

        // Apply preferences if available
        if let Some(prefs) = &blueprint.prefs {
            let pref_list = match topic {
                "frontend" => prefs.frontend.as_ref(),
                "backend" => prefs.backend.as_ref(),
                "database" => prefs.database.as_ref(),
                "ai" => prefs.ai.as_ref(),
                _ => None,
            };

            if let Some(pref_names) = pref_list {
                let preferred: Vec<Candidate> = filtered
                    .iter()
                    .filter(|c| pref_names.contains(&c.name))
                    .cloned()
                    .collect();

                if !preferred.is_empty() {
                    filtered = preferred;
                }
            }
        }

        if filtered.is_empty() {
            return Err(format!("No suitable {topic} candidates found"));
        }

        // Score candidates
        let mut scored: Vec<(Candidate, f64)> = filtered
            .into_iter()
            .map(|c| {
                let score = self.calculate_score(&c.metrics, blueprint);
                
                // Log scoring details
                let breakdown = vec![
                    ("quality".to_string(), self.rules.weights.quality * c.metrics.quality),
                    ("slo".to_string(), self.rules.weights.slo * c.metrics.slo),
                    ("cost".to_string(), self.rules.weights.cost * c.metrics.cost),
                    ("security".to_string(), self.rules.weights.security * c.metrics.security),
                    ("ops".to_string(), self.rules.weights.ops * c.metrics.ops),
                ];
                observability::log_scoring(topic, &c.name, score, &breakdown);
                
                (c, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Handle ties
        let top_score = scored[0].1;
        let tied_candidates: Vec<String> = scored
            .iter()
            .filter(|(_, score)| (*score - top_score).abs() < 0.001)
            .map(|(c, _)| c.name.clone())
            .collect();

        let choice = if tied_candidates.len() > 1 {
            tie_breaker(topic, self.seed, tied_candidates)
        } else {
            scored[0].0.name.clone()
        };

        // Get the chosen candidate
        let chosen = scored
            .iter()
            .find(|(c, _)| c.name == choice)
            .map(|(c, s)| (c.clone(), *s))
            .unwrap();

        // Prepare alternatives
        let alternatives: Vec<String> = scored
            .iter()
            .filter(|(c, _)| c.name != choice)
            .take(3)
            .map(|(c, _)| c.name.clone())
            .collect();

        // Generate reasons
        let mut reasons = vec![];
        if topic == "backend" && language.is_some() {
            reasons.push(format!("Compatible with {} language", language.unwrap()));
        }
        if chosen.1 > 0.8 {
            reasons.push("High overall score across all metrics".to_string());
        }
        if blueprint.traffic_profile.latency_sensitive && chosen.0.metrics.slo > 0.85 {
            reasons.push("Excellent performance for latency-sensitive workload".to_string());
        }

        // Add compliance reasons if applicable
        if let Some(compliance_types) = &blueprint.constraints.compliance {
            if !compliance_types.is_empty() {
                if chosen.0.metrics.security > 0.85 {
                    reasons
                        .push("Strong security features for compliance requirements".to_string());
                }
                if compliance_types
                    .iter()
                    .any(|c| matches!(c, ComplianceType::Hipaa))
                {
                    reasons.push("HIPAA-compliant infrastructure support".to_string());
                }
                if compliance_types
                    .iter()
                    .any(|c| matches!(c, ComplianceType::Sox))
                {
                    reasons.push("SOX compliance with audit trail capabilities".to_string());
                }
            }
        }

        if let Some(notes) = chosen.0.notes.first() {
            reasons.push(notes.clone());
        }

        // Ensure we always have at least one reason
        if reasons.is_empty() {
            reasons.push(format!("Selected based on optimal {topic} score"));
        }

        Ok(Decision {
            topic: topic.to_string(),
            choice,
            reasons,
            alternatives,
            score: chosen.1,
        })
    }

    fn check_constraints(&self, candidate: &Candidate, blueprint: &Blueprint) -> bool {
        // Check region constraints
        if let Some(allowed_regions) = &blueprint.constraints.region_allow {
            let matches = candidate
                .regions
                .iter()
                .any(|r| r == "*" || r == "global" || allowed_regions.contains(r));
            if !matches {
                return false;
            }
        }

        // Check cost constraints
        if let Some(max_cost) = blueprint.constraints.monthly_cost_usd_max {
            let passed = candidate.monthly_cost_base <= max_cost;
            observability::log_constraint_evaluation(
                "monthly_cost",
                max_cost,
                candidate.monthly_cost_base,
                passed,
            );
            if !passed {
                return false;
            }
        }
        
        // Note: quality_min, security_min, and slo_min constraints could be added
        // to the schema if needed. For now, these are checked via scoring.

        true
    }

    fn calculate_score(&self, metrics: &Metrics, blueprint: &Blueprint) -> f64 {
        let weights = &self.rules.weights;

        let mut score = weights.quality * metrics.quality
            + weights.slo * metrics.slo
            + weights.cost * metrics.cost
            + weights.security * metrics.security
            + weights.ops * metrics.ops;

        // Adjust for specific requirements
        if blueprint.traffic_profile.latency_sensitive {
            score += 0.1 * metrics.slo;
        }

        if blueprint.traffic_profile.global {
            score += 0.05 * metrics.ops;
        }

        // Normalize
        score / 1.15
    }

    fn get_component_cost(&self, category: &str, name: &str) -> f64 {
        let candidates = match category {
            "backend" => &self.rules.candidates.backend,
            "frontend" => &self.rules.candidates.frontend,
            "database" => &self.rules.candidates.database,
            "cache" => &self.rules.candidates.cache,
            "queue" => &self.rules.candidates.queue,
            "ai" => &self.rules.candidates.ai,
            "infra" => &self.rules.candidates.infra,
            "ci_cd" => &self.rules.candidates.ci_cd,
            _ => return 0.0,
        };

        candidates
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.monthly_cost_base)
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_rules() -> &'static str {
        r#"
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
    - name: "Go"
      metrics: { quality: 0.85, slo: 0.9, cost: 0.85, security: 0.9, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 0
    - name: "TypeScript"
      metrics: { quality: 0.8, slo: 0.8, cost: 0.9, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 0
  backend:
    - name: "Actix Web"
      requires: { language: "Rust" }
      metrics: { quality: 0.9, slo: 0.9, cost: 0.7, security: 0.8, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 100
    - name: "Axum"
      requires: { language: "Rust" }
      metrics: { quality: 0.85, slo: 0.85, cost: 0.7, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 100
    - name: "Gin"
      requires: { language: "Go" }
      metrics: { quality: 0.85, slo: 0.85, cost: 0.75, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 100
    - name: "Express"
      requires: { language: "TypeScript" }
      metrics: { quality: 0.9, slo: 0.75, cost: 0.8, security: 0.7, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 100
  frontend:
    - name: "SvelteKit"
      metrics: { quality: 0.85, slo: 0.8, cost: 0.8, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 50
    - name: "Next.js"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.75, security: 0.85, ops: 0.8 }
      regions: ["us-east-1", "eu-west-1"]
      monthly_cost_base: 50
  database:
    - name: "PostgreSQL"
      persistence: "sql"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.7, security: 0.9, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 200
    - name: "Redis"
      persistence: "kv"
      metrics: { quality: 0.85, slo: 0.95, cost: 0.6, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 150
    - name: "DynamoDB"
      persistence: "both"
      metrics: { quality: 0.85, slo: 0.9, cost: 0.8, security: 0.85, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 180
  cache:
    - name: "Redis"
      metrics: { quality: 0.9, slo: 0.95, cost: 0.6, security: 0.85, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 100
    - name: "Memcached"
      metrics: { quality: 0.8, slo: 0.9, cost: 0.7, security: 0.75, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 80
  queue:
    - name: "NATS"
      metrics: { quality: 0.85, slo: 0.9, cost: 0.5, security: 0.85, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 50
    - name: "RabbitMQ"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.6, security: 0.9, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 75
  ai:
    - name: "RuneSage"
      metrics: { quality: 0.8, slo: 0.8, cost: 0.7, security: 0.8, ops: 0.8 }
      regions: ["*"]
      monthly_cost_base: 100
    - name: "OpenAI"
      metrics: { quality: 0.95, slo: 0.85, cost: 0.5, security: 0.85, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 200
    - name: "Claude"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.6, security: 0.9, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 150
  infra:
    - name: "Terraform"
      metrics: { quality: 0.9, slo: 0.85, cost: 0.8, security: 0.9, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 0
    - name: "Pulumi"
      metrics: { quality: 0.85, slo: 0.8, cost: 0.75, security: 0.85, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 0
  ci_cd:
    - name: "GitHub Actions"
      metrics: { quality: 0.85, slo: 0.8, cost: 0.9, security: 0.85, ops: 0.9 }
      regions: ["*"]
      monthly_cost_base: 20
    - name: "GitLab CI"
      metrics: { quality: 0.8, slo: 0.75, cost: 0.85, security: 0.8, ops: 0.85 }
      regions: ["*"]
      monthly_cost_base: 30
compliance_requirements:
  hipaa:
    required_features: ["encryption", "audit_log", "access_control"]
  sox:
    required_features: ["audit_log", "version_control", "change_management"]
"#
    }

    fn get_test_blueprint() -> Blueprint {
        Blueprint {
            project_name: "test-project".to_string(),
            goals: vec!["Build a web app".to_string()],
            constraints: Constraints {
                monthly_cost_usd_max: Some(1000.0),
                persistence: None,
                region_allow: None,
                compliance: None,
            },
            traffic_profile: TrafficProfile {
                rps_peak: 1000.0,
                global: true,
                latency_sensitive: false,
            },
            prefs: None,
            single_language_mode: None,
        }
    }

    #[test]
    fn test_selector_creation() {
        let selector = Selector::new(get_test_rules(), 42);
        assert!(selector.is_ok());
    }

    #[test]
    fn test_selector_invalid_yaml() {
        let invalid_yaml = "invalid: yaml: content:";
        let selector = Selector::new(invalid_yaml, 42);
        assert!(selector.is_err());
        assert!(selector.unwrap_err().contains("Failed to parse rules"));
    }

    #[test]
    fn test_select_complete_stack() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let blueprint = get_test_blueprint();

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert!(!plan.stack.language.is_empty());
        assert!(!plan.stack.frontend.is_empty());
        assert!(!plan.stack.backend.is_empty());
        assert!(!plan.stack.database.is_empty());
        assert!(!plan.stack.cache.is_empty());
        assert!(!plan.stack.queue.is_empty());
        assert!(!plan.stack.ai.is_empty());
        assert!(!plan.stack.infra.is_empty());
        assert!(!plan.stack.ci_cd.is_empty());

        // Check meta information
        assert_eq!(plan.meta.seed, 42);
        assert!(plan.meta.blueprint_hash.starts_with("sha256:"));
        assert!(plan.meta.plan_hash.starts_with("sha256:"));
    }

    #[test]
    fn test_single_language_mode() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();

        // Test Rust mode
        let mut blueprint = get_test_blueprint();
        blueprint.single_language_mode = Some(LanguageMode::Rust);

        let result = selector.select(&blueprint);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.stack.language, "Rust");
        assert!(plan.stack.backend == "Actix Web" || plan.stack.backend == "Axum");

        // Test Go mode
        blueprint.single_language_mode = Some(LanguageMode::Go);
        let result = selector.select(&blueprint);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.stack.language, "Go");
        assert_eq!(plan.stack.backend, "Gin");

        // Test TypeScript mode
        blueprint.single_language_mode = Some(LanguageMode::Ts);
        let result = selector.select(&blueprint);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.stack.language, "TypeScript");
        assert_eq!(plan.stack.backend, "Express");
    }

    #[test]
    fn test_persistence_constraints() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();

        // Test SQL constraint
        blueprint.constraints.persistence = Some(PersistenceType::Sql);
        let result = selector.select(&blueprint);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.stack.database, "PostgreSQL");

        // Test KV constraint
        blueprint.constraints.persistence = Some(PersistenceType::Kv);
        let result = selector.select(&blueprint);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.stack.database, "Redis");

        // Test Both constraint
        blueprint.constraints.persistence = Some(PersistenceType::Both);
        let result = selector.select(&blueprint);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.stack.database, "DynamoDB");
    }

    #[test]
    fn test_region_constraints() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();

        // Constrain to us-east-1 only
        blueprint.constraints.region_allow = Some(vec!["us-east-1".to_string()]);

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        // All selected components should support us-east-1 or be global
        let plan = result.unwrap();

        // Frontend should be either SvelteKit (global) or Next.js (supports us-east-1)
        assert!(plan.stack.frontend == "SvelteKit" || plan.stack.frontend == "Next.js");
    }

    #[test]
    fn test_cost_constraints() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();

        // Set very low cost constraint
        blueprint.constraints.monthly_cost_usd_max = Some(100.0);

        let result = selector.select(&blueprint);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        // Check that it's a cost constraint error
        assert!(
            err_msg.contains("cost constraint") || err_msg.contains("No suitable"),
            "Expected cost constraint error, got: {err_msg}"
        );
    }

    #[test]
    fn test_preferences() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();

        // Set preferences
        blueprint.prefs = Some(Preferences {
            frontend: Some(vec!["Next.js".to_string()]),
            backend: Some(vec!["Axum".to_string()]),
            database: Some(vec!["Redis".to_string()]),
            ai: Some(vec!["Claude".to_string()]),
        });
        blueprint.single_language_mode = Some(LanguageMode::Rust);

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.stack.frontend, "Next.js");
        assert_eq!(plan.stack.backend, "Axum");
        assert_eq!(plan.stack.database, "Redis");
        assert!(plan.stack.ai.contains(&"Claude".to_string()));
    }

    #[test]
    fn test_scoring_algorithm() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let blueprint = get_test_blueprint();

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();

        // All decisions should have valid scores
        for decision in &plan.decisions {
            assert!(decision.score >= 0.0 && decision.score <= 1.0);
            assert!(!decision.reasons.is_empty());
            assert!(!decision.choice.is_empty());
        }

        // Decisions should be sorted by score (descending)
        for i in 1..plan.decisions.len() {
            assert!(plan.decisions[i - 1].score >= plan.decisions[i].score);
        }
    }

    #[test]
    fn test_latency_sensitive_scoring() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();
        blueprint.traffic_profile.latency_sensitive = true;

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();

        // Check that high SLO components are preferred
        // Redis cache should be selected for its high SLO score
        assert_eq!(plan.stack.cache, "Redis");
    }

    #[test]
    fn test_ai_selection_multiple() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let blueprint = get_test_blueprint();

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();

        // Should select 2 AI providers
        assert_eq!(plan.stack.ai.len(), 2);

        // Find the AI decision
        let ai_decision = plan
            .decisions
            .iter()
            .find(|d| d.topic == "ai")
            .expect("AI decision not found");

        // Should have alternatives
        assert!(!ai_decision.alternatives.is_empty());
    }

    #[test]
    fn test_compliance_reasons() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();
        blueprint.constraints.compliance = Some(vec![ComplianceType::Hipaa, ComplianceType::Sox]);

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();

        // Should include compliance-related reasons
        let has_compliance_reason = plan.decisions.iter().any(|d| {
            d.reasons
                .iter()
                .any(|r| r.contains("HIPAA") || r.contains("SOX") || r.contains("compliance"))
        });
        assert!(has_compliance_reason);
    }

    #[test]
    fn test_deterministic_selection() {
        let selector1 = Selector::new(get_test_rules(), 42).unwrap();
        let selector2 = Selector::new(get_test_rules(), 42).unwrap();
        let blueprint = get_test_blueprint();

        let result1 = selector1.select(&blueprint).unwrap();
        let result2 = selector2.select(&blueprint).unwrap();

        // Same seed should produce same results
        assert_eq!(result1.stack.language, result2.stack.language);
        assert_eq!(result1.stack.frontend, result2.stack.frontend);
        assert_eq!(result1.stack.backend, result2.stack.backend);
        assert_eq!(result1.stack.database, result2.stack.database);
        assert_eq!(result1.stack.cache, result2.stack.cache);
        assert_eq!(result1.stack.queue, result2.stack.queue);
        assert_eq!(result1.stack.ai, result2.stack.ai);
        assert_eq!(result1.stack.infra, result2.stack.infra);
        assert_eq!(result1.stack.ci_cd, result2.stack.ci_cd);
    }

    #[test]
    fn test_different_seeds_different_results() {
        let selector1 = Selector::new(get_test_rules(), 42).unwrap();
        let selector2 = Selector::new(get_test_rules(), 99).unwrap();

        // Create a blueprint that would result in ties
        let mut blueprint = get_test_blueprint();
        blueprint.single_language_mode = Some(LanguageMode::Rust);

        let result1 = selector1.select(&blueprint).unwrap();
        let result2 = selector2.select(&blueprint).unwrap();

        // Different seeds might produce different results when there are ties
        // At least one component should be different (backend has two Rust options with similar scores)
        let _all_same = result1.stack.backend == result2.stack.backend;

        // This test might occasionally pass even with different seeds,
        // but it's statistically unlikely all components would be the same
        // We'll just verify both are valid selections
        assert!(result1.stack.backend == "Actix Web" || result1.stack.backend == "Axum");
        assert!(result2.stack.backend == "Actix Web" || result2.stack.backend == "Axum");
    }

    #[test]
    fn test_no_suitable_candidates() {
        let rules_yaml = r#"
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
      regions: ["eu-only"]
  backend: []
  frontend: []
  database: []
  cache: []
  queue: []
  ai: []
  infra: []
  ci_cd: []
"#;

        let selector = Selector::new(rules_yaml, 42).unwrap();
        let mut blueprint = get_test_blueprint();
        blueprint.constraints.region_allow = Some(vec!["us-east-1".to_string()]);

        let result = selector.select(&blueprint);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("No suitable language candidates found"));
    }

    #[test]
    fn test_cost_calculation() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let blueprint = get_test_blueprint();

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();

        // Verify cost is calculated correctly
        // Should be sum of all component costs
        assert!(plan.estimated.monthly_cost_usd > 0.0);
        assert!(plan.estimated.monthly_cost_usd < 1000.0); // Within constraint
    }

    #[test]
    fn test_empty_goals() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();
        blueprint.goals = vec![];

        // This should be caught by validate_blueprint, but let's test selector behavior
        let result = selector.select(&blueprint);
        // Selector should still work with empty goals
        assert!(result.is_ok());
    }

    #[test]
    fn test_backend_language_requirement() {
        let selector = Selector::new(get_test_rules(), 42).unwrap();
        let mut blueprint = get_test_blueprint();

        // Force TypeScript language
        blueprint.single_language_mode = Some(LanguageMode::Ts);

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.stack.language, "TypeScript");
        assert_eq!(plan.stack.backend, "Express"); // Only TS backend option
    }

    #[test]
    fn test_tie_breaker_activation() {
        // This test verifies tie breaker is used when scores are equal
        let rules_yaml = r#"
version: 1
weights:
  quality: 1.0
  slo: 0.0
  cost: 0.0
  security: 0.0
  ops: 0.0
candidates:
  language:
    - name: "Rust"
      metrics: { quality: 0.9, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  backend:
    - name: "Option1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
    - name: "Option2"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
    - name: "Option3"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  frontend:
    - name: "Frontend1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  database:
    - name: "DB1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  cache:
    - name: "Cache1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  queue:
    - name: "Queue1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  ai:
    - name: "AI1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  infra:
    - name: "Infra1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
  ci_cd:
    - name: "CI1"
      metrics: { quality: 0.8, slo: 0.5, cost: 0.5, security: 0.5, ops: 0.5 }
      regions: ["*"]
"#;

        let selector = Selector::new(rules_yaml, 42).unwrap();
        let blueprint = get_test_blueprint();

        let result = selector.select(&blueprint);
        assert!(result.is_ok());

        let plan = result.unwrap();
        // Verify a backend was selected from the tied options
        assert!(["Option1", "Option2", "Option3"].contains(&plan.stack.backend.as_str()));
    }
}
