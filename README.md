# Runeforge

A CLI tool that reads Blueprint requirements and returns optimal technology stack recommendations as JSON.

[![CI](https://github.com/NishizukaKoichi/Runeforge/actions/workflows/ci.yml/badge.svg)](https://github.com/NishizukaKoichi/Runeforge/actions/workflows/ci.yml)
[![Security Audit](https://github.com/NishizukaKoichi/Runeforge/actions/workflows/security.yml/badge.svg)](https://github.com/NishizukaKoichi/Runeforge/actions/workflows/security.yml)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

## Features

- **Blueprint Validation**: Validates input requirements against JSON schema
- **Optimal Stack Selection**: Multi-metric weighted scoring algorithm
- **Deterministic Output**: Seed-based reproducible results
- **Constraint Support**: Cost limits, region restrictions, compliance requirements
- **Flexible Input**: Supports both YAML and JSON input formats

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/NishizukaKoichi/Runeforge.git
cd Runeforge

# Build and install
cargo install --path .
```

### Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/NishizukaKoichi/Runeforge/releases).

## Usage

```bash
runeforge plan \
  -f blueprint.yaml   # Input blueprint file (required)
  --seed 42           # Random seed for deterministic output (default: 42)
  --out plan.json     # Output file (default: stdout)
  --strict            # Enable strict schema validation
```

### Example

```bash
# Generate a technology stack plan
runeforge plan -f examples/baseline.yaml --seed 42 --out plan.json

# View the output
cat plan.json
```

## Input Schema

The input blueprint must conform to the schema defined in [`schemas/blueprint.schema.json`](schemas/blueprint.schema.json).

### Minimal Example

```yaml
project_name: "my-app"
goals:
  - "Build a web application"
constraints:
  monthly_cost_usd_max: 500
traffic_profile:
  rps_peak: 1000
  global: false
  latency_sensitive: false
```

### Full Example

```yaml
project_name: "enterprise-app"
goals:
  - "Build scalable e-commerce platform"
  - "Support global users"
  - "Ensure HIPAA compliance"
constraints:
  monthly_cost_usd_max: 2000
  persistence: "sql"
  region_allow: ["us-east", "eu-west"]
  compliance: ["hipaa", "audit-log", "sbom"]
traffic_profile:
  rps_peak: 50000
  global: true
  latency_sensitive: true
prefs:
  backend: ["Rust", "Go"]
  database: ["PostgreSQL", "MySQL"]
single_language_mode: "rust"
```

## Output Schema

The output conforms to [`schemas/stack.schema.json`](schemas/stack.schema.json):

```json
{
  "decisions": [
    {
      "topic": "backend",
      "choice": "Actix Web",
      "reasons": ["High throughput", "Rust ecosystem"],
      "alternatives": ["Axum"],
      "score": 0.862
    }
  ],
  "stack": {
    "language": "Rust",
    "frontend": "SvelteKit",
    "backend": "Actix Web",
    "database": "PostgreSQL",
    "cache": "Redis",
    "queue": "NATS JetStream",
    "ai": ["OpenAI GPT-4o"],
    "infra": "Terraform + Cloudflare Workers",
    "ci_cd": "GitHub Actions"
  },
  "estimated": {
    "monthly_cost_usd": 450
  },
  "meta": {
    "seed": 42,
    "blueprint_hash": "sha256:...",
    "plan_hash": "sha256:..."
  }
}
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Input schema validation error |
| 2 | Output schema validation error |
| 3 | No suitable stack found |

## API Reference

### Library Usage

```rust
use runeforge::{schema, selector::Selector};

// Validate and parse blueprint
let blueprint = schema::validate_blueprint(&input_yaml)?;

// Create selector with rules and seed
let selector = Selector::new(&rules_yaml, seed)?;

// Generate technology stack plan
let plan = selector.select(&blueprint)?;

// Validate output
schema::validate_stack_plan(&plan)?;
```

### Key Types

- `Blueprint`: Input requirements structure
- `StackPlan`: Output technology stack recommendation
- `Decision`: Individual technology choice with reasoning
- `Selector`: Main selection engine

## Configuration

### Rules Configuration

Technology candidates and scoring weights are defined in `resources/rules.yaml`:

```yaml
version: 1
weights:
  quality: 0.30
  slo: 0.25
  cost: 0.20
  security: 0.15
  ops: 0.10
```

## Development

### Prerequisites

- Rust 1.82+ (MSRV)
- cargo-audit for security scanning

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with example
cargo run -- plan -f examples/baseline.yaml
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test acceptance

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html --output-dir coverage
```

## Performance

Typical performance characteristics:

- P95 latency: < 100ms for standard blueprints
- Memory usage: < 50MB
- Supports blueprints up to 10MB

## Security

- All dependencies are audited via `cargo audit`
- SBOM (Software Bill of Materials) is generated for each release
- Signed container images available via cosign

## License

This project is dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.