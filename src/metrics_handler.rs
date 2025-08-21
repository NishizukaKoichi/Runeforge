#[cfg(feature = "std")]
use crate::observability::Metrics;
use std::sync::{Arc, Mutex};

/// A simple metrics handler that can be used to expose metrics
#[cfg(feature = "std")]
pub struct MetricsHandler {
    metrics: Arc<Mutex<Metrics>>,
}

#[cfg(feature = "std")]
impl MetricsHandler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Metrics::default())),
        }
    }
    
    pub fn get_metrics(&self) -> Arc<Mutex<Metrics>> {
        Arc::clone(&self.metrics)
    }
    
    /// Export metrics in Prometheus format
    pub fn export_prometheus(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        
        format!(
            r#"# HELP runeforge_blueprint_validations_total Total number of blueprint validations
# TYPE runeforge_blueprint_validations_total counter
runeforge_blueprint_validations_total {}

# HELP runeforge_successful_selections_total Total number of successful stack selections
# TYPE runeforge_successful_selections_total counter
runeforge_successful_selections_total {}

# HELP runeforge_failed_selections_total Total number of failed stack selections
# TYPE runeforge_failed_selections_total counter
runeforge_failed_selections_total {}

# HELP runeforge_selection_duration_milliseconds Average duration of stack selection
# TYPE runeforge_selection_duration_milliseconds gauge
runeforge_selection_duration_milliseconds {}

# HELP runeforge_constraint_violations_total Total number of constraint violations
# TYPE runeforge_constraint_violations_total counter
runeforge_constraint_violations_total {}
"#,
            metrics.blueprint_validations,
            metrics.successful_selections,
            metrics.failed_selections,
            metrics.average_selection_time_ms,
            metrics.constraint_violations
        )
    }
    
    /// Export metrics in JSON format
    pub fn export_json(&self) -> String {
        let metrics = self.metrics.lock().unwrap();
        
        serde_json::json!({
            "blueprint_validations": metrics.blueprint_validations,
            "successful_selections": metrics.successful_selections,
            "failed_selections": metrics.failed_selections,
            "average_selection_time_ms": metrics.average_selection_time_ms,
            "constraint_violations": metrics.constraint_violations
        }).to_string()
    }
}