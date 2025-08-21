use hex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Calculate SHA256 hash of a serializable object and return as hex string
pub fn calculate_hash<T: Serialize>(data: &T) -> Result<String, String> {
    let json =
        serde_json::to_string(data).map_err(|e| format!("Failed to serialize for hashing: {e}"))?;

    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();

    Ok(hex::encode(result))
}

/// Create a deterministic RNG from a seed
pub fn create_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// Tie breaker for equal scores using deterministic randomization
pub fn tie_breaker(topic: &str, seed: u64, candidates: Vec<String>) -> String {
    if candidates.is_empty() {
        panic!("No candidates provided for tie breaker");
    }

    if candidates.len() == 1 {
        return candidates[0].clone();
    }

    // Create a deterministic seed based on topic and base seed
    let mut hasher = Sha256::new();
    hasher.update(topic.as_bytes());
    hasher.update(seed.to_le_bytes());
    let topic_hash = hasher.finalize();

    // Use first 8 bytes of hash as seed
    let topic_seed = u64::from_le_bytes(topic_hash[0..8].try_into().unwrap());
    let mut rng = create_rng(topic_seed);

    // Select random candidate
    let index = rng.gen_range(0..candidates.len());
    candidates[index].clone()
}

/// Calculate blueprint hash for deterministic identification
pub fn calculate_blueprint_hash(blueprint_content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(blueprint_content.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}

/// Calculate plan hash for output verification
pub fn calculate_plan_hash(plan_json: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(plan_json.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;

    #[test]
    fn test_deterministic_hash() {
        let data = Stack {
            language: "rust".to_string(),
            frontend: "SvelteKit".to_string(),
            backend: "Actix Web".to_string(),
            database: "PostgreSQL".to_string(),
            cache: "Redis".to_string(),
            queue: "NATS".to_string(),
            ai: vec!["RuneSage".to_string()],
            infra: "Terraform".to_string(),
            ci_cd: "GitHub Actions".to_string(),
        };

        let hash1 = calculate_hash(&data).unwrap();
        let hash2 = calculate_hash(&data).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_data() {
        let data1 = Stack {
            language: "rust".to_string(),
            frontend: "SvelteKit".to_string(),
            backend: "Actix Web".to_string(),
            database: "PostgreSQL".to_string(),
            cache: "Redis".to_string(),
            queue: "NATS".to_string(),
            ai: vec!["RuneSage".to_string()],
            infra: "Terraform".to_string(),
            ci_cd: "GitHub Actions".to_string(),
        };

        let data2 = Stack {
            language: "go".to_string(), // Different language
            frontend: "SvelteKit".to_string(),
            backend: "Actix Web".to_string(),
            database: "PostgreSQL".to_string(),
            cache: "Redis".to_string(),
            queue: "NATS".to_string(),
            ai: vec!["RuneSage".to_string()],
            infra: "Terraform".to_string(),
            ci_cd: "GitHub Actions".to_string(),
        };

        let hash1 = calculate_hash(&data1).unwrap();
        let hash2 = calculate_hash(&data2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_format() {
        let data = Stack {
            language: "rust".to_string(),
            frontend: "SvelteKit".to_string(),
            backend: "Actix Web".to_string(),
            database: "PostgreSQL".to_string(),
            cache: "Redis".to_string(),
            queue: "NATS".to_string(),
            ai: vec!["RuneSage".to_string()],
            infra: "Terraform".to_string(),
            ci_cd: "GitHub Actions".to_string(),
        };

        let hash = calculate_hash(&data).unwrap();

        // SHA256 hash should be 64 hex characters
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blueprint_hash_format() {
        let blueprint_json = r#"{"project_name":"test","goals":["test"]}"#;
        let hash = calculate_blueprint_hash(blueprint_json);

        assert!(hash.starts_with("sha256:"));
        let hex_part = &hash[7..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_plan_hash_format() {
        let plan_json = r#"{"decisions":[],"stack":{}}"#;
        let hash = calculate_plan_hash(plan_json);

        assert!(hash.starts_with("sha256:"));
        let hex_part = &hash[7..];
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_blueprint_hash_deterministic() {
        let blueprint_json = r#"{"project_name":"test","goals":["build","deploy"]}"#;

        let hash1 = calculate_blueprint_hash(blueprint_json);
        let hash2 = calculate_blueprint_hash(blueprint_json);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_plan_hash_deterministic() {
        let plan_json = r#"{"decisions":[{"topic":"language","choice":"Rust"}]}"#;

        let hash1 = calculate_plan_hash(plan_json);
        let hash2 = calculate_plan_hash(plan_json);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_deterministic_rng() {
        let seed = 42;
        let mut rng1 = create_rng(seed);
        let mut rng2 = create_rng(seed);

        assert_eq!(rng1.gen::<u32>(), rng2.gen::<u32>());
    }

    #[test]
    fn test_deterministic_rng_different_seeds() {
        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(99);

        // Different seeds should produce different sequences
        let val1 = rng1.gen::<u32>();
        let val2 = rng2.gen::<u32>();

        // While theoretically possible, extremely unlikely to be equal
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_rng_range() {
        let mut rng = create_rng(42);

        // Test that gen_range works correctly
        for _ in 0..100 {
            let val = rng.gen_range(0..10);
            assert!(val < 10);
            assert!(val >= 0);
        }
    }

    #[test]
    fn test_tie_breaker_deterministic() {
        let candidates = vec![
            "Option1".to_string(),
            "Option2".to_string(),
            "Option3".to_string(),
        ];

        let result1 = tie_breaker("backend", 42, candidates.clone());
        let result2 = tie_breaker("backend", 42, candidates.clone());

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_tie_breaker_different_topics() {
        let candidates = vec!["Option1".to_string(), "Option2".to_string()];

        // Different topics might produce different results
        let result1 = tie_breaker("backend", 42, candidates.clone());
        let result2 = tie_breaker("frontend", 42, candidates.clone());

        // They could be same or different, but must be deterministic
        assert!(candidates.contains(&result1));
        assert!(candidates.contains(&result2));
    }

    #[test]
    fn test_tie_breaker_single_candidate() {
        let candidates = vec!["OnlyOption".to_string()];

        let result = tie_breaker("backend", 42, candidates.clone());
        assert_eq!(result, "OnlyOption");
    }

    #[test]
    #[should_panic(expected = "No candidates provided for tie breaker")]
    fn test_tie_breaker_no_candidates() {
        let candidates: Vec<String> = vec![];
        tie_breaker("backend", 42, candidates);
    }

    #[test]
    fn test_tie_breaker_distribution() {
        // Test that tie breaker produces reasonable distribution
        let candidates = vec![
            "Option1".to_string(),
            "Option2".to_string(),
            "Option3".to_string(),
        ];

        let mut counts = std::collections::HashMap::new();

        // Run tie breaker with different seeds
        for seed in 0..100 {
            let result = tie_breaker("backend", seed, candidates.clone());
            *counts.entry(result).or_insert(0) += 1;
        }

        // Each option should be selected at least once
        assert_eq!(counts.len(), 3);
        for candidate in &candidates {
            assert!(counts.contains_key(candidate));
            assert!(*counts.get(candidate).unwrap() > 0);
        }
    }

    #[test]
    fn test_tie_breaker_same_seed_different_topics() {
        let candidates = vec![
            "Option1".to_string(),
            "Option2".to_string(),
            "Option3".to_string(),
        ];

        let backend_result = tie_breaker("backend", 42, candidates.clone());
        let frontend_result = tie_breaker("frontend", 42, candidates.clone());
        let database_result = tie_breaker("database", 42, candidates.clone());

        // All should be valid selections
        assert!(candidates.contains(&backend_result));
        assert!(candidates.contains(&frontend_result));
        assert!(candidates.contains(&database_result));

        // With same seed but different topics, results should be deterministic
        let backend_result2 = tie_breaker("backend", 42, candidates.clone());
        let frontend_result2 = tie_breaker("frontend", 42, candidates.clone());
        let database_result2 = tie_breaker("database", 42, candidates.clone());

        assert_eq!(backend_result, backend_result2);
        assert_eq!(frontend_result, frontend_result2);
        assert_eq!(database_result, database_result2);
    }

    #[test]
    fn test_hash_complex_structure() {
        #[derive(Serialize)]
        struct ComplexData {
            name: String,
            values: Vec<i32>,
            nested: NestedData,
        }

        #[derive(Serialize)]
        struct NestedData {
            flag: bool,
            count: u64,
        }

        let data = ComplexData {
            name: "test".to_string(),
            values: vec![1, 2, 3, 4, 5],
            nested: NestedData {
                flag: true,
                count: 42,
            },
        };

        let hash = calculate_hash(&data).unwrap();
        assert_eq!(hash.len(), 64);

        // Changing any field should change the hash
        let data2 = ComplexData {
            name: "test2".to_string(), // Changed
            values: vec![1, 2, 3, 4, 5],
            nested: NestedData {
                flag: true,
                count: 42,
            },
        };

        let hash2 = calculate_hash(&data2).unwrap();
        assert_ne!(hash, hash2);
    }

    #[test]
    fn test_hash_order_matters() {
        #[derive(Serialize)]
        struct OrderTest {
            a: String,
            b: String,
        }

        let data1 = OrderTest {
            a: "first".to_string(),
            b: "second".to_string(),
        };

        let data2 = OrderTest {
            a: "second".to_string(),
            b: "first".to_string(),
        };

        let hash1 = calculate_hash(&data1).unwrap();
        let hash2 = calculate_hash(&data2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_tie_breaker_long_candidate_list() {
        let candidates: Vec<String> = (0..100).map(|i| format!("Option{}", i)).collect();

        let result = tie_breaker("backend", 42, candidates.clone());
        assert!(candidates.contains(&result));

        // Same seed should produce same result
        let result2 = tie_breaker("backend", 42, candidates.clone());
        assert_eq!(result, result2);
    }

    #[test]
    fn test_blueprint_hash_whitespace_sensitive() {
        let json1 = r#"{"project_name":"test","goals":["test"]}"#;
        let json2 = r#"{"project_name": "test", "goals": ["test"]}"#; // Added spaces

        let hash1 = calculate_blueprint_hash(json1);
        let hash2 = calculate_blueprint_hash(json2);

        // Different whitespace should produce different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_plan_hash_empty_content() {
        let empty_json = "";
        let hash = calculate_plan_hash(empty_json);

        assert!(hash.starts_with("sha256:"));
        let hex_part = &hash[7..];
        assert_eq!(hex_part.len(), 64);
    }
}
