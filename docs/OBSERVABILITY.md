# Observability Guide

Runeforge includes built-in observability features for monitoring and debugging technology stack selection.

## Features

### Structured Logging

Runeforge uses the `tracing` crate for structured logging with JSON output format.

#### Enabling Logs

Set the `RUST_LOG` environment variable:

```bash
# Basic info logging
export RUST_LOG=info

# Debug logging for runeforge
export RUST_LOG=info,runeforge=debug

# Trace-level logging for deep debugging
export RUST_LOG=trace
```

#### Log Events

The following events are logged:

- **Blueprint Validation**: Content length, format (YAML/JSON)
- **Selection Start**: Project name, random seed
- **Constraint Evaluation**: Type, required vs actual values, pass/fail
- **Scoring Details**: Component, candidate, score breakdown
- **Final Selection**: Complete stack summary, total cost
- **Errors**: Context and error details

### Metrics Collection

Runeforge tracks the following metrics:

- `blueprint_validations`: Total number of blueprint validations
- `successful_selections`: Count of successful stack selections
- `failed_selections`: Count of failed stack selections
- `average_selection_time_ms`: Rolling average of selection duration
- `constraint_violations`: Number of constraint violations encountered

#### Exporting Metrics

```rust
use runeforge::metrics_handler::MetricsHandler;

let handler = MetricsHandler::new();
let metrics = handler.get_metrics();

// Export as Prometheus format
let prometheus_output = handler.export_prometheus();

// Export as JSON
let json_output = handler.export_json();
```

### Integration with Observability Platforms

#### OpenTelemetry

Runeforge's structured logs can be collected by OpenTelemetry collectors:

```yaml
# otel-collector-config.yaml
receivers:
  filelog:
    include: ["/var/log/runeforge/*.log"]
    operators:
      - type: json_parser

exporters:
  otlp:
    endpoint: "your-otlp-endpoint:4317"

service:
  pipelines:
    logs:
      receivers: [filelog]
      exporters: [otlp]
```

#### Datadog

For Datadog integration, use the JSON log format:

```bash
RUST_LOG=info runeforge plan -f blueprint.yaml 2>&1 | tee runeforge.log
```

Configure Datadog Agent to collect JSON logs:

```yaml
logs:
  - type: file
    path: /var/log/runeforge/*.log
    service: runeforge
    source: rust
    sourcecategory: application
```

#### Grafana Loki

For Grafana Loki, use promtail to ship logs:

```yaml
# promtail-config.yaml
clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: runeforge
    static_configs:
      - targets:
          - localhost
        labels:
          job: runeforge
          __path__: /var/log/runeforge/*.log
    pipeline_stages:
      - json:
          expressions:
            level: level
            timestamp: timestamp
            message: message
```

### Performance Monitoring

Monitor selection performance with duration spans:

```rust
use runeforge::observability::DurationSpan;

let _span = DurationSpan::new("custom_operation");
// Your code here
// Duration is automatically logged when _span is dropped
```

### Debugging Tips

1. **Enable debug logs** for detailed scoring information:
   ```bash
   RUST_LOG=debug runeforge plan -f blueprint.yaml
   ```

2. **Track constraint violations** to understand why selections fail:
   - Look for "Constraint not satisfied" warnings
   - Check the constraint type and actual vs required values

3. **Monitor selection performance**:
   - Watch the "Operation completed" logs with duration_ms
   - Use metrics to track average selection times

4. **Analyze scoring details**:
   - Debug logs show score breakdowns for each candidate
   - Useful for understanding why certain technologies were chosen

### Example: Running with Full Observability

```bash
# Set up environment
export RUST_LOG=info,runeforge=debug

# Run with observability demo
cargo run --example observability_demo

# Run actual selection with detailed logs
runeforge plan -f blueprint.yaml --out plan.json 2>&1 | tee runeforge-$(date +%Y%m%d-%H%M%S).log
```

The logs can then be analyzed with tools like `jq`:

```bash
# Extract all constraint violations
cat runeforge-*.log | jq 'select(.fields.context == "constraint_evaluation" and .fields.passed == false)'

# Get average selection time
cat runeforge-*.log | jq 'select(.fields.operation == "run_plan") | .fields.duration_ms' | awk '{sum+=$1; count++} END {print sum/count}'
```