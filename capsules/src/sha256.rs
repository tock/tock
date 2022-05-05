//! Software implementation of SHA-256.
//!
//! Implementation is based on the
//! Wikipedia description of the algorithm. It performs the hash using
//! 32-bit native values, translating the input data into the
//! endianness of the processor and translating the output into
//! big endian format.

use core::cell::Cell;
use kernel::debug;
use kernel::dynamic_deferred_call::{
    DeferredCallHandle, DynamicDeferredCall, DynamicDeferredCallClient,
};
use kernel::hil::digest::Client;
use kernel::hil::digest::{Digest, DigestData, DigestHash, DigestVerify};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Data,
    Hash,
    Verify,
}

const SHA_BLOCK_LEN_BYTES: usize = 64;
const SHA_256_OUTPUT_LEN_BYTES: usize = 32;

pub struct Sha256Software<'a> {
    state: Cell<State>,

    client: OptionalCell<&'a dyn Client<'a, SHA_256_OUTPUT_LEN_BYTES>>,
    input_data: OptionalCell<LeasableBuffer<'static, u8>>,
    data_buffer: MapCell<[u8; SHA_BLOCK_LEN_BYTES]>,
    buffered_length: Cell<usize>,
    total_length: Cell<usize>,
        
    
    // Used to store the hash or the hash to compare against with verify
    output_data: Cell<Option<&'static mut [u8; SHA_256_OUTPUT_LEN_BYTES]>>,

    hash_values: Cell<[u32; 8]>,
    round_constants: MapCell<[u32; SHA_BLOCK_LEN_BYTES]>,

    deferred_caller: &'a DynamicDeferredCall,
    handle: OptionalCell<DeferredCallHandle>,
}

impl<'a> Sha256Software<'a> {
    pub fn new(call: &'a DynamicDeferredCall) -> Sha256Software<'a> {
        Sha256Software {
            state: Cell::new(State::Idle),
            client: OptionalCell::empty(),
            input_data: OptionalCell::empty(),
            data_buffer: MapCell::new([0; SHA_BLOCK_LEN_BYTES]),
            buffered_length: Cell::new(0),
            total_length: Cell::new(0),
            
            output_data: Cell::new(None),

            hash_values: Cell::new([0; 8]),
            round_constants: MapCell::new([0; 64]),

            deferred_caller: call,
            handle: OptionalCell::empty(),
        }
    }

    pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
        self.handle.replace(handle);
    }

    pub fn busy(&self) -> bool {
        match self.state.get() {
            State::Idle => false,
            _ => true,
        }
    }

    pub fn initialize(&self) -> Result<(), ErrorCode> {
        if !self.busy() {
            self.buffered_length.set(0);
            self.data_buffer.map(|b| {
                for i in 0..SHA_BLOCK_LEN_BYTES {
                    b[i] = 0;
                }
            });
            self.hash_values.set([
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ]);
            self.round_constants.map(|k| {
                k[0] = 0x428a2f98;
                k[1] = 0x71374491;
                k[2] = 0xb5c0fbcf;
                k[3] = 0xe9b5dba5;
                k[4] = 0x3956c25b;
                k[5] = 0x59f111f1;
                k[6] = 0x923f82a4;
                k[7] = 0xab1c5ed5;
                k[8] = 0xd807aa98;
                k[9] = 0x12835b01;
                k[10] = 0x243185be;
                k[11] = 0x550c7dc3;
                k[12] = 0x72be5d74;
                k[13] = 0x80deb1fe;
                k[14] = 0x9bdc06a7;
                k[15] = 0xc19bf174;
                k[16] = 0xe49b69c1;
                k[17] = 0xefbe4786;
                k[18] = 0x0fc19dc6;
                k[19] = 0x240ca1cc;
                k[20] = 0x2de92c6f;
                k[21] = 0x4a7484aa;
                k[22] = 0x5cb0a9dc;
                k[23] = 0x76f988da;
                k[24] = 0x983e5152;
                k[25] = 0xa831c66d;
                k[26] = 0xb00327c8;
                k[27] = 0xbf597fc7;
                k[28] = 0xc6e00bf3;
                k[29] = 0xd5a79147;
                k[30] = 0x06ca6351;
                k[31] = 0x14292967;
                k[32] = 0x27b70a85;
                k[33] = 0x2e1b2138;
                k[34] = 0x4d2c6dfc;
                k[35] = 0x53380d13;
                k[36] = 0x650a7354;
                k[37] = 0x766a0abb;
                k[38] = 0x81c2c92e;
                k[39] = 0x92722c85;
                k[40] = 0xa2bfe8a1;
                k[41] = 0xa81a664b;
                k[42] = 0xc24b8b70;
                k[43] = 0xc76c51a3;
                k[44] = 0xd192e819;
                k[45] = 0xd6990624;
                k[46] = 0xf40e3585;
                k[47] = 0x106aa070;
                k[48] = 0x19a4c116;
                k[49] = 0x1e376c08;
                k[50] = 0x2748774c;
                k[51] = 0x34b0bcb5;
                k[52] = 0x391c0cb3;
                k[53] = 0x4ed8aa4a;
                k[54] = 0x5b9cca4f;
                k[55] = 0x682e6ff3;
                k[56] = 0x748f82ee;
                k[57] = 0x78a5636f;
                k[58] = 0x84c87814;
                k[59] = 0x8cc70208;
                k[60] = 0x90befffa;
                k[61] = 0xa4506ceb;
                k[62] = 0xbef9a3f7;
                k[63] = 0xc67178f2;
            });
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }


    // Complete the hash and produce a final hash result.
    fn complete_sha256(&self) {        
        let mut buffered_length = self.buffered_length.get();
        // This shouldn't be necessary, as temp buffer should never be
        // full. But if it is fill, appending the 1 will be an
        // out-of-bounds access and panic, so check and clear
        // the buffered block just in case.
        if buffered_length == 64 {
            self.data_buffer.map(|b| {
                self.compute_block(b);
                for i in 0..SHA_BLOCK_LEN_BYTES {
                    b[i] = 0;
                }
            });
            buffered_length = buffered_length - 64;
        }
        if buffered_length < 64 {
            self.data_buffer.map(|b| {
                for i in buffered_length..SHA_BLOCK_LEN_BYTES {
                    b[i] = 0;
                }
            });
        }

        self.data_buffer.map(|b| {
            // Append the 1
            b[buffered_length] = 0x80;
            buffered_length = buffered_length + 1;
            // The length is 56 because of the 8 bytes appended.
            // Since a block is 64 bytes, this means the last block
            // must have at most 56 bytes including the appended 1, or
            // it will bleed into the next block.
            if buffered_length > 56 {
                for i in buffered_length..SHA_BLOCK_LEN_BYTES {
                    b[i] = 0;
                }
                self.compute_block(b);
                for i in 0..SHA_BLOCK_LEN_BYTES {
                    b[i] = 0;
                }
                buffered_length = 0;
            }
            let total_length = self.total_length.get();
            let length64 = (total_length * 8) as u64;
            let len_high: u32 = (length64 >> 32) as u32;
            let len_low: u32 = (length64 & 0xffffffff) as u32;
            b[56] = (len_high >> 24 & 0xff) as u8;
            b[57] = (len_high >> 16 & 0xff) as u8;
            b[58] = (len_high >>  8 & 0xff) as u8;
            b[59] = (len_high >>  0 & 0xff) as u8;
            b[60] = (len_low  >> 24 & 0xff) as u8;
            b[61] = (len_low  >> 16 & 0xff) as u8;
            b[62] = (len_low  >>  8 & 0xff) as u8;
            b[63] = (len_low  >>  0 & 0xff) as u8;
            self.compute_block(b); 
        });
    }
    
    // This method computes SHA256 on data in input_data,
    // updating the internal hash state. `data_buffer`
    // contains input data that did or does not fill a block:
    // the implementation first fills temp_buffer and computes
    // on it, then operates on input_data. If the end of
    // input_data does not complete a block then the remainder
    // is stored in data_buffer.
    fn compute_sha256(&self) {
        let mut data = self.input_data.take().unwrap();
        let data_length = data.len();
        self.total_length.set(self.total_length.get() + data_length);
        let mut buffered_length = self.buffered_length.get();
        if buffered_length != 0 {
            // Copy bytes into the front of the temp buffer and
            // compute if it fills.
            self.data_buffer.map(|b| {
                let copy_len = if data_length + buffered_length >= SHA_BLOCK_LEN_BYTES {
                    SHA_BLOCK_LEN_BYTES - buffered_length
                } else {
                    data_length
                };

                for i in 0..copy_len {
                    b[i + buffered_length] = data[i];
                }
                data.slice(copy_len..data.len());
                buffered_length += copy_len;
                //debug!("SHA256: Copying {} bytes into SHA buffer, {} bytes remain", copy_len, data.len());
                
                if buffered_length == SHA_BLOCK_LEN_BYTES {
                    self.compute_block(b);
                }
            });
        }
        // Process blocks
        while data.len() >= 64 {
            self.compute_buffer(&data[0..64]);
            data.slice(64..data.len());
        }
        // Process tail end of block
        if data.len() != 0 {
            self.data_buffer.map(|b| {
                for i in 0..data.len() {
                    b[i] = data[i];
                }
                buffered_length = data.len();
                // Go to end of data.
                data.slice(data.len()..data.len());
            });
        }
        self.input_data.set(data);
        self.buffered_length.set(buffered_length);
    }

    fn right_rotate(&self, x: u32, rotate: u32) -> u32 {
        (x >> rotate) | (x << (32 - rotate))
    }

    // Note: slice MUST be >= 64 bytes long
    fn compute_buffer(&self, buffer: &[u8]) {
        debug!("SHA256: Computing buffer.");
        // This is clearly inefficient (copy a u8 array into a u32
        // array), but it's better than using unsafe.  This
        // implementation is not intended to be high performance.
        let mut message_schedule: [u32; 64] = [0; 64];
        for i in 0..16 {
            let val: u32 = (buffer[i * 4 + 0] as u32) << 24 |
                           (buffer[i * 4 + 1] as u32) << 16 |
                           (buffer[i * 4 + 2] as u32) << 8 |
                           (buffer[i * 4 + 3] as u32);
            message_schedule[i] = val;
        }
        self.perform_sha(&mut message_schedule);
    }

    fn compute_block(&self, data: &mut [u8; 64]) {
        self.compute_buffer(data);
            
/*        // This is clearly inefficient (copy a u8 array into a u32
        // array), but it's better than using unsafe.  This
        // implementation is not intended to be high performance.
        let mut message_schedule: [u32; 64] = [0; 64];
        for i in 0..16 {
            let val: u32 = (data[i * 4 + 0] as u32) << 24 |
            (data[i * 4 + 1] as u32) << 16 |
            (data[i * 4 + 2] as u32) << 8 |
            (data[i * 4 + 3] as u32);
            message_schedule[i] = val;
        }
        self.perform_sha(&mut message_schedule);*/
    }
    
    fn perform_sha(&self, message_schedule: &mut [u32; 64]) {
        // Message schedule
        for i in 16..64 {
            let mut s0 = self.right_rotate(message_schedule[i - 15], 7);
            s0 ^= self.right_rotate(message_schedule[i - 15], 18);
            s0 ^= message_schedule[i - 15] >> 3;
            let mut s1 = self.right_rotate(message_schedule[i - 2], 17);
            s1 ^= self.right_rotate(message_schedule[i - 2], 19);
            s1 ^= message_schedule[i - 2] >> 10;
            message_schedule[i] = message_schedule[i - 16] + s0 + message_schedule[i - 7] + s1;
        }

        // Compression
        let mut hashes = self.hash_values.get();
        for i in 0..64 {
            let s1 = self.right_rotate(hashes[4], 6)
                ^ self.right_rotate(hashes[4], 11)
                ^ self.right_rotate(hashes[4], 25);
            let ch = (hashes[4] & hashes[5]) ^ ((!hashes[4]) & hashes[6]);
            let constant = self.round_constants.map_or(0, |k| k[i]);
            let temp1 = hashes[7] + s1 + ch + constant + message_schedule[i];
            let s0 = self.right_rotate(hashes[0], 2)
                ^ self.right_rotate(hashes[0], 13)
                ^ self.right_rotate(hashes[0], 22);
            let maj = (hashes[0] & hashes[1]) ^ (hashes[0] & hashes[2]) ^ (hashes[1] & hashes[2]);
            let temp2 = s0 + maj;

            hashes[7] = hashes[6];
            hashes[6] = hashes[5];
            hashes[5] = hashes[4];
            hashes[4] = hashes[3].wrapping_add(temp1);
            hashes[3] = hashes[2];
            hashes[2] = hashes[1];
            hashes[1] = hashes[0];
            hashes[0] = temp1.wrapping_add(temp2);
        }

        let mut new_hashes = self.hash_values.get();
        for i in 0..8 {
            new_hashes[i] = new_hashes[i].wrapping_add(hashes[i]);
        }
        self.hash_values.set(new_hashes);
    }
}

impl<'a> DigestData<'a, 32> for Sha256Software<'a> {
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        if self.busy() {
            Err((ErrorCode::BUSY, data.take()))
        } else {
            self.state.set(State::Data);
            if self.handle.is_none() {
                Err((ErrorCode::FAIL, data.take()))
            } else {
                let len = data.len();
                self.handle.map(|handle| {
                    self.deferred_caller.set(*handle);
                    self.input_data.set(data);
                    self.compute_sha256();
                });
                Ok(len)
            }
        }
    }

    fn clear_data(&self) {
        let _ = self.initialize();
    }
}

impl<'a> DigestHash<'a, 32> for Sha256Software<'a> {
    fn run(
        &'a self,
        digest: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        if self.busy() {
            Err((ErrorCode::BUSY, digest))
        } else {
            self.state.set(State::Hash);
            if self.handle.is_none() {
                Err((ErrorCode::FAIL, digest))
            } else {
                self.handle.map(|handle| {
                    self.complete_sha256();
                    for i in 0..8 {
                        let val = self.hash_values.get()[i];
                        digest[4 * i + 3] = (val >> 0 & 0xff) as u8;
                        digest[4 * i + 2] = (val >> 8 & 0xff) as u8;
                        digest[4 * i + 1] = (val >> 16 & 0xff) as u8;
                        digest[4 * i + 0] = (val >> 24 & 0xff) as u8;
                    }
                    self.output_data.set(Some(digest));
                    self.deferred_caller.set(*handle);
                });
                Ok(())
            }
        }
    }
}

impl<'a> DigestVerify<'a, 32> for Sha256Software<'a> {
    fn verify(
        &'a self,
        compare: &'static mut [u8; 32],
    ) -> Result<(), (ErrorCode, &'static mut [u8; 32])> {
        if self.busy() {
            Err((ErrorCode::BUSY, compare))
        } else {
            self.state.set(State::Verify);
            if self.handle.is_none() {
                Err((ErrorCode::FAIL, compare))
            } else {
                self.handle.map(|handle| {
                    self.complete_sha256();
                    self.output_data.set(Some(compare));
                    self.deferred_caller.set(*handle);
                });
                Ok(())
            }
        }
    }
}

impl<'a> Digest<'a, 32> for Sha256Software<'a> {
    fn set_client(&'a self, client: &'a dyn Client<'a, 32>) {
        self.client.set(client);
    }
}

impl<'a> DynamicDeferredCallClient for Sha256Software<'a> {
    fn call(&self, _handle: DeferredCallHandle) {
        let prior = self.state.get();
        self.state.set(State::Idle);
        match prior {
            State::Idle => {}
            State::Verify => {
                // Do the verification here so we don't have to store
                // the result across the callback.
                let output = self.output_data.replace(None).unwrap();
                let mut pass = true;
                for i in 0..8 {
                    let hashval = self.hash_values.get()[i];
                    if output[4 * i + 3] != (hashval >> 0 & 0xff) as u8
                        || output[4 * i + 2] != (hashval >> 8 & 0xff) as u8
                        || output[4 * i + 1] != (hashval >> 16 & 0xff) as u8
                        || output[4 * i + 0] != (hashval >> 24 & 0xff) as u8
                    {
                        pass = false;
                        let oval = (output[4 * i + 0] as u32) << 24
                            | (output[4 * i + 1] as u32) << 16
                            | (output[4 * i + 2] as u32) << 8
                            | output[4 * i + 3] as u32;
                        debug!("Mismatched output {}: {:08x} not {:08x}", i, hashval, oval);
                        break;
                    }
                }

                self.client.map(|c| {
                    c.verification_done(Ok(pass), output);
                });
            }
            State::Data => {
                // Data already computed in method call
                let data = self.input_data.take().unwrap();
                self.client.map(|client| {
                    client.add_data_done(Ok(()), data.take());
                });
            }
            State::Hash => {
                // Hash already copied in method call.
                let output = self.output_data.replace(None).unwrap();
                self.client.map(|c| {
                    c.hash_done(Ok(()), output);
                });
            }
        }
    }
}
