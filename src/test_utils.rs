// Test utilities for main.rs testing
#[cfg(test)]
pub mod test_helpers {
    use std::fs;
    use tempfile::TempDir;

    #[allow(dead_code)]
    pub fn create_test_blueprint(dir: &TempDir, filename: &str, content: &str) -> String {
        let file_path = dir.path().join(filename);
        fs::write(&file_path, content).unwrap();
        file_path.to_str().unwrap().to_string()
    }

    #[allow(dead_code)]
    pub fn create_test_rules(dir: &TempDir) -> String {
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
        rules_path.to_str().unwrap().to_string()
    }
}
