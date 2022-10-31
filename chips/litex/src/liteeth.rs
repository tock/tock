//! LiteX LiteEth peripheral
//!
//! The hardware source and any documentation can be found in the
//! [LiteEth Git
//! repository](https://github.com/enjoy-digital/liteeth).

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{LiteXSoCRegisterConfiguration, Read, Write};
use core::cell::Cell;
use core::slice;
use kernel::debug;
use kernel::hil::ethernet::{EthernetAdapter, EthernetAdapterClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

// Both events have the same index since they are located on different
// event manager instances
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
pub struct LiteEthPhyRegisters<R: LiteXSoCRegisterConfiguration> {
    /// ETHPHY_CRG_RESET
    reset: R::WriteOnly8,
    /// ETHPHY_MDIO_W
    mdio_w: R::ReadWrite8, //<EthPhyMDIOW>,
    /// ETHPHY_MDIO_R
    mdio_r: R::ReadOnly8, //<EthPhyMDIOR>,
}

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
    fn rx_ev<'a>(&'a self) -> LiteEthRXEV<'a, R> {
        LiteEthRXEV::<R>::new(&self.rx_ev_status, &self.rx_ev_pending, &self.rx_ev_enable)
    }

    fn tx_ev<'a>(&'a self) -> LiteEthTXEV<'a, R> {
        LiteEthTXEV::<R>::new(&self.tx_ev_status, &self.tx_ev_pending, &self.tx_ev_enable)
    }
}

pub struct LiteEth<'a, R: LiteXSoCRegisterConfiguration> {
    mac_regs: StaticRef<LiteEthMacRegisters<R>>,
    mac_memory_base: usize,
    mac_memory_len: usize,
    slot_size: usize,
    rx_slots: usize,
    tx_slots: usize,
    client: OptionalCell<&'a dyn EthernetAdapterClient>,
    tx_packet: TakeCell<'static, [u8]>,
    tx_packet_info: TakeCell<'static, [(usize, u16)]>,
    initialized: Cell<bool>,
}

impl<'a, R: LiteXSoCRegisterConfiguration> LiteEth<'a, R> {
    pub unsafe fn new(
        mac_regs: StaticRef<LiteEthMacRegisters<R>>,
        mac_memory_base: usize,
        mac_memory_len: usize,
        slot_size: usize,
        rx_slots: usize,
        tx_slots: usize,
        tx_packet_info: &'static mut [(usize, u16)],
    ) -> LiteEth<'a, R> {
        LiteEth {
            mac_regs,
            mac_memory_base,
            mac_memory_len,
            slot_size,
            rx_slots,
            tx_slots,
            client: OptionalCell::empty(),
            tx_packet: TakeCell::empty(),
            tx_packet_info: TakeCell::new(tx_packet_info),
            initialized: Cell::new(false),
        }
    }

    pub fn initialize(&self) {
        // Sanity check the memory parameters
        //
        // Technically the constructor is unsafe as it will (over the
        // lifetime of this struct) "cast" the raw mac_memory pointer
        // (and slot offsets) into pointers and access them
        // directly. However checking it at runtime once seems like a
        // good idea.
        assert!(
            (self.rx_slots + self.tx_slots) * self.slot_size <= self.mac_memory_len,
            "LiteEth: slots would exceed assigned MAC memory area"
        );

        assert!(self.rx_slots > 0, "LiteEth: no RX slot");
        assert!(self.tx_slots > 0, "LiteEth: no TX slot");

        // Sanity check the length of the packet info buffer, must be
        // the same as the number of tx slots
        assert!(
            self.tx_packet_info.map(|i| i.len()).unwrap() == self.tx_slots,
            "LiteEth: tx_packet_info.len() must be equal to tx_slots"
        );

        // Disable TX events (first enabled when a packet is sent)
        self.mac_regs.tx_ev().disable_event(LITEETH_TX_EVENT);

        // Clear all pending RX & TX events (there might be leftovers
        // from the bootloader or a reboot, for which we don't want to
        // generate an event)
        //
        // This is not sufficient to guarantee that all events will be
        // cleared then. A packet could still be in reception or
        // transmit.
        while self.mac_regs.rx_ev().event_pending(LITEETH_RX_EVENT) {
            self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);
        }
        while self.mac_regs.tx_ev().event_pending(LITEETH_TX_EVENT) {
            self.mac_regs.tx_ev().clear_event(LITEETH_TX_EVENT);
        }

        // Enable RX events
        self.mac_regs.rx_ev().enable_event(LITEETH_RX_EVENT);

        self.initialized.set(true);
    }

    unsafe fn get_slot_buffer<'s>(&'s self, tx: bool, slot_id: usize) -> Option<&'s mut [u8]> {
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

        // Obtain the packet slot id
        let slot_id: usize = self.mac_regs.rx_slot.get().into();

        // // Obtain the packet reception timestamp
        // let timestamp = self.mac_regs.rx_timestamp.get();

        // Get the slot buffer reference
        let slot = unsafe {
            self.get_slot_buffer(false, slot_id)
                .expect("LiteEth: invalid RX slot id")
        };

        // Give the client read-only access to the packet data
        self.client
            .map(|client| client.rx_packet(&slot[..(pkt_len as usize)], None));

        // Since all data is copied, acknowledge the interrupt
        // so that the slot is ready for use again
        self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);

        // Just in case it was disabled
        self.mac_regs.rx_ev().enable_event(LITEETH_RX_EVENT);
    }

    fn tx_interrupt(&self) {
        // // Store information about the packet that has been sent (from
        // // the return channel)
        // let res_slot = self.mac_regs.tx_timestamp_slot.get();
        // let res_timestamp = self.mac_regs.tx_timestamp.get();

        // Acknowledge the event, removing the tx_res fields from the FIFO
        self.mac_regs.tx_ev().clear_event(LITEETH_TX_EVENT);

        if self.tx_packet.is_none() {
            debug!("LiteEth: tx interrupt called without tx_packet set");
        }

        // We use only one slot, so this event is unambiguous
        let packet = self
            .tx_packet
            .take()
            .expect("LiteEth: TakeCell empty in tx callback");

        // Retrieve the previously stored packet information for this
        // slot.
        let slot_id = 0; // currently only use one TX slot
        let (packet_identifier, len) = self
            .tx_packet_info
            .map(|pkt_info| pkt_info[slot_id])
            .unwrap();

        self.client
            .map(move |client| client.tx_done(Ok(()), packet, len, packet_identifier, None));
    }

    pub fn service_interrupt(&self) {
        // The interrupt could've been generated by both a packet
        // being received or finished transmitting. Check and handle
        // both cases

        while self.mac_regs.rx_ev().event_asserted(LITEETH_RX_EVENT) {
            self.rx_interrupt();
        }

        while self.mac_regs.tx_ev().event_asserted(LITEETH_TX_EVENT) {
            self.tx_interrupt();
        }
    }
}

impl<'a, R: LiteXSoCRegisterConfiguration> EthernetAdapter<'a> for LiteEth<'a, R> {
    fn set_client(&self, client: &'a dyn EthernetAdapterClient) {
        self.client.set(client);
    }

    /// Transmit an ethernet packet over the interface
    ///
    /// For now this will only use a single slot on the interface and
    /// is therefore blocking. A client must wait until a callback to
    /// `tx_done` prior to sending a new packet.
    fn transmit(
        &self,
        packet: &'static mut [u8],
        len: u16,
        packet_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if packet.len() < (len as usize) {
            return Err((ErrorCode::INVAL, packet));
        }

        if self.tx_packet.is_some() {
            return Err((ErrorCode::BUSY, packet));
        }

        // For now, we always use slot 0
        let slot_id = 0;

        let slot = unsafe { self.get_slot_buffer(true, slot_id) }.expect("LiteEth: no TX slot");
        if slot.len() < (len as usize) {
            return Err((ErrorCode::SIZE, packet));
        }

        // Set the slot's packet information
        self.tx_packet_info
            .map(|pkt_info| {
                pkt_info[slot_id as usize] = (packet_identifier, len);
            })
            .unwrap();

        // Copy the packet into the slot HW buffer
        slot[..(len as usize)].copy_from_slice(&packet[..(len as usize)]);

        // Put the currently transmitting packet into the designated
        // TakeCell
        self.tx_packet.replace(packet);

        // Set the slot and packet length
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
