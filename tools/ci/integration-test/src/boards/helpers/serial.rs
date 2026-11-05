// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use super::console_buffer::ConsoleBuffer;
use crate::boards::Uart;
use anyhow::{Context, Result};
use std::io::Read;
use std::time::{Duration, Instant};

/// [`Uart`] backed by a physical board's serial port (exposed as a character
/// device or COM port).
pub struct SerialUart {
    port: Box<dyn serialport::SerialPort>,
    output: ConsoleBuffer,
}

impl SerialUart {
    /// Open the serial port `device` with the baud rate `baud`, and discard any
    /// input it has received before.
    pub fn open(device: &str, baud: u32) -> Result<Self> {
        let port = serialport::new(device, baud)
            .timeout(Duration::from_millis(200))
            .open()
            .with_context(|| format!("failed to open {device}"))?;

        port.clear(serialport::ClearBuffer::Input)
            .with_context(|| format!("failed to clear the input of {device}"))?;

        Ok(SerialUart {
            port,
            output: ConsoleBuffer::new(),
        })
    }
}

impl Uart for SerialUart {
    fn receive_until(&mut self, needle: &str, timeout: Duration) -> Result<String> {
        let deadline = Instant::now() + timeout;
        let mut chunk = [0u8; 256];

        while Instant::now() < deadline {
            match self.port.read(&mut chunk) {
                Ok(n) => self.output.push(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(e.into()),
            }

            if let Some(out) = self.output.take_through(needle) {
                return Ok(out);
            }
        }

        Err(self.output.timeout_error(needle))
    }

    fn transcript(&mut self) -> &[u8] {
        self.output.transcript()
    }
}
