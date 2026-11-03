//! Debug-only, evidence-preserving AION 2 capture analysis.

pub mod analysis;
mod compression;
mod interpretation;
pub mod live;
pub mod network;
pub mod opcode;
pub mod pcapng;
pub mod source;
pub mod storage;
mod synchronization;

#[cfg(test)]
mod tests;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub const DECODER_VERSION: &str = "aion2-experimental-4";

pub fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}
