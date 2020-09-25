//! LiteX LiteEth peripheral

use crate::event_manager::LiteXEventManager;
use crate::litex_registers::{LiteXSoCRegisterConfiguration, Read, Write};
use core::cell::Cell;
use core::slice;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::ReturnCode;

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
    tx_ready: R::ReadOnly8,
    tx_level: R::ReadOnly8,
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

pub trait LiteEthClient {
    fn tx_done(&self, rc: ReturnCode, packet_buffer: &'static mut [u8]);
    fn rx_packet(&self, packet: &'static mut [u8], len: usize);
}

pub struct LiteEth<'a, R: LiteXSoCRegisterConfiguration> {
    mac_regs: StaticRef<LiteEthMacRegisters<R>>,
    mac_memory_base: usize,
    mac_memory_len: usize,
    slot_size: usize,
    rx_slots: usize,
    tx_slots: usize,
    client: OptionalCell<&'a dyn LiteEthClient>,
    tx_packet: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
    initialized: Cell<bool>,
}

impl<'a, R: LiteXSoCRegisterConfiguration> LiteEth<'a, R> {
    pub const unsafe fn new(
        mac_regs: StaticRef<LiteEthMacRegisters<R>>,
        mac_memory_base: usize,
        mac_memory_len: usize,
        slot_size: usize,
        rx_slots: usize,
        tx_slots: usize,
        rx_buffer: &'static mut [u8],
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
            rx_buffer: TakeCell::new(rx_buffer),
            initialized: Cell::new(false),
        }
    }

    pub fn set_client(&self, client: &'a dyn LiteEthClient) {
        self.client.set(client);
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

        // Clear any pending EV events
        self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);
        self.mac_regs.tx_ev().clear_event(LITEETH_TX_EVENT);

        // Disable TX events (only enabled when a packet is sent)
        self.mac_regs.tx_ev().disable_event(LITEETH_TX_EVENT);

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

    pub fn return_rx_buffer(&self, rx_buffer: &'static mut [u8]) {
        // Assert that we won't overwrite a buffer
        assert!(
            self.rx_buffer.is_none(),
            "LiteEth: return RX buffer while one is registered"
        );

        // Put the buffer back
        self.rx_buffer.replace(rx_buffer);

        // In case we received a packet RX interrupt but couldn't
        // handle it due to the missing buffer, reenable RX interrupts
        self.mac_regs.rx_ev().enable_event(LITEETH_RX_EVENT);
    }

    fn rx_interrupt(&self) {
        // Check whether we have a buffer to read the packet into. If
        // not, we must disable, but not clear the event and enable it
        // again as soon as we get the buffer back from the client
        if self.rx_buffer.is_none() {
            self.mac_regs.rx_ev().disable_event(LITEETH_RX_EVENT);
        } else {
            // Get the buffer first to be able to check the length
            let rx_buffer = self.rx_buffer.take().unwrap();

            // Get the frame length. If it exceeds the length of the
            // rx_buffer, discard the packet, put the buffer back
            let pkt_len = self.mac_regs.rx_length.get() as usize;
            if pkt_len > rx_buffer.len() {
                debug!("LiteEth: discarding ethernet packet with len {}", pkt_len);

                // Acknowledge the interrupt so that the HW may use the slot again
                self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);

                // Replace the buffer
                self.rx_buffer.replace(rx_buffer);
            } else {
                // Obtain the packet slot id
                let slot_id: usize = self.mac_regs.rx_slot.get().into();

                // Get the slot buffer reference
                let slot = unsafe {
                    self.get_slot_buffer(false, slot_id)
                        .expect("LiteEth: invalid RX slot id")
                };

                // Copy the packet into the buffer
                rx_buffer[..pkt_len].copy_from_slice(&slot[..pkt_len]);

                // Since all data is copied, acknowledge the interrupt
                // so that the slot is ready for use again
                self.mac_regs.rx_ev().clear_event(LITEETH_RX_EVENT);

                self.client
                    .map(move |client| client.rx_packet(rx_buffer, pkt_len));
            }
        }
    }

    /// Transmit an ethernet packet over the interface
    ///
    /// For now this will only use a single slot on the interface and
    /// is therefore blocking. A client must wait until a callback to
    /// `tx_done` prior to sending a new packet.
    pub fn transmit(
        &self,
        packet: &'static mut [u8],
        len: usize,
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        if packet.len() < len || len > u16::MAX as usize {
            return Err((ReturnCode::EINVAL, packet));
        }

        if self.tx_packet.is_some() {
            return Err((ReturnCode::EBUSY, packet));
        }

        let slot = unsafe { self.get_slot_buffer(true, 0) }.expect("LiteEth: no TX slot");
        if slot.len() < len {
            return Err((ReturnCode::ESIZE, packet));
        }

        // Copy the packet into the slot HW buffer
        slot[..len].copy_from_slice(&packet[..len]);

        // Put the currently transmitting packet into the designated
        // TakeCell
        self.tx_packet.replace(packet);

        // Set the slot and packet length
        self.mac_regs.tx_slot.set(0);
        self.mac_regs.tx_length.set(len as u16);

        // Wait for the device to be ready to transmit
        while self.mac_regs.tx_ready.get() == 0 {}

        // Enable TX interrupts
        self.mac_regs.tx_ev().enable_event(LITEETH_TX_EVENT);

        // Start the transmission
        self.mac_regs.tx_start.set(1);

        Ok(())
    }

    fn tx_interrupt(&self) {
        // Deassert the interrupt, but can be left enabled
        self.mac_regs.tx_ev().clear_event(LITEETH_TX_EVENT);

        if self.tx_packet.is_none() {
            debug!("LiteEth: tx interrupt called without tx_packet set");
        }

        // We use only one slot, so this event is unambiguous
        let packet = self
            .tx_packet
            .take()
            .expect("LiteEth: TakeCell empty in tx callback");
        self.client
            .map(move |client| client.tx_done(ReturnCode::SUCCESS, packet));
    }

    pub fn service_interrupt(&self) {
        // The interrupt could've been generated by both a packet
        // being received or finished transmitting. Check and handle
        // both cases

        if self.mac_regs.rx_ev().event_enabled(LITEETH_RX_EVENT)
            && self.mac_regs.rx_ev().event_pending(LITEETH_RX_EVENT)
        {
            self.rx_interrupt();
        }

        if self.mac_regs.tx_ev().event_enabled(LITEETH_TX_EVENT)
            && self.mac_regs.tx_ev().event_pending(LITEETH_TX_EVENT)
        {
            self.tx_interrupt();
        }
    }
}
