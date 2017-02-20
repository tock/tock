use chip;
use core::cell::Cell;
use core::mem;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil::aes::{AESDriver, Client};
use kernel::returncode::ReturnCode;
use nvic;
use peripheral_interrupts::NvicIdx;
use peripheral_registers::{AESCCM_REGS, AESCCM_BASE};


// maybe make this to a struct later
// byte  0-15       ;;  Key
// byte  16-24      ;;  Packet counters
// byte  25-32      ;;  IV
static mut CCM_DATA: [u8; 32] = [0; 32];


// byte 0           ;;  Header
// byte 1           ;;  Length
// byte 2           ;;  NOT used
// byte 3-30        ;;  PAYLOAD
// TOTAL PACKET =  Header(1 byte) + Length(1 byte) + RFU (1 byte) + PAYLOAD(27 bytes) = 30
static mut IN_DATA: [u8; 30] = [0; 30];

// byte 0           ;;  Header
// byte 1           ;;  Length+4
// byte 2           ;;  NOT used
// byte 3-30        ;; Encrypted PAYLOAD
// byte 3-34        ;;  MIC
// TOTAL PACKET =  Header(1 byte) + Length(1 byte) + RFU (1 byte) + PAYLOAD(27 bytes) + MIC 4 bytes = 34
static mut OUT_DATA: [u8; 34] = [0; 34];

// scratchdata for temp usage
static mut TMP: [u8; 32] = [0; 32];

pub struct AesCCM {
    regs: *mut AESCCM_REGS,
    client: Cell<Option<&'static Client>>,
    // TODO didn't got it work, i.e. to mutate data in struct as "&mut self"
    // ccm_data: [u8; 32],
    // in_data: [u8; 30],
    // out_data: [u8; 34],
    // tmp: [u8; 32],
    len: Cell<u8>,
}

pub static mut AESCCM: AesCCM = AesCCM::new();

impl AesCCM {
    const fn new() -> AesCCM {
        AesCCM {
            regs: AESCCM_BASE as *mut AESCCM_REGS,
            client: Cell::new(None),
            // ccm_data: [0; 32],
            // in_data: [0; 30],
            // out_data: [0; 34],
            // tmp: [0; 32],
            len: Cell::new(0),
        }
    }

    pub fn ccm_init(&self) {
        let regs: &mut AESCCM_REGS = unsafe { mem::transmute(self.regs) };
        unsafe {
            regs.CNFPTR.set((&CCM_DATA as *const u8) as u32);
            regs.INPTR.set((&IN_DATA as *const u8) as u32);
            regs.OUTPTR.set((&OUT_DATA as *const u8) as u32);
            regs.SCRATCHPTR.set((&TMP as *const u8) as u32);
        }
        // enable aes_ccm
        regs.ENABLE.set(0x02);
    }
    fn set_key(&self, key: &'static mut [u8], len: u8) {
        assert_eq!(len, 16);
        for (i, c) in key.as_ref()[0..16].iter().enumerate() {
            unsafe {
                CCM_DATA[i] = *c;
            }
        }
        unsafe {
            self.client
                .get()
                .map(|client| client.set_key_done(&mut CCM_DATA[0..len as usize], len));
        }
    }

    fn encrypt(&self, pt: &'static mut [u8], len: u8) {
        // TODO features for bigger payload than 27 bytes preferable handled in capsules
        if len > 27 {
            panic!("UN-SUPPORTED UNECR PAYLOAD LEN\r\n");
        }

        let regs: &mut AESCCM_REGS = unsafe { mem::transmute(self.regs) };

        self.len.set(len);

        // set header
        unsafe {
            IN_DATA[1] = self.len.get();
        }

        // mutate payload
        for (i, c) in pt.as_ref()[0..self.len.get() as usize].iter().enumerate() {
            unsafe {
                IN_DATA[i + 3] = *c;
            }
        }

        if regs.ERROR.get() != 0 {
            panic!("ENCRYPTION ERROR before CRYPT {}\r\n", regs.ERROR.get());
        }

        // set encryption mode
        regs.MODE.set(0x00);
        regs.ENDKSGEN.set(0);
        regs.ENDCRYPT.set(0);

        self.enable_nvic();
        self.enable_interrupts();

        regs.KSGEN.set(1);
    }

    fn decrypt(&self, ct: &'static mut [u8], len: u8) {
        // TODO features for bigger payload than 27 bytes preferable handled in capsules
        if len > 31 {
            panic!("UN-SUPPORTED ENC PAYLOAD LEN\r\n");
        }
        let regs: &mut AESCCM_REGS = unsafe { mem::transmute(self.regs) };

        self.len.set(len);

        unsafe {
            IN_DATA[1] = self.len.get();
        }
        // mutate payload
        for (i, c) in ct.as_ref()[0..self.len.get() as usize].iter().enumerate() {
            unsafe {
                IN_DATA[i + 3] = *c;
            }
        }

        if regs.ERROR.get() != 0 {
            panic!("ENCRYPTION ERROR  before CRYPT {}\r\n", regs.ERROR.get());
        }

        // set decryption mode
        regs.MODE.set(0x01);
        regs.ENDKSGEN.set(0);
        regs.ENDCRYPT.set(0);

        self.enable_nvic();
        self.enable_interrupts();

        regs.KSGEN.set(1);
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut AESCCM_REGS = unsafe { mem::transmute(self.regs) };

        if regs.ENDKSGEN.get() == 1 {

            // disable endksgen interrupts, may be un-necessary
            regs.INTENCLR.set(0x01);
            regs.ENDKSGEN.set(0);

            // start encryption/decryption
            regs.ENDCRYPT.set(0);
            regs.CRYPT.set(1);
        }

        if regs.ENDCRYPT.get() == 1 {
            // disable all interrupts related to AES CCM

            self.disable_nvic();
            self.disable_interrupts();
            regs.ENDCRYPT.set(0);

            // Encryption Mode
            if regs.MODE.get() == 0 {
                unsafe {
                    // ct + MIC
                    // panic!("LEN: {:?}\r\n OUT_DATA: {:?}\r\n", self.len.get(), OUT_DATA);
                    self.client
                        .get()
                        .map(|client| client.encrypt_done(&mut OUT_DATA[3..], self.len.get() + 4));
                }
            }
            // Decryption Mode
            else if regs.MODE.get() == 1 {
                unsafe {
                    // pt
                    self.client
                        .get()
                        .map(|client| client.decrypt_done(&mut OUT_DATA[3..], self.len.get() - 4));
                }
            }
        }

        if regs.ERROR.get() == 1 {
            panic!("error AES CCM CRYPT \r\n");
        }

        nvic::clear_pending(NvicIdx::CCM_AAR);
    }

    fn enable_interrupts(&self) {
        let regs: &mut AESCCM_REGS = unsafe { mem::transmute(self.regs) };
        // Enable ENDSKGGEN, ENDSCRYPT and Error Interrupt
        regs.INTENSET.set(1 | 1 << 1 | 1 << 2); // <-> 1 + 2 + 4
    }

    fn disable_interrupts(&self) {
        let regs: &mut AESCCM_REGS = unsafe { mem::transmute(self.regs) };
        regs.INTENCLR.set(1 | 1 << 1 | 1 << 2);
    }

    fn enable_nvic(&self) {
        nvic::enable(NvicIdx::CCM_AAR);
    }

    fn disable_nvic(&self) {
        nvic::disable(NvicIdx::CCM_AAR);
    }

    pub fn set_client<C: Client>(&self, client: &'static C) {
        // test::test_aes_ecb_test();
        self.client.set(Some(client));
    }
}
// Methods of RadioDummy Trait/Interface and are shared between Capsules and Chips
impl AESDriver for AesCCM {
    // This Function is called once Tock is booted
    fn init(&self) {
        self.ccm_init()
    }

    fn set_key(&self, key: &'static mut [u8], len: u8) {
        self.set_key(key, len)
    }

    // This Function is called once a radio packet is to be sent
    fn encrypt(&self, plaintext: &'static mut [u8], len: u8) {
        self.encrypt(plaintext, len)
    }

    // This Function is called once a radio packet is to be sent
    fn decrypt(&self, ciphertext: &'static mut [u8], len: u8) {
        self.decrypt(ciphertext, len)
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn CCM_AAR_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::CCM_AAR);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::CCM_AAR);
}
