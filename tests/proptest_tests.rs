use proptest::prelude::*;
use rand::Rng;
use runeforge::schema::*;
use runeforge::util::*;
use std::collections::HashSet;

// Property-based test strategies
prop_compose! {
    fn arb_traffic_profile()(
        rps_peak in 0.0..10000.0,
        global in any::<bool>(),
        latency_sensitive in any::<bool>()
    ) -> TrafficProfile {
        TrafficProfile { rps_peak, global, latency_sensitive }
    }
}

prop_compose! {
    fn arb_constraints()(
        monthly_cost_usd_max in prop::option::of(0.0..10000.0),
        persistence in prop::option::of(prop_oneof![
            Just(PersistenceType::Kv),
            Just(PersistenceType::Sql),
            Just(PersistenceType::Both),
        ]),
        region_allow in prop::option::of(prop::collection::vec(
            prop_oneof![
                Just("us-east-1".to_string()),
                Just("us-west-2".to_string()),
                Just("eu-west-1".to_string()),
                Just("ap-northeast-1".to_string()),
            ],
            0..3
        )),
        compliance in prop::option::of(prop::collection::vec(
            prop_oneof![
                Just(ComplianceType::AuditLog),
                Just(ComplianceType::Sbom),
                Just(ComplianceType::Pci),
                Just(ComplianceType::Sox),
                Just(ComplianceType::Hipaa),
            ],
            0..5
        ))
    ) -> Constraints {
        Constraints {
            monthly_cost_usd_max,
            persistence,
            region_allow,
            compliance,
        }
    }
}

prop_compose! {
    fn arb_preferences()(
        frontend in prop::option::of(prop::collection::vec("[A-Za-z]+", 0..3)),
        backend in prop::option::of(prop::collection::vec("[A-Za-z]+", 0..3)),
        database in prop::option::of(prop::collection::vec("[A-Za-z]+", 0..3)),
        ai in prop::option::of(prop::collection::vec("[A-Za-z]+", 0..3))
    ) -> Preferences {
        Preferences { frontend, backend, database, ai }
    }
}

prop_compose! {
    fn arb_blueprint()(
        project_name in "[A-Za-z][A-Za-z0-9-]{0,50}",
        goals in prop::collection::vec("[A-Za-z ]{5,50}", 1..5),
        constraints in arb_constraints(),
        traffic_profile in arb_traffic_profile(),
        prefs in prop::option::of(arb_preferences()),
        single_language_mode in prop::option::of(prop_oneof![
            Just(LanguageMode::Rust),
            Just(LanguageMode::Go),
            Just(LanguageMode::Ts),
        ])
    ) -> Blueprint {
        Blueprint {
            project_name,
            goals,
            constraints,
            traffic_profile,
            prefs,
            single_language_mode,
        }
    }
}

proptest! {
    #[test]
    fn test_blueprint_validation_doesnt_panic(blueprint in arb_blueprint()) {
        // Validation should either succeed or return an error, never panic
        let json = serde_json::to_string(&blueprint).unwrap();
        let _ = validate_blueprint(&json);
    }

    #[test]
    fn test_hash_determinism(data in prop::collection::vec(any::<u8>(), 0..1000)) {
        let json = serde_json::to_string(&data).unwrap();
        let hash1 = calculate_blueprint_hash(&json);
        let hash2 = calculate_blueprint_hash(&json);

        // Same input should always produce same hash
        prop_assert_eq!(&hash1, &hash2);

        // Hash should have correct format
        prop_assert!(hash1.starts_with("sha256:"));
        prop_assert_eq!(hash1.len(), 71); // "sha256:" + 64 hex chars
    }

    #[test]
    fn test_tie_breaker_properties(
        topic in "[a-z]+",
        seed in any::<u64>(),
        num_candidates in 1..20usize
    ) {
        let candidates: Vec<String> = (0..num_candidates)
            .map(|i| format!("Option{i}"))
            .collect();

        // Should always return one of the candidates
        let result = tie_breaker(&topic, seed, candidates.clone());
        prop_assert!(candidates.contains(&result));

        // Should be deterministic
        let result2 = tie_breaker(&topic, seed, candidates.clone());
        prop_assert_eq!(result, result2);
    }

    #[test]
    fn test_tie_breaker_distribution(seed in any::<u64>()) {
        let candidates = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let mut results = HashSet::new();

        // Try with different topics
        for i in 0..10 {
            let topic = format!("topic{i}");
            let result = tie_breaker(&topic, seed, candidates.clone());
            results.insert(result);
        }

        // With 10 different topics, we should see some variety
        // (not all results should be the same)
        prop_assert!(results.len() > 1);
    }

    #[test]
    fn test_stack_plan_validation_properties(
        monthly_cost in 0.0..10000.0f64,
        seed in any::<i64>(),
        num_decisions in 0..10usize
    ) {
        let decisions: Vec<Decision> = (0..num_decisions)
            .map(|i| Decision {
                topic: format!("topic{i}"),
                choice: format!("choice{i}"),
                reasons: vec!["reason".to_string()],
                alternatives: vec![],
                score: 0.5, // Valid score
            })
            .collect();

        let plan = StackPlan {
            decisions,
            stack: Stack {
                language: "Rust".to_string(),
                frontend: "SvelteKit".to_string(),
                backend: "Actix".to_string(),
                database: "PostgreSQL".to_string(),
                cache: "Redis".to_string(),
                queue: "NATS".to_string(),
                ai: vec!["AI1".to_string()],
                infra: "Terraform".to_string(),
                ci_cd: "GitHub".to_string(),
            },
            estimated: Estimated { monthly_cost_usd: monthly_cost },
            meta: Meta {
                seed,
                blueprint_hash: "sha256:test".to_string(),
                plan_hash: "sha256:test".to_string(),
            },
        };

        let result = validate_stack_plan(&plan);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn test_rng_properties(seed in any::<u64>()) {
        let mut rng1 = create_rng(seed);
        let mut rng2 = create_rng(seed);

        // Same seed should produce same sequence
        for _ in 0..10 {
            prop_assert_eq!(rng1.gen::<u32>(), rng2.gen::<u32>());
        }
    }

    #[test]
    fn test_score_bounds(
        quality in 0.0..1.0f64,
        slo in 0.0..1.0f64,
        cost in 0.0..1.0f64,
        security in 0.0..1.0f64,
        ops in 0.0..1.0f64
    ) {
        let decision = Decision {
            topic: "test".to_string(),
            choice: "test".to_string(),
            reasons: vec!["test".to_string()],
            alternatives: vec![],
            score: (quality + slo + cost + security + ops) / 5.0,
        };

        // Average of values in [0,1] should also be in [0,1]
        prop_assert!(decision.score >= 0.0);
        prop_assert!(decision.score <= 1.0);
    }

    #[test]
    fn test_blueprint_serialization_roundtrip(blueprint in arb_blueprint()) {
        // Serialize to JSON and back
        let json = serde_json::to_string(&blueprint).expect("Failed to serialize blueprint");
        let deserialized: Blueprint = serde_json::from_str(&json).expect("Failed to deserialize blueprint");

        // Should be equivalent
        prop_assert_eq!(blueprint.project_name, deserialized.project_name);
        prop_assert_eq!(blueprint.goals, deserialized.goals);
        // For floating point, allow small differences due to JSON serialization
        prop_assert!((blueprint.traffic_profile.rps_peak - deserialized.traffic_profile.rps_peak).abs() < 0.0001,
                     "rps_peak mismatch: {} vs {}", blueprint.traffic_profile.rps_peak, deserialized.traffic_profile.rps_peak);
        prop_assert_eq!(blueprint.traffic_profile.global, deserialized.traffic_profile.global);
        prop_assert_eq!(blueprint.traffic_profile.latency_sensitive, deserialized.traffic_profile.latency_sensitive);
    }

    #[test]
    fn test_yaml_json_equivalence(blueprint in arb_blueprint()) {
        // Serialize to both YAML and JSON
        let yaml = serde_yaml::to_string(&blueprint).unwrap();
        let json = serde_json::to_string(&blueprint).unwrap();

        // Both should deserialize to equivalent blueprints
        let from_yaml: Blueprint = serde_yaml::from_str(&yaml).unwrap();
        let from_json: Blueprint = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(from_yaml.project_name, from_json.project_name);
        prop_assert_eq!(from_yaml.goals, from_json.goals);
    }
}
