// For details on LoRa parameters, see Semtech SX1276/77/78/79 Datasheet.
// For understanding how the `Radio` capsule is architected around the `SpiMasterDevice`, see similiar implementation for the RF233.

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::debug_gpio;
use kernel::hil::gpio;
use kernel::hil::lora::{PacketConfig, RadioConfig, RadioData};
use kernel::hil::radio;
use kernel::hil::spi;
use kernel::hil::spi::{SpiMasterClient, SpiMasterDevice};
use kernel::ReturnCode;

// not written to registers unlike Modes
// only help in programming for SPI layer ops
#[derive(Copy, Clone, PartialEq)]
enum InternalState {
    TxOn,
    RxOn,
    Sleep,
    Ready,
}

// registers
pub enum RegMap {
    RegFifo = 0x00,
    RegOpMode = 0x01,
    RegFrfMsb = 0x06,
    RegFrfMid = 0x07,
    RegFrfLsb = 0x08,
    RegPaConfig = 0x09,
    RegOcp = 0x0b,
    RegLna = 0x0c,
    RegFifoAddrPtr = 0x0d,
    RegFifoTxBaseAddr = 0x0e,
    RegFifoRxBaseAddr = 0x0f,
    RegFifoRxCurrentAddr = 0x10,
    RegIrqFlags = 0x12,
    RegRxNbBytes = 0x13,
    RegPktSnrValue = 0x19,
    RegPktRssiValue = 0x1a,
    RegModemConfig1 = 0x1d,
    RegModemConfig2 = 0x1e,
    RegPreambleMsb = 0x20,
    RegPreambleLsb = 0x21,
    RegPayloadLength = 0x22,
    RegModemConfig3 = 0x26,
    RegFreqErrorMsb = 0x28,
    RegFreqErrorMid = 0x29,
    RegFreqErrorLsb = 0x2a,
    RegRssiWideband = 0x2c,
    RegDetectionOptimize = 0x31,
    RegInvertiq = 0x33,
    RegDetectionThreshold = 0x37,
    RegSyncWord = 0x39,
    RegInvertiq2 = 0x3b,
    RegDioMapping1 = 0x40,
    RegVersion = 0x42,
    RegPaDac = 0x4d,
}

// modes
enum Mode {
    ModeLongRangeMode = 0x80,
    ModeSleep = 0x00,
    ModeStdby = 0x01,
    ModeTx = 0x03,
    ModeRxContinuous = 0x05,
    ModeRxSingle = 0x06,
}

// Irq masks
enum Irq {
    IrqTxDoneMask = 0x08,
    IrqPayloadCrcErrorMask = 0x20,
    IrqRxDoneMask = 0x40,
}

// Other config
const PA_BOOST: u8 = 0x80;
const MAX_PKT_LENGTH: u8 = 255;

// The modem
pub struct Radio<'a> {
    spi: &'a dyn SpiMasterDevice,
    spi_buf: TakeCell<'static, [u8]>,
    spi_rx: TakeCell<'static, [u8]>,
    spi_tx: TakeCell<'static, [u8]>,
    spi_busy: Cell<bool>,
    //Pins
    //cs_pin: &'a dyn gpio::Pin,
    reset_pin: &'a dyn gpio::Pin,
    //irq_pin: &'a dyn gpio::InterruptPin,
    //State params
    sleep_pending: Cell<bool>,
    wake_pending: Cell<bool>,
    interrupt_handling: Cell<bool>,
    interrupt_pending: Cell<bool>,
    state: Cell<InternalState>,
    //LoRa params
    frequency: Cell<u64>,
    packet_index: Cell<u8>,
    implicit_header: Cell<bool>,
    tx_done: bool,
    rx_done: bool,
}

//
// SPI functions
// Implementing a public trait to cleanly pass to board. TO DO: move to kernel/hil
// Note: initialize, start, stop are specific to SPI
//

impl RadioConfig for Radio<'_> {
    fn initialize(
        &self,
        buf: &'static mut [u8],
        reg_write: &'static mut [u8],
        reg_read: &'static mut [u8],
    ) -> ReturnCode {
        if buf.len() < radio::MAX_BUF_SIZE || reg_read.len() != 2 || reg_write.len() != 2 {
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
        for _i in 0..10000 {
            self.reset_pin.clear();
        }
        self.reset_pin.set();
        ReturnCode::SUCCESS
    }

    fn start(&self) -> ReturnCode {
        self.set_frequency(self.frequency);
        // set base addresses
        self.register_write(RegMap::RegFifoTxBaseAddr, 0);
        self.register_write(RegMap::RegFifoRxBaseAddr, 0);
        // set Lna boost
        self.register_write(
            RegMap::RegLna,
            self.register_return(RegMap::RegLna) as u8 | 0x03,
        );
        // set auto Agc
        self.register_write(RegMap::RegModemConfig3, 0x04);
        // set output power to 17 dBm
        self.set_power(17, 0);

        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        // TODO fix
        // let version = self.register_return(RegMap::RegVersion) as u8;
        // if version != 0x12 {
        //  return ReturnCode::FAIL;
        // }

        self.sleep();
        if self.state.get() != InternalState::Sleep {
            return ReturnCode::EALREADY;
        }
        self.ready();

        ReturnCode::SUCCESS
    }

    //
    // State functions
    //
    fn ready(&self) -> ReturnCode {
        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeStdby as u8,
        );
        if self.state.get() == InternalState::Sleep {
            self.state.set(InternalState::Ready);
        } else {
            // Delay wakeup until the radio turns all the way off
            self.wake_pending.set(true);
        }
        ReturnCode::SUCCESS
    }

    fn sleep(&self) -> ReturnCode {
        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeSleep as u8,
        );
        self.sleep_pending.set(false);
        ReturnCode::SUCCESS
    }

    fn set_header_mode(&self, implicit: bool) {
        let cfg: u8;
        if implicit {
            cfg = self.register_return(RegMap::RegModemConfig1) as u8 | 0x01;
        } else {
            cfg = self.register_return(RegMap::RegModemConfig1) as u8 & 0xfe;
        }
        self.register_write(RegMap::RegModemConfig1, cfg);
        self.implicit_header.set(false);
    }

    fn is_on(&self) -> bool {
        if (self.register_return(RegMap::RegOpMode) & Mode::ModeTx as u8) == Mode::ModeTx as u8 {
            return true;
        }

        if (self.register_return(RegMap::RegIrqFlags) & Irq::IrqTxDoneMask as u8) != 0 {
            // clear Irq's
            self.register_write(RegMap::RegIrqFlags, Irq::IrqTxDoneMask as u8);
        }

        return false;
    }

    // Radio operations for handling LoRa IRQ
    fn handle_packet_irq(&self) -> ReturnCode {
        let irq_flags = self.register_return(RegMap::RegIrqFlags) as u8;

        // clear Irq's
        self.register_write(RegMap::RegIrqFlags, irq_flags);

        if (irq_flags & Irq::IrqPayloadCrcErrorMask as u8) == 0 {
            if (irq_flags & Irq::IrqRxDoneMask as u8) != 0 {
                // received a packet
                self.packet_index.set(0);

                // read packet length
                let packet_length;
                if self.implicit_header.get() == true {
                    packet_length = self.register_return(RegMap::RegPayloadLength) as u8;
                } else {
                    packet_length = self.register_return(RegMap::RegRxNbBytes) as u8;
                }

                // set Fifo address to current Rx address
                self.register_write(
                    RegMap::RegFifoAddrPtr,
                    self.register_return(RegMap::RegFifoRxCurrentAddr),
                );

                if self.rx_done && packet_length != 0 {
                    self.handle_interrupt();
                }

                // reset Fifo address
                self.register_write(RegMap::RegFifoAddrPtr, 0);
            } else if (irq_flags & Irq::IrqTxDoneMask as u8) != 0 {
                if self.tx_done {
                    self.handle_interrupt();
                }
            }
        }

        ReturnCode::SUCCESS
    }
}

//
// SPI functions
// Implementing a public trait to cleanly pass to board. TO DO: move to kernel/hil
// Note: initialize, start, stop are specific to SPI
//

impl RadioData for Radio<'_> {
    fn transmit(&self, implicit: bool) -> ReturnCode {
        if self.is_on() {
            return ReturnCode::FAIL;
        }
        self.ready();
        self.set_header_mode(implicit);
        self.register_write(RegMap::RegFifoAddrPtr, 0);
        self.register_write(RegMap::RegPayloadLength, 0);
        ReturnCode::SUCCESS
    }

    fn transmit_done(&self, asyn: bool) -> ReturnCode {
        if (asyn) && (self.tx_done) {
            self.register_write(RegMap::RegDioMapping1, 0x40);
        }
        // put in Tx mode
        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeTx as u8,
        );
        if !asyn {
            // wait for Tx done
            while self.register_return(RegMap::RegIrqFlags) & (Irq::IrqTxDoneMask as u8) == 0 {}
            // clear Irq's
            self.register_write(RegMap::RegIrqFlags, Irq::IrqTxDoneMask as u8);
        }
        ReturnCode::SUCCESS
    }

    fn receive(&self, size: u8) {
        self.register_write(RegMap::RegDioMapping1, 0x00);

        if size > 0 {
            self.set_header_mode(true);
            self.register_write(RegMap::RegPayloadLength, size & 0xff);
        } else {
            self.set_header_mode(false);
        }

        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeRxContinuous as u8,
        );
    }

    fn receive_done(&self, size: usize) -> u8 {
        let mut packet_length = 0 as u8;
        let irq_flags = self.register_return(RegMap::RegIrqFlags) as u8;

        if size > 0 {
            self.set_header_mode(true);
            self.register_write(RegMap::RegPayloadLength, size as u8 & 0xff);
        } else {
            self.set_header_mode(false);
        }

        // clear Irq's
        self.register_write(RegMap::RegIrqFlags, irq_flags);

        if (irq_flags & Irq::IrqRxDoneMask as u8 != 0)
            && (irq_flags & Irq::IrqPayloadCrcErrorMask as u8 != 0)
        {
            // received a packet
            self.packet_index.set(0);

            // read packet length
            if self.implicit_header.get() == true {
                packet_length = self.register_return(RegMap::RegPayloadLength) as u8;
            } else {
                packet_length = self.register_return(RegMap::RegRxNbBytes) as u8;
            }

            // set Fifo address to current Rx address
            self.register_write(
                RegMap::RegFifoAddrPtr,
                self.register_return(RegMap::RegFifoRxCurrentAddr),
            );

            self.ready();
        } else if self.register_return(RegMap::RegOpMode)
            != (Mode::ModeLongRangeMode as u8 | Mode::ModeRxSingle as u8)
        {
            // not currently in Rx mode

            // reset Fifo address
            self.register_write(RegMap::RegFifoAddrPtr, 0);

            // put in single Rx mode
            self.register_write(
                RegMap::RegOpMode,
                Mode::ModeLongRangeMode as u8 | Mode::ModeRxSingle as u8,
            );
        }

        return packet_length;
    }

    // manipulate FIFO

    fn read(&self) -> u8 {
        self.packet_index.set(self.packet_index.get() + 1);
        self.register_return(RegMap::RegFifo)
    }

    fn write(&self, buf: &[u8], mut size: u8) {
        let current_length = self.register_return(RegMap::RegPayloadLength) as u8;
        // check size
        if (current_length + size) > MAX_PKT_LENGTH {
            size = MAX_PKT_LENGTH - current_length;
        }
        // write data
        for i in 0..size {
            self.register_write(RegMap::RegFifo, buf[i as usize]);
        }
        // update length
        self.register_write(RegMap::RegPayloadLength, current_length + size);
    }
}

impl<'a> Radio<'a> {
    fn new(
        spi: &'a dyn SpiMasterDevice,
        //cs: &'a dyn gpio::Pin,
        reset: &'a dyn gpio::Pin,
        //irq: &'a dyn gpio::InterruptPin,
    ) -> Radio<'a> {
        Radio {
            spi: spi,
            spi_buf: TakeCell::empty(),
            spi_rx: TakeCell::empty(),
            spi_tx: TakeCell::empty(),
            spi_busy: Cell::new(false),
            //cs_pin: cs,
            reset_pin: reset,
            //irq_pin: irq,
            sleep_pending: Cell::new(false),
            wake_pending: Cell::new(false),
            interrupt_handling: Cell::new(false),
            interrupt_pending: Cell::new(false),
            state: Cell::new(InternalState::Sleep),
            frequency: Cell::new(0),
            packet_index: Cell::new(0),
            implicit_header: Cell::new(false),
            tx_done: false,
            rx_done: false,
        }
    }

    // SPI operations for handling LoRa IRQ
    fn handle_interrupt(&self) {
        if self.spi_busy.get() == false {
            // Need to disable reception until we've completed read
            if self.state.get() != InternalState::RxOn {
                self.interrupt_handling.set(true);
            }
        } else {
            self.interrupt_pending.set(true);
        }
    }

    fn register_write(&self, reg: RegMap, val: u8) -> ReturnCode {
        if self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none() {
            return ReturnCode::EBUSY;
        }
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | 0x80;
        wbuf[1] = val;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    fn register_read(&self, reg: RegMap) -> ReturnCode {
        if self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none() {
            return ReturnCode::EBUSY;
        }
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | 0x7f;
        wbuf[1] = 0;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    fn register_return(&self, reg: RegMap) -> u8 {
        if self.register_read(reg) == ReturnCode::SUCCESS {
            self.spi_rx.take().unwrap()[1] as u8
        } else {
            return 0;
        }
    }

    fn state_transition_write(&self, reg: RegMap, val: u8, state: InternalState) {
        self.state.set(state);
        self.register_write(reg, val);
    }

    fn state_transition_read(&self, reg: RegMap, state: InternalState) {
        self.state.set(state);
        self.register_read(reg);
    }
}

// TODO make transmit_done sync
impl SpiMasterClient for Radio<'_> {
    fn read_write_done(
        &self,
        _write: &'static mut [u8],
        _read: Option<&'static mut [u8]>,
        _len: usize,
    ) {
        self.spi_busy.set(false);
        let rbuf = _read.take().unwrap();
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
        if !handling && state == InternalState::RxOn {
            self.spi_buf.replace(_write);
            self.spi_rx.replace(rbuf);
            self.receive_done(_len);
            self.rx_done = true;
        } else if !handling && state == InternalState::TxOn {
            self.spi_buf.replace(_write);
            self.spi_rx.replace(rbuf);
            self.transmit_done(true);
            self.tx_done = true;
        } else {
            self.spi_rx.replace(rbuf);
            self.spi_tx.replace(_write);
        }

        if self.interrupt_pending.get() && state == InternalState::Sleep {
            self.interrupt_pending.set(false);
            self.handle_interrupt();
            return;
        }

        if state == InternalState::Ready {
            self.wake_pending.set(false);
            if self.sleep_pending.get() {
                self.sleep_pending.set(false);
            }
        }
    }
}
