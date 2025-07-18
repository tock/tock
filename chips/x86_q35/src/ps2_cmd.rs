//! Shared command‑queue helper for PS/2 host‑to‑device transactions
//!
//! Centralises the ACK/RESEND handshake and retry logic required by
//! LED, typematic‑rate, scan‑set and similar commands.

use kernel::errorcode::ErrorCode;
use kernel::hil::ps2_traits::PS2Traits;

/// Maximum number of bytes the command helper supports
/// (opcode + parameters + response).
pub const MAX_CMD: usize = 8;

/// Simple fixed‑capacity response buffer.
#[derive(Copy, Clone, Debug)]
pub struct Resp {
    buf: [u8; MAX_CMD],
    len: usize,
}
impl Resp {
    pub const fn new() -> Self {
        Self { buf: [0; MAX_CMD], len: 0 }
    }
    pub fn push(&mut self, b: u8) {
        if self.len < MAX_CMD {
            self.buf[self.len] = b;
            self.len += 1;
        }
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.len]
    }
    pub fn len(&self) -> usize {
        self.len
    }

}

/// Send `cmd` (opcode + optional data) and collect `resp_len` bytes.
/// Automatically retries the entire sequence on `0xFE` (RESEND)
/// up to 3 times.
pub fn send<C: PS2Traits>(
    _ctl: &C, // reference kept for type inference; methods are static
    cmd: &[u8],
    resp_len: usize,
) -> Result<Resp, ErrorCode> {
    const MAX_RETRIES: usize = 3;
    assert!(cmd.len() <= MAX_CMD);
    assert!(resp_len <= MAX_CMD);

    let mut retries = 0;
    let mut resp = Resp::new();

    let _ = _ctl; // suppress unused warning

    'retry: loop {
        // host => device
        for &b in cmd {
            C::wait_input_ready();
            C::write_data(b);

            C::wait_output_ready();
            match C::read_data() {
                0xFA => {}           // ACK – proceed
                0xFE => {
                    retries += 1;
                    if retries > MAX_RETRIES {
                        return Err(ErrorCode::FAIL);
                    }
                    continue 'retry; // restart whole sequence
                }
                _ => return Err(ErrorCode::FAIL), // unexpected byte
            }
        }

        // device => host response
        resp.len = 0; // reset
        for _ in 0..resp_len {
            C::wait_output_ready();
            resp.push(C::read_data());
        }
        return Ok(resp);
    }
}

/// Testing the cmd
#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;
    use kernel::errorcode::ErrorCode;
    use kernel::hil::ps2_traits::PS2Traits;

    /// Dummy controller that ACKs every byte except the first time, where it RESENDs once.
    struct StubCtl {
        step: Cell<u8>,
    }
    impl StubCtl {
        const fn new() -> Self { Self { step: Cell::new(0) } }
    }
    impl PS2Traits for StubCtl {
        fn wait_input_ready() {}
        fn wait_output_ready() {}
        fn write_data(_b: u8) {}
        fn write_command(_: u8) {}
        fn read_data() -> u8 {
            // first call => 0xFE (RESEND), then 0xFA (ACK)
            static mut COUNT: u8 = 0;
            unsafe {
                COUNT += 1;
                if COUNT == 1 { 0xFE } else { 0xFA }
            }
        }
        fn init(&self) {}
        fn handle_interrupt(&self) -> Result<(), ErrorCode> { Ok(()) }
        fn pop_scan_code(&self) -> Option<u8> { None }
        fn push_code(&self, _: u8) -> Result<(), ErrorCode> { Ok(()) }
    }

    #[test]
    fn resend_retry_succeeds() {
        let ctl = StubCtl::new();
        // Send “Set LEDs” (0xED) with dummy mask, expect no data bytes
        let r = super::send(&ctl, &[0xED, 0x00], 0).unwrap();
        assert_eq!(r.len(), 0);
    }
}

