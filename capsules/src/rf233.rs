// I like them sometimes, for formatting -pal
#![allow(unused_parens)]

///
/// Capsule for sending 802.15.4 packets with an Atmel RF233.
///
/// This implementation is completely non-blocking. This means that
/// the state machine is somewhat complex, as it must interleave interrupt
/// handling with requests and radio state management. See the SPI
/// read_write_done handler for details.
///
/// To do items:
///    - Support TX power control
///    - Support channel selection
///    - Support link-layer acknowledgements
///    - Support power management (turning radio off)
// Capsule for sending 802.15.4 packets with an Atmel RF233.
//
// Author: Philip Levis
// Date: Jan 12 2017
//

use core::cell::Cell;
use kernel::common::take_cell::TakeCell;
use kernel::hil::gpio;
use kernel::hil::radio;
use kernel::hil::spi;
use kernel::returncode::ReturnCode;
use rf233_const::*;

const INTERRUPT_ID: usize = 0x2154;

#[allow(non_camel_case_types,dead_code)]
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
    START_CSMA_SEEDED,
    START_RPC_SET,

    // Radio is configured, turning it on.
    ON_STATUS_READ,
    ON_PLL_WAITING,
    ON_PLL_SET,

    // Radio is in the RX_AACK_ON state, ready to receive packets.
    READY,

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
    CONFIG_POWER_SET,
    CONFIG_DONE,

    // RX is a short-lived state for when software has detected
    // the chip is receiving a packet (by internal state) but has
    // not received the interrupt yet. I.e., the SFD has been
    // received but not the rest of the packet yet.
    RX,
    // The packet has been successfully received
    RX_START_READING, // Starting to read a packet out of the radio
    RX_READING_FRAME_LEN, // We've read the length of the frame
    RX_READING_FRAME_LEN_DONE,
    RX_READING_FRAME, // Reading the packet out of the radio
    RX_READING_FRAME_DONE,
    UNKNOWN,
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

pub struct RF233<'a, S: spi::SpiMasterDevice + 'a> {
    spi: &'a S,
    radio_on: Cell<bool>,
    transmitting: Cell<bool>,
    receiving: Cell<bool>,
    spi_busy: Cell<bool>,
    interrupt_handling: Cell<bool>,
    interrupt_pending: Cell<bool>,
    config_pending: Cell<bool>,
    reset_pin: &'a gpio::Pin,
    sleep_pin: &'a gpio::Pin,
    irq_pin: &'a gpio::Pin,
    irq_ctl: &'a gpio::PinCtl,
    state: Cell<InternalState>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_len: Cell<u8>,
    tx_client: Cell<Option<&'static radio::TxClient>>,
    rx_client: Cell<Option<&'static radio::RxClient>>,
    cfg_client: Cell<Option<&'static radio::ConfigClient>>,
    addr: Cell<u16>,
    pan: Cell<u16>,
    tx_power: Cell<i8>,
    channel: Cell<u8>,
    seq: Cell<u8>,
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

fn interrupt_included(mask: u8, interrupt: u8) -> bool {
    (mask & interrupt) == interrupt
}

impl<'a, S: spi::SpiMasterDevice + 'a> spi::SpiMasterClient for RF233<'a, S> {
    fn read_write_done(&self,
                       mut _write: &'static mut [u8],
                       mut read: Option<&'static mut [u8]>,
                       _len: usize) {
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

        // This first case is when an interrupt fired during an SPI operation:
        // we wait for the SPI operation to complete then handle the
        // interrupt by reading the IRQ_STATUS register over the SPI.
        // Since itself is an SPI operation, return.
        if self.interrupt_pending.get() == true {
            self.interrupt_pending.set(false);
            self.handle_interrupt();
            return;
        }
        // This second case is when the SPI operation is reading the
        // IRQ_STATUS register from handling an interrupt. Note that
        // we're done handling the interrupt and continue with the
        // state machine. This is an else because handle_interrupt
        // sets interrupt_handling to true.
        if handling {
            self.interrupt_handling.set(false);
            let state = self.state.get();
            let interrupt = result;
            if state == InternalState::ON_PLL_WAITING {
                if interrupt_included(interrupt, IRQ_0_PLL_LOCK) {
                    self.state.set(InternalState::ON_PLL_SET);
                }
            } else if (state == InternalState::TX_TRANSMITTING) &&
                      interrupt_included(interrupt, IRQ_3_TRX_END) {
                self.state.set(InternalState::TX_DONE);
            }
            if interrupt_included(interrupt, IRQ_2_RX_START) {
                // Start of frame
                self.receiving.set(true);
                self.state.set(InternalState::RX);
            }

            // We've received  an entire frame into the frame buffer.
            // There are three cases:
            //   1. we have a receive buffer: copy it out
            //   2. no receive buffer, but transmission pending: send
            //   3. no receive buffer, no transmission: return to waiting
            if (interrupt_included(interrupt, IRQ_3_TRX_END) && self.receiving.get()) {
                self.receiving.set(false);
                if self.rx_buf.is_some() {
                    self.state.set(InternalState::RX_START_READING);
                } else if self.transmitting.get() {
                    self.state_transition_read(RF233Register::MIN,
                                               InternalState::TX_STATUS_PRECHECK1);
                    return;
                } else {
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                    return;
                }
            }
        }

        match self.state.get() {
            // Default on state; wait for transmit() call or receive
            // interrupt
            InternalState::READY => {
                self.radio_on.set(true);
                if self.config_pending.get() == true {
                    self.state_transition_write(RF233Register::SHORT_ADDR_0,
                                                (self.addr.get() & 0xff) as u8,
                                                InternalState::CONFIG_SHORT0_SET);
                }
                // Useful debug code to test radio can transmit without
                // an app/calling system calls
                //unsafe {
                //    self.transmit(0xFFFF, &mut app_buf, 20);
                //}
            }

            // Starting state, begin start sequence.
            InternalState::START => {
                self.state_transition_read(RF233Register::IRQ_STATUS,
                                           InternalState::START_PART_READ);
            }
            InternalState::START_PART_READ => {
                self.state_transition_read(RF233Register::TRX_STATUS,
                                           InternalState::START_STATUS_READ);
            }
            InternalState::START_STATUS_READ => {
                if status == ExternalState::ON as u8 {
                    self.state_transition_write(RF233Register::TRX_STATE,
                                                RF233TrxCmd::OFF as u8,
                                                InternalState::START_TURNING_OFF);
                } else {
                    self.state_transition_write(RF233Register::TRX_CTRL_1,
                                                TRX_CTRL_1,
                                                InternalState::START_CTRL1_SET);
                }
            }
            InternalState::START_TURNING_OFF => {
                self.irq_pin.make_input();
                self.irq_pin.clear();
                self.irq_ctl.set_input_mode(gpio::InputMode::PullNone);
                self.irq_pin.enable_interrupt(INTERRUPT_ID, gpio::InterruptMode::RisingEdge);

                self.state_transition_write(RF233Register::TRX_CTRL_1,
                                            TRX_CTRL_1,
                                            InternalState::START_CTRL1_SET);
            }
            InternalState::START_CTRL1_SET => {
                let val = self.channel.get() | PHY_CC_CCA_MODE_CS_OR_ED;
                self.state_transition_write(RF233Register::PHY_CC_CCA,
                                            val,
                                            InternalState::START_CCA_SET);
            }
            InternalState::START_CCA_SET => {
                let val = power_to_setting(self.tx_power.get());
                self.state_transition_write(RF233Register::PHY_TX_PWR,
                                            val,
                                            InternalState::START_PWR_SET);
            }
            InternalState::START_PWR_SET => {
                self.state_transition_write(RF233Register::TRX_CTRL_2,
                                            TRX_CTRL_2,
                                            InternalState::START_CTRL2_SET)
            }
            InternalState::START_CTRL2_SET => {
                self.state_transition_write(RF233Register::IRQ_MASK,
                                            IRQ_MASK,
                                            InternalState::START_IRQMASK_SET);
            }

            InternalState::START_IRQMASK_SET => {
                self.state_transition_write(RF233Register::XAH_CTRL_1,
                                            XAH_CTRL_1,
                                            InternalState::START_XAH1_SET);
            }

            InternalState::START_XAH1_SET => {
                // This encapsulates the frame retry and CSMA retry
                // settings in the RF233 C code
                self.state_transition_write(RF233Register::XAH_CTRL_0,
                                            XAH_CTRL_0,
                                            InternalState::START_XAH0_SET);
            }
            InternalState::START_XAH0_SET => {
                self.state_transition_write(RF233Register::PAN_ID_0,
                                            (self.pan.get() >> 8) as u8,
                                            InternalState::START_PANID0_SET);
            }
            InternalState::START_PANID0_SET => {
                self.state_transition_write(RF233Register::PAN_ID_1,
                                            (self.pan.get() & 0xff) as u8,
                                            InternalState::START_PANID1_SET);
            }
            InternalState::START_PANID1_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_0,
                                            IEEE_ADDR_0,
                                            InternalState::START_IEEE0_SET);
            }
            InternalState::START_IEEE0_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_1,
                                            IEEE_ADDR_1,
                                            InternalState::START_IEEE1_SET);
            }
            InternalState::START_IEEE1_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_2,
                                            IEEE_ADDR_2,
                                            InternalState::START_IEEE2_SET);
            }
            InternalState::START_IEEE2_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_3,
                                            IEEE_ADDR_3,
                                            InternalState::START_IEEE3_SET);
            }
            InternalState::START_IEEE3_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_4,
                                            IEEE_ADDR_4,
                                            InternalState::START_IEEE4_SET);
            }
            InternalState::START_IEEE4_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_5,
                                            IEEE_ADDR_5,
                                            InternalState::START_IEEE5_SET);
            }
            InternalState::START_IEEE5_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_6,
                                            IEEE_ADDR_6,
                                            InternalState::START_IEEE6_SET);
            }
            InternalState::START_IEEE6_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_7,
                                            IEEE_ADDR_7,
                                            InternalState::START_IEEE7_SET);
            }
            InternalState::START_IEEE7_SET => {
                self.state_transition_write(RF233Register::SHORT_ADDR_0,
                                            (self.addr.get() & 0xff) as u8,
                                            InternalState::START_SHORT0_SET);
            }
            InternalState::START_SHORT0_SET => {
                self.state_transition_write(RF233Register::SHORT_ADDR_1,
                                            (self.addr.get() >> 8) as u8,
                                            InternalState::START_SHORT1_SET);
            }
            InternalState::START_SHORT1_SET => {
                self.state_transition_write(RF233Register::CSMA_SEED_0,
                                            SHORT_ADDR_0 + SHORT_ADDR_1,
                                            InternalState::START_CSMA_SEEDED);
            }
            InternalState::START_CSMA_SEEDED => {
                self.state_transition_write(RF233Register::TRX_RPC,
                                            TRX_RPC,
                                            InternalState::START_RPC_SET);
            }
            InternalState::START_RPC_SET => {
                // If asleep, turn on
                self.state_transition_read(RF233Register::TRX_STATUS,
                                           InternalState::ON_STATUS_READ);
            }
            InternalState::ON_STATUS_READ => {
                self.state_transition_write(RF233Register::TRX_STATE,
                                            RF233TrxCmd::PLL_ON as u8,
                                            InternalState::ON_PLL_WAITING);
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
                self.state_transition_write(RF233Register::TRX_STATE,
                                            RF233TrxCmd::RX_AACK_ON as u8,
                                            InternalState::READY);
            }
            InternalState::TX_STATUS_PRECHECK1 => {
                if (status == ExternalState::BUSY_RX_AACK as u8 ||
                    status == ExternalState::BUSY_TX_ARET as u8 ||
                    status == ExternalState::BUSY_RX as u8) {
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
                self.state_transition_read(RF233Register::TRX_STATUS,
                                           InternalState::TX_STATUS_PRECHECK2);
            }
            InternalState::TX_STATUS_PRECHECK2 => {
                if (status == ExternalState::BUSY_RX_AACK as u8 ||
                    status == ExternalState::BUSY_TX_ARET as u8 ||
                    status == ExternalState::BUSY_RX as u8) {
                    self.receiving.set(true);
                    self.state.set(InternalState::RX);
                } else {
                    self.state_transition_write(RF233Register::TRX_STATE,
                                                RF233TrxCmd::PLL_ON as u8,
                                                InternalState::TX_PLL_START);
                }
            }
            InternalState::TX_PLL_START => {
                self.state_transition_read(RF233Register::TRX_STATUS, InternalState::TX_PLL_WAIT);
            }
            InternalState::TX_PLL_WAIT => {
                self.transmitting.set(true);
                if status == ExternalState::STATE_TRANSITION_IN_PROGRESS as u8 {
                    self.state_transition_read(RF233Register::TRX_STATUS,
                                               InternalState::TX_PLL_WAIT);
                } else if status != ExternalState::PLL_ON as u8 {
                    self.state_transition_write(RF233Register::TRX_STATE,
                                                RF233TrxCmd::PLL_ON as u8,
                                                InternalState::TX_PLL_WAIT);

                } else {
                    self.state_transition_write(RF233Register::TRX_STATE,
                                                RF233TrxCmd::TX_ARET_ON as u8,
                                                InternalState::TX_ARET_ON);
                }
            }
            InternalState::TX_ARET_ON => {
                self.state_transition_write(RF233Register::TRX_STATE,
                                            RF233TrxCmd::TX_START as u8,
                                            InternalState::TX_TRANSMITTING);
            }
            InternalState::TX_TRANSMITTING => {
                // Do nothing, wait for TRX_END interrupt denoting transmission
                // completed. The code at the top of this SPI handler for
                // interrupt handling will transition to the TX_DONE state.
            }
            InternalState::TX_DONE => {
                self.state_transition_write(RF233Register::TRX_STATE,
                                            RF233TrxCmd::RX_AACK_ON as u8,
                                            InternalState::TX_RETURN_TO_RX);
            }
            InternalState::TX_RETURN_TO_RX => {
                if status == ExternalState::RX_AACK_ON as u8 {
                    self.transmitting.set(false);
                    let buf = self.tx_buf.take();
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);

                    self.tx_client
                        .get()
                        .map(|c| { c.send_done(buf.unwrap(), ReturnCode::SUCCESS); });
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

            // Read the length out
            InternalState::RX_START_READING => {
                self.state.set(InternalState::RX_READING_FRAME_LEN);
                self.frame_read(self.rx_buf.take().unwrap(), 1);
            }

            InternalState::RX_READING_FRAME_LEN => {} // Should not get this
            InternalState::RX_READING_FRAME_LEN_DONE => {
                // Because the first byte of a frame read is
                // the status of the chip, the first byte of the
                // packet, the length field, is at index 1.
                // Subtract 2 for CRC, 1 for length byte.
                let len = result - 2 + 1;
                // If the packet isn't too long, read it
                if (len <= radio::MAX_PACKET_SIZE && len >= radio::MIN_PACKET_SIZE) {
                    self.state.set(InternalState::RX_READING_FRAME);
                    let rbuf = self.rx_buf.take().unwrap();
                    self.frame_read(rbuf, len);
                } else if self.transmitting.get() {
                    // Packet was too long and a transmission is pending,
                    // start the transmission
                    self.state_transition_read(RF233Register::TRX_STATUS,
                                               InternalState::TX_STATUS_PRECHECK1);
                } else {
                    // Packet was too long and no pending transmission,
                    // return to waiting for packets.
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                }
            }
            InternalState::RX_READING_FRAME => {} // Should never get this state
            InternalState::RX_READING_FRAME_DONE => {
                self.receiving.set(false);
                // Just read a packet: if a transmission is pending,
                // start the transmission state machine
                if self.transmitting.get() {
                    self.state_transition_read(RF233Register::TRX_STATUS,
                                               InternalState::TX_STATUS_PRECHECK1);
                } else {
                    self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                }
                self.rx_client.get().map(|client| {
                    let rbuf = self.rx_buf.take().unwrap();
                    // Subtract the CRC and add the length byte
                    let len = rbuf[1] - 2 + 1;
                    client.receive(rbuf, len, ReturnCode::SUCCESS);
                });
            }

            InternalState::CONFIG_SHORT0_SET => {
                self.state_transition_write(RF233Register::SHORT_ADDR_1,
                                            (self.addr.get() >> 8) as u8,
                                            InternalState::CONFIG_SHORT1_SET);
            }
            InternalState::CONFIG_SHORT1_SET => {
                self.state_transition_write(RF233Register::PAN_ID_0,
                                            (self.pan.get() & 0xff) as u8,
                                            InternalState::CONFIG_PAN0_SET);
            }
            InternalState::CONFIG_PAN0_SET => {
                self.state_transition_write(RF233Register::PAN_ID_1,
                                            (self.pan.get() >> 8) as u8,
                                            InternalState::CONFIG_PAN1_SET);
            }
            InternalState::CONFIG_PAN1_SET => {
                let val = power_to_setting(self.tx_power.get());
                self.state_transition_write(RF233Register::PHY_TX_PWR,
                                            val,
                                            InternalState::CONFIG_POWER_SET);
            }

            InternalState::CONFIG_POWER_SET => {
                let val = self.channel.get() | PHY_CC_CCA_MODE_CS_OR_ED;
                self.state_transition_write(RF233Register::PHY_CC_CCA,
                                            val,
                                            InternalState::CONFIG_DONE);
            }

            InternalState::CONFIG_DONE => {
                self.config_pending.set(false);
                self.state_transition_read(RF233Register::TRX_STATUS, InternalState::READY);
                self.cfg_client.get().map(|c| { c.config_done(ReturnCode::SUCCESS); });
            }

            InternalState::UNKNOWN => {}
        }
    }
}

impl<'a, S: spi::SpiMasterDevice + 'a> gpio::Client for RF233<'a, S> {
    fn fired(&self, identifier: usize) {
        if identifier == INTERRUPT_ID {
            self.handle_interrupt();
        }
    }
}

impl<'a, S: spi::SpiMasterDevice + 'a> RF233<'a, S> {
    pub fn new(spi: &'a S,
               reset: &'a gpio::Pin,
               sleep: &'a gpio::Pin,
               irq: &'a gpio::Pin,
               ctl: &'a gpio::PinCtl)
               -> RF233<'a, S> {
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
            state: Cell::new(InternalState::START),
            interrupt_handling: Cell::new(false),
            interrupt_pending: Cell::new(false),
            config_pending: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_client: Cell::new(None),
            rx_client: Cell::new(None),
            cfg_client: Cell::new(None),
            addr: Cell::new(0),
            pan: Cell::new(0),
            tx_power: Cell::new(setting_to_power(PHY_TX_PWR)),
            channel: Cell::new(PHY_CHANNEL),
            seq: Cell::new(0),
            spi_rx: TakeCell::empty(),
            spi_tx: TakeCell::empty(),
            spi_buf: TakeCell::empty(),
        }
    }

    fn handle_interrupt(&self) {
        // Because the first thing we do on handling an interrupt is
        // read the IRQ status, we defer handling the state transition
        // to the SPI handler
        if self.spi_busy.get() == false {
            self.interrupt_handling.set(true);
            self.register_read(RF233Register::IRQ_STATUS);
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

    fn frame_write(&self, buf: &'static mut [u8], buf_len: u8) -> ReturnCode {
        if self.spi_busy.get() {
            return ReturnCode::EBUSY;
        }

        let op_len = (buf_len + 1) as usize;
        buf[0] = RF233BusCommand::FRAME_WRITE as u8;
        self.spi.read_write_bytes(buf, self.spi_buf.take(), op_len);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    fn frame_read(&self, buf: &'static mut [u8], buf_len: u8) -> ReturnCode {
        if self.spi_busy.get() {
            return ReturnCode::EBUSY;
        }
        let op_len = (buf_len + 1) as usize;
        let wbuf = self.spi_buf.take().unwrap();
        wbuf[0] = RF233BusCommand::FRAME_READ as u8;
        self.spi.read_write_bytes(wbuf, Some(buf), op_len);
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

    /// Generate the 802.15.4 header and set up the radio's state to
    /// be able to send the packet (store reference, etc.).
    // For details on frame format, the old CC2420 datasheet is a
    // very good guide. -pal
    fn prepare_packet(&self, buf: &'static mut [u8], len: u8, dest: u16) {
        buf[0] = 0x00; // Where the frame command will go.
        buf[1] = len + 2 - 1; // plus 2 for CRC, - 1 for length byte  1/6/17 PAL
        buf[2] = 0x61; // 0x40: intra-PAN; 0x20: ack requested; 0x01: data frame
        buf[3] = 0x88; // 0x80: 16-bit src addr; 0x08: 16-bit dest addr
        buf[4] = self.seq.get();
        buf[5] = (self.pan.get() & 0xFF) as u8; // PAN id is 16 bits
        buf[6] = (self.pan.get() >> 8) as u8;
        buf[7] = (dest & 0xff) as u8;
        buf[8] = (dest >> 8) as u8;
        buf[9] = (self.addr.get() & 0xFF) as u8;
        buf[10] = (self.addr.get() >> 8) as u8;

        self.seq.set(self.seq.get() + 1);
        self.tx_buf.replace(buf);
        self.tx_len.set(len);
    }
}

impl<'a, S: spi::SpiMasterDevice + 'a> radio::Radio for RF233<'a, S> {
    fn initialize(&self,
                  buf: &'static mut [u8],
                  reg_write: &'static mut [u8],
                  reg_read: &'static mut [u8])
                  -> ReturnCode {
        if (buf.len() < radio::MAX_BUF_SIZE || reg_read.len() != 2 || reg_write.len() != 2) {
            return ReturnCode::ESIZE;
        }
        self.spi_buf.replace(buf);
        self.spi_rx.replace(reg_read);
        self.spi_tx.replace(reg_write);
        ReturnCode::SUCCESS
    }

    fn reset(&self) -> ReturnCode {
        self.spi.configure(spi::ClockPolarity::IdleLow,
                           spi::ClockPhase::SampleLeading,
                           100000);
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
        if self.state.get() != InternalState::START {
            return ReturnCode::FAIL;
        }
        self.register_read(RF233Register::PART_NUM);
        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn set_transmit_client(&self, client: &'static radio::TxClient) {
        self.tx_client.set(Some(client));
    }

    fn set_receive_client(&self, client: &'static radio::RxClient, buffer: &'static mut [u8]) {
        self.rx_client.set(Some(client));
        self.rx_buf.replace(buffer);
    }

    fn set_config_client(&self, client: &'static radio::ConfigClient) {
        self.cfg_client.set(Some(client));
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.rx_buf.replace(buffer);
    }

    fn config_set_address(&self, addr: u16) {
        self.addr.set(addr);
    }

    fn config_set_pan(&self, id: u16) {
        self.pan.set(id);
    }

    fn config_set_tx_power(&self, power: i8) -> ReturnCode {
        if (power > 4 || power < -17) {
            ReturnCode::EINVAL
        } else {
            self.tx_power.set(power);
            ReturnCode::SUCCESS
        }
    }

    fn config_set_channel(&self, chan: u8) -> ReturnCode {
        if chan >= 11 && chan <= 26 {
            self.channel.set(chan);
            ReturnCode::SUCCESS
        } else {
            ReturnCode::EINVAL
        }
    }


    fn config_address(&self) -> u16 {
        self.addr.get()
    }
    /// The 16-bit PAN ID
    fn config_pan(&self) -> u16 {
        self.pan.get()
    }
    /// The transmit power, in dBm
    fn config_tx_power(&self) -> i8 {
        self.tx_power.get()
    }
    /// The 802.15.4 channel
    fn config_channel(&self) -> u8 {
        self.channel.get()
    }

    fn config_commit(&self) -> ReturnCode {
        let pending = self.config_pending.get();
        if !pending {
            self.config_pending.set(true);
            let state = self.state.get();

            if state == InternalState::READY {
                // Start configuration commit
                self.state_transition_write(RF233Register::SHORT_ADDR_0,
                                            (self.addr.get() & 0xff) as u8,
                                            InternalState::CONFIG_SHORT0_SET);
            } else {
                // Do nothing --
                // Configuration will be pushed automatically on boot,
                // or pending flag will be checked on return to READY
                // and commit started
            }
        }
        ReturnCode::SUCCESS
    }

    // + 1 because we need space for the frame read/write byte for
    // the SPI command. Otherwise, if the packet begins at byte 0, we
    // have to copy it into a buffer whose byte 0 is the frame read/write
    // command.
    fn payload_offset(&self) -> u8 {
        radio::HEADER_SIZE + 1
    }

    fn header_size(&self) -> u8 {
        radio::HEADER_SIZE
    }

    fn ready(&self) -> bool {
        self.radio_on.get() && self.state.get() == InternalState::READY
    }

    fn transmit(&self, dest: u16, payload: &'static mut [u8], len: u8) -> ReturnCode {
        let state = self.state.get();
        if !self.radio_on.get() {
            return ReturnCode::EOFF;
        } else if self.tx_buf.is_some() || self.transmitting.get() {
            return ReturnCode::EBUSY;
        } else if (len + 2) as usize >= payload.len() {
            // Not enough room for CRC
            return ReturnCode::ESIZE;
        }

        self.prepare_packet(payload, len, dest);
        self.transmitting.set(true);
        if !self.receiving.get() && state == InternalState::READY {
            self.state_transition_read(RF233Register::TRX_STATUS,
                                       InternalState::TX_STATUS_PRECHECK1);
        }
        return ReturnCode::SUCCESS;
    }
}
