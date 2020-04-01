use crate::hash::{HashType, Hasher, HasherClient};

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::ReturnCode;
/// Extend a u32 byte array representation
///
/// This function takes a former_value, interprets it as an array of u8s
/// and overwrites it with additional u8 taken from iter, beginning from the byte_offset-th byte (first byte is 0).
/// It returns the number of bytes that have been taken from the iterator as well as the new u32.
pub fn extend_u32(
    former_value: u32,
    byte_offset: usize,
    iter: &mut dyn Iterator<Item = u8>,
) -> (u32, usize) {
    let mut values_as_bytes = former_value.to_be_bytes();
    let bytes_written =
        values_as_bytes
            .iter_mut()
            .skip(byte_offset)
            .zip(iter)
            .fold(0, |acc, (byte, new_byte)| {
                *byte = new_byte;
                acc + 1
            });
    (u32::from_be_bytes(values_as_bytes), bytes_written)
}

/// Fill buffer with zero-bit padding
///
/// If with_trailing_one is set to true, the padding begins with a one-bit.
/// In case the function succeeds, it returns the size of the padding.
pub fn zero_pad_block_buffer(
    buffer: &BlockBuffer,
    with_trailing_one: bool,
) -> Result<usize, ReturnCode> {
    let mut padding_iterator_with_trailing_one =
        core::iter::once(1 << 7 as u8).chain(core::iter::repeat(0));
    let mut padding_iterator_without_trailing_one = core::iter::repeat(0);

    let (bytes, result) = match with_trailing_one {
        true => buffer.append_bytes(&mut padding_iterator_with_trailing_one)?,
        false => buffer.append_bytes(&mut padding_iterator_without_trailing_one)?,
    };

    assert!(result);
    Ok(bytes)
}

/// Fill buffer with a zero-padding and message_size
///
/// The last 8 bytes will consist of message_size in big endian.
/// If with_trailing_one is set to true, the padding will begin with a zero-bit.
/// In case the function succeeds, it returns the size of the padding.
pub fn message_size_pad_block_buffer(
    buffer: &BlockBuffer,
    message_size: u64,
    with_trailing_one: bool,
) -> Result<usize, ReturnCode> {
    let padding_iterator_without_trailing_one = core::iter::repeat(0);
    let padding_iterator_with_trailing_one =
        core::iter::once(1 << 7 as u8).chain(core::iter::repeat(0));

    if buffer.filled_bytes() > 56 {
        //If false, the message size does not fit into the buffer.
        Err(ReturnCode::ENOMEM)
    } else {
        //Fill buffer with zero-padding until only the message size does fit.
        let padding_size = 56 - buffer.filled_bytes();

        let (bytes, result) = match with_trailing_one {
            true => {
                buffer.append_bytes(&mut padding_iterator_with_trailing_one.take(padding_size))?
            }

            false => buffer
                .append_bytes(&mut padding_iterator_without_trailing_one.take(padding_size))?,
        };

        //Fill buffer with message size.
        assert!(!result);
        let (message_size_bytes, result) =
            buffer.append_bytes(&mut message_size.to_be_bytes().iter().cloned())?;

        assert!(result);
        Ok(bytes + message_size_bytes)
    }
}

/// Copy the contents of the input_slice into the output_slice byte-by-byte
///
/// The content of the input_slice is interpreted as big-endian.
/// The function stops when either the whole input_slice is copies or the output_slice is full.
pub fn convert_u32_slice_to_u8_slice(input_slice: &[u32], output_slice: &mut [u8]) {
    input_slice
        .iter()
        .zip(output_slice.chunks_exact_mut(4))
        .for_each(|(input, output_chunk)| {
            let bytes = input.to_be_bytes();
            output_chunk
                .iter_mut()
                .zip(bytes.iter())
                .for_each(|(output_byte, byte)| *output_byte = *byte);
        });
}

/// Buffer of sixteen u32 values
///
/// This buffer offers an interface for appending data byte-by-byte.
/// In the end, the buffer returns these bytes interpreted as
/// 16 big-endian u32 values. This is useful as this representation
/// is used internally inside SHA, SHA2-256 and SHA2-224.
pub struct BlockBuffer<'a> {
    block_buffer: TakeCell<'a, [u32; 16]>,
    byte_counter: Cell<usize>,
}

impl<'a> BlockBuffer<'a> {
    /// Append content from iter to the block buffer until either the buffer is full or the iterator is exhausted.
    /// Returns the number of bytes taken from iterator.
    /// Second return value is true when the buffer is full after append.
    pub fn append_bytes(
        &self,
        iter: &mut dyn Iterator<Item = u8>,
    ) -> Result<(usize, bool), ReturnCode> {
        let former_byte_counter = self.byte_counter.get();

        self.block_buffer
            .map(|block_buffer| {
                //First, fill a partly filled u32
                let (new_value, bytes_written) = extend_u32(
                    block_buffer[former_byte_counter / 4],
                    former_byte_counter % 4,
                    iter,
                );
                block_buffer[former_byte_counter / 4] = new_value;
                self.byte_counter
                    .update(|byte_counter| byte_counter + bytes_written);

                let mut block_buffer_iter = block_buffer[self.byte_counter.get() / 4..].iter_mut();

                //Fill until either buffer is full or iter exhausted
                while let Some(word) = block_buffer_iter.next() {
                    if let Some(next_byte) = iter.next() {
                        //FIXME: THIS IS UGLY AS HELL!
                        let (new_value, bytes_written) =
                            extend_u32(u32::from_be_bytes([next_byte, 0, 0, 0]), 1, iter);
                        *word = new_value;
                        self.byte_counter
                            .update(|byte_counter| byte_counter + bytes_written + 1);
                    } else {
                        break;
                    }
                }
            })
            .ok_or(ReturnCode::EBUSY)?;

        assert!(self.byte_counter.get() <= 64);
        let bytes_taken = self.byte_counter.get() - former_byte_counter;
        if self.full() {
            Ok((bytes_taken, true))
        } else {
            Ok((bytes_taken, false))
        }
    }

    /// Erase contents of blockbuffer
    pub fn reset(&self) {
        self.byte_counter.set(0)
    }

    /// Return content of blockbuffer when full, then reset.
    pub fn flush_and_reset(&self) -> Result<[u32; 16], ReturnCode> {
        match self.byte_counter.get() {
            64 => self
                .block_buffer
                .map(|block_buffer| {
                    self.reset();
                    block_buffer.clone()
                })
                .ok_or(ReturnCode::EBUSY),
            _ => Err(ReturnCode::ENOMEM),
        }
    }

    /// Create a new empty blockbuffer using provided buffer.
    pub fn new(buffer: &'a mut [u32; 16]) -> BlockBuffer<'a> {
        BlockBuffer {
            block_buffer: TakeCell::new(buffer),
            byte_counter: Cell::new(0),
        }
    }

    /// Return the current number of filled bytes inside buffer
    pub fn filled_bytes(&self) -> usize {
        self.byte_counter.get()
    }

    /// Returns true when block buffer is full.
    pub fn full(&self) -> bool {
        self.filled_bytes() == 64
    }
}

/// Generic trait for hash functions that work like SHA
///
/// The SoftShaHasher provides an interface for SHA-like hash functions independent
/// from the size of the returned hash. It simplifies the implementation
/// of the hash function because it separates buffer handling from the actual implementation
/// which needs to work on full blocks only.
pub trait SoftShaHasher<'a, H: HashType> {
    fn set_client(&'a self, client: &'a dyn HasherClient<H>);
    fn fill_buffer(
        &self,
        iter: &mut dyn Iterator<Item = H::Input>,
    ) -> Result<(usize, bool), ReturnCode>;
    fn get_hash(&self) -> Result<(), ReturnCode>;
    fn reset(&self);
    fn process_round(&self) -> Result<(), ReturnCode>;
    fn call_data_processed_callback(&self);
}

impl<'a, T, H> Hasher<'a, H> for T
where
    T: SoftShaHasher<'a, H>,
    H: HashType,
{
    fn set_client(&'a self, client: &'a dyn HasherClient<H>) {
        SoftShaHasher::set_client(self, client)
    }

    fn input_data(
        &self,
        iter: &mut dyn Iterator<Item = H::Input>,
    ) -> Result<(usize, bool), ReturnCode> {
        let (bytes_processed, block_ready) = self.fill_buffer(iter)?;

        if block_ready {
            self.process_round()?;
        }

        self.call_data_processed_callback();
        Ok((bytes_processed, true))
    }

    fn get_hash(&self) -> Result<(), ReturnCode> {
        SoftShaHasher::get_hash(self).and_then(|_| {
            SoftShaHasher::reset(self);
            Ok(())
        })
    }

    fn reset(&self) {
        SoftShaHasher::reset(self)
    }
}
