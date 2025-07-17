use core::cell::RefCell;
use core::marker::PhantomData;
use kernel::debug;
use kernel::errorcode::ErrorCode;
use kernel::hil::ps2_kb::KBReceiver;
use kernel::hil::ps2_traits::PS2Traits;

/// Internal decoder state for PS/2 Set 2 scan codes
pub struct DecoderState {
    // TODO: add prefix flags (extended, break)
    // TODO: add modifier state (shift, ctrl, alt, caps)
}

impl DecoderState {
    /// Create a new decoder state
    pub fn new() -> Self {
        DecoderState {
            // initialize flags and state
        }
    }

    /// Process one raw scan code byte
    /// Returns Some(u8) when a complete ASCII/key byte is ready
    pub fn process(&mut self, raw: u8) -> Option<u8> {
        // TODO: handle E0/F0 prefixes
        // TODO: lookup in scan-code table + modifiers
        None
    }
}

/// PS/2 Keyboard wrapper using any `PS2Traits` implementer
pub struct Keyboard<'a, C: PS2Traits> {
    ps2: &'a C,
    decoder: RefCell<DecoderState>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, C: PS2Traits> Keyboard<'a, C> {
    ///Keyboard wrapping 4 PS2 controller
    pub fn new(ps2: &'a C) -> Self {
        Keyboard {
            ps2,
            decoder: RefCell::new(DecoderState::new()),
            _marker: PhantomData,
        }
    }

    /// Set keyboard LEDs: bit0=ScrollLock, bit1=NumLock, bit2=CapsLock
    pub fn set_leds(&self, mask: u8) -> Result<(), ErrorCode> {
        // 1️⃣ Send "Set LEDs" command (0xED)
        C::write_data(0xED);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            // Resend request
            C::write_data(0xED);
            C::wait_output_ready();
            let _ = C::read_data();
        } else if resp != 0xFA {
            debug!("Unexpected LED command ACK: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        C::write_data(mask & 0x07);
        C::wait_output_ready();
        let resp2 = C::read_data();
        if resp2 == 0xFE {
            C::write_data(mask & 0x07);
            C::wait_output_ready();
            let _ = C::read_data();
        } else if resp2 != 0xFA {
            debug!("Unexpected LED mask ACK: {:02x}", resp2);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }
    /// Keyboard detection system!!!!
    pub fn probe_echo(&self) -> Result<(), ErrorCode> {
        C::write_data(0xEE);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            // Keyboard asked to resend
            C::write_data(0xEE);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xEE {
                debug!("Echo resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xEE {
            debug!("Echo failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }

    /// Check for keyboard presence via echo. Returns true if present.
    pub fn is_present(&self) -> bool {
        self.probe_echo().is_ok()
    }

    /// Identify the keyboard: send 0xF2 and collect up to 3 ID bytes
    /// Returns (buffer, count) on success, or Err on failure
    pub fn identify(&self) -> Result<([u8; 3], usize), ErrorCode> {
        // 1) Send Identify command
        C::write_data(0xF2);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp != 0xFA {
            debug!("Identify command not ACKed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        // 2) Read ID bytes; typical keyboards send 0–2 bytes
        let mut ids = [0u8; 3];
        let mut count = 0;
        for _ in 0..3 {
            C::wait_output_ready();
            let b = C::read_data();
            ///MUST REVIEW!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
            if b == 0xFA {
                continue;
            }
            ids[count] = b;
            count += 1;
        }
        Ok((ids, count))
    }
    ///SCAN MODE!!!!!!!!!!!
    ///
    ///
    ///
    ///
    ///
    pub fn scan_code_set(&self, cmd: u8) -> Result<u8, ErrorCode> {
        // 1️⃣ Send 0xF0 and sub-command
        C::write_data(0xF0);
        C::wait_output_ready();
        let ack = C::read_data();
        if ack == 0xFE {
            // Resend F0
            C::write_data(0xF0);
            C::wait_output_ready();
            let _ = C::read_data();
        } else if ack != 0xFA {
            debug!("Scan-set cmd not ACKed: {:02x}", ack);
            return Err(ErrorCode::FAIL);
        }
        C::write_data(cmd);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            // Resend subcmd
            C::write_data(cmd);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Scan-set subcmd not ACKed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
            // Next byte is the current set
            C::wait_output_ready();
            let set = C::read_data();
            return Ok(set);
        } else if resp == 0xFA {
            // On GET (cmd=0) the controller then returns the set number
            C::wait_output_ready();
            let set = C::read_data();
            return Ok(set);
        } else if cmd != 0 && resp == cmd {
            // On SET, some keyboards echo back the new set directly
            return Ok(resp);
        }
        // Unexpected response
        Err(ErrorCode::FAIL)
    }

    pub fn set_typematic(&self, rate_delay: u8) -> Result<(), ErrorCode> {
        let cmd = rate_delay & 0x7F;
        C::write_data(0xF3);
        C::wait_output_ready();
        let ack = C::read_data();
        if ack == 0xFE {
            C::write_data(0xF3);
            C::wait_output_ready();
            let _ = C::read_data();
        } else if ack != 0xFA {
            debug!("Typematic cmd not ACKed: {:02x}", ack);
            return Err(ErrorCode::FAIL);
        }
        C::write_data(cmd);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            C::write_data(cmd);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Typematic data not ACKed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Typematic data ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }

    /// Enable scanning (0xF4): keyboard will send scan codes on key events
    /// Returns Ok(()) on success
    pub fn enable_scanning(&self) -> Result<(), ErrorCode> {
        C::write_data(0xF4);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            C::write_data(0xF4);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Enable scanning resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Enable scanning ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }

    /// Disable scanning (0xF5): keyboard will stop sending scan codes
    /// Returns Ok(()) on success
    pub fn disable_scanning(&self) -> Result<(), ErrorCode> {
        C::write_data(0xF5);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            C::write_data(0xF5);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Disable scanning resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Disable scanning ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }
    /// Set default keyboard parameters (0xF6), restoring defaults
    /// Returns Ok(()) on success
    pub fn set_defaults(&self) -> Result<(), ErrorCode> {
        C::write_data(0xF6);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            C::write_data(0xF6);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Set defaults resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Set defaults ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }
    /// Set all keys to typematic/autorepeat only (0xF7) — scancode set 3 only
    /// Returns Ok(()) on success
    pub fn set_typematic_only(&self) -> Result<(), ErrorCode> {
        C::write_data(0xF7);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            C::write_data(0xF7);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Typematic-only resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Typematic-only ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }
        Ok(())
    }

    /// Set all keys to make/release (0xF8) — scancode set 3 only
    /// Returns Ok(()) on success, or Err(FAIL) if the ACK/Resend handshake fails.
    pub fn set_make_release(&self) -> Result<(), ErrorCode> {
        // Send the 0xF8 command
        C::write_data(0xF8);
        C::wait_output_ready();
        let resp = C::read_data();

        // Handle Resend (0xFE) or unexpected replies
        if resp == 0xFE {
            // Keyboard wants us to resend
            C::write_data(0xF8);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Make/release resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            // Anything other than ACK is a failure
            debug!("Make/release ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }

    pub fn set_make_only(&self) -> Result<(), ErrorCode> {
        // Send the 0xF9 command
        C::write_data(0xF9);
        C::wait_output_ready();
        let resp = C::read_data();

        if resp == 0xFE {
            // Keyboard wants us to resend
            C::write_data(0xF9);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Make-only resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            // Anything other than ACK is a failure
            debug!("Make-only ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }
    pub fn set_full_mode(&self) -> Result<(), ErrorCode> {
        // Send the 0xFA command
        C::write_data(0xFA);
        C::wait_output_ready();
        let resp = C::read_data();

        // Handle possible Resend (0xFE) or unexpected replies
        if resp == 0xFE {
            // Keyboard asked to resend
            C::write_data(0xFA);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Full‐mode resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            // Anything other than ACK is a failure
            debug!("Full‐mode ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }
    /// Set a specific key to typematic/autorepeat only (0xFB) — scancode set 3 only.
    /// `scancode` is the make-code of the key to configure.
    /// Returns Ok(()) on success, or Err(FAIL) if the ACK/Resend handshake fails.
    pub fn set_key_typematic_only(&self, scancode: u8) -> Result<(), ErrorCode> {
        // Send the 0xFB command
        C::write_data(0xFB);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            // Keyboard asked us to resend the command
            C::write_data(0xFB);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Typematic‑only (key) resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Typematic‑only (key) ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        // Send the key’s scancode
        C::write_data(scancode);
        C::wait_output_ready();
        let resp3 = C::read_data();
        if resp3 == 0xFE {
            // Resend the scancode
            C::write_data(scancode);
            C::wait_output_ready();
            let resp4 = C::read_data();
            if resp4 != 0xFA {
                debug!("Typematic‑only (key data) resend failed: {:02x}", resp4);
                return Err(ErrorCode::FAIL);
            }
        } else if resp3 != 0xFA {
            debug!("Typematic‑only (key data) ACK failed: {:02x}", resp3);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }

    /// Set a specific key to make/release only (0xFC) — scancode set 3 only.
    /// `scancode` is the make-code of the key to configure.
    /// Returns Ok(()) on success, or Err(FAIL) if the ACK/Resend handshake fails.
    pub fn set_key_make_release(&self, scancode: u8) -> Result<(), ErrorCode> {
        // Send the 0xFC command
        C::write_data(0xFC);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            // Keyboard asked us to resend the command
            C::write_data(0xFC);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Make/release (key) resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Make/release (key) ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        // Send the key’s scancode
        C::write_data(scancode);
        C::wait_output_ready();
        let resp3 = C::read_data();
        if resp3 == 0xFE {
            // Resend the scancode
            C::write_data(scancode);
            C::wait_output_ready();
            let resp4 = C::read_data();
            if resp4 != 0xFA {
                debug!("Make/release (key data) resend failed: {:02x}", resp4);
                return Err(ErrorCode::FAIL);
            }
        } else if resp3 != 0xFA {
            debug!("Make/release (key data) ACK failed: {:02x}", resp3);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }

    /// Set a specific key to make-only (0xFD) — scancode set 3 only.
    /// `scancode` is the make-code of the key to configure.
    /// Returns Ok(()) on success, or Err(FAIL) if the ACK/Resend handshake fails.
    pub fn set_key_make_only(&self, scancode: u8) -> Result<(), ErrorCode> {
        // Send the 0xFD command
        C::write_data(0xFD);
        C::wait_output_ready();
        let resp = C::read_data();
        if resp == 0xFE {
            // Keyboard asked us to resend the command
            C::write_data(0xFD);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Make-only (key) resend failed: {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Make-only (key) ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        // Send the key’s scancode
        C::write_data(scancode);
        C::wait_output_ready();
        let resp3 = C::read_data();
        if resp3 == 0xFE {
            // Resend the scancode
            C::write_data(scancode);
            C::wait_output_ready();
            let resp4 = C::read_data();
            if resp4 != 0xFA {
                debug!("Make-only (key data) resend failed: {:02x}", resp4);
                return Err(ErrorCode::FAIL);
            }
        } else if resp3 != 0xFA {
            debug!("Make-only (key data) ACK failed: {:02x}", resp3);
            return Err(ErrorCode::FAIL);
        }

        Ok(())
    }

    /// Request the keyboard to resend its last-produced byte (0xFE).
    /// Returns the resent byte on success, or Err(FAIL) if the ACK/Resend handshake itself fails.
    pub fn resend_last_byte(&self) -> Result<u8, ErrorCode> {
        // Send the Resend command
        C::write_data(0xFE);
        C::wait_output_ready();
        let resp = C::read_data();

        // If the keyboard itself asks for a resend, retry
        let byte = if resp == 0xFE {
            // Keyboard wants us to resend the 0xFE command
            C::write_data(0xFE);
            C::wait_output_ready();
            C::read_data()
        } else {
            // Otherwise, resp *is* the resent data byte
            resp
        };

        Ok(byte)
    }
    /// Reset keyboard and run self‑test (0xFF).
    /// Expects: 0xFA (ACK), then 0xAA (pass) or 0xFC/0xFD (fail).
    /// Returns Ok(()) only if self‑test passes.
    pub fn reset_and_self_test(&self) -> Result<(), ErrorCode> {
        // Send Reset command
        C::write_data(0xFF);
        C::wait_output_ready();
        let resp = C::read_data();

        // Handle Resend or missing ACK
        if resp == 0xFE {
            // Keyboard asked us to resend
            C::write_data(0xFF);
            C::wait_output_ready();
            let resp2 = C::read_data();
            if resp2 != 0xFA {
                debug!("Reset resend failed (no ACK): {:02x}", resp2);
                return Err(ErrorCode::FAIL);
            }
        } else if resp != 0xFA {
            debug!("Reset ACK failed: {:02x}", resp);
            return Err(ErrorCode::FAIL);
        }

        // Wait for self‑test result
        C::wait_output_ready();
        let test = C::read_data();
        match test {
            0xAA => {
                // Self‑test passed
                Ok(())
            }
            0xFC | 0xFD => {
                debug!("Keyboard self‑test failed: {:02x}", test);
                Err(ErrorCode::FAIL)
            }
            other => {
                debug!("Unexpected self‑test response: {:02x}", other);
                Err(ErrorCode::FAIL)
            }
        }
    }
}

/// Implement the KBReceiver trait so Console can poll for keystrokes
impl<'a, C: PS2Traits> KBReceiver for Keyboard<'a, C> {
    /// Fetch one decoded ASCII/key byte, or `None` if no data
    fn receive(&self) -> Option<u8> {
        // Pop raw scan code from the PS/2 controller's buffer
        if let Some(raw) = self.ps2.pop_scan_code() {
            // Decode into ASCII/key
            self.decoder.borrow_mut().process(raw)
        } else {
            None
        }
    }
}
