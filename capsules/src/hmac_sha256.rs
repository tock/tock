use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn test() {
    // Create `Mac` trait implementation, namely HMAC-SHA256
    let mut mac = Hmac::<Sha256>::new(b"my secret and secure key");
    mac.input(b"input message");

    // `result` has type `MacResult` which is a thin wrapper around array of
    // bytes for providing constant time equality check
    let result = mac.result();

    // To get &[u8] use `code` method, but be carefull, since incorrect use
    // of the code value may permit timing attacks which defeat the security
    // provided by the `MacResult`.
    let code_bytes = result.code();

    // Verify the message
    let mut mac = Hmac::<Sha256>::new(b"my secret and secure key");
    mac.input(b"input message");

    // This just wraps `code_bytes` in a `MacResult` and constant-time-compares
    // to `mac.result()`
    let is_code_correct = mac.verify(code_bytes);

    assert!(is_code_correct);

    debug!("HMAC-SHA256 test OK");
}
