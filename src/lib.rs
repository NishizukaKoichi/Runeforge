#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod ports;

#[cfg(all(feature = "std", not(target_arch = "wasm32")))]
pub mod adapters;

mod check_forbidden_apis;

#[cfg(feature = "std")]
pub mod schema;
#[cfg(feature = "std")]
pub mod selector;
#[cfg(feature = "std")]
pub mod util;

#[cfg(test)]
mod test_utils;

pub fn is_wasm() -> bool {
    cfg!(target_arch = "wasm32")
}
