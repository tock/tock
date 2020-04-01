use crate::sha::common::{
    convert_u32_slice_to_u8_slice, message_size_pad_block_buffer, zero_pad_block_buffer,
    BlockBuffer, SoftShaHasher,
};
use crate::sha::sha_constants::{get_k_sha1, SHA_1_INITIALISATION_VECTOR};

use crate::hash::{hash_functions, HasherClient};
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::ReturnCode;

pub struct Sha1<'a> {
    client: OptionalCell<&'a dyn HasherClient<hash_functions::SHA1>>,
    hash_buffer: TakeCell<'a, [u32; 5]>,
    block_counter: Cell<usize>,
    block_buffer: BlockBuffer<'a>,
    dynamic_deferred_call: &'a DynamicDeferredCall,
    deferred_call_handle: OptionalCell<DeferredCallHandle>,
    deferred_call_type: Cell<DeferredCallType>,
}

impl<'a> SoftShaHasher<'a, hash_functions::SHA1> for Sha1<'a> {
    fn set_client(&'a self, client: &'a dyn HasherClient<hash_functions::SHA1>) {
        self.client.set(client);
    }

    fn fill_buffer(&self, iter: &mut dyn Iterator<Item = u8>) -> Result<(usize, bool), ReturnCode> {
        self.block_buffer.append_bytes(iter)
    }

    fn process_round(&self) -> Result<(), ReturnCode> {
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
                let mut hash_byte_buffer: [u8; 20] = [0; 20];
                convert_u32_slice_to_u8_slice(hash_buffer, &mut hash_byte_buffer);

                self.deferred_call_type
                    .set(DeferredCallType::HashReady(Ok(hash_byte_buffer)));
                self.deferred_call_handle
                    .map(|handle| self.dynamic_deferred_call.set(*handle));
            },
        );

        result
    }

    fn reset(&self) {
        self.hash_buffer
            .map(|hb| hb.copy_from_slice(&SHA_1_INITIALISATION_VECTOR));
        self.block_buffer.reset();
        self.block_counter.set(0);
    }
}

impl<'a> DynamicDeferredCallClient for Sha1<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        use crate::sha::sha1::DeferredCallType::{DataProcessed, HashReady};

        let ctype = self.deferred_call_type.take();

        match ctype {
            DataProcessed(opt) => self.client.map(|c| c.data_processed(opt)),
            HashReady(Ok(slice)) => self.client.map(|c| {
                let mut arr: [u8; 20] = [0; 20];
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
    HashReady(Result<[u8; 20], ReturnCode>),
}
impl Default for DeferredCallType {
    fn default() -> Self {
        DeferredCallType::None
    }
}

impl<'a> Sha1<'a> {
    fn ft(t: usize, x: u32, y: u32, z: u32) -> u32 {
        match t {
            0..=19 => (x & y) | (!x & z),
            20..=39 | 60..=79 => x ^ y ^ z,
            40..=59 => (x & y) | (x & z) | (y & z),
            _ => panic!("Invalid index!"),
        }
    }

    fn block_round(&self, message_block: &[u32; 16]) -> Result<(), ReturnCode> {
        let mut w: [u32; 16] = [0; 16];

        w.copy_from_slice(message_block);
        self.hash_buffer
            .map(|hash_buffer| {
                let mask = 0xf; // x & mask == x % 16
                let mut c = hash_buffer.clone();

                for t in 0..80 {
                    //let s = t % 16;
                    let s = t & mask;

                    if t >= 16 {
                        w[s] = (w[(s + 13) & mask] ^ w[(s + 8) & mask] ^ w[(s + 2) & mask] ^ w[s])
                            .rotate_left(1);
                    }

                    let temp = (c[0].rotate_left(5))
                        .wrapping_add(Self::ft(t, c[1], c[2], c[3]))
                        .wrapping_add(c[4])
                        .wrapping_add(get_k_sha1(t))
                        .wrapping_add(w[s]);

                    c[4] = c[3];
                    c[3] = c[2];
                    c[2] = c[1].rotate_left(30);
                    c[1] = c[0];
                    c[0] = temp;
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
        hash_buffer: &'a mut [u32; 5],
        block_buffer: &'a mut [u32; 16],
        dynamic_deferred_call: &'a DynamicDeferredCall,
    ) -> Sha1<'a> {
        hash_buffer.copy_from_slice(&SHA_1_INITIALISATION_VECTOR);

        Sha1 {
            client: OptionalCell::empty(),
            hash_buffer: TakeCell::new(hash_buffer),
            block_counter: Cell::new(0),
            block_buffer: BlockBuffer::new(block_buffer),
            dynamic_deferred_call,
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
}
