#![no_main]
use libfuzzer_sys::fuzz_target;
use runeforge::schema::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Try to deserialize and validate StackPlan
        if let Ok(stack_plan) = serde_json::from_str::<StackPlan>(s) {
            let _ = validate_stack_plan(&stack_plan);
        }
    }
});