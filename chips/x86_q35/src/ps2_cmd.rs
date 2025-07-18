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
