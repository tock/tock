//! Interface for CRC computation.

use crate::returncode::ReturnCode;

/// CRC algorithms
///
/// In all cases, input bytes are bit-reversed (i.e., consumed from LSB to MSB.)
///
/// Algorithms prefixed with `Sam4L` are native to that chip and thus require
/// no software post-processing on platforms using it.
///
#[derive(Copy, Clone)]
pub enum CrcAlg {
    /// Polynomial 0x04C11DB7, output reversed then inverted ("CRC-32")
    Crc32,
    /// Polynomial 0x1EDC6F41, output reversed then inverted ("CRC-32C" / "Castagnoli")
    Crc32C,

    /// Polynomial 0x1021, no output post-processing
    Sam4L16,
    /// Polynomial 0x04C11DB7, no output post-processing
    Sam4L32,
    /// Polynomial 0x1EDC6F41, no output post-processing
    Sam4L32C,
}

pub trait CRC {
    /// Initiate a CRC calculation
    fn compute(&self, data: &[u8], _: CrcAlg) -> ReturnCode;

    /// Disable the CRC unit until compute() is next called
    fn disable(&self);
}

pub trait Client {
    /// Receive the successful result of a CRC calculation
    fn receive_result(&self, _: u32);
}
