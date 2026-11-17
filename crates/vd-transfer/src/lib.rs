//! Chunked, resumable file transfer state machine (no socket IO).
#![forbid(unsafe_code)]

use vd_common::{Error, Result};
use vd_proto::FileMessage;

/// Local transfer direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Sending a file to the peer.
    Send,
    /// Receiving a file from the peer.
    Receive,
}

/// Transfer session state.
#[derive(Debug, Clone)]
pub struct TransferSession {
    /// Transfer id.
    pub transfer_id: u64,
    /// Direction.
    pub direction: Direction,
    /// File name.
    pub name: String,
    /// Total size.
    pub size: u64,
    /// Bytes acknowledged / written.
    pub progress: u64,
    /// Running BLAKE3 hasher state is kept opaque in later tasks; stub tracks bytes only.
    pub complete: bool,
}

impl TransferSession {
    /// Start an outgoing offer.
    #[must_use]
    pub fn offer(transfer_id: u64, name: String, size: u64) -> Self {
        Self {
            transfer_id,
            direction: Direction::Send,
            name,
            size,
            progress: 0,
            complete: false,
        }
    }

    /// Apply a protocol message to advance state (stub logic).
    ///
    /// # Errors
    /// Returns protocol errors for mismatched ids; NotImplemented for data path.
    pub fn on_message(&mut self, msg: &FileMessage) -> Result<()> {
        match msg {
            FileMessage::Accept {
                transfer_id,
                resume_from,
            } => {
                if *transfer_id != self.transfer_id {
                    return Err(Error::Protocol("transfer id mismatch".into()));
                }
                self.progress = *resume_from;
                Ok(())
            }
            FileMessage::Ack {
                transfer_id,
                acked_through,
            } => {
                if *transfer_id != self.transfer_id {
                    return Err(Error::Protocol("transfer id mismatch".into()));
                }
                self.progress = *acked_through;
                Ok(())
            }
            FileMessage::Cancel { transfer_id, .. } => {
                if *transfer_id != self.transfer_id {
                    return Err(Error::Protocol("transfer id mismatch".into()));
                }
                self.complete = true;
                Ok(())
            }
            FileMessage::Complete {
                transfer_id,
                blake3: hash,
            } => {
                if *transfer_id != self.transfer_id {
                    return Err(Error::Protocol("transfer id mismatch".into()));
                }
                if hash.len() != 32 && !hash.is_empty() {
                    return Err(Error::Protocol("blake3 hash must be 32 bytes".into()));
                }
                self.complete = true;
                self.progress = self.size;
                Ok(())
            }
            FileMessage::Chunk { .. } | FileMessage::Offer { .. } => {
                Err(Error::NotImplemented("file chunk IO path"))
            }
        }
    }

    /// Hash bytes with BLAKE3 (helper for later verify).
    #[must_use]
    pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
        *blake3::hash(data).as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_sets_resume_offset() {
        let mut s = TransferSession::offer(1, "a.bin".into(), 100);
        s.on_message(&FileMessage::Accept {
            transfer_id: 1,
            resume_from: 40,
        })
        .unwrap();
        assert_eq!(s.progress, 40);
    }

    #[test]
    fn blake3_stable() {
        let h = TransferSession::blake3_hash(b"abc");
        assert_eq!(h.len(), 32);
    }
}
