//! Driver for sending 802.15.4 packets with an Atmel RF233.
//!
//! This implementation is completely non-blocking. This means that the state
//! machine is somewhat complex, as it must interleave interrupt handling with
//! requests and radio state management. See the SPI `read_write_done` handler
//! for details.
//!
//! To do items:
//!
//! - Support TX power control
//! - Support channel selection
//! - Support link-layer acknowledgements
//
// Author: Philip Levis
// Date: Jan 12 2017
//

#![allow(unused_parens)]

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::gpio;
use kernel::hil::radio;
use kernel::hil::spi;
use kernel::ReturnCode;
use rf233_const::{ExternalState, InteruptFlags, RF233BusCommand, RF233Register, RF233TrxCmd};
// n.b. This is a fairly "C"-like interface presently. Ideally it should move
// over to the Tock register interface eventually, but this code does work as
// written. Do not follow this as an example when implementing new code.
use rf233_const::CSMA_SEED_1;
use rf233_const::IRQ_MASK;
use rf233_const::PHY_CC_CCA_MODE_CS_OR_ED;
use rf233_const::PHY_CHANNEL;
use rf233_const::PHY_RSSI_RX_CRC_VALID;
use rf233_const::PHY_TX_PWR;
use rf233_const::SHORT_ADDR_0;
use rf233_const::SHORT_ADDR_1;
use rf233_const::TRX_CTRL_1;
use rf233_const::TRX_CTRL_2;
use rf233_const::TRX_RPC;
use rf233_const::TRX_TRAC_CHANNEL_ACCESS_FAILURE;
use rf233_const::TRX_TRAC_MASK;
use rf233_const::XAH_CTRL_0;
use rf233_const::XAH_CTRL_1;

const INTERRUPT_ID: usize = 0x2154;

#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Clone, PartialEq)]
enum InternalState {
    // There are 6 high-level states:
    // START -- the initialization sequence
    // ON    -- turning the radio on to receive
    // READY -- waiting to receive packets
    // RX    -- receiving a packet
    // TX    -- transmitting a packet
    // CONFIG -- reconfiguring the radio
    START,
    START_PART_READ,
    START_STATUS_READ,
    START_TURNING_OFF,
    START_CTRL1_SET,
    START_CCA_SET,
    START_PWR_SET,
    START_CTRL2_SET,
    START_IRQMASK_SET,
    START_XAH1_SET,
    START_XAH0_SET,
    START_PANID0_SET,
    START_PANID1_SET,
    START_IEEE0_SET,
    START_IEEE1_SET,
    START_IEEE2_SET,
    START_IEEE3_SET,
    START_IEEE4_SET,
    START_IEEE5_SET,
    START_IEEE6_SET,
    START_IEEE7_SET,
    START_SHORT0_SET,
    START_SHORT1_SET,
    START_CSMA_0_SEEDED,
    START_CSMA_1_SEEDED,
    START_RPC_SET,

    // Radio is configured, turning it on.
    ON_STATUS_READ,
    ON_PLL_WAITING,
    ON_PLL_SET,

    // Radio is in the RX_AACK_ON state, ready to receive packets.
    READY,

    // States that transition the radio to and from SLEEP
    SLEEP_TRX_OFF,
    SLEEP,
    SLEEP_WAKE,

    // States pertaining to packet transmission.
    // Note that this state machine can be aborted due to
    // an incoming packet; self.transmitting keeps track
    // of whether a transmission is pending.
    TX_STATUS_PRECHECK1,
    TX_WRITING_FRAME,
    TX_WRITING_FRAME_DONE,
    TX_STATUS_PRECHECK2,
    TX_PLL_START,
    TX_PLL_WAIT,
    TX_ARET_ON,
    TX_TRANSMITTING,
    TX_READ_ACK,
    TX_DONE,
    TX_RETURN_TO_RX,

    // This state denotes we began a transmission, but
    // before we could transition to PLL_ON a packet began
    // to be received. When we handle the initial RX interrupt,
    // we'll transition to the correct state. We can't return to READY
    // because we need to block other operations.
    TX_PENDING,

    // Intermediate states when committing configuration from RAM
    // to the chiP; short address, PAN address, tx power and channel
    CONFIG_SHORT0_SET,
    CONFIG_SHORT1_SET,
    CONFIG_PAN0_SET,
    CONFIG_PAN1_SET,
    CONFIG_IEEE0_SET,
    CONFIG_IEEE1_SET,
    CONFIG_IEEE2_SET,
    CONFIG_IEEE3_SET,
    CONFIG_IEEE4_SET,
    CONFIG_IEEE5_SET,
    CONFIG_IEEE6_SET,
    CONFIG_IEEE7_SET,
    CONFIG_POWER_SET,
    CONFIG_DONE,

    // RX is a short-lived state for when software has detected
    // the chip is receiving a packet (by internal state) but has
    // not received the interrupt yet. I.e., the SFD has been
    // received but not the rest of the packet yet.
    RX,
    // The packet has been successfully received
    RX_TURNING_OFF,       // Disabling packet reception
    RX_READY_TO_READ,     // Reception disabled, handle interrupt and start reading
    RX_START_READING,     // Starting to read a packet out of the radio
    RX_READING_FRAME_LEN, // We've read the length of the frame
    RX_READING_FRAME_LEN_DONE,
    RX_READING_FRAME,      // Reading the packet out of the radio
    RX_READING_FRAME_DONE, // Now read a register to verify FCS
    RX_READING_FRAME_FCS_DONE,
    RX_ENABLING_RECEPTION, // Re-enabling reception
}

// There are two tricky parts to this capsule: buffer management
// and the finite state machine.
//
// Buffer management is tricky because the implementation tries to
// minimize the different buffers it uses. It needs to be able to send
// 2-byte register reads and writes on initialization. So it needs 2
// 2-byte buffers for these. When it is transmitting a packet, it
// performs one long write over SPI to move the packet to the radio.
// It needs a read buffer of equal length so it can check the radio
// state.  Similarly, when it reads a packet out of RAM into a buffer,
// it needs an equal length buffer for the SPI write. Finally, it
// needs a buffer to receive packets into, so it doesn't drop a packet
// just because an application didn't read in time. Therefore, the
// structure needs four buffers: 2 2-byte buffers and two
// packet-length buffers.  Since the SPI callback does not distinguish
// which buffers are being used, the read_write_done callback checks
// which state the stack is in and places the buffers back
// accodingly. A bug here would mean a memory leak and later panic
// when a buffer that should be present has been lost.
//
// The finite state machine is tricky for two reasons. First, the
// radio can issue an interrupt at any time, and the stack handles the
// interrupt (clearing it) by reading the IRQ_STATUS
// register. Therefore, when an interrupt occurs, the next SPI
// operation needs to read IRQ_STATUS (and potentially change
// self.state) before returning to the main state
// machine. self.interrupt_pending indicates if an interrupt has fired
// and therefore must be handled by reading IRQ_STATUS and acting
// accordingly. self.interrupt_handling indicates that a read of
// IRQ_STATUS is pending and so the read_write_done should enact state
// transitions based on the interrupt.
//
// Second, it is possible that a packet starts arriving while the
// stack is preparing a transmission. In this case, the transmission
// needs to be aborted, but restarted once the reception
// completes. The stack keeps track of this with self.transmitting.
// The final state before transmission is TX_ARET_ON; the next step is
// to start transmission. If a start-of-frame interrupt is handled at
// any point in the TX state machine, the stack moves to the RX state
// and waits for the interrupt specifying the entire packet has been
// received.

pub struct RF233<'a, S: spi::SpiMasterDevice> {
    spi: &'a S,
    radio_on: Cell<bool>,
    transmitting: Cell<bool>,
    receiving: Cell<bool>,
    spi_busy: Cell<bool>,
    crc_valid: Cell<bool>,
    interrupt_handling: Cell<bool>,
    interrupt_pending: Cell<bool>,
    config_pending: Cell<bool>,
    sleep_pending: Cell<bool>,
    wake_pending: Cell<bool>,
    power_client_pending: Cell<bool>,
    reset_pin: &'a gpio::Pin,
    sleep_pin: &'a gpio::Pin,
    irq_pin: &'a gpio::Pin,
    irq_ctl: &'a gpio::PinCtl,
    state: Cell<InternalState>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_len: Cell<u8>,
    tx_client: OptionalCell<&'static radio::TxClient>,
    rx_client: OptionalCell<&'static radio::RxClient>,
    cfg_client: OptionalCell<&'static radio::ConfigClient>,
    power_client: OptionalCell<&'static radio::PowerClient>,
    addr: Cell<u16>,
    addr_long: Cell<[u8; 8]>,
    pan: Cell<u16>,
    tx_power: Cell<i8>,
    channel: Cell<u8>,
    spi_rx: TakeCell<'static, [u8]>,
    spi_tx: TakeCell<'static, [u8]>,
    spi_buf: TakeCell<'static, [u8]>,
}

fn setting_to_power(setting: u8) -> i8 {
    match setting {
        0x00 => 4,
        0x01 => 4,
        0x02 => 3,
        0x03 => 3,
        0x04 => 2,
        0x05 => 2,
        0x06 => 1,
        0x07 => 0,
        0x08 => -1,
        0x09 => -2,
        0x0A => -3,
        0x0B => -4,
        0x0C => -6,
        0x0D => -8,
        0x0E => -12,
        0x0F => -17,
        _ => -127,
    }
}

fn power_to_setting(power: i8) -> u8 {
    if (power >= 4) {
        return 0x00;
    } else if (power >= 3) {
        return 0x03;
    } else if (power >= 2) {
        return 0x05;
    } else if (power >= 1) {
        return 0x06;
    } else if (power >= 0) {
        return 0x07;
    } else if (power >= -1) {
        return 0x08;
    } else if (power >= -2) {
        return 0x09;
    } else if (power >= -3) {
        return 0x0A;
    } else if (power >= -4) {
        return 0x0B;
    } else if (power >= -6) {
        return 0x0C;
    } else if (power >= -8) {
        return 0x0D;
    } else if (power >= -12) {
        return 0x0E;
    } else {
        return 0x0F;
    }
}

fn interrupt_included(mask: u8, interrupt: InteruptFlags) -> bool {
    let int = interrupt as u8;
    (mask & int) == int
}

impl<S: spi::SpiMasterDevice> spi::SpiMasterClient for RF233<'a, S> {
    // This function is a bit confusing because the order of the logic in the
    // function is different than the order of operations during transmission
    // and reception.
    fn read_write_done(
        &self,
        mut _write: &'static mut [u8],
        mut read: Option<&'static mut [u8]>,
        _len: usize,
    ) {
        self.spi_busy.set(false);
        let rbuf = read.take().unwrap();
        let status = rbuf[0] & 0x1f;
        let result = rbuf[1];

        // Need to put buffers back. Four cases:
        // 1. a frame read completed, need to put RX buf back and put the
        //    used write buf back into spi_buf
        // 2. a frame length read completed, need to put RX buf back and
        //    put the used write buf back into spi_buf
        // 3. a frame write completed, need to put TX buf back and put the
        //    used read buf back into spi_buf
        // 4. a register op completed, need to but the used read buf back into
        //    spi_rx and the used write buf into spi_tx. interrupt handling
        //    is implicitly a register op.
        // Note that in cases 1-3, we need to enact a state transition
        // so that, if an interrupt is pending, we don't put the buffers
        // back again. The _DONE states denote that the frame transfer
        // has completed. So we'll put the buffers back only once.
        let state = self.state.get();

        let handling = self.interrupt_handling.get();
        if !handling && state == InternalState::RX_READING_FRAME_LEN {
            self.spi_buf.replace(_write);
            self.rx_buf.replace(rbuf);
            self.state.set(InternalState::RX_READING_FRAME_LEN_DONE);
        } else if !handling && state == InternalState::RX_READING_FRAME {
            self.spi_buf.replace(_write);
            self.rx_buf.replace(rbuf);
            self.state.set(InternalState::RX_READING_FRAME_DONE);
        } else if !handling && state == InternalState::TX_WRITING_FRAME {
            self.spi_buf.replace(rbuf);
            self.tx_buf.replace(_write);
            self.state.set(InternalState::TX_WRITING_FRAME_DONE);
        } else {
            self.spi_rx.replace(rbuf);
            self.spi_tx.replace(_write);
        }

        let state = self.state.get();

        // This case is when the SPI operation is reading the IRQ_STATUS
        // register from handling an interrupt. Note that we're done handling
        // the interrupt and continue with the state machine.
        if handling {
            self.interrupt_handling.set(false);

            let interrupt = result;

            // If we're going to sleep, ignore the interrupt and continue
            if state != InternalState::SLEEP_TRX_OFF && state != InternalState::SLEEP {
                if state == InternalState::ON_PLL_WAITING {
                    if interrupt_included(interrupt, InteruptFlags::IRQ_0_PLL_LOCK) {
                        self.state.set(InternalState::ON_PLL_SET);
                    }
                } else if state == InternalState::TX_TRANSMITTING
                    && interrupt_included(interrupt, InteruptFlags::IRQ_3_TRX_END)
                {
                    self.state.set(InternalState::TX_DONE);
                }
                if interrupt_included(interrupt, InteruptFlags::IRQ_2_RX_START) {
                    // Start of frame
                    self.receiving.set(true);
                    self.state.set(InternalState::RX);
                }

                // We've received  an entire frame into the frame buffer. This should be
                // in the InternalState::RX_READY_TO_READ state.
                // There are three cases:
                //   1. we have a receive buffer: copy it out
                //   2. no receive buffer, but transmission pending: send
                //   3. no receive buffer, no transmission: return to waiting
                if (interrupt_included(interrupt, InteruptFlags::IRQ_3_TRX_END)
                    && self.receiving.get())
                {
                    self.receiving.set(false);
                    if self.rx_buf.is_some() {
                        self.state.set(InternalState::RX_START_READING);
                    } else if self.transmitting.get() {
                        self.state_transition_read(
                            RF233Register::MIN,
                            InternalState::TX_STATUS_PRECHECK1,
                        );
                        return;
                    } else {
                        self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                        return;
                    }
                }
            }
        }

        // No matter what, if the READY state is reached, the radio is on. This
        // needs to occur before handling the interrupt below.
        if self.state.get() == InternalState::READY {
            self.wake_pending.set(false);

            // If we just woke up, note that we need to call the PowerClient
            if !self.radio_on.get() {
                self.power_client_pending.set(true);
            }
            self.radio_on.set(true);
        }

        // An interrupt can only be pending if an interrupt was fired during an
        // SPI operation: we wait for the SPI operation to complete then handle
        // the interrupt by reading the IRQ_STATUS register over the SPI.
        //
        // However, we should not handle the interrupt if we are in the midst of
        // receiving a frame.
        if self.interrupt_pending.get() {
            match self.state.get() {
                InternalState::RX_TURNING_OFF
                | InternalState::RX_START_READING
                | InternalState::RX_READING_FRAME_DONE
                | InternalState::RX_READING_FRAME_FCS_DONE => {}
                _ => {
                    self.interrupt_pending.set(false);
                    self.handle_interrupt();
                    return;
                }
            }
        }
        // Similarly, if a configuration is pending, we only start the
        // configuration process when we are in a state where it is legal to
        // start the configuration process.
        if self.config_pending.get() && self.state.get() == InternalState::READY {
            self.state_transition_write(
                RF233Register::SHORT_ADDR_0,
                (self.addr.get() & 0xff) as u8,
                InternalState::CONFIG_SHORT0_SET,
            );
        }

        match self.state.get() {
            // Default on state; wait for transmit() call or receive interrupt
            InternalState::READY => {
                // If stop() was called, start turning off the radio.
                if self.sleep_pending.get() {
                    self.sleep_pending.set(false);
                    self.radio_on.set(false);
                    self.state_transition_write(
                        RF233Register::TRX_STATE,
                        RF233TrxCmd::OFF as u8,
                        InternalState::SLEEP_TRX_OFF,
                    );
                } else if self.power_client_pending.get() {
                    // fixes bug where client would start transmitting before this state completed
                    self.power_client_pending.set(false);
                    self.power_client.map(|p| {
                        p.changed(self.radio_on.get());
                    });
                }
            }
            // Starting state, begin start sequence.
            InternalState::START => {
                self.state_transition_read(
                    RF233Register::IRQ_STATUS,
                    InternalState::START_PART_READ,
                );
            }
            InternalState::START_PART_READ => {
                self.state_transition_read(
                    RF233Register::TRX_STATUS,
                    InternalState::START_STATUS_READ,
                );
            }
            InternalState::START_STATUS_READ => {
                if status == ExternalState::ON as u8 {
                    self.state_transition_write(
                        RF233Register::TRX_STATE,
                        RF233TrxCmd::OFF as u8,
                        InternalState::START_TURNING_OFF,
                    );
                } else {
                    self.state_transition_write(
                        RF233Register::TRX_CTRL_1,
                        TRX_CTRL_1,
                        InternalState::START_CTRL1_SET,
                    );
                }
            }
            InternalState::START_TURNING_OFF => {
                self.irq_pin.make_input();
                self.irq_pin.clear();
                self.irq_ctl.set_input_mode(gpio::InputMode::PullNone);
                self.irq_pin
                    .enable_interrupt(INTERRUPT_ID, gpio::InterruptMode::RisingEdge);

                self.state_transition_write(
                    RF233Register::TRX_CTRL_1,
                    TRX_CTRL_1,
                    InternalState::START_CTRL1_SET,
                );
            }
            InternalState::START_CTRL1_SET => {
                let val = self.channel.get() | PHY_CC_CCA_MODE_CS_OR_ED;
                self.state_transition_write(
                    RF233Register::PHY_CC_CCA,
                    val,
                    InternalState::START_CCA_SET,
                );
            }
            InternalState::START_CCA_SET => {
                let val = power_to_setting(self.tx_power.get());
                self.state_transition_write(
                    RF233Register::PHY_TX_PWR,
                    val,
                    InternalState::START_PWR_SET,
                );
            }
            InternalState::START_PWR_SET => self.state_transition_write(
                RF233Register::TRX_CTRL_2,
                TRX_CTRL_2,
                InternalState::START_CTRL2_SET,
            ),
            InternalState::START_CTRL2_SET => {
                self.state_transition_write(
                    RF233Register::IRQ_MASK,
                    IRQ_MASK,
                    InternalState::START_IRQMASK_SET,
                );
            }

            InternalState::START_IRQMASK_SET => {
                self.state_transition_write(
                    RF233Register::XAH_CTRL_1,
                    XAH_CTRL_1,
                    InternalState::START_XAH1_SET,
                );
            }

            InternalState::START_XAH1_SET => {
                // This encapsulates the frame retry and CSMA retry
                // settings in the RF233 C code
                self.state_transition_write(
                    RF233Register::XAH_CTRL_0,
                    XAH_CTRL_0,
                    InternalState::START_XAH0_SET,
                );
            }
            InternalState::START_XAH0_SET => {
                self.state_transition_write(
                    RF233Register::PAN_ID_0,
                    (self.pan.get() >> 8) as u8,
                    InternalState::START_PANID0_SET,
                );
            }
            InternalState::START_PANID0_SET => {
                self.state_transition_write(
                    RF233Register::PAN_ID_1,
                    (self.pan.get() & 0xff) as u8,
                    InternalState::START_PANID1_SET,
                );
            }
            InternalState::START_PANID1_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_0,
                    self.addr_long.get()[0],
                    InternalState::START_IEEE0_SET,
                );
            }
            InternalState::START_IEEE0_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_1,
                    self.addr_long.get()[1],
                    InternalState::START_IEEE1_SET,
                );
            }
            InternalState::START_IEEE1_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_2,
                    self.addr_long.get()[2],
                    InternalState::START_IEEE2_SET,
                );
            }
            InternalState::START_IEEE2_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_3,
                    self.addr_long.get()[3],
                    InternalState::START_IEEE3_SET,
                );
            }
            InternalState::START_IEEE3_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_4,
                    self.addr_long.get()[4],
                    InternalState::START_IEEE4_SET,
                );
            }
            InternalState::START_IEEE4_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_5,
                    self.addr_long.get()[5],
                    InternalState::START_IEEE5_SET,
                );
            }
            InternalState::START_IEEE5_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_6,
                    self.addr_long.get()[6],
                    InternalState::START_IEEE6_SET,
                );
            }
            InternalState::START_IEEE6_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_7,
                    self.addr_long.get()[7],
                    InternalState::START_IEEE7_SET,
                );
            }
            InternalState::START_IEEE7_SET => {
                self.state_transition_write(
                    RF233Register::SHORT_ADDR_0,
                    (self.addr.get() & 0xff) as u8,
                    InternalState::START_SHORT0_SET,
                );
            }
            InternalState::START_SHORT0_SET => {
                self.state_transition_write(
                    RF233Register::SHORT_ADDR_1,
                    (self.addr.get() >> 8) as u8,
                    InternalState::START_SHORT1_SET,
                );
            }
            InternalState::START_SHORT1_SET => {
                self.state_transition_write(
                    RF233Register::CSMA_SEED_0,
                    SHORT_ADDR_0 + SHORT_ADDR_1,
                    InternalState::START_CSMA_0_SEEDED,
                );
            }
            InternalState::START_CSMA_0_SEEDED => {
                self.state_transition_write(
                    RF233Register::CSMA_SEED_1,
                    CSMA_SEED_1,
                    InternalState::START_CSMA_1_SEEDED,
                );
            }
            InternalState::START_CSMA_1_SEEDED => {
                self.state_transition_write(
                    RF233Register::TRX_RPC,
                    TRX_RPC,
                    InternalState::START_RPC_SET,
                );
            }
            InternalState::START_RPC_SET => {
                // If asleep, turn on
                self.state_transition_read(
                    RF233Register::TRX_STATUS,
                    InternalState::ON_STATUS_READ,
                );
            }
            InternalState::ON_STATUS_READ => {
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::PLL_ON as u8,
                    InternalState::ON_PLL_WAITING,
                );
            }
            InternalState::ON_PLL_WAITING => {
                // Waiting for the PLL interrupt, do nothing
            }

            // Final startup state, transition to READY and turn radio on.
            InternalState::ON_PLL_SET => {
                // We've completed the SPI operation to read the
                // IRQ_STATUS register, triggered by an interrupt
                // denoting moving to the PLL_ON state, so move
                // to RX_ON (see Sec 7, pg 36 of RF233 datasheet
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::RX_AACK_ON as u8,
                    InternalState::READY,
                );
            }
            InternalState::SLEEP_TRX_OFF => {
                // Toggle the sleep pin to put the radio into sleep mode
                self.sleep_pin.set();

                // If start() was called while we were shutting down,
                // immediately start turning the radio back on
                if self.wake_pending.get() {
                    self.state_transition_read(
                        RF233Register::TRX_STATUS,
                        InternalState::SLEEP_WAKE,
                    );
                // Inform power client that the radio turned off successfully
                } else {
                    self.state.set(InternalState::SLEEP);
                    self.power_client.map(|p| {
                        p.changed(self.radio_on.get());
                    });
                }
            }
            // Do nothing; a call to start() is required to restart radio
            InternalState::SLEEP => {}

            InternalState::SLEEP_WAKE => {
                // Toggle the sleep pin to take the radio out of sleep mode into
                // InternalState::TRX_OFF, then transition directly to RX_AACK_ON.
                self.sleep_pin.clear();
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::RX_AACK_ON as u8,
                    InternalState::READY,
                );
            }
            InternalState::TX_STATUS_PRECHECK1 => {
                if (status == ExternalState::BUSY_RX_AACK as u8
                    || status == ExternalState::BUSY_TX_ARET as u8
                    || status == ExternalState::BUSY_RX as u8)
                {
                    self.state.set(InternalState::TX_PENDING);
                } else {
                    // Something wrong here?
                    self.state.set(InternalState::TX_WRITING_FRAME);
                    let wbuf = self.tx_buf.take().unwrap();
                    self.frame_write(wbuf, self.tx_len.get());
                }
            }
            InternalState::TX_WRITING_FRAME => {} // Should never get here
            InternalState::TX_WRITING_FRAME_DONE => {
                self.state_transition_read(
                    RF233Register::TRX_STATUS,
                    InternalState::TX_STATUS_PRECHECK2,
                );
            }
            InternalState::TX_STATUS_PRECHECK2 => {
                if (status == ExternalState::BUSY_RX_AACK as u8
                    || status == ExternalState::BUSY_TX_ARET as u8
                    || status == ExternalState::BUSY_RX as u8)
                {
                    self.receiving.set(true);
                    self.state.set(InternalState::RX);
                } else {
                    self.state_transition_write(
                        RF233Register::TRX_STATE,
                        RF233TrxCmd::PLL_ON as u8,
                        InternalState::TX_PLL_START,
                    );
                }
            }
            InternalState::TX_PLL_START => {
                self.state_transition_read(RF233Register::TRX_STATUS, InternalState::TX_PLL_WAIT);
            }
            InternalState::TX_PLL_WAIT => {
                self.transmitting.set(true);
                if status == ExternalState::STATE_TRANSITION_IN_PROGRESS as u8 {
                    self.state_transition_read(
                        RF233Register::TRX_STATUS,
                        InternalState::TX_PLL_WAIT,
                    );
                } else if status != ExternalState::PLL_ON as u8 {
                    self.state_transition_write(
                        RF233Register::TRX_STATE,
                        RF233TrxCmd::PLL_ON as u8,
                        InternalState::TX_PLL_WAIT,
                    );
                } else {
                    self.state_transition_write(
                        RF233Register::TRX_STATE,
                        RF233TrxCmd::TX_ARET_ON as u8,
                        InternalState::TX_ARET_ON,
                    );
                }
            }
            InternalState::TX_ARET_ON => {
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::TX_START as u8,
                    InternalState::TX_TRANSMITTING,
                );
            }
            InternalState::TX_TRANSMITTING => {
                // Do nothing, wait for TRX_END interrupt denoting transmission
                // completed. The code at the top of this SPI handler for
                // interrupt handling will transition to the TX_DONE state.
            }
            InternalState::TX_DONE => {
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::RX_AACK_ON as u8,
                    InternalState::TX_READ_ACK,
                );
            }
            InternalState::TX_READ_ACK => {
                self.state_transition_read(
                    RF233Register::TRX_STATE,
                    InternalState::TX_RETURN_TO_RX,
                );
            }

            // Insert read of TRX_STATUS here, checking TRAC
            InternalState::TX_RETURN_TO_RX => {
                let ack: bool = (result & TRX_TRAC_MASK) == 0;
                if status == ExternalState::RX_AACK_ON as u8 {
                    let return_code = if (result & TRX_TRAC_MASK) == TRX_TRAC_CHANNEL_ACCESS_FAILURE
                    {
                        ReturnCode::FAIL
                    } else {
                        ReturnCode::SUCCESS
                    };

                    self.transmitting.set(false);
                    let buf = self.tx_buf.take();
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);

                    self.tx_client.map(|c| {
                        c.send_done(buf.unwrap(), ack, return_code);
                    });
                } else {
                    self.register_read(RF233Register::TRX_STATUS);
                }
            }

            // This state occurs when, in the midst of starting a
            // transmission, we discovered that the radio had moved into
            // a receive state. Since this will trigger interrupts,
            // we enter this dead state and just wait for the interrupt
            // handlers.
            InternalState::TX_PENDING => {}

            // No operations in the RX state, an SFD interrupt should
            // take us out of it.
            InternalState::RX => {}
            InternalState::RX_TURNING_OFF => {
                // This is the case when the driver turns off reception in
                // response to receiving a frame, to make sure it is not
                // overwritten. Now we are reading to handle the interrupt and
                // start reading out the frame.
                self.state_transition_read(
                    RF233Register::IRQ_STATUS,
                    InternalState::RX_READY_TO_READ,
                );
                self.interrupt_handling.set(true);
            }
            // This state is when the driver handles the pending TRX_END interrupt
            // on reception, so is handled above in the interrupt logic.
            // the pending interrupt will be handled
            InternalState::RX_READY_TO_READ => {}

            // Read the length out
            InternalState::RX_START_READING => {
                self.state.set(InternalState::RX_READING_FRAME_LEN);
                // A frame read of frame_length 0 results in the received SPI
                // buffer only containing two bytes, the chip status and the
                // frame length.
                self.frame_read(self.rx_buf.take().unwrap(), 0);
            }

            InternalState::RX_READING_FRAME_LEN => {} // Should not get this
            InternalState::RX_READING_FRAME_LEN_DONE => {
                // A frame read starts with a 1-byte chip status followed by a
                // 1-byte PHY header, which is the length of the frame.
                // Then, the frame follows, and there are 3 more bytes at the
                // end corresponding to LQI, ED, and RX_STATUS. Performing a
                // shorter frame read just drops these bytes.
                let frame_len = result;
                // If the packet isn't too long to fit in the SPI buffer, read it
                if (frame_len <= radio::MAX_FRAME_SIZE as u8
                    && frame_len >= radio::MIN_FRAME_SIZE as u8)
                {
                    self.state.set(InternalState::RX_READING_FRAME);
                    let rbuf = self.rx_buf.take().unwrap();
                    self.frame_read(rbuf, frame_len);
                } else if self.transmitting.get() {
                    // Packet was too long and a transmission is pending,
                    // start the transmission
                    self.state_transition_read(
                        RF233Register::TRX_STATUS,
                        InternalState::TX_STATUS_PRECHECK1,
                    );
                } else {
                    // Packet was too long and no pending transmission,
                    // return to waiting for packets.
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                }
            }
            InternalState::RX_READING_FRAME => {} // Should never get this state
            InternalState::RX_READING_FRAME_DONE => {
                // Now read the PHY_RSSI register to obtain the RX_CRC_VALID bit
                self.state_transition_read(
                    RF233Register::PHY_RSSI,
                    InternalState::RX_READING_FRAME_FCS_DONE,
                );
            }
            InternalState::RX_READING_FRAME_FCS_DONE => {
                // Store whether the CRC was valid, then turn the radio back on.
                self.crc_valid.set((result & PHY_RSSI_RX_CRC_VALID) != 0);
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::RX_AACK_ON as u8,
                    InternalState::RX_ENABLING_RECEPTION,
                );
            }
            InternalState::RX_ENABLING_RECEPTION => {
                self.receiving.set(false);

                // Stay awake if we receive a packet, another call to stop()
                // is therefore necessary to shut down the radio. Currently
                // mainly benefits the XMAC wrapper that would like to avoid
                // a shutdown when in the expected case the radio should stay
                // awake.
                self.sleep_pending.set(false);

                // Just read a packet: if a transmission is pending,
                // start the transmission state machine
                if self.transmitting.get() {
                    self.state_transition_read(
                        RF233Register::TRX_STATUS,
                        InternalState::TX_STATUS_PRECHECK1,
                    );
                } else {
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                }
                self.rx_client.map(|client| {
                    let rbuf = self.rx_buf.take().unwrap();
                    let frame_len = rbuf[1] as usize - radio::MFR_SIZE;
                    client.receive(rbuf, frame_len, self.crc_valid.get(), ReturnCode::SUCCESS);
                });
            }

            InternalState::CONFIG_SHORT0_SET => {
                self.state_transition_write(
                    RF233Register::SHORT_ADDR_1,
                    (self.addr.get() >> 8) as u8,
                    InternalState::CONFIG_SHORT1_SET,
                );
            }
            InternalState::CONFIG_SHORT1_SET => {
                self.state_transition_write(
                    RF233Register::PAN_ID_0,
                    (self.pan.get() & 0xff) as u8,
                    InternalState::CONFIG_PAN0_SET,
                );
            }
            InternalState::CONFIG_PAN0_SET => {
                self.state_transition_write(
                    RF233Register::PAN_ID_1,
                    (self.pan.get() >> 8) as u8,
                    InternalState::CONFIG_PAN1_SET,
                );
            }
            InternalState::CONFIG_PAN1_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_0,
                    self.addr_long.get()[0],
                    InternalState::CONFIG_IEEE0_SET,
                );
            }
            InternalState::CONFIG_IEEE0_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_1,
                    self.addr_long.get()[1],
                    InternalState::CONFIG_IEEE1_SET,
                );
            }
            InternalState::CONFIG_IEEE1_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_2,
                    self.addr_long.get()[2],
                    InternalState::CONFIG_IEEE2_SET,
                );
            }
            InternalState::CONFIG_IEEE2_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_3,
                    self.addr_long.get()[3],
                    InternalState::CONFIG_IEEE3_SET,
                );
            }
            InternalState::CONFIG_IEEE3_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_4,
                    self.addr_long.get()[4],
                    InternalState::CONFIG_IEEE4_SET,
                );
            }
            InternalState::CONFIG_IEEE4_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_5,
                    self.addr_long.get()[5],
                    InternalState::CONFIG_IEEE5_SET,
                );
            }
            InternalState::CONFIG_IEEE5_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_6,
                    self.addr_long.get()[6],
                    InternalState::CONFIG_IEEE6_SET,
                );
            }
            InternalState::CONFIG_IEEE6_SET => {
                self.state_transition_write(
                    RF233Register::IEEE_ADDR_7,
                    self.addr_long.get()[7],
                    InternalState::CONFIG_IEEE7_SET,
                );
            }
            InternalState::CONFIG_IEEE7_SET => {
                let val = power_to_setting(self.tx_power.get());
                self.state_transition_write(
                    RF233Register::PHY_TX_PWR,
                    val,
                    InternalState::CONFIG_POWER_SET,
                );
            }
            InternalState::CONFIG_POWER_SET => {
                let val = self.channel.get() | PHY_CC_CCA_MODE_CS_OR_ED;
                self.state_transition_write(
                    RF233Register::PHY_CC_CCA,
                    val,
                    InternalState::CONFIG_DONE,
                );
            }
            InternalState::CONFIG_DONE => {
                self.config_pending.set(false);
                self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                self.cfg_client.map(|c| {
                    c.config_done(ReturnCode::SUCCESS);
                });
            }
        }
    }
}

impl<S: spi::SpiMasterDevice> gpio::Client for RF233<'a, S> {
    fn fired(&self, identifier: usize) {
        if identifier == INTERRUPT_ID {
            self.handle_interrupt();
        }
    }
}

impl<S: spi::SpiMasterDevice> RF233<'a, S> {
    pub fn new(
        spi: &'a S,
        reset: &'a gpio::Pin,
        sleep: &'a gpio::Pin,
        irq: &'a gpio::Pin,
        ctl: &'a gpio::PinCtl,
    ) -> RF233<'a, S> {
        RF233 {
            spi: spi,
            reset_pin: reset,
            sleep_pin: sleep,
            irq_pin: irq,
            irq_ctl: ctl,
            radio_on: Cell::new(false),
            transmitting: Cell::new(false),
            receiving: Cell::new(false),
            spi_busy: Cell::new(false),
            crc_valid: Cell::new(false),
            state: Cell::new(InternalState::START),
            interrupt_handling: Cell::new(false),
            interrupt_pending: Cell::new(false),
            config_pending: Cell::new(false),
            sleep_pending: Cell::new(false),
            wake_pending: Cell::new(false),
            power_client_pending: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            cfg_client: OptionalCell::empty(),
            power_client: OptionalCell::empty(),
            addr: Cell::new(0),
            addr_long: Cell::new([0x00; 8]),
            pan: Cell::new(0),
            tx_power: Cell::new(setting_to_power(PHY_TX_PWR)),
            channel: Cell::new(PHY_CHANNEL),
            spi_rx: TakeCell::empty(),
            spi_tx: TakeCell::empty(),
            spi_buf: TakeCell::empty(),
        }
    }

    fn handle_interrupt(&self) {
        // In most cases, the first thing the driver does on handling an interrupt is
        // read the IRQ status; this pushes most logic to the SPI handler.
        // The one exception is when the radio receives a packet; to prevent this
        // packet from being overwritten before reading it from the radio,
        // the driver needs to disable reception. This has to be done in the first
        // SPI operation.
        if self.spi_busy.get() == false {
            if self.state.get() == InternalState::RX {
                // We've received a complete frame; need to disable
                // reception until we've read it out from RAM,
                // otherwise subsequent packets may corrupt it.
                // Dynamic Frame Buffer protection (RF233 manual, Sec
                // 11.8) is insufficient because we perform multiple
                // SPI operations to read a frame, and the RF233
                // releases its protection after the first SPI
                // operation.
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::PLL_ON as u8,
                    InternalState::RX_TURNING_OFF,
                );
            } else {
                self.interrupt_handling.set(true);
                self.register_read(RF233Register::IRQ_STATUS);
            }
        } else {
            self.interrupt_pending.set(true);
        }
    }

    fn register_write(&self, reg: RF233Register, val: u8) -> ReturnCode {
        if (self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none()) {
            return ReturnCode::EBUSY;
        }
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | RF233BusCommand::REGISTER_WRITE as u8;
        wbuf[1] = val;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);

        ReturnCode::SUCCESS
    }

    fn register_read(&self, reg: RF233Register) -> ReturnCode {
        if (self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none()) {
            return ReturnCode::EBUSY;
        }

        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | RF233BusCommand::REGISTER_READ as u8;
        wbuf[1] = 0;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);

        ReturnCode::SUCCESS
    }

    fn frame_write(&self, buf: &'static mut [u8], frame_len: u8) -> ReturnCode {
        if self.spi_busy.get() {
            return ReturnCode::EBUSY;
        }

        let buf_len = radio::PSDU_OFFSET + frame_len as usize;
        buf[0] = RF233BusCommand::FRAME_WRITE as u8;
        self.spi.read_write_bytes(buf, self.spi_buf.take(), buf_len);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    fn frame_read(&self, buf: &'static mut [u8], frame_len: u8) -> ReturnCode {
        if self.spi_busy.get() {
            return ReturnCode::EBUSY;
        }

        let buf_len = radio::PSDU_OFFSET + frame_len as usize;
        let wbuf = self.spi_buf.take().unwrap();
        wbuf[0] = RF233BusCommand::FRAME_READ as u8;
        self.spi.read_write_bytes(wbuf, Some(buf), buf_len);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    fn state_transition_write(&self, reg: RF233Register, val: u8, state: InternalState) {
        self.state.set(state);
        self.register_write(reg, val);
    }

    fn state_transition_read(&self, reg: RF233Register, state: InternalState) {
        self.state.set(state);
        self.register_read(reg);
    }
}

impl<S: spi::SpiMasterDevice> radio::Radio for RF233<'a, S> {}

impl<S: spi::SpiMasterDevice> radio::RadioConfig for RF233<'a, S> {
    fn initialize(
        &self,
        buf: &'static mut [u8],
        reg_write: &'static mut [u8],
        reg_read: &'static mut [u8],
    ) -> ReturnCode {
        if (buf.len() < radio::MAX_BUF_SIZE || reg_read.len() != 2 || reg_write.len() != 2) {
            return ReturnCode::ESIZE;
        }
        self.spi_buf.replace(buf);
        self.spi_rx.replace(reg_read);
        self.spi_tx.replace(reg_write);
        ReturnCode::SUCCESS
    }

    fn reset(&self) -> ReturnCode {
        self.spi.configure(
            spi::ClockPolarity::IdleLow,
            spi::ClockPhase::SampleLeading,
            100000,
        );
        self.reset_pin.make_output();
        self.sleep_pin.make_output();
        for _i in 0..10000 {
            self.reset_pin.clear();
        }
        self.reset_pin.set();
        self.sleep_pin.clear();
        self.transmitting.set(false);
        ReturnCode::SUCCESS
    }

    fn start(&self) -> ReturnCode {
        self.sleep_pending.set(false);

        if self.state.get() != InternalState::START && self.state.get() != InternalState::SLEEP {
            return ReturnCode::EALREADY;
        }

        if self.state.get() == InternalState::SLEEP {
            self.state_transition_read(RF233Register::TRX_STATUS, InternalState::SLEEP_WAKE);
        } else {
            // Delay wakeup until the radio turns all the way off
            self.wake_pending.set(true);
            self.register_read(RF233Register::PART_NUM);
        }

        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        if self.state.get() == InternalState::SLEEP
            || self.state.get() == InternalState::SLEEP_TRX_OFF
        {
            return ReturnCode::EALREADY;
        }

        match self.state.get() {
            InternalState::READY | InternalState::ON_PLL_WAITING => {
                self.radio_on.set(false);
                self.state_transition_write(
                    RF233Register::TRX_STATE,
                    RF233TrxCmd::OFF as u8,
                    InternalState::SLEEP_TRX_OFF,
                );
            }
            _ => {
                self.sleep_pending.set(true);
            }
        }

        ReturnCode::SUCCESS
    }

    fn is_on(&self) -> bool {
        self.radio_on.get()
    }

    fn busy(&self) -> bool {
        self.state.get() != InternalState::READY && self.state.get() != InternalState::SLEEP
    }

    fn set_config_client(&self, client: &'static radio::ConfigClient) {
        self.cfg_client.set(client);
    }

    fn set_power_client(&self, client: &'static radio::PowerClient) {
        self.power_client.set(client);
    }

    fn set_address(&self, addr: u16) {
        self.addr.set(addr);
    }

    fn set_address_long(&self, addr: [u8; 8]) {
        self.addr_long.set(addr);
    }

    fn set_pan(&self, id: u16) {
        self.pan.set(id);
    }

    fn set_tx_power(&self, power: i8) -> ReturnCode {
        if (power > 4 || power < -17) {
            ReturnCode::EINVAL
        } else {
            self.tx_power.set(power);
            ReturnCode::SUCCESS
        }
    }

    fn set_channel(&self, chan: u8) -> ReturnCode {
        if chan >= 11 && chan <= 26 {
            self.channel.set(chan);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EINVAL
        }
    }

    fn get_address(&self) -> u16 {
        self.addr.get()
    }

    fn get_address_long(&self) -> [u8; 8] {
        self.addr_long.get()
    }

    /// The 16-bit PAN ID
    fn get_pan(&self) -> u16 {
        self.pan.get()
    }
    /// The transmit power, in dBm
    fn get_tx_power(&self) -> i8 {
        self.tx_power.get()
    }
    /// The 802.15.4 channel
    fn get_channel(&self) -> u8 {
        self.channel.get()
    }

    fn config_commit(&self) {
        let pending = self.config_pending.get();
        if !pending {
            self.config_pending.set(true);
            let state = self.state.get();

            if state == InternalState::READY {
                // Start configuration commit
                self.state_transition_write(
                    RF233Register::SHORT_ADDR_0,
                    (self.addr.get() & 0xff) as u8,
                    InternalState::CONFIG_SHORT0_SET,
                );
            } else {
                // Do nothing --
                // Configuration will be pushed automatically on boot,
                // or pending flag will be checked on return to READY
                // and commit started
            }
        }
    }
}

impl<S: spi::SpiMasterDevice> radio::RadioData for RF233<'a, S> {
    fn set_transmit_client(&self, client: &'static radio::TxClient) {
        self.tx_client.set(client);
    }

    fn set_receive_client(&self, client: &'static radio::RxClient, buffer: &'static mut [u8]) {
        self.rx_client.set(client);
        self.rx_buf.replace(buffer);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.rx_buf.replace(buffer);
    }

    // The payload length is the length of the MAC payload, not the PSDU
    fn transmit(
        &self,
        spi_buf: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        let state = self.state.get();
        let frame_len = frame_len + radio::MFR_SIZE;

        if !self.radio_on.get() {
            return (ReturnCode::EOFF, Some(spi_buf));
        } else if self.tx_buf.is_some() || self.transmitting.get() {
            return (ReturnCode::EBUSY, Some(spi_buf));
        } else if radio::PSDU_OFFSET + frame_len >= spi_buf.len() {
            // Not enough room for CRC
            return (ReturnCode::ESIZE, Some(spi_buf));
        }

        // Set PHY header to be the frame length
        spi_buf[1] = frame_len as u8;
        self.tx_buf.replace(spi_buf);
        self.tx_len.set(frame_len as u8);
        self.transmitting.set(true);

        if !self.receiving.get() && state == InternalState::READY {
            self.state_transition_read(
                RF233Register::TRX_STATUS,
                InternalState::TX_STATUS_PRECHECK1,
            );
        }
        (ReturnCode::SUCCESS, None)
    }
}
