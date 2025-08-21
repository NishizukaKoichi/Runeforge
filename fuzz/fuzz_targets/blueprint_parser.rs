#![no_main]
use libfuzzer_sys::fuzz_target;
use runeforge::schema;

fuzz_target!(|data: &[u8]| {
    // Parse blueprint from fuzzed data
    if let Ok(s) = std::str::from_utf8(data) {
        // Try parsing as YAML
        let _ = schema::validate_blueprint(s);
        
        // Try parsing as JSON
        let _ = schema::validate_blueprint(s);
    }
});
