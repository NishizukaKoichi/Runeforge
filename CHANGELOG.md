# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-08-21

### Added
- Initial implementation of Runeforge CLI
- Blueprint validation with JSON schema support for YAML and JSON formats
- Multi-metric weighted scoring algorithm (quality, SLO, cost, security, ops)
- Constraint filtering system (cost limits, region restrictions, compliance)
- Deterministic output generation using seed-based RNG
- Support for technology preferences and single-language mode
- Comprehensive test suite (unit, integration, acceptance, property-based)
- Exit codes for different error scenarios (0-3)
- Example blueprints (baseline, latency-sensitive, compliance-heavy)
- JSON schemas for input validation and output structure
- Rules configuration system for technology candidates
- SBOM generation and container signing workflows
- Multi-architecture Docker support
- Security scanning with cargo-audit and cargo-deny

### Security
- Implemented secure dependency management with deny.toml
- Added automated security scanning in CI pipeline
- Configured SBOM generation for supply chain transparency

[Unreleased]: https://github.com/NishizukaKoichi/Runeforge/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/NishizukaKoichi/Runeforge/releases/tag/v0.1.0