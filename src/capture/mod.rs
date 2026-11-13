//! Packet acquisition shared by the privileged service and user-side analysis.
mod packet;
pub use packet::{CapturedPacket, LinkType, PacketTimestamp};
#[cfg(any(windows, test))]
mod lifecycle;
pub mod network;
#[cfg(any(windows, test))]
mod protocol;
#[cfg(windows)]
pub mod service;
pub mod shared;
pub mod widgets;
#[cfg(windows)]
mod windows;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub name: String,
    pub address: std::net::Ipv4Addr,
    pub index: u32,
}

pub struct CaptureBatch {
    pub packets: Vec<CapturedPacket>,
    pub status: String,
    pub finished: bool,
    pub error: Option<String>,
}

/// Adapters capture evidence; protocol interpretation belongs to the shared analyzer.
pub trait CaptureSource: Send {
    fn poll(&mut self) -> Result<CaptureBatch>;
}
