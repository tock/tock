// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! LiteX LiteEth peripheral
//!
//! The hardware source and any documentation can be found in the [LiteEth Git
//! repository](https://github.com/enjoy-digital/liteeth).

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{LiteXSoCRegisterConfiguration, Read, Write};
use core::cell::Cell;
use core::slice;
use kernel::debug;
use kernel::hil::ethernet::{EthernetAdapterDatapath, EthernetAdapterDatapathClient};
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// Both events have the same index since they are located on different event
// manager instances
const LITEETH_TX_EVENT: usize = 0;
const LITEETH_RX_EVENT: usize = 0;

type LiteEthRXEV<'a, R> = LiteXEventManager<
    'a,
    u8,
    <R as LiteXSoCRegisterConfiguration>::ReadOnly8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
    <R as LiteXSoCRegisterConfiguration>::ReadWrite8,
>;
type LiteEthTXEV<'a, R> = LiteEthRXEV<'a, R>;

#[repr(C)]
pub struct LiteEthMacRegisters<R: LiteXSoCRegisterConfiguration> {
    /// ETHMAC_SRAM_WRITER_SLOT
    rx_slot: R::ReadOnly8,
    /// ETHMAC_SRAM_WRITER_LENGTH
    rx_length: R::ReadOnly32,
    /// ETHMAC_SRAM_WRITER_ERRORS
    rx_errors: R::ReadOnly32,
    /// ETHMAC_SRAM_WRITER_EV
    rx_ev_status: R::ReadOnly8,
    rx_ev_pending: R::ReadWrite8,
    rx_ev_enable: R::ReadWrite8,

    /// ETHMAC_SRAM_READER_START
    tx_start: R::ReadWrite8,
    /// ETHMAC_SRAM_READER_READY
    tx_ready: R::ReadOnly8,
    /// ETHMAC_SRAM_READER_LEVEL
    tx_level: R::ReadOnly8,
    /// ETHMAC_SRAM_READER_SLOT
    tx_slot: R::ReadWrite8,
    /// ETHMAC_SRAM_READER_LENGTH
    tx_length: R::ReadWrite16,
    /// ETHMAC_SRAM_READER_EV
    tx_ev_status: R::ReadOnly8,
    tx_ev_pending: R::ReadWrite8,
    tx_ev_enable: R::ReadWrite8,

    /// ETHMAC_PREAMBLE_CRC
    preamble_crc: R::ReadWrite8,
    /// ETHMAC_PREAMBLE_ERRORS
    preamble_errors: R::ReadOnly8,
    /// ETHMAC_CRC_ERRORS
    crc_errors: R::ReadOnly32,
}

impl<R: LiteXSoCRegisterConfiguration> LiteEthMacRegisters<R> {
    fn rx_ev(&self) -> LiteEthRXEV<'_, R> {
        LiteEthRXEV::<R>::new(&self.rx_ev_status, &self.rx_ev_pending, &self.rx_ev_enable)
    }

    fn tx_ev(&self) -> LiteEthTXEV<'_, R> {
        LiteEthTXEV::<R>::new(&self.tx_ev_status, &self.tx_ev_pending, &self.tx_ev_enable)
    }
}

pub struct LiteEth<'a, const MAX_TX_SLOTS: usize, R: LiteXSoCRegisterConfiguration> {
    mac_regs: StaticRef<LiteEthMacRegisters<R>>,
    mac_memory_base: usize,
    mac_memory_len: usize,
    slot_size: usize,
    rx_slots: usize,
    tx_slots: usize,
    client: OptionalCell<&'a dyn EthernetAdapterDatapathClient>,
    tx_frame: TakeCell<'static, [u8]>,
    tx_frame_info: MapCell<[(usize, u16); MAX_TX_SLOTS]>,
    initialized: Cell<bool>,
}

impl<const MAX_TX_SLOTS: usize, R: LiteXSoCRegisterConfiguration> LiteEth<'_, MAX_TX_SLOTS, R> {
    pub unsafe fn new(
        mac_regs: StaticRef<LiteEthMacRegisters<R>>,
        mac_memory_base: usize,
        mac_memory_len: usize,
        slot_size: usize,
        rx_slots: usize,
        tx_slots: usize,
    ) -> Self {
        LiteEth {
            mac_regs,
            mac_memory_base,
            mac_memory_len,
            slot_size,
            rx_slots,
            tx_slots,
            client: OptionalCell::empty(),
            tx_frame: TakeCell::empty(),
            tx_frame_info: MapCell::new([(0, 0); MAX_TX_SLOTS]),
            initialized: Cell::new(false),
        }
    }

    pub fn initialize(&self) {
        // Sanity check the memory parameters
        //
        // Technically the constructor is unsafe as it will (over the lifetime
        // of this struct) "cast" the raw mac_memory pointer (and slot offsets)
        // into pointers and access them directly. However checking it at
        // runtime once seems like a good idea.
        assert!(
            (self.rx_slots + self.tx_slots) * self.slot_size <= self.mac_memory_len,
            "LiteEth: slots would exceed assigned MAC memory area"
        );

        assert!(self.rx_slots > 0, "LiteEth: no RX slot");
        assert!(self.tx_slots > 0, "LiteEth: no TX slot");

        // Sanity check the length of the frame_info buffer, must be able to fit
        // all `tx_slots` requested at runtime.
        assert!(
            MAX_TX_SLOTS >= self.tx_slots,
            "LiteEth: MAX_TX_SLOTS ({}) must be larger or equal to tx_slots ({})",
            MAX_TX_SLOTS,
            self.tx_slots,
        );

        // Disable TX events (first enabled when a frame is sent)
        self.mac_regs.tx_ev().disable_event(LITEETH_TX_EVENT);

        // Clear all pending RX & TX events (there might be leftovers from the
        // bootloader or a reboot, for which we don't want to generate an event)
        //
        // This is not sufficient to guarantee that all events will be cleared
        // then. A frame could still be in reception or transmit.
        while self.mac_regs.rx_ev().event_pending(LITEETH_RX_EVENT) {
            self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);
        }
        while self.mac_regs.tx_ev().event_pending(LITEETH_TX_EVENT) {
            self.mac_regs.tx_ev().clear_event(LITEETH_TX_EVENT);
        }

        self.initialized.set(true);
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn get_slot_buffer(&self, tx: bool, slot_id: usize) -> Option<&mut [u8]> {
        if (tx && slot_id > self.tx_slots) || (!tx && slot_id > self.rx_slots) {
            return None;
        }

        let slots_offset = if tx {
            self.mac_memory_base + self.slot_size * self.rx_slots
        } else {
            self.mac_memory_base
        };

        let slot_addr = slots_offset + slot_id * self.slot_size;
        Some(slice::from_raw_parts_mut(
            slot_addr as *mut u8,
            self.slot_size,
        ))
    }

    fn rx_interrupt(&self) {
        // Get the frame length
        let pkt_len = self.mac_regs.rx_length.get();

        // Obtain the frame slot id
        let slot_id: usize = self.mac_regs.rx_slot.get().into();

        // Obtain the frame reception timestamp. The `rx_timestamp` register is
        // optional and disabled by default.
        //
        // let timestamp = self.mac_regs.rx_timestamp.get();

        // Get the slot buffer reference
        let slot = unsafe {
            self.get_slot_buffer(false, slot_id)
                .expect("LiteEth: invalid RX slot id")
        };

        // Give the client read-only access to the frame data
        self.client
            .map(|client| client.received_frame(&slot[..(pkt_len as usize)], None));

        // Since all data is copied, acknowledge the interrupt so that the slot
        // is ready for use again
        self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);
    }

    fn tx_interrupt(&self) {
        // Store information about the frame that has been sent (from the return
        // channel). Uncomment the below lines if hardware timestamping is
        // enabled and frame TX timestamps are supposed to be recorded.
        //
        // let res_slot = self.mac_regs.tx_timestamp_slot.get();
        // let res_timestamp = self.mac_regs.tx_timestamp.get();

        // Acknowledge the event, removing the tx_res fields from the FIFO
        self.mac_regs.tx_ev().clear_event(LITEETH_TX_EVENT);

        if self.tx_frame.is_none() {
            debug!("LiteEth: tx interrupt called without tx_frame set");
        }

        // We use only one slot, so this event is unambiguous
        let frame = self
            .tx_frame
            .take()
            .expect("LiteEth: TakeCell empty in tx callback");

        // Retrieve the previously stored frame information for this slot.
        let slot_id = 0; // currently only use one TX slot
        let (frame_identifier, len) = self
            .tx_frame_info
            .map(|pkt_info| pkt_info[slot_id])
            .unwrap();

        self.client.map(move |client| {
            client.transmit_frame_done(Ok(()), frame, len, frame_identifier, None)
        });
    }

    pub fn service_interrupt(&self) {
        // The interrupt could've been generated by both a frame being received
        // or finished transmitting. Check and handle both cases.

        while self.mac_regs.tx_ev().event_asserted(LITEETH_TX_EVENT) {
            self.tx_interrupt();
        }

        // `event_asserted` checks that the event is both pending _and_ enabled
        // (raising a CPU interrupt). This means that reception is enabled, and
        // we must handle it:
        while self.mac_regs.rx_ev().event_asserted(LITEETH_RX_EVENT) {
            self.rx_interrupt();
        }
    }
}

impl<'a, const MAX_TX_SLOTS: usize, R: LiteXSoCRegisterConfiguration> EthernetAdapterDatapath<'a>
    for LiteEth<'a, MAX_TX_SLOTS, R>
{
    fn set_client(&self, client: &'a dyn EthernetAdapterDatapathClient) {
        self.client.set(client);
    }

    fn enable_receive(&self) {
        // Enable RX event interrupts:
        if !self.initialized.get() {
            panic!("LiteEth: cannot enable_receive without prior initialization!");
        }

        self.mac_regs.rx_ev().enable_event(LITEETH_RX_EVENT);
    }

    fn disable_receive(&self) {
        // Disable RX event interrupts:
        self.mac_regs.rx_ev().disable_event(LITEETH_RX_EVENT);
    }

    /// Transmit an Ethernet frame over the interface
    ///
    /// For now this will only use a single slot on the interface and is
    /// therefore blocking. A client must wait until a callback to `tx_done`
    /// prior to sending a new frame.
    fn transmit_frame(
        &self,
        frame: &'static mut [u8],
        len: u16,
        frame_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if frame.len() < (len as usize) {
            return Err((ErrorCode::INVAL, frame));
        }

        if self.tx_frame.is_some() {
            return Err((ErrorCode::BUSY, frame));
        }

        // For now, we always use slot 0
        let slot_id = 0;

        let slot = unsafe { self.get_slot_buffer(true, slot_id) }.expect("LiteEth: no TX slot");
        if slot.len() < (len as usize) {
            return Err((ErrorCode::SIZE, frame));
        }

        // Set the slot's frame information
        self.tx_frame_info
            .map(|pkt_info| {
                pkt_info[slot_id] = (frame_identifier, len);
            })
            .unwrap();

        // Copy the frame into the slot HW buffer
        slot[..(len as usize)].copy_from_slice(&frame[..(len as usize)]);

        // Put the currently transmitting frame into the designated TakeCell
        self.tx_frame.replace(frame);

        // Set the slot and frame length
        self.mac_regs.tx_slot.set(0);
        self.mac_regs.tx_length.set(len);

        // Wait for the device to be ready to transmit
        while self.mac_regs.tx_ready.get() == 0 {}

        // Enable TX events
        self.mac_regs.tx_ev().enable_event(LITEETH_TX_EVENT);

        // Start the transmission
        self.mac_regs.tx_start.set(1);

        Ok(())
    }
}
