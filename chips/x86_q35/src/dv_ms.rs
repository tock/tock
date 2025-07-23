// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Your Name 2024.

//! PS/2 mouse wrapper for the 8042 controller

use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use kernel::errorcode::ErrorCode;
use kernel::hil::ps2_traits::{MouseEvent, PS2Mouse, PS2Traits};

const RAW_BUF_SIZE: usize = 32; // depth of raw‐byte ring
const PACKET_BUF_SIZE: usize = 16; // depth of 3‐byte packet ring

/// Raw‐byte FIFO
struct RawFifo {
    buf: [u8; RAW_BUF_SIZE],
    head: usize,
    tail: usize,
    full: bool,
}

impl RawFifo {
    const fn new() -> Self {
        Self {
            buf: [0; RAW_BUF_SIZE],
            head: 0,
            tail: 0,
            full: false,
        }
    }
    fn push(&mut self, b: u8) {
        self.buf[self.head] = b;
        self.head = (self.head + 1) % RAW_BUF_SIZE;
        if self.full {
            self.tail = (self.tail + 1) % RAW_BUF_SIZE;
        } else if self.head == self.tail {
            self.full = true;
        }
    }
    fn pop(&mut self) -> Option<u8> {
        if !self.full && self.head == self.tail {
            None
        } else {
            let b = self.buf[self.tail];
            self.tail = (self.tail + 1) % RAW_BUF_SIZE;
            self.full = false;
            Some(b)
        }
    }
}

/// 3‑byte packet FIFO
struct PacketFifo {
    buf: [[u8; 3]; PACKET_BUF_SIZE],
    head: usize,
    tail: usize,
    full: bool,
}

impl PacketFifo {
    const fn new() -> Self {
        Self {
            buf: [[0; 3]; PACKET_BUF_SIZE],
            head: 0,
            tail: 0,
            full: false,
        }
    }
    fn push(&mut self, pkt: [u8; 3]) {
        self.buf[self.head] = pkt;
        self.head = (self.head + 1) % PACKET_BUF_SIZE;
        if self.full {
            self.tail = (self.tail + 1) % PACKET_BUF_SIZE;
        } else if self.head == self.tail {
            self.full = true;
        }
    }
    fn pop(&mut self) -> Option<[u8; 3]> {
        if !self.full && self.head == self.tail {
            None
        } else {
            let pkt = self.buf[self.tail];
            self.tail = (self.tail + 1) % PACKET_BUF_SIZE;
            self.full = false;
            Some(pkt)
        }
    }
}

/// Main PS/2 mouse driver
pub struct Mouse<'a, C: PS2Traits> {
    controller: &'a C,
    raw: RefCell<RawFifo>,
    packet_fifo: RefCell<PacketFifo>,
    state: Cell<usize>,
    pkt: Cell<[u8; 3]>,
    _marker: PhantomData<&'a ()>,
}

impl<'a, C: PS2Traits> Mouse<'a, C> {
    pub fn new(controller: &'a C) -> Self {
        Self {
            controller,
            raw: RefCell::new(RawFifo::new()),
            packet_fifo: RefCell::new(PacketFifo::new()),
            state: Cell::new(0),
            pkt: Cell::new([0; 3]),
            _marker: PhantomData,
        }
    }

    /// Top‑half: call from IRQ stub. Reads one byte & assembles 3‑byte packets.
    pub fn handle_interrupt(&self) {
        let _ = self.controller.handle_interrupt();
        if let Some(b) = self.controller.pop_scan_code() {
            let mut st = self.state.get();
            let mut buf_pkt = self.pkt.get();
            buf_pkt[st] = b;
            st += 1;
            if st == 3 {
                // full packet ready
                self.packet_fifo.borrow_mut().push(buf_pkt);
                st = 0;
            }
            self.pkt.set(buf_pkt);
            self.state.set(st);
        }
    }

    /// Bottom‑half: non‑blocking decode of next packet → MouseEvent
    pub fn poll(&self) -> Option<MouseEvent> {
        if let Some(pkt) = self.packet_fifo.borrow_mut().pop() {
            // TODO: translate raw pkt bytes into MouseEvent fields
            let event = MouseEvent {
                buttons: pkt[0] & 0x07,
                x_movement: ((pkt[0] as i8 as i16) << 8 | pkt[1] as i16) as i8,
                y_movement: ((pkt[0] as i8 as i16) << 8 | pkt[2] as i16) as i8,
            };
            Some(event)
        } else {
            None
        }
    }
}
impl<'a, C: PS2Traits> PS2Mouse for Mouse<'a, C> {
    /// 1:1 scaling
    fn set_scaling_1_1(&self) -> Result<(), ErrorCode> {
        crate::ps2_cmd::send::<C>(self.controller, &[0xE6], 0).map(|_| ())
    }

    /// 2:1 scaling
    fn set_scaling_2_1(&self) -> Result<(), ErrorCode> {
        crate::ps2_cmd::send::<C>(self.controller, &[0xE7], 0).map(|_| ())
    }

    /// Resolution (0–3)
    fn set_resolution(&self, res: u8) -> Result<(), ErrorCode> {
        crate::ps2_cmd::send::<C>(self.controller, &[0xE8, res], 0).map(|_| ())
    }

    /// Status request → 3‑byte response
    fn status_request(&self) -> Result<[u8; 3], ErrorCode> {
        let resp = crate::ps2_cmd::send::<C>(self.controller, &[0xE9], 3)?;
        let mut out = [0u8; 3];
        out.copy_from_slice(resp.as_slice());
        Ok(out)
    }

    /// Read one “packet” (3 raw bytes assembled in your FIFO → decode elsewhere)
    fn read_data(&self) -> Result<MouseEvent, ErrorCode> {
        self.poll().ok_or(ErrorCode::NOMEM)
    }

    /// Remote‐mode sample‐rate setter (0xF3)
    fn set_sample_rate(&self, rate: u8) -> Result<(), ErrorCode> {
        crate::ps2_cmd::send::<C>(self.controller, &[0xF3, rate], 0).map(|_| ())
    }

    /// Put the device into “streaming” (push on movement)
    fn enable_streaming(&self) -> Result<(), ErrorCode> {
        crate::ps2_cmd::send::<C>(self.controller, &[0xF4], 0).map(|_| ())
    }

    /// Halt streaming updates
    fn disable_streaming(&self) -> Result<(), ErrorCode> {
        crate::ps2_cmd::send::<C>(self.controller, &[0xF5], 0).map(|_| ())
    }

    /// Reset device and verify self‐test
    fn reset(&self) -> Result<(), ErrorCode> {
        let resp = crate::ps2_cmd::send::<C>(self.controller, &[0xFF], 2)?;
        // resp.as_slice() == &[0xFA, 0xAA]
        if resp.as_slice() == &[0xAA] {
            Ok(())
        } else {
            Err(ErrorCode::FAIL)
        }
    }
}
