use chip;
use core::cell::Cell;
// use kernel::common::take_cell::TakeCell;
use kernel::hil::symmetric_encryption::{SymmetricEncryptionDriver, Client};
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{AESECB_REGS, AESECB_BASE};

#[deny(no_mangle_const_items)]

static mut ECB_DATA: [u8; 48] = [0; 48];
// key 0-15
// cleartext 16-32
// ciphertext 32-47

static mut BUF: [u8; 128] = [0; 128];
static mut DATA: [u8; 128] = [0; 128];

#[no_mangle]
pub struct AesECB {
    regs: *mut AESECB_REGS,
    client: Cell<Option<&'static Client>>,
    ctr: Cell<[u8; 16]>,
    // data: TakeCell<'static, [u8]>,
    remaining: Cell<u8>,
    len: Cell<u8>,
    offset: Cell<u8>,
}

pub static mut AESECB: AesECB = AesECB::new();

impl AesECB {
    const fn new() -> AesECB {
        AesECB {
            regs: AESECB_BASE as *mut AESECB_REGS,
            client: Cell::new(None),
            ctr: Cell::new([0; 16]),
            // data: TakeCell::empty(),
            remaining: Cell::new(0),
            len: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    pub fn ecb_init(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.ECBDATAPTR.set((&ECB_DATA as *const u8) as u32);
        }
    }

    fn update_ctr(&self) {
        // from 15 to 0...
        let mut ctr = self.ctr.get();
        for i in (0..16).rev() {
            ctr[i] += 1;
            if ctr[i] != 0 {
                break;
            }
        }
        self.ctr.set(ctr);
    }


    // check components/drivers_nrf/hal/nrf_ecb.c for inspiration :)
    #[inline(never)]
    #[no_mangle]
    fn crypt(&self) {
        let regs = unsafe { &*self.regs };

        let ctr = self.ctr.get();

        for i in 0..16 {
            unsafe {
                ECB_DATA[i + 16] = ctr[i];
            }
        }

        regs.ENDECB.set(0);
        regs.STARTECB.set(1);

        self.enable_nvic();
        self.enable_interrupts();
    }

    fn set_key(&self, key: &'static mut [u8]) {
        for (i, c) in key.as_ref()[0..16].iter().enumerate() {
            unsafe {
                ECB_DATA[i] = *c;
            }
        }

        unsafe {
            // BUF.copy_from_slice(&ECB_DATA[0..16]);
            self.client.get().map(|client| client.set_key_done(&mut BUF[0..16], 16));
        }

    }

    #[inline(never)]
    #[no_mangle]
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };

        // disable interrupts
        self.disable_nvic();
        self.disable_interrupts();
        nvic::clear_pending(NvicIdx::ECB);

        if regs.ENDECB.get() == 1 {

            let rem = self.remaining.get() as usize;
            let offset = self.offset.get() as usize;
            let take = if rem >= 16 { 16 } else { rem };


            // THIS DON'T WORK FOR MORE THAN 1 BLOCK BECAUSE TAKE EATS UP THE ENTIRE BUF
            // ------------------------------------------------------------------------------------
            // guard that more bytes exist
            // if self.data.is_some() && take > 0 {
            //     self.data.take().map(|buf| {
            //         if buf.len() >= offset + take {
            //             // take at most 16 bytes and XOR with the keystream
            //             for (i, c) in buf.as_ref()[offset .. offset+take].iter().enumerate() {
            //                 // m XOR ECB(k || ctr)
            //                 unsafe { BUF[i] = ECB_DATA[i-offset+32] ^ *c; }
            //                 debug!("{}\r\n", i);
            //             }
            //             self.offset.set( (offset + take) as u8 );
            //             self.remaining.set( (rem - take) as u8 );
            //         }
            //     });
            // }

            // TEMP solution
            if take > 0 {
                for i in offset..offset + take {
                    unsafe {
                        BUF[i] = ECB_DATA[i - offset + 32] ^ DATA[i];
                    }
                }
                self.offset.set((offset + take) as u8);
                self.remaining.set((rem - take) as u8);
                self.update_ctr();
            }

            // USE THIS PRINT TO TEST THAT THE CTR UPDATES ACCORDINGLY
            // debug!("ctr {:?}\r\n", self.ctr.get());
            
            // More bytes to encrypt!!!
            if self.remaining.get() > 0 {
                self.crypt();
            }
            // DONE
            else {
                // panic!("done\r\n");
                unsafe {
                    self.client.get().map(|client| {
                        client.crypt_done(&mut BUF[0..self.len.get() as usize], self.len.get())
                    });
                }
            }
        }
        // else ERROR encrypt error do nothing
    }

    fn enable_interrupts(&self) {
        // set ENDECB bit and ERROR bit
        let regs = unsafe { &*self.regs };
        regs.INTENSET.set(1 | 1 << 1);
    }

    fn disable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.INTENCLR.set(1 | 1 << 1);
    }

    fn enable_nvic(&self) {
        nvic::enable(NvicIdx::ECB);
    }

    fn disable_nvic(&self) {
        nvic::disable(NvicIdx::ECB);
    }

    pub fn set_client<C: Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn set_initial_ctr(&self, iv: &'static mut [u8]) {
        let mut ctr: [u8; 16] = [0; 16];
        for (i, c) in iv.as_ref()[0..16].iter().enumerate() {
            ctr[i] = *c;
        }
        self.ctr.set(ctr);
    }
}

impl SymmetricEncryptionDriver for AesECB {
    // This Function is called once Tock is booted
    fn init(&self) {
        self.ecb_init();
    }

    // capsule ensures that the key is 16 bytes
    fn set_key(&self, key: &'static mut [u8]) {
        self.set_key(key)
    }

    fn crypt_ctr(&self, data: &'static mut [u8], iv: &'static mut [u8], len: u8) {
        self.remaining.set(len);
        self.len.set(len);
        self.offset.set(0);
        // self.data.replace(data);
        
        // append data to "enc/dec" to a the global buf
        for (i, c) in data.as_ref()[0..len as usize].iter().enumerate() {
            unsafe {
                DATA[i] = *c;
            }
        }
        self.set_initial_ctr(iv);
        self.crypt();
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn ECB_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::ECB);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::ECB);
}
