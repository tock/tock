use crate::sha::common::{
    convert_u32_slice_to_u8_slice, message_size_pad_block_buffer, zero_pad_block_buffer,
    BlockBuffer, SoftShaHasher,
};
use crate::sha::sha_constants::{
    K_32, SHA_224_INITIALISATION_VECTOR, SHA_256_INITIALISATION_VECTOR,
};

use crate::hash::{hash_functions, HashType, HasherClient};
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::ReturnCode;

pub struct Sha2_32<'a, H> {
    client: OptionalCell<&'a dyn HasherClient<H>>,
    hash_buffer: TakeCell<'a, [u32; 8]>,
    block_buffer: BlockBuffer<'a>,
    block_counter: Cell<usize>,
    dynamic_deferred_call: &'a DynamicDeferredCall,
    deferred_call_handle: OptionalCell<DeferredCallHandle>,
    deferred_call_type: Cell<DeferredCallType>,
}

impl<'a> DynamicDeferredCallClient for Sha2_32<'a, hash_functions::SHA2_224> {
    fn call(&self, _handle: DeferredCallHandle) {
        use crate::sha::sha2_32::DeferredCallType::{DataProcessed, HashReady};

        let ctype = self.deferred_call_type.take();

        match ctype {
            DataProcessed(opt) => self.client.map(|c| c.data_processed(opt)),
            HashReady(Ok(slice)) => self.client.map(|c| {
                let mut arr: [u8; 28] = [0; 28];
                slice
                    .iter()
                    .zip(arr.iter_mut())
                    .for_each(|(src, dst)| *dst = *src);
                c.hash_ready(Ok(&arr))
            }),
            HashReady(Err(e)) => self.client.map(|c| c.hash_ready(Err(e))),
            DeferredCallType::None => None,
        };
    }
}

impl<'a> DynamicDeferredCallClient for Sha2_32<'a, hash_functions::SHA2_256> {
    fn call(&self, _handle: DeferredCallHandle) {
        use crate::sha::sha2_32::DeferredCallType::{DataProcessed, HashReady};

        let ctype = self.deferred_call_type.take();

        match ctype {
            DataProcessed(opt) => self.client.map(|c| c.data_processed(opt)),
            HashReady(Ok(slice)) => self.client.map(|c| {
                let mut arr: [u8; 32] = [0; 32];
                slice
                    .iter()
                    .zip(arr.iter_mut())
                    .for_each(|(src, dst)| *dst = *src);
                c.hash_ready(Ok(&arr))
            }),
            HashReady(Err(e)) => self.client.map(|c| c.hash_ready(Err(e))),
            DeferredCallType::None => None,
        };
    }
}

enum DeferredCallType {
    None,
    DataProcessed(Option<ReturnCode>),
    HashReady(Result<[u8; 32], ReturnCode>),
}
impl Default for DeferredCallType {
    fn default() -> Self {
        DeferredCallType::None
    }
}

impl<'a> SoftShaHasher<'a, hash_functions::SHA2_256> for Sha2_32<'a, hash_functions::SHA2_256> {
    fn set_client(&'a self, client: &'a dyn HasherClient<hash_functions::SHA2_256>) {
        Self::set_client(self, client)
    }

    fn fill_buffer(&self, iter: &mut dyn Iterator<Item = u8>) -> Result<(usize, bool), ReturnCode> {
        Self::fill_buffer(self, iter)
    }
    fn get_hash(&self) -> Result<(), ReturnCode> {
        Self::get_hash(self)
    }
    fn reset(&self) {
        self.hash_buffer
            .map(|hb| hb.copy_from_slice(&SHA_256_INITIALISATION_VECTOR));
        Self::reset(self)
    }
    fn process_round(&self) -> Result<(), ReturnCode> {
        Self::process_round(self)
    }
    fn call_data_processed_callback(&self) {
        Self::call_data_processed_callback(self)
    }
}

impl<'a> SoftShaHasher<'a, hash_functions::SHA2_224> for Sha2_32<'a, hash_functions::SHA2_224> {
    fn set_client(&'a self, client: &'a dyn HasherClient<hash_functions::SHA2_224>) {
        Self::set_client(self, client)
    }

    fn fill_buffer(&self, iter: &mut dyn Iterator<Item = u8>) -> Result<(usize, bool), ReturnCode> {
        Self::fill_buffer(self, iter)
    }
    fn get_hash(&self) -> Result<(), ReturnCode> {
        Self::get_hash(self)
    }
    fn reset(&self) {
        self.hash_buffer
            .map(|hb| hb.copy_from_slice(&SHA_224_INITIALISATION_VECTOR));
        Self::reset(self)
    }
    fn process_round(&self) -> Result<(), ReturnCode> {
        Self::process_round(self)
    }
    fn call_data_processed_callback(&self) {
        Self::call_data_processed_callback(self)
    }
}

impl<'a, H: HashType> Sha2_32<'a, H> {
    fn ch(x: u32, y: u32, z: u32) -> u32 {
        (x & y) ^ (!x & z)
    }

    fn maj(x: u32, y: u32, z: u32) -> u32 {
        (x & y) ^ (x & z) ^ (y & z)
    }

    fn bsig0(x: u32) -> u32 {
        x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
    }

    fn bsig1(x: u32) -> u32 {
        x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
    }

    fn ssig0(x: u32) -> u32 {
        x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
    }

    fn ssig1(x: u32) -> u32 {
        x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
    }

    fn block_round(&self, message_block: &[u32; 16]) -> Result<(), ReturnCode> {
        let mut w: [u32; 64] = [0; 64];

        w[..16].copy_from_slice(message_block);

        self.hash_buffer
            .map(|hash_buffer| {
                for i in 16..64 {
                    w[i] = Self::ssig1(w[i - 2])
                        .wrapping_add(w[i - 7])
                        .wrapping_add(Self::ssig0(w[i - 15]))
                        .wrapping_add(w[i - 16]);
                }

                let mut c = hash_buffer.clone();

                for i in 0..64 {
                    let t1 = c[7]
                        .wrapping_add(Self::bsig1(c[4]))
                        .wrapping_add(Self::ch(c[4], c[5], c[6]))
                        .wrapping_add(K_32[i])
                        .wrapping_add(w[i]);

                    let t2 = Self::bsig0(c[0]).wrapping_add(Self::maj(c[0], c[1], c[2]));

                    c[7] = c[6];
                    c[6] = c[5];
                    c[5] = c[4];
                    c[4] = c[3].wrapping_add(t1);
                    c[3] = c[2];
                    c[2] = c[1];
                    c[1] = c[0];
                    c[0] = t1.wrapping_add(t2);
                }

                hash_buffer
                    .iter_mut()
                    .zip(c.iter())
                    .for_each(|(h, ch)| *h = h.wrapping_add(*ch));
            })
            .ok_or(ReturnCode::EBUSY)?;
        self.block_counter.update(|block_counter| block_counter + 1);

        Ok(())
    }

    pub fn new(
        hash_buffer: &'a mut [u32; 8],
        block_buffer: &'a mut [u32; 16],
        dynamic_deferred_call: &'a DynamicDeferredCall,
    ) -> Sha2_32<'a, H> {
        match H::output_bits() {
            224 => {
                hash_buffer.copy_from_slice(&SHA_224_INITIALISATION_VECTOR);
            }

            256 => {
                hash_buffer.copy_from_slice(&SHA_256_INITIALISATION_VECTOR);
            }

            _ => panic!("Unknown hash size!"),
        }

        Sha2_32 {
            client: OptionalCell::empty(),
            hash_buffer: TakeCell::new(hash_buffer),
            block_buffer: BlockBuffer::new(block_buffer),
            block_counter: Cell::new(0),
            dynamic_deferred_call: dynamic_deferred_call,
            deferred_call_handle: OptionalCell::empty(),
            deferred_call_type: Cell::default(),
        }
    }

    pub fn set_handle(&self, handle: DeferredCallHandle) {
        self.deferred_call_handle.set(handle);
    }

    fn append_padding_and_finalise(&self) -> Result<(), ReturnCode> {
        let padding_two_blocks_long = self.block_buffer.filled_bytes() >= 56;
        let message_size =
            (self.block_counter.get() * 512 + self.block_buffer.filled_bytes() * 8) as u64;

        if padding_two_blocks_long {
            zero_pad_block_buffer(&self.block_buffer, true)?;
            self.process_round()?;
            message_size_pad_block_buffer(&self.block_buffer, message_size, false)?;
        } else {
            message_size_pad_block_buffer(&self.block_buffer, message_size, true)?;
        }

        self.process_round()
    }

    fn reset(&self) {
        self.block_buffer.reset();
        self.block_counter.set(0);
    }

    fn set_client(&'a self, client: &'a dyn HasherClient<H>) {
        self.client.set(client);
    }

    fn fill_buffer(&self, iter: &mut dyn Iterator<Item = u8>) -> Result<(usize, bool), ReturnCode> {
        self.block_buffer.append_bytes(iter)
    }

    fn process_round(&self) -> Result<(), ReturnCode> {
        debug_assert!(self.block_buffer.full());
        let message_block = self.block_buffer.flush_and_reset()?;

        self.block_round(&message_block)
    }

    fn call_data_processed_callback(&self) {
        self.deferred_call_type
            .set(DeferredCallType::DataProcessed(None));
        self.deferred_call_handle
            .map(|handle| self.dynamic_deferred_call.set(*handle));
    }

    fn get_hash(&self) -> Result<(), ReturnCode> {
        let result = self.append_padding_and_finalise();

        self.hash_buffer.map_or_else(
            || {
                self.deferred_call_type
                    .set(DeferredCallType::HashReady(Err(ReturnCode::EBUSY)));
                self.deferred_call_handle
                    .map(|handle| self.dynamic_deferred_call.set(*handle));
            },
            |hash_buffer| {
                let mut hash_byte_buffer: [u8; 32] = [0; 32];
                convert_u32_slice_to_u8_slice(hash_buffer, &mut hash_byte_buffer);

                self.deferred_call_type
                    .set(DeferredCallType::HashReady(Ok(hash_byte_buffer)));
                self.deferred_call_handle
                    .map(|handle| self.dynamic_deferred_call.set(*handle));
            },
        );

        result
    }
}
