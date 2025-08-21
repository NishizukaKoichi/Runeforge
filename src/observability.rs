#[cfg(feature = "std")]
use tracing::{debug, error, info, instrument, warn};

#[cfg(feature = "std")]
use std::time::Instant;

/// Initialize observability with structured logging
#[cfg(feature = "std")]
pub fn init_observability() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    // Check if we're in test mode by looking for TEST_MODE env var
    let is_test_mode =
        std::env::var("CARGO_CFG_TEST").is_ok() || std::env::var("TEST_MODE").is_ok() || cfg!(test);

    // Get log level from env var or default to INFO (or ERROR for tests)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if is_test_mode {
            EnvFilter::new("error")
        } else {
            EnvFilter::new("info")
        }
    });

    if is_test_mode {
        // Use compact format for tests
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .compact();

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    } else {
        // Use JSON format for production
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .json();

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }

    Ok(())
}

/// A span guard that logs duration when dropped
#[cfg(feature = "std")]
pub struct DurationSpan {
    name: &'static str,
    start: Instant,
}

#[cfg(feature = "std")]
impl DurationSpan {
    pub fn new(name: &'static str) -> Self {
        info!(operation = name, "Starting operation");
        Self {
            name,
            start: Instant::now(),
        }
    }
}

#[cfg(feature = "std")]
impl Drop for DurationSpan {
    fn drop(&mut self) {
        let duration_ms = self.start.elapsed().as_millis();
        info!(
            operation = self.name,
            duration_ms = duration_ms,
            "Operation completed"
        );
    }
}

/// Log blueprint validation
#[cfg(feature = "std")]
#[instrument]
pub fn log_blueprint_validation(content_len: usize, format: &str) {
    info!(
        content_length = content_len,
        format = format,
        "Validating blueprint"
    );
}

/// Log selection process
#[cfg(feature = "std")]
#[instrument]
pub fn log_selection_start(project_name: &str, seed: u64) {
    info!(
        project_name = project_name,
        seed = seed,
        "Starting technology stack selection"
    );
}

/// Log constraint evaluation
#[cfg(feature = "std")]
#[instrument]
pub fn log_constraint_evaluation(
    constraint_type: &str,
    required_value: f64,
    actual_value: f64,
    passed: bool,
) {
    if passed {
        debug!(
            constraint_type = constraint_type,
            required = required_value,
            actual = actual_value,
            "Constraint satisfied"
        );
    } else {
        warn!(
            constraint_type = constraint_type,
            required = required_value,
            actual = actual_value,
            "Constraint not satisfied"
        );
    }
}

/// Log scoring details
#[cfg(feature = "std")]
#[instrument]
pub fn log_scoring(component: &str, candidate: &str, score: f64, breakdown: &[(String, f64)]) {
    debug!(
        component = component,
        candidate = candidate,
        total_score = score,
        score_breakdown = ?breakdown,
        "Calculated candidate score"
    );
}

/// Log final selection
#[cfg(feature = "std")]
#[instrument]
pub fn log_final_selection(stack_summary: &[(String, String)], total_cost: f64) {
    info!(
        stack = ?stack_summary,
        total_monthly_cost_usd = total_cost,
        "Technology stack selected"
    );
}

/// Log errors with context
#[cfg(feature = "std")]
#[instrument]
pub fn log_error(context: &str, error: &str) {
    error!(context = context, error = error, "Error occurred");
}

/// Metrics collection structure
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct Metrics {
    pub blueprint_validations: u64,
    pub successful_selections: u64,
    pub failed_selections: u64,
    pub average_selection_time_ms: f64,
    pub constraint_violations: u64,
}

#[cfg(feature = "std")]
impl Default for Metrics {
    fn default() -> Self {
        Self {
            blueprint_validations: 0,
            successful_selections: 0,
            failed_selections: 0,
            average_selection_time_ms: 0.0,
            constraint_violations: 0,
        }
    }
}

#[cfg(feature = "std")]
impl Metrics {
    pub fn record_validation(&mut self) {
        self.blueprint_validations += 1;
    }

    pub fn record_selection(&mut self, success: bool, duration_ms: u128) {
        if success {
            self.successful_selections += 1;
        } else {
            self.failed_selections += 1;
        }

        // Update rolling average
        let total_selections = self.successful_selections + self.failed_selections;
        let current_total = self.average_selection_time_ms * (total_selections - 1) as f64;
        self.average_selection_time_ms =
            (current_total + duration_ms as f64) / total_selections as f64;
    }

    pub fn record_constraint_violation(&mut self) {
        self.constraint_violations += 1;
    }

    pub fn log_summary(&self) {
        info!(
            blueprint_validations = self.blueprint_validations,
            successful_selections = self.successful_selections,
            failed_selections = self.failed_selections,
            average_selection_time_ms = self.average_selection_time_ms,
            constraint_violations = self.constraint_violations,
            "Metrics summary"
        );
    }
}

// No-op implementations for no_std
#[cfg(not(feature = "std"))]
pub fn init_observability() -> Result<(), &'static str> {
    Ok(())
}

#[cfg(not(feature = "std"))]
pub fn log_blueprint_validation(_content_len: usize, _format: &str) {}

#[cfg(not(feature = "std"))]
pub fn log_selection_start(_project_name: &str, _seed: u64) {}

#[cfg(not(feature = "std"))]
pub fn log_constraint_evaluation(
    _constraint_type: &str,
    _required_value: f64,
    _actual_value: f64,
    _passed: bool,
) {
}

#[cfg(not(feature = "std"))]
pub fn log_scoring(
    _component: &str,
    _candidate: &str,
    _score: f64,
    _breakdown: &[(alloc::string::String, f64)],
) {
}

#[cfg(not(feature = "std"))]
pub fn log_final_selection(
    _stack_summary: &[(alloc::string::String, alloc::string::String)],
    _total_cost: f64,
) {
}

#[cfg(not(feature = "std"))]
pub fn log_error(_context: &str, _error: &str) {}
