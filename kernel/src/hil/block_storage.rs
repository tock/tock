//! Interface for reading, writing, and erasing storage blocks
//! on devices without a Flash Translation Layer.
//!
//! Operates on discardable and writeable blocks, where areas must be
//! discarded before overwriting.
//!
//! Here's an example implementation for a raw flash chip:
//!
//! ```rust,ignore
//! use kernel::hil;
//! use kernel::ErrorCode;
//!
//! const WRITE_BLOCK_BYTES: usize = 256;
//! const DISCARD_BLOCK_BYTES: usize = 4096;
//!
//! struct RawFlashChip {};
//!
//! type WriteBlockIndex = hil::block_storage::BlockIndex<WRITE_BLOCK_BYTES>;
//! type DiscardBlockIndex = hil::block_storage::BlockIndex<DISCARD_BLOCK_BYTES>;
//!
//! impl hil::block_storage::BlockStorage<WRITE_BLOCK_BYTES, DISCARD_BLOCK_BYTES> for RawFlashChip {
//!     //(implement associated functions here)
//! }
//! ```
use crate::ErrorCode;
use core::ops::Add;

/// An index to a block within device composed of `S`-sized blocks.
/// Stores the number of blocks from the start of the device.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockIndex<const S: usize>(pub u32);

impl<const S: usize> BlockIndex<S> {
    /// Returns the index that contains the address.
    pub fn new_containing(address: u64) -> Self {
        Self((address / S as u64) as u32)
    }

    /// Returns the index starting at the given address, if any.
    pub fn new_starting_at(address: u64) -> Option<Self> {
        if address % S as u64 == 0 {
            Some(Self::new_containing(address))
        } else {
            None
        }
    }
}

impl<const S: usize> From<BlockIndex<S>> for u64 {
    fn from(index: BlockIndex<S>) -> Self {
        index.0 as u64 * S as u64
    }
}

impl<const S: usize> Add<u32> for BlockIndex<S> {
    type Output = Self;
    fn add(self, other: u32) -> Self {
        BlockIndex(self.0 + other)
    }
}

/// Readable persistent block storage device.
///
/// The device is formed from equally-sized storage blocks,
/// which are arranged one after another, without gaps or overlaps,
/// to form a linear storage of bytes.
///
/// Every byte on the device belongs to exactly one block.
///
/// `R`: The size of a read block in bytes.
pub trait ReadableStorage<const R: usize> {
    /// Read data from a block, and into the buffer.
    ///
    /// `ErrorCode::INVAL` will be returned if
    /// - `region` exceeds the end of the device, or
    /// - `buf` is shorter than `region`.
    ///
    /// Returns `ErrorCode::BUSY` when another operation is in progress.
    ///
    /// On success, triggers `ReadableClient::read_complete` once.
    fn read(
        &self,
        region: &BlockIndex<R>,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Returns the size of the device in bytes.
    fn get_size(&self) -> u64;
}

/// Writeable persistent block storage device.
///
/// The device is formed from equally-sized storage blocks,
/// which are arranged one after another, without gaps or overlaps,
/// to form a linear storage of bytes.
///
/// The device is split into blocks in two ways, into:
/// - discard blocks, which are the smallest unit of space
/// that can be discarded (see `BlockStorage::discard`)
/// - write blocks, which are the smallest unit of space that can be written
///
/// Every byte on the device belongs to exactly one discard block,
/// and to exactly one write block at the same time.
///
/// `W`: The size of a write block in bytes.
/// `D`: The size of a discard block in bytes.
pub trait WriteableStorage<const W: usize, const D: usize> {
    /// Write data from a buffer to storage.
    ///
    /// This function writes the contents of `buf` to memory,
    /// at the chosen `region`.
    ///
    /// `ErrorCode::INVAL` will be returned if
    /// - `region` exceeds the end of the device, or
    /// - `buf` is shorter than `region`.
    ///
    /// This function SHALL NOT discard the block first.
    /// The user of this function MUST ensure that the relevant block
    /// has been successfully discarded first (see `discard`).
    ///
    /// Once a byte has been written as part of a write block,
    /// it MUST NOT be written again until it's discarded
    /// as part of a discard block.
    /// Multiple consecutive writes to the same block
    /// are forbidden by this trait, but the restriction is not enforced.
    ///
    /// **Note** about raw flash devices: writes can turn bits from `1` to `0`.
    /// To change a bit from `0` to `1`, a region must be erased (discarded).
    ///
    /// Returns `ErrorCode::BUSY` when another operation is in progress.
    ///
    /// On success, triggers `WriteableClient::write_complete` once.
    fn write(
        &self,
        region: &BlockIndex<W>,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;

    /// Makes a region ready for writing.
    ///
    /// This corresponds roughly to the erase operation on raw flash devices.
    ///
    /// A successful `discard` leaves bytes in the selected `region` undefined.
    /// The user of this API must not assume any property
    /// of the discarded bytes.
    ///
    /// If `region` exceeds the size of the device, returns `ErrorCode::INVAL`.
    ///
    /// Returns `ErrorCode::BUSY` when another operation is in progress.
    ///
    /// On success, triggers `WriteableClient::discard_complete` once.
    fn discard(&self, region: &BlockIndex<D>) -> Result<(), ErrorCode>;
}

pub trait Storage<const W: usize, const D: usize>:
    ReadableStorage<W> + WriteableStorage<W, D>
{
}

impl<const W: usize, const D: usize, T: ReadableStorage<W> + WriteableStorage<W, D>> Storage<W, D>
    for T
{
}

/// Specifies a storage area with byte granularity.
pub struct AddressRange {
    /// Offset from the beginning of the storage device.
    pub start_address: u64,
    /// Length of the range.
    pub length_bytes: u32,
}

impl AddressRange {
    pub fn get_end_address(&self) -> u64 {
        self.start_address + self.length_bytes as u64
    }
}

impl<const C: usize> From<BlockIndex<C>> for AddressRange {
    fn from(region: BlockIndex<C>) -> Self {
        AddressRange {
            start_address: region.0 as u64 * C as u64,
            length_bytes: C as u32,
        }
    }
}

/// Devices which can read arbitrary byte-indexed ranges.
pub trait ReadRange {
    /// Read data from storage into a buffer.
    ///
    /// This function will read data stored in storage at `range` into `buf`.
    ///
    /// `ErrorCode::INVAL` will be returned if
    /// - `range` exceeds the end of the device, or
    /// - `buf` is shorter than `range`.
    ///
    /// Returns `ErrorCode::BUSY` when another operation is in progress.
    ///
    /// On success, triggers `ReadableClient::read_complete` once.
    fn read_range(
        &self,
        range: &AddressRange,
        buf: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])>;
}

pub trait HasClient<'a, C> {
    /// Set the client for this peripheral. The client will be called
    /// when operations complete.
    fn set_client(&'a self, client: &'a C);
}

/// Implement this to receive callbacks from `ReadableStorage` and `ReadRange`.
pub trait ReadableClient {
    /// This will be called when a read operation is complete.
    ///
    /// If the device is unable to read the region, returns `ErrorCode::FAIL`.
    ///
    /// On errors, the buffer contents are undefined.
    fn read_complete(&self, read_buffer: &'static mut [u8], ret: Result<(), ErrorCode>);
}

/// Implement this to receive callbacks from `ReadableStorage`.
pub trait WriteableClient {
    /// This will be called when the write operation is complete.
    ///
    /// If the device is unable to write to the region,
    /// returns `ErrorCode::FAIL`.
    ///
    /// On errors, the contents of the storage region are undefined,
    /// and the region must be considered written.
    fn write_complete(&self, write_buffer: &'static mut [u8], ret: Result<(), ErrorCode>);

    /// This will be called when the discard operation is complete.
    ///
    /// If the device is unable to discard the region,
    /// returns `ErrorCode::FAIL`.
    ///
    /// On errors, the contents of the storage region are undefined,
    /// and the region's discarded status must be considered unchanged.
    fn discard_complete(&self, ret: Result<(), ErrorCode>);
}

pub trait Client: WriteableClient + ReadableClient {}

impl<T: WriteableClient + ReadableClient> Client for T {}
