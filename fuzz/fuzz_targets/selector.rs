#![no_main]
use libfuzzer_sys::fuzz_target;
use runeforge::{schema::*, selector::Selector};

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    
    // Extract seed from first 8 bytes
    let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
    
    // Use remaining data as blueprint input
    if let Ok(s) = std::str::from_utf8(&data[8..]) {
        if let Ok(blueprint) = schema::validate_blueprint(s) {
            // Load default rules
            let rules_content = include_str!("../../../resources/rules.yaml");
            
            // Try to create selector and generate plan
            if let Ok(selector) = Selector::new(rules_content, seed) {
                let _ = selector.select(&blueprint);
            }
        }
    }
});