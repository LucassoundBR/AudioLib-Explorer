// tools/fast-tests/reaper_contracts.rs
//! Real production modules in a small, GUI-free test target. No dispatch mocks.
// Both independently compiled endpoints import the same wire constants by path.
#![allow(clippy::duplicate_mod)]
extern crate self as audiolib_windows;

#[path = "../../crates/audiolib-windows/src/reaper_native.rs"]
pub mod reaper_native;
pub use reaper_native::NativeReaper;

#[path = "../../crates/audiolib-windows/src/reaper_extension_host.rs"]
pub mod reaper_extension_host;

#[path = "../../crates/audiolib-windows/src/reaper_focus.rs"]
pub mod reaper_focus;

// GUI-only entrypoints are intentionally unused in this test executable.
// Keep all other compiler and Clippy diagnostics enabled.
#[allow(dead_code)]
#[path = "../../app/audiolib-desktop/src/reaper_bridge.rs"]
mod reaper_bridge;
