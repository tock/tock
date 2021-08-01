//! Interface for CRC computation.

use crate::utilities::leasable_buffer::LeasableBuffer;
use crate::ErrorCode;

/// Client for CRC algorithm implementations
///
/// Implement this trait and use [`Crc::set_client`] in order to
/// receive callbacks from the CRC implementation.
pub trait Client {
    /// Called when the current data chunk has been processed by the
    /// CRC engine. Further data may be supplied when this callback is
    /// received.
    fn input_done(&self, result: Result<(), ErrorCode>, buffer: LeasableBuffer<'static, u8>);

    /// Called when the CRC computation is finished.
    fn crc_done(&self, result: Result<CrcOutput, ErrorCode>);
}

/// CRC algorithms
///
/// In all cases, input bytes are bit-reversed (i.e., consumed from LSB to MSB.)
///
/// Algorithms prefixed with `Sam4L` are native to that chip and thus require
/// no software post-processing on platforms using it.
///
#[derive(Copy, Clone)]
pub enum CrcAlgorithm {
    /// Polynomial 0x04C11DB7, output reversed then inverted
    /// ("CRC-32")
    Crc32,
    /// Polynomial 0x1EDC6F41, output reversed then inverted
    /// ("CRC-32C" / "Castagnoli")
    Crc32C,
    /// Polynomial 0x1021, no output post-processing ("CRC-16-CCITT")
    Crc16CCITT,
}

/// CRC output type
///
/// Individual CRC algorithms can have different output lengths. This
/// type represents the different [`CrcAlgorithm`] outputs
/// respectively.
#[derive(Copy, Clone)]
pub enum CrcOutput {
    /// Output of [`CrcAlgorithm::Crc32`]
    Crc32(u32),
    /// Output of [`CrcAlgorithm::Crc32C`]
    Crc32C(u32),
    /// Output of [`CrcAlgorithm::Crc16CCITT`]
    Crc16CCITT(u16),
}

impl CrcOutput {
    pub fn algorithm(&self) -> CrcAlgorithm {
        match self {
            CrcOutput::Crc32(_) => CrcAlgorithm::Crc32,
            CrcOutput::Crc32C(_) => CrcAlgorithm::Crc32C,
            CrcOutput::Crc16CCITT(_) => CrcAlgorithm::Crc16CCITT,
        }
    }
}

pub trait Crc<'a> {
    /// Set the client to be used for callbacks of the CRC
    /// implementation.
    fn set_client(&self, client: &'a dyn Client);
    /// Check whether a given CRC algorithm is supported by a CRC
    /// implementation.
    ///
    /// Returns true if the algorithm specified is supported.
    fn algorithm_supported(&self, algorithm: CrcAlgorithm) -> bool;

    /// Set the CRC algorithm to use.
    ///
    /// Calling this method may enable the CRC engine in case of a
    /// physical unit.
    ///
    /// If the device is currently processing a chunk of data or
    /// calculating a CRC, this operation will be refused, returning
    /// [`ErrorCode::BUSY`]. If a CRC calculation currently has
    /// pending data, it will be cancelled and the CRC engine's state
    /// reset.
    ///
    /// [`ErrorCode::NOSUPPORT`] will be returned if the algorithm
    /// requested is not supported. To non-invasively check whether a
    /// given algorithm is supported by a CRC implementation, use
    /// [`Crc::algorithm_supported`].
    fn set_algorithm(&self, algorithm: CrcAlgorithm) -> Result<(), ErrorCode>;

    /// Input chunked data into the CRC implementation.
    ///
    /// Calling this method may enable the CRC engine in case of a
    /// physical unit.
    ///
    /// If [`Crc::set_algorithm`] has not been invoked before, this
    /// method must return [`ErrorCode::RESERVE`].
    ///
    /// If the device is currently already processing a chunk of data
    /// or calculating a CRC, [`ErrorCode::BUSY`] must be returned.
    ///
    /// After the chunk of data has been processed,
    /// [`Client::input_done`] is called.
    ///
    /// The implementation may only read a part of the passed
    /// [`LeasableBuffer`]. It will return the bytes read and will
    /// resize the returned [`LeasableBuffer`] appropriately prior to
    /// passing it back through [`Client::input_done`].
    fn input(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ErrorCode, LeasableBuffer<'static, u8>)>;

    /// Request calculation of the CRC.
    ///
    /// Calling this method may enable the CRC engine in case of a
    /// physical unit.
    ///
    /// If [`Crc::set_algorithm`] has not been invoked before, this
    /// method must return [`ErrorCode::RESERVE`].
    ///
    /// If the device is currently processing a chunk of data or
    /// calculating a CRC, [`ErrorCode::BUSY`] must be returned.
    ///
    /// After the CRC has been calculated, [`Client::crc_done`] is
    /// called.
    fn compute(&self) -> Result<(), ErrorCode>;

    /// Disable the CRC unit until susequent calls to methods which
    /// will enable the CRC unit again.
    fn disable(&self);
}
