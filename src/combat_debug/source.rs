//! File and network adapters supply the same original-packet contract.
use super::{Result, pcapng};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub use crate::capture::{CaptureBatch, CaptureSource};

pub struct PcapngSource {
    pub name: String,
    pub fingerprint: String,
    pending: Option<pcapng::PcapngImport>,
}

impl PcapngSource {
    pub fn open(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        let fingerprint = format!("sha256:{:x}", Sha256::digest(&bytes));
        Ok(Self {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            fingerprint,
            pending: Some(pcapng::read(&bytes)),
        })
    }
}

impl CaptureSource for PcapngSource {
    fn poll(&mut self) -> Result<CaptureBatch> {
        let parsed = self.pending.take().unwrap_or(pcapng::PcapngImport {
            packets: Vec::new(),
            blocks_read: 0,
            error: None,
        });
        Ok(CaptureBatch {
            status: if parsed.is_partial() {
                "partielle"
            } else {
                "terminée"
            }
            .into(),
            packets: parsed.packets,
            finished: true,
            error: parsed.error,
        })
    }
}
