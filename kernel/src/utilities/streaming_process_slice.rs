// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

use core::ops::{Range, RangeFrom};

use crate::processbuffer::WriteableProcessSlice;
use crate::utilities::registers::{register_bitfields, LocalRegisterCopy};
use crate::ErrorCode;

/// A wrapper around a [`WriteableProcessSlice`] for streaming data from the
/// kernel to a userspace process.
///
/// Applications like ADC sampling or network stacks require the kernel to
/// provide a process with a continuous, lossless stream of data from a source
/// that is not rate-controlled by the process. This wrapper implements the
/// kernel-side of a simple protocol to achieve this goal, without requiring
/// kernel-side buffering and by utilizing the atomic swap semantics of Tock's
/// `allow` system call. The protocol is versioned; the semantics for version 0
/// are as follows:
///
/// 1. To receive a data stream from the kernel, a userspace process allocates
///    two buffers.
///
/// 2. The first buffer is prepared according to the format below. The `flags`
///    field's version bits are set to `0`. The process clears the `exceeded`
///    flag. It may set or clear the `halt` flag. All reserved flags must be set
///    to `0`. Finally, the `offset` bytes (intepreted as a u32 value in native
///    endianness) are set to `0`.
///
/// 3. The process `allow`s this buffer to a kernel driver.
///
/// 4. The kernel driver writes incoming data starting at the `data` field +
///    `offset` bytes. After each write, the kernel increments `offset` by the
///    number of bytes written.
///
///    For each *chunk* written to the buffer (where a *chunk* is an
///    application-defined construct, such as a network packet), the kernel only
///    increments `offset` if the full chunk was successfully written into the
///    buffer. The kernel may or may not modify any data after the current
///    `offset` value, regardless of whether any metadata was updated. The
///    kernel never modifies any data in the region of
///    `[data.start; data.start + offset)`.
///
///    Should the write of a chunk fail because the buffer has insufficient
///    space left, the kernel will set the `exceeded` flag bit (index 0).
///
///    The `halt` flag bit as set by the process governs the kernel's behavior
///    once the `exceeded` flag is set: if `halt` is cleared, the kernel will
///    attempt to write future, smaller chunks to be buffer (and thus implicitly
///    discarding some packets). If `halt` and `exceeded` are both set, the
///    kernel will stop writing any data into the buffer.
///
/// 5. The kernel will schedule an upcall to the process, indicating that a
///    write to the buffer (or setting the `exceeded`) flag occurred. The kernel
///    may schedule only one upcall for the first chunk written to the buffer,
///    or multiple upcalls (e.g., one upcall per chunk written). A process must
///    not rely on the number of upcalls received and instead rely on the buffer
///    metadata (`offset` and the `flags` bits) to determine the amount of data
///    written to the buffer.
///
/// 6. The process prepares its second buffer, following step 2. The process
///    then issues an `allow` operation that atomically swaps the current
///    allowed buffer by its second buffer.
///
/// 7. The process can now process the received chunks contained in the initial
///    buffer, while the kernel receives new chunks in the other, newly allowed
///    buffer.
///
///
/// The version 0 buffer format is specified as follows:
/// ```text,ignore
/// 0          1          2          3          4          5
/// +----------+----------+----------+----------+----------+-------------------...
/// | flags    | buffer offset                             | data
/// +----------+----------+----------+----------+----------+-------------------...
/// | VVVVxxHE | 32 bits (native endian)                   |
/// ```
///
/// The flags field is structured as follows:
/// - `V`: version bits. This kernel only supports version `0`.
/// - `H`: `halt` flag. If this flag is set alongside the `exceeded` flag, the
///   kernel will not write any further data to this buffer.
/// - `E`: `exceeded` flag. The kernel sets this flag when the remaining buffer
///   capacity is insufficient to append the current chunk.
/// - `x`: reserved flags. Unless specified otherwise, processes must clear
///   these flags prior to `allow`ing a buffer to the kernel. A kernel that does
///   not know of a reserved flag must refuse to operate on a buffer that has
///   such a flag set.
#[repr(transparent)]
pub struct StreamingProcessSlice<'a> {
    slice: &'a WriteableProcessSlice,
}

register_bitfields![
    u8,
    pub StreamingProcessSliceFlags [
        VERSION OFFSET(4) NUMBITS(4) [
            V0 = 0x00,
        ],
        RESERVED OFFSET(2) NUMBITS(2) [
            RESERVED0 = 0x00,
        ],
        HALT OFFSET(1) NUMBITS(1) [],
        EXCEEDED OFFSET(1) NUMBITS(1) [],
    ]
];

#[derive(Debug, Clone, Copy)]
pub struct StreamingProcessSliceMeta {
    pub version: StreamingProcessSliceFlags::VERSION::Value,
    pub halt: bool,
    pub exceeded: bool,
    pub offset: u32,
}

impl<'a> StreamingProcessSlice<'a> {
    const RANGE_FLAGS: Range<usize> = 0..1;
    const RANGE_OFFSET: Range<usize> = (Self::RANGE_FLAGS.end)..(Self::RANGE_FLAGS.end + 4);
    const RANGE_DATA: RangeFrom<usize> = (Self::RANGE_OFFSET.end + 1)..;

    pub fn new(slice: &'a WriteableProcessSlice) -> StreamingProcessSlice {
        StreamingProcessSlice { slice }
    }

    /// Checks whether the buffer is valid (of sufficient size to contain at
    /// least the `flags` and `offset` fields), and extract the `flags` and
    /// `offset` field values.
    ///
    /// This function fails with
    /// - `INVAL`: if the version is not `0`, or the reserved flags are not
    ///   cleared.
    /// - `SIZE`: if the underlying slice is not large enough to fit the
    ///   flags field and the offset field (5 bytes).
    pub fn get_meta(&self) -> Result<StreamingProcessSliceMeta, ErrorCode> {
        let mut flags_bytes = [0_u8];
        self.slice
            .get(Self::RANGE_FLAGS)
            .ok_or(ErrorCode::SIZE)?
            .copy_to_slice_or_err(&mut flags_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        let flags: LocalRegisterCopy<u8, StreamingProcessSliceFlags::Register> =
            LocalRegisterCopy::new(flags_bytes[0]);

        let version = match flags.read_as_enum(StreamingProcessSliceFlags::VERSION) {
            Some(v @ StreamingProcessSliceFlags::VERSION::Value::V0) => v,
            _ => return Err(ErrorCode::INVAL),
        };

        if flags.read_as_enum(StreamingProcessSliceFlags::RESERVED)
            != Some(StreamingProcessSliceFlags::RESERVED::Value::RESERVED0)
        {
            return Err(ErrorCode::INVAL);
        }

        let mut offset_bytes = [0_u8; Self::RANGE_OFFSET.end - Self::RANGE_OFFSET.start];
        self.slice
            .get(Self::RANGE_OFFSET)
            .ok_or(ErrorCode::SIZE)?
            .copy_to_slice_or_err(&mut offset_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        Ok(StreamingProcessSliceMeta {
            version,
            halt: flags.read(StreamingProcessSliceFlags::HALT) != 0,
            exceeded: flags.read(StreamingProcessSliceFlags::EXCEEDED) != 0,
            offset: u32::from_ne_bytes(offset_bytes),
        })
    }

    /// Write updated metadata (`flags` and `offset`) back to the underlying
    /// slice.
    ///
    /// This function does not perform any sanity checks on the `meta` argument.
    /// In particular, users of this function must ensure that they previously
    /// extracted the written-back [`StreamingProcessSliceMeta`] argument from
    /// the buffer, do not modify the version, do not change any flags that are
    /// controlled by the process or otherwise violate the protocol, and
    /// correctly increment the `offset` value.
    ///
    /// - `SIZE`: if the underlying slice is not large enough to fit the
    ///   flags field and the offset field (5 bytes).
    pub fn set_meta(&self, meta: StreamingProcessSliceMeta) -> Result<(), ErrorCode> {
        // Write the offset first, to avoid modifying the buffer if it's too
        // small to fit the offset, but large enough to hold the flags byte:
        let offset_bytes = u32::to_ne_bytes(meta.offset);
        self.slice
            .get(Self::RANGE_OFFSET)
            .ok_or(ErrorCode::SIZE)?
            .copy_from_slice_or_err(&offset_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        let flags_bytes: [u8; 1] = [(StreamingProcessSliceFlags::VERSION.val(meta.version as u8)
            + StreamingProcessSliceFlags::RESERVED::RESERVED0
            + StreamingProcessSliceFlags::HALT.val(meta.halt as u8)
            + StreamingProcessSliceFlags::EXCEEDED.val(meta.exceeded as u8))
        .value];
        self.slice
            .get(Self::RANGE_FLAGS)
            .ok_or(ErrorCode::SIZE)?
            .copy_from_slice_or_err(&flags_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        Ok(())
    }

    /// Access the payload data portion of the underlying slice.
    ///
    /// This method does not perform any validation of the buffer version or
    /// metadata. It must only be used on slices of version 0. If the underlying
    /// slice is too small to hold the metadata fields, this will return an
    /// empty slice.
    fn payload_slice(&self) -> &WriteableProcessSlice {
        self.slice
            .get(Self::RANGE_DATA)
            .unwrap_or((&mut [][..]).into())
    }

    /// Append a chunk of data to the slice.
    ///
    /// If the underlying slice has a correct `flags` and `offset` value, is not
    /// halted, and has sufficient space for this `data` chunk, this function
    /// returns the updated buffer metadata.
    ///
    /// This function fails with:
    /// - `INVAL`: if the version is not `0`, or the reserved flags are not
    ///   cleared.
    /// - `BUSY`: if both the `halt` and `exceeded` flags are set. In this case,
    ///   the slice will not be modified.
    /// - `SIZE`: if the underlying slice is not large enough to fit the
    ///   flags field and the offset field (5 bytes). In this case, the
    ///   `exceeded` flag will be set and the slice will not be modified.
    /// - `FAIL`: would need to increment offset beyond 2**32 - 1. Neither the
    ///   payload slice nor any metadata will be modified.
    pub fn append_chunk(&self, chunk: &[u8]) -> Result<StreamingProcessSliceMeta, ErrorCode> {
        // This includes general sanity checks:
        let mut meta = self.get_meta()?;

        // Check whether we are instructed to halt:
        if meta.exceeded && meta.halt {
            return Err(ErrorCode::BUSY);
        }

        let new_offset: u32 = (meta.offset as usize)
            .checked_add(chunk.len())
            .ok_or(ErrorCode::FAIL)?
            .try_into()
            .map_err(|_| ErrorCode::FAIL)?;

        // Attempt to append the chunk to the slice, otherwise fail with SIZE:
        if let Some(dst) = self
            .payload_slice()
            .get((meta.offset as usize)..(new_offset as usize))
        {
            // We do have sufficient space to append this chunk to the slice:
            dst.copy_from_slice(chunk);
            meta.offset = new_offset;
            self.set_meta(meta)?;
            Ok(meta)
        } else {
            // We don't have sufficient space to append this chunk to the slice.
            // Do not update meta.offset, but set meta.exceeded:
            meta.exceeded = true;
            self.set_meta(meta)?;
            Err(ErrorCode::SIZE)
        }
    }
}
