// crates/audiolib-windows/src/reaper_protocol.rs
//! Shared local protocol constants for the REAPER extension and desktop client.
pub const CLASS_NAME: &str = "AudioLibReaperBridgeV1";
pub const HELLO: u32 = 0x8001;
pub const READY: u32 = 0x8002;
pub const VERSION: usize = 0x414c0001;
pub const PACKET: usize = 0x414c0001;
pub const ACCEPTED: usize = 1;
pub const REJECTED: usize = 2;
pub const NO_TRACK: usize = 3;
pub const BUSY: usize = 4;
pub const UNCONFIRMED: usize = 5;
