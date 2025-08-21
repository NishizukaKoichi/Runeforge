# Runeforge Acceptance Tests

This directory contains comprehensive acceptance tests that verify the Runeforge implementation meets all requirements specified in SPEC.md.

## Test Structure

The acceptance tests are organized into the following modules:

- **test_schema_validation.rs**: Tests for input/output schema validation and file format support
- **test_determinism.rs**: Tests for deterministic output generation with seeds
- **test_constraints.rs**: Tests for cost, region, compliance, and language constraints
- **test_output_validation.rs**: Tests for output structure, required fields, and schema compliance
- **test_scoring_algorithm.rs**: Tests for scoring logic, preferences, and traffic profile handling

## Running the Tests

### Run all acceptance tests:
```bash
cargo test --test acceptance
```

### Run specific test module:
```bash
cargo test --test acceptance test_schema_validation
cargo test --test acceptance test_determinism
cargo test --test acceptance test_constraints
cargo test --test acceptance test_output_validation
cargo test --test acceptance test_scoring_algorithm
```

### Run with verbose output:
```bash
cargo test --test acceptance -- --nocapture
```

## Test Fixtures

Test fixtures are located in `tests/acceptance/fixtures/`:

### Valid Fixtures:
- `valid_baseline.yaml` - Standard web application requirements
- `valid_latency_sensitive.yaml` - Low-latency requirements (trading platform)
- `valid_compliance_heavy.yaml` - Healthcare app with compliance requirements
- `valid_minimal.yaml` - Minimal valid blueprint
- `valid_cost_constraint.yaml` - Budget-constrained application
- `valid_region_constraint.yaml` - Region-specific deployment

### Invalid Fixtures:
- `invalid_missing_required.yaml` - Missing required fields
- `invalid_schema_type.yaml` - Invalid data types

## Requirements Coverage

The `acceptance_map.json` file maps each requirement from SPEC.md to its corresponding tests:

- **Exit Codes**: Tests verify correct exit codes (0, 1, 2, 3)
- **Schema Validation**: Input and output schema compliance
- **Determinism**: Same input + seed produces identical output
- **Constraints**: Cost, region, compliance, and single-language mode
- **Scoring Algorithm**: Weighted scoring with quality, SLO, cost, security, and ops metrics
- **Decision Output**: Complete decision information with reasons and alternatives
- **CLI Flags**: --seed, --out, --strict functionality

## Test Dependencies

The acceptance tests require:
- `serde_json` for JSON parsing
- `jsonschema` for schema validation (optional, can use schemars)

## Adding New Tests

1. Create a new test function in the appropriate module
2. Add test fixtures if needed in `tests/acceptance/fixtures/`
3. Update `acceptance_map.json` to map the requirement to the test
4. Run the test to ensure it passes

## Integration with CI

These tests should be run as part of the CI pipeline to ensure:
- All requirements are continuously validated
- No regressions are introduced
- Output format remains stable