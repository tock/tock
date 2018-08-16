//! Test the AES CCM implementation on top of AES hardware.

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil::symmetric_encryption::{CCMClient, AES128CCM, AES128_KEY_SIZE, CCM_NONCE_LENGTH};
use kernel::ReturnCode;

pub struct Test<'a, A: AES128CCM<'a>> {
    aes_ccm: &'a A,

    buf: TakeCell<'static, [u8]>,
    current_test: Cell<usize>,
    encrypting: Cell<bool>,

    // (a_data, m_data, c_data, nonce, confidential, mic_len)
    tests: [(
        &'static [u8],
        &'static [u8],
        &'static [u8],
        &'static [u8],
        bool,
        usize,
    ); 3],
}

impl<A: AES128CCM<'a>> Test<'a, A> {
    pub fn new(aes_ccm: &'a A, buf: &'static mut [u8]) -> Test<'a, A> {
        Test {
            aes_ccm: aes_ccm,
            buf: TakeCell::new(buf),
            current_test: Cell::new(0),
            encrypting: Cell::new(true),
            tests: [
                (
                    &BEACON_UNSECURED[0..26],
                    &BEACON_UNSECURED[26..26],
                    &BEACON_SECURED[26..34],
                    &BEACON_NONCE,
                    false,
                    8,
                ),
                (
                    &DATA_UNSECURED[0..26],
                    &DATA_UNSECURED[26..30],
                    &DATA_SECURED[26..30],
                    &DATA_NONCE,
                    true,
                    0,
                ),
                (
                    &MAC_UNSECURED[0..29],
                    &MAC_UNSECURED[29..30],
                    &MAC_SECURED[29..38],
                    &MAC_NONCE,
                    true,
                    8,
                ),
            ],
        }
    }

    pub fn run(&self) {
        debug!("AES CCM* encryption/decryption tests");
        self.trigger_test();
    }

    fn next_test(&self) -> bool {
        if self.encrypting.get() {
            self.encrypting.set(false);
        } else {
            self.encrypting.set(true);
            self.current_test.set(self.current_test.get() + 1);
            if self.current_test.get() >= self.tests.len() {
                return false;
            }
        }
        return true;
    }

    fn trigger_test(&self) {
        let (a_data, m_data, c_data, nonce, confidential, mic_len) =
            self.tests[self.current_test.get()];
        let (a_off, m_off, m_len) = (0, a_data.len(), m_data.len());
        let encrypting = self.encrypting.get();

        let buf = match self.buf.take() {
            None => panic!("Test failed: buffer is not present."),
            Some(buf) => buf,
        };

        if encrypting {
            buf[a_off..m_off].copy_from_slice(a_data);
            buf[m_off..m_off + m_len].copy_from_slice(m_data);
        } else {
            buf[a_off..m_off].copy_from_slice(a_data);
            buf[m_off..m_off + m_len + mic_len].copy_from_slice(c_data);
        }

        if self.aes_ccm.set_key(&KEY) != ReturnCode::SUCCESS
            || self.aes_ccm.set_nonce(&nonce) != ReturnCode::SUCCESS
        {
            panic!("Test failed: cannot set key or nonce.");
        }

        let (res, opt_buf) =
            self.aes_ccm
                .crypt(buf, a_off, m_off, m_len, mic_len, confidential, encrypting);
        if res != ReturnCode::SUCCESS {
            debug!("Failed to start test.")
        }
        if let Some(buf) = opt_buf {
            self.buf.replace(buf);
        }
    }

    fn check_test(&self, tag_is_valid: bool) {
        let (a_data, m_data, c_data, _nonce, _confidential, mic_len) =
            self.tests[self.current_test.get()];
        let (a_off, m_off, m_len) = (0, a_data.len(), m_data.len());
        let encrypting = self.encrypting.get();

        let buf = match self.buf.take() {
            None => panic!("Test failed: buffer is not present."),
            Some(buf) => buf,
        };

        if encrypting {
            let a_matches = buf[a_off..m_off]
                .iter()
                .zip(a_data.iter())
                .all(|(a, b)| *a == *b);
            let c_matches = buf[m_off..m_off + m_len + mic_len]
                .iter()
                .zip(c_data.iter())
                .all(|(a, b)| *a == *b);
            if a_matches && c_matches && tag_is_valid {
                debug!(
                    "OK! (current_test={}, encrypting={}, tag_is_valid={})",
                    self.current_test.get(),
                    self.encrypting.get(),
                    tag_is_valid
                );
            } else {
                debug!("Failed: a_matches={}, c_matches={}, (current_test={}, encrypting={}, tag_is_valid={}",
                       a_matches,
                       c_matches,
                       self.current_test.get(),
                       self.encrypting.get(),
                       tag_is_valid);
                for (a, b) in buf[m_off..m_off + m_len + mic_len]
                    .iter()
                    .zip(c_data.iter())
                {
                    debug!("{:x} vs {:x}", *a, *b);
                }
            }
        } else {
            let a_matches = buf[a_off..m_off]
                .iter()
                .zip(a_data.iter())
                .all(|(a, b)| *a == *b);
            let m_matches = buf[m_off..m_off + m_len]
                .iter()
                .zip(m_data.iter())
                .all(|(a, b)| *a == *b);
            if a_matches && m_matches && tag_is_valid {
                debug!(
                    "OK! (current_test={}, encrypting={}, tag_is_valid={})",
                    self.current_test.get(),
                    self.encrypting.get(),
                    tag_is_valid
                );
            } else {
                debug!("Failed: a_matches={}, m_matches={}, (current_test={}, encrypting={}, tag_is_valid={}",
                       a_matches,
                       m_matches,
                       self.current_test.get(),
                       self.encrypting.get(),
                       tag_is_valid);
            }
        }

        self.buf.replace(buf);
    }
}

impl<A: AES128CCM<'a>> CCMClient for Test<'a, A> {
    fn crypt_done(&self, buf: &'static mut [u8], res: ReturnCode, tag_is_valid: bool) {
        self.buf.replace(buf);
        if res != ReturnCode::SUCCESS {
            debug!("Test failed: crypt_done returned {:?}", res);
        } else {
            self.check_test(tag_is_valid);
            if self.next_test() {
                self.trigger_test()
            }
        }
    }
}

static KEY: [u8; AES128_KEY_SIZE] = [
    0xC0, 0xC1, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF,
];

// IEEE 802.15.4-2015, Annex C.2.1.1, Secured beacon frame
static BEACON_SECURED: [u8; 34] = [
    0x08, 0xD0, 0x84, 0x21, 0x43, 0x01, 0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0x02, 0x05, 0x00,
    0x00, 0x00, 0x55, 0xCF, 0x00, 0x00, 0x51, 0x52, 0x53, 0x54, 0x22, 0x3B, 0xC1, 0xEC, 0x84, 0x1A,
    0xB5, 0x53,
];

// IEEE 802.15.4-2015, Annex C.2.1.2, Unsecured beacon frame with auxiliary
// security header included and the security bits set
static BEACON_UNSECURED: [u8; 26] = [
    0x08, 0xD0, 0x84, 0x21, 0x43, 0x01, 0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0x02, 0x05, 0x00,
    0x00, 0x00, 0x55, 0xCF, 0x00, 0x00, 0x51, 0x52, 0x53, 0x54,
];

// IEEE 802.15.4-2015, Annex C.2.1.3, Nonce for beacon frame
static BEACON_NONCE: [u8; CCM_NONCE_LENGTH] = [
    0xAC, 0xDE, 0x48, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x02,
];

// IEEE 802.15.4-2015, Annex C.2.2.1, Secured data frame
static DATA_SECURED: [u8; 30] = [
    0x69, 0xDC, 0x84, 0x21, 0x43, 0x02, 0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x48, 0xDE, 0xAC, 0x04, 0x05, 0x00, 0x00, 0x00, 0xD4, 0x3E, 0x02, 0x2B,
];

// IEEE 802.15.4-2015, Annex C.2.2.2, Unsecured data frame with auxiliary
// security header included and the security bits set
static DATA_UNSECURED: [u8; 30] = [
    0x69, 0xDC, 0x84, 0x21, 0x43, 0x02, 0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0x01, 0x00, 0x00,
    0x00, 0x00, 0x48, 0xDE, 0xAC, 0x04, 0x05, 0x00, 0x00, 0x00, 0x61, 0x62, 0x63, 0x64,
];

// IEEE 802.15.4-2015, Annex C.2.2.2, Nonce for data frame
static DATA_NONCE: [u8; CCM_NONCE_LENGTH] = [
    0xAC, 0xDE, 0x48, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x04,
];

// IEEE 802.15.4-2015, Annex C.2.3.1, Secured MAC command frame
static MAC_SECURED: [u8; 38] = [
    0x2B, 0xDC, 0x84, 0x21, 0x43, 0x02, 0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0xFF, 0xFF, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0x06, 0x05, 0x00, 0x00, 0x00, 0x01, 0xD8, 0x4F, 0xDE,
    0x52, 0x90, 0x61, 0xF9, 0xC6, 0xF1,
];

// IEEE 802.15.4-2015, Annex C.2.3.2, Unsecured MAC command frame with auxiliary
// security header included and the security bits set
static MAC_UNSECURED: [u8; 30] = [
    0x2B, 0xDC, 0x84, 0x21, 0x43, 0x02, 0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0xFF, 0xFF, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x48, 0xDE, 0xAC, 0x06, 0x05, 0x00, 0x00, 0x00, 0x01, 0xCE,
];

// IEEE 802.15.4-2015, Annex C.2.3.2, Nonce for MAC frame
static MAC_NONCE: [u8; CCM_NONCE_LENGTH] = [
    0xAC, 0xDE, 0x48, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x06,
];
