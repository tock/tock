// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Module containing the [`StreamingProcessSlice`] abstraction and
//! related infrastructure. See the documentation on
//! [`StreamingProcessSlice`] for more information.

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
///    to `0`. Finally, the `offset` bytes (interpreted as a u32 value in native
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
///    `offset` value, regardless of whether any header fields were updated. The
///    kernel never modifies any data in the region of
///    `[data.start; data.start + offset)`.
///
///    Should the write of a chunk fail because the buffer has insufficient
///    space left, the kernel will set the `exceeded` flag bit (index 0).
///
///    The `halt` flag bit as set by the process governs the kernel's behavior
///    once the `exceeded` flag is set: if `halt` is cleared, the kernel will
///    attempt to write future, smaller chunks to the buffer (and thus implicitly
///    discarding some packets). If `halt` and `exceeded` are both set, the
///    kernel will stop writing any data into the buffer.
///
/// 5. The kernel will schedule an upcall to the process, indicating that a
///    write to the buffer (or setting the `exceeded`) flag occurred. The kernel
///    may schedule only one upcall for the first chunk written to the buffer,
///    or multiple upcalls (e.g., one upcall per chunk written). A process must
///    not rely on the number of upcalls received and instead rely on the buffer
///    header (`offset` and the `flags` bits) to determine the amount of data
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
/// As the kernel cannot track if an `allow`ed buffer for a particular
/// [`SyscallDriver`](crate::syscall_driver::SyscallDriver) implementation is intended to be a
/// [`StreamingProcessSlice`], the kernel must use the header in the buffer as
/// provided by the process. The implementation of [`StreamingProcessSlice`]
/// ensures that an incorrect header will not cause a panic, but incoming
/// packets could be dropped. A process using a syscall API that uses a
/// [`StreamingProcessSlice`] must ensure it has properly initialized the header
/// before `allow`ing the buffer.
///
/// The version 0 buffer format is specified as follows:
/// ```text,ignore
/// 0           2           4           6           8
/// +-----------+-----------+-----------------------+----------...
/// | version   | flags     | write offset (32 bit) | data
/// +-----------+-----------+-----------------------+----------...
/// | 000...000 | x{14},H,E | <native endian u32>   |
/// +-----------+-----------+-----------------------+----------...
/// ```
///
/// The `version` field is a u16 integer stored in the target's native
/// endianness. The `flags` field is a bitfield laid out as shown in the
/// diagram above (laid out in big endian, with `E` being the least significant
/// bit at byte 3). The `offset` field is a u32 integer stored in the target's
/// native endianness.
///
/// The kernel does not impose any alignment restrictions on
/// `StreamingProcessSlice`s of version 0.
///
/// The flags field is structured as follows:
/// - `V`: version bits. This kernel only supports version `0`.
/// - `H`: `halt` flag. If this flag is set and the `exceeded` flag is set, the
///   kernel will not write any further data to this buffer.
/// - `E`: `exceeded` flag. The kernel sets this flag when the remaining buffer
///   capacity is insufficient to append the current chunk.
/// - `x{14}`: reserved flag bits. Unless specified otherwise, processes must clear
///   these flags prior to `allow`ing a buffer to the kernel. A kernel that does
///   not know of a reserved flag must refuse to operate on a buffer that has
///   such a flag set.
#[repr(transparent)]
pub struct StreamingProcessSlice<'a> {
    slice: &'a WriteableProcessSlice,
}

register_bitfields![
    u16,
    pub StreamingProcessSliceFlags [
        RESERVED OFFSET(2) NUMBITS(14) [
            RESERVED0 = 0x00,
        ],
        HALT OFFSET(1) NUMBITS(1) [],
        EXCEEDED OFFSET(0) NUMBITS(1) [],
    ]
];

/// Fields in the `StreamingProcessSlice` buffer header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamingProcessSliceHeader {
    pub version: u16,
    pub halt: bool,
    pub exceeded: bool,
    pub offset: u32,
}

impl<'a> StreamingProcessSlice<'a> {
    const RANGE_VERSION: Range<usize> = 0..2;
    const RANGE_FLAGS: Range<usize> = (Self::RANGE_VERSION.end)..(Self::RANGE_VERSION.end + 2);
    const RANGE_OFFSET: Range<usize> = (Self::RANGE_FLAGS.end)..(Self::RANGE_FLAGS.end + 4);
    const RANGE_DATA: RangeFrom<usize> = (Self::RANGE_OFFSET.end)..;

    pub fn new(slice: &'a WriteableProcessSlice) -> StreamingProcessSlice<'a> {
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
    ///   flags field and the offset field.
    fn get_header(&self) -> Result<StreamingProcessSliceHeader, ErrorCode> {
        let mut version_bytes = [0_u8; 2];
        self.slice
            .get(Self::RANGE_VERSION)
            .ok_or(ErrorCode::SIZE)?
            .copy_to_slice_or_err(&mut version_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        let version = u16::from_be_bytes(version_bytes);
        if version != 0 {
            return Err(ErrorCode::INVAL);
        }

        let mut flags_bytes = [0_u8; 2];
        self.slice
            .get(Self::RANGE_FLAGS)
            .ok_or(ErrorCode::SIZE)?
            .copy_to_slice_or_err(&mut flags_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        let flags: LocalRegisterCopy<u16, StreamingProcessSliceFlags::Register> =
            LocalRegisterCopy::new(u16::from_be_bytes(flags_bytes));

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

        Ok(StreamingProcessSliceHeader {
            version,
            halt: flags.read(StreamingProcessSliceFlags::HALT) != 0,
            exceeded: flags.read(StreamingProcessSliceFlags::EXCEEDED) != 0,
            offset: u32::from_ne_bytes(offset_bytes),
        })
    }

    /// Write updated header (`version`, `flags` and `offset`) back to the
    /// underlying slice.
    ///
    /// This function does not perform any sanity checks on the `header`
    /// argument.  In particular, users of this function must ensure that they
    /// previously extracted the written-back [`StreamingProcessSliceHeader`]
    /// argument from the buffer, do not modify the version, do not change any
    /// flags that are controlled by the process or otherwise violate the
    /// protocol, and correctly increment the `offset` value.
    ///
    /// - `SIZE`: if the underlying slice is not large enough to fit the
    ///   flags field and the offset field.
    fn write_header(&self, header: StreamingProcessSliceHeader) -> Result<(), ErrorCode> {
        // Write the offset first, to avoid modifying the buffer if it's too
        // small to fit the offset, but large enough to hold the flags byte:
        let offset_bytes = u32::to_ne_bytes(header.offset);
        self.slice
            .get(Self::RANGE_OFFSET)
            .ok_or(ErrorCode::SIZE)?
            .copy_from_slice_or_err(&offset_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        let version_bytes: [u8; 2] = u16::to_ne_bytes(header.version);
        self.slice
            .get(Self::RANGE_VERSION)
            .ok_or(ErrorCode::SIZE)?
            .copy_from_slice_or_err(&version_bytes)
            .map_err(|_| ErrorCode::SIZE)?;

        let flags_bytes: [u8; 2] = u16::to_be_bytes(
            (StreamingProcessSliceFlags::RESERVED::RESERVED0
                + StreamingProcessSliceFlags::HALT.val(header.halt as u16)
                + StreamingProcessSliceFlags::EXCEEDED.val(header.exceeded as u16))
            .value,
        );
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
    /// data. It must only be used on slices of version 0. If the underlying
    /// slice is too small to hold the header fields, this will return an empty
    /// slice.
    fn payload_slice(&self) -> &WriteableProcessSlice {
        self.slice
            .get(Self::RANGE_DATA)
            .unwrap_or((&mut [][..]).into())
    }

    /// Append a chunk of data to the slice.
    ///
    /// If the underlying slice has a correct `flags` and `offset` value, is not
    /// halted, and has sufficient space for this `data` chunk, this function
    /// returns the updated buffer offset (set to one past the last written
    /// byte).
    ///
    /// This function returns whether this chunk was the first non-zero-length
    /// `chunk` appended to the slice, and the offset after the append operation
    /// (where the next chunk would be written in the data section).
    ///
    /// This function fails with:
    /// - `INVAL`: if the version is not `0`, or the reserved flags are not
    ///   cleared.
    /// - `BUSY`: if both the `halt` and `exceeded` flags are set. In this case,
    ///   the slice will not be modified.
    /// - `SIZE`: if the underlying slice is not large enough to fit the
    ///   flags field and the offset field. In this case, the
    ///   `exceeded` flag will be set and the slice will not be modified.
    /// - `FAIL`: would need to increment offset beyond 2**32 - 1. Neither the
    ///   payload slice nor any header fields will be modified.
    ///
    /// Appending a zero-length `chunk` will be treated as every other chunk,
    /// but appending it will not set the exceeded flag, even if `offset` is at
    /// the maximum position for this buffer. A zero-length append operation can
    /// still fail due to the buffer being halted, having an improper header,
    /// etc. A zero-length `chunk` will never be treated as the first chunk
    /// appended to a buffer.
    pub fn append_chunk(&self, chunk: &[u8]) -> Result<(bool, u32), ErrorCode> {
        // This includes general sanity checks:
        let mut header = self.get_header()?;

        // Check whether we are instructed to halt:
        if header.exceeded && header.halt {
            return Err(ErrorCode::BUSY);
        }

        let previous_offset = header.offset;

        let new_offset: u32 = (previous_offset as usize)
            .checked_add(chunk.len())
            .ok_or(ErrorCode::FAIL)?
            .try_into()
            .map_err(|_| ErrorCode::FAIL)?;

        // Attempt to append the chunk to the slice, otherwise fail with SIZE:
        if let Some(dst) = self
            .payload_slice()
            .get((previous_offset as usize)..(new_offset as usize))
        {
            // We do have sufficient space to append this chunk to the slice:
            dst.copy_from_slice(chunk);
            header.offset = new_offset;
            self.write_header(header)?;
            Ok((previous_offset == 0 && chunk.len() != 0, new_offset))
        } else {
            // We don't have sufficient space to append this chunk to the slice.
            // Do not update header.offset, but set header.exceeded:
            header.exceeded = true;
            self.write_header(header)?;
            Err(ErrorCode::SIZE)
        }
    }

    /// Append a chunk of data from an iterator.
    ///
    /// If the underlying slice has a correct `flags` and `offset` value, is not
    /// halted, and has sufficient space for this `data` chunk, this function
    /// returns the updated buffer offset (set to one past the last written
    /// byte).
    ///
    /// This function returns whether this chunk was the first non-zero-length
    /// `chunk` appended to the slice, and the offset after the append operation
    /// (where the next chunk would be written in the data section).
    ///
    /// If the buffer does not have enough space, this function will still
    /// partially copy this chunk and modify the slice payload data after
    /// `offset`. It will not update the `offset` header field though, and
    /// instead set the `exceeded` flag.
    ///
    /// This function fails with:
    /// - `INVAL`: if the version is not `0`, or the reserved flags are not
    ///   cleared.
    /// - `BUSY`: if both the `halt` and `exceeded` flags are set. In this case,
    ///   the slice will not be modified.
    /// - `SIZE`: if the underlying slice is not large enough to fit the
    ///   flags field and the offset field. In this case, the
    ///   `exceeded` flag will be set and the slice will not be modified.
    /// - `FAIL`: would need to increment offset beyond 2**32 - 1. Neither the
    ///   payload slice nor any header fields will be modified.
    ///
    /// Appending a zero-length `chunk` will be treated as every other chunk,
    /// but appending it will not set the exceeded flag, even if `offset` is at
    /// the maximum position for this buffer. A zero-length append operation can
    /// still fail due to the buffer being halted, having an improper header,
    /// etc. A zero-length `chunk` will never be treated as the first chunk
    /// appended to a buffer.
    pub fn append_chunk_from_iter<I: IntoIterator<Item = u8>>(
        &self,
        src: I,
    ) -> Result<(bool, u32), ErrorCode> {
        // This includes general sanity checks:
        let mut header = self.get_header()?;

        // Check whether we are instructed to halt:
        if header.exceeded && header.halt {
            return Err(ErrorCode::BUSY);
        }

        // Create a subslice over the remaining payload space:
        let remaining_payload_slice = self
            .payload_slice()
            .get((header.offset as usize)..)
            // If the iterator yields 0 elements, even if the offset
            // lies at the end or outside of the payload slice, we
            // still don't want to return an error.
            .unwrap_or((&mut [][..]).into());

        // Create a mutable iterator over the remaining payload space:
        let mut remaining_payload_iter = remaining_payload_slice.iter();

        // We don't know how many bytes the `src` iterator will yield. Try to
        // copy from it and abort if we run out of space on the payload iter.
        //
        // We don't use `zip` here, as that would silently truncate the `src`
        // iter if the `payload` iter runs out of elements.
        let bytes_written_or_out_of_space = src
            .into_iter()
            // Combine this byte with one of the payload slice. This is
            // different from `zip` in that we keep iterating even if we hit
            // `None` on the payload iter:
            .map(|src_byte| {
                remaining_payload_iter
                    .next()
                    .map(|payload_byte| (payload_byte, src_byte))
            })
            // If we managed to get a `Some(Cell<u8>)`, write a byte from the
            // `src` to the payload slice and return `true`, else `false`.
            .map(|opt| opt.map(|(dst, src)| dst.set(src)).is_some())
            // Finally, count how many `true`s the iterator yields. Upon hitting
            // the first `false`, we instead return `None`.
            .try_fold(0, |acc, val| if val { Some(acc + 1) } else { None });

        if let Some(bytes_written) = bytes_written_or_out_of_space {
            // We did have sufficient space to append this chunk to the
            // slice. Update the offset contained in the header.
            let previous_offset = header.offset;

            header.offset += bytes_written;
            self.write_header(header)?;

            Ok((previous_offset == 0 && bytes_written != 0, header.offset))
        } else {
            // We don't have sufficient space to append this chunk to the slice.
            // Do not update header.offset, but set header.exceeded:
            header.exceeded = true;
            self.write_header(header)?;
            Err(ErrorCode::SIZE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamingProcessSlice;
    use crate::processbuffer::WriteableProcessSlice;
    use crate::ErrorCode;

    #[test]
    fn test_empty_process_slice() {
        let process_slice: &WriteableProcessSlice = (&mut [][..]).into();
        let s = StreamingProcessSlice::new(process_slice);

        assert_eq!(s.append_chunk(b"The cake is a lie."), Err(ErrorCode::SIZE));
        assert_eq!(
            s.append_chunk_from_iter(b"The cake is a lie.".iter().copied()),
            Err(ErrorCode::SIZE)
        );
    }

    #[test]
    fn test_header_only_process_slice() {
        let mut buffer = [0_u8; 8];
        let process_slice: &WriteableProcessSlice = (&mut buffer[..]).into();

        let s = StreamingProcessSlice::new(process_slice);
        let hdr = s.get_header().unwrap();
        assert_eq!(hdr.version, 0);
        assert_eq!(hdr.offset, 0);
        assert_eq!(hdr.halt, false);
        assert_eq!(hdr.exceeded, false);

        assert_eq!(s.append_chunk(b""), Ok((false, 0)));
        let hdr = s.get_header().unwrap();
        assert_eq!(hdr.version, 0);
        assert_eq!(hdr.offset, 0);
        assert_eq!(hdr.halt, false);
        assert_eq!(hdr.exceeded, false);

        assert_eq!(
            s.append_chunk_from_iter(b"".iter().copied()),
            Ok((false, 0))
        );
        let hdr = s.get_header().unwrap();
        assert_eq!(hdr.version, 0);
        assert_eq!(hdr.offset, 0);
        assert_eq!(hdr.halt, false);
        assert_eq!(hdr.exceeded, false);

        let prev_hdr = s.get_header().unwrap();
        assert_eq!(s.append_chunk(b"The cake is a lie."), Err(ErrorCode::SIZE));
        let hdr = s.get_header().unwrap();
        assert_eq!(hdr.version, 0);
        assert_eq!(hdr.offset, 0);
        assert_eq!(hdr.halt, false);
        assert_eq!(hdr.exceeded, true);

        // Reset the header:
        s.write_header(prev_hdr).unwrap();
        let hdr = s.get_header().unwrap();
        assert_eq!(prev_hdr, hdr);

        assert_eq!(
            s.append_chunk_from_iter(b"The cake is a lie.".iter().copied()),
            Err(ErrorCode::SIZE)
        );
        let hdr = s.get_header().unwrap();
        assert_eq!(hdr.version, 0);
        assert_eq!(hdr.offset, 0);
        assert_eq!(hdr.halt, false);
        assert_eq!(hdr.exceeded, true);
    }
}
