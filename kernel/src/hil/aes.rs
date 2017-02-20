use returncode::ReturnCode;

pub trait AESDriver {
    fn init(&self);
    fn set_key(&self, key: &'static mut [u8], len: u8);
    fn encrypt(&self, plaintext: &'static mut [u8], len: u8);
    fn decrypt(&self, ciphertext: &'static mut [u8], len: u8);
}

pub trait Client {
    fn encrypt_done(&self, ct: &'static mut [u8], len: u8) -> ReturnCode;
    fn decrypt_done(&self, pt: &'static mut [u8], len: u8) -> ReturnCode;
    fn set_key_done(&self, key: &'static mut [u8], len: u8) -> ReturnCode;
}
