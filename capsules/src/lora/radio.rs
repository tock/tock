// For details on LoRa parameters, see Semtech SX1276/77/78/79 Datasheet.
// For understanding how the `Radio` capsule is architected around the `SpiMasterDevice`, see similiar implementation for the RF233.

// TODOs
// fix pub enum
// fix pub fn (Radio impl)

use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil::gpio;
use kernel::hil::radio;
use kernel::hil::spi;
use kernel::hil::spi::SpiMasterDevice;
use kernel::ReturnCode;

// not written to registers unlike Modes
// only help in programming for SPI layer ops
#[derive(Copy, Clone, PartialEq)]
enum InternalState {
    Start,
    //TxOn,
    TxOff,
    RxOn,
    RxOff,
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

//
// SPI functions
// Implementing a public trait to cleanly pass to board. TO DO: move to kernel/hil
// Note: initialize, start, stop are specific to SPI
//
pub trait RadioConfig {
    /// buf must be at least MAX_BUF_SIZE in length, and
    /// reg_read and reg_write must be 2 bytes.
    fn initialize(
        &self,
        spi_buf: &'static mut [u8],
        reg_write: &'static mut [u8],
        reg_read: &'static mut [u8],
    ) -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
}

impl<S: spi::SpiMasterDevice> RadioConfig for Radio<'a, S> {
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
        self.sleep_pending.set(false);

        if self.state.get() != InternalState::Start && self.state.get() != InternalState::Sleep {
            return ReturnCode::EALREADY;
        }

        if self.state.get() == InternalState::Sleep {
            self.state.set(InternalState::Ready);
        } else {
            // Delay wakeup until the radio turns all the way off
            self.wake_pending.set(true);
        }

        ReturnCode::SUCCESS
    }

    fn stop(&self) -> ReturnCode {
        if self.state.get() == InternalState::Sleep || self.state.get() == InternalState::TxOff {
            return ReturnCode::EALREADY;
        }

        match self.state.get() {
            InternalState::Ready => {
                self.state.set(InternalState::Start);
            }
            _ => {
                self.sleep_pending.set(true);
            }
        }

        ReturnCode::SUCCESS
    }
}

// The modem
pub struct Radio<'a, S: SpiMasterDevice> {
    spi: &'a S,
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

impl<S: SpiMasterDevice> Radio<'a, S> {
    pub fn new(
        spi: &'a S,
        //cs: &'a dyn gpio::Pin,
        reset: &'a dyn gpio::Pin,
        //irq: &'a dyn gpio::InterruptPin,
    ) -> Radio<'a, S> {
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
            state: Cell::new(InternalState::Start),
            frequency: Cell::new(0),
            packet_index: Cell::new(0),
            implicit_header: Cell::new(false),
            tx_done: false,
            rx_done: false,
        }
    }

    // SPI operations for handling LoRa IRQ
    pub fn handle_interrupt(&self) {
        if self.spi_busy.get() == false {
            if self.state.get() == InternalState::RxOn {
                // We've received a complete frame; need to disable
                // reception until we've read it out from RAM,
                // otherwise subsequent packets may corrupt it.
                self.state.set(InternalState::RxOff);
            } else {
                self.interrupt_handling.set(true);
            }
        } else {
            self.interrupt_pending.set(true);
        }
    }

    pub fn register_write(&self, reg: RegMap, val: u8) -> ReturnCode {
        //if self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none() {
        //    return ReturnCode::EBUSY;
        //}
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | 0x80;
        wbuf[1] = val;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    pub fn register_read(&self, reg: RegMap) -> ReturnCode {
        //if self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none() {
        //    return ReturnCode::EBUSY;
        //}
        let wbuf = self.spi_tx.take().unwrap();
        let rbuf = self.spi_rx.take().unwrap();
        wbuf[0] = (reg as u8) | 0x7f;
        wbuf[1] = 0;
        self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
        self.spi_busy.set(true);
        ReturnCode::SUCCESS
    }

    pub fn register_return(&self, reg: RegMap) -> u8 {
        if self.register_read(reg) == ReturnCode::SUCCESS {
            self.spi_rx.take().unwrap()[1] as u8
        } else {
            return 0;
        }
    }

    //
    // State functions
    //
    pub fn idle(&self) {
        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeStdby as u8,
        );
    }

    pub fn sleep(&self) -> ReturnCode {
        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeSleep as u8,
        );

        ReturnCode::SUCCESS
    }

    pub fn explicit_header_mode(&self) {
        self.implicit_header.set(false);
        self.register_write(
            RegMap::RegModemConfig1,
            self.register_return(RegMap::RegModemConfig1) as u8 & 0xfe,
        );
    }

    pub fn implicit_header_mode(&self) {
        self.implicit_header.set(true);
        self.register_write(
            RegMap::RegModemConfig1,
            self.register_return(RegMap::RegModemConfig1) as u8 | 0x01,
        );
    }

    //
    // Packet functions
    //
    pub fn begin_packet(&self, implicit_header: bool) -> ReturnCode {
        if self.is_transmitting() {
            return ReturnCode::FAIL;
        }

        self.idle();

        if implicit_header {
            self.implicit_header_mode();
        } else {
            self.explicit_header_mode();
        }

        // reset Fifo address and paload length
        self.register_write(RegMap::RegFifoAddrPtr, 0);
        self.register_write(RegMap::RegPayloadLength, 0);

        ReturnCode::SUCCESS
    }

    pub fn end_packet(&self, asyn: bool) -> ReturnCode {
        if (asyn) && (self.tx_done) {
            self.register_write(RegMap::RegDioMapping1, 0x40); // Dio0 => Txdone
        }
        // put in Tx mode
        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeTx as u8,
        );

        if !asyn {
            // wait for Tx done
            while self.register_return(RegMap::RegIrqFlags) & (Irq::IrqTxDoneMask as u8) == 0 {
                //yield;
            }
            // clear Irq's
            self.register_write(RegMap::RegIrqFlags, Irq::IrqTxDoneMask as u8);
        }

        ReturnCode::SUCCESS
    }

    pub fn is_transmitting(&self) -> bool {
        if (self.register_return(RegMap::RegOpMode) & Mode::ModeTx as u8) == Mode::ModeTx as u8 {
            return true;
        }

        if (self.register_return(RegMap::RegIrqFlags) & Irq::IrqTxDoneMask as u8) != 0 {
            // clear Irq's
            self.register_write(RegMap::RegIrqFlags, Irq::IrqTxDoneMask as u8);
        }

        return false;
    }

    pub fn parse_packet(&self, size: u8) -> u8 {
        let mut packet_length = 0 as u8;
        let irq_flags = self.register_return(RegMap::RegIrqFlags) as u8;

        if size > 0 {
            self.implicit_header_mode();
            self.register_write(RegMap::RegPayloadLength, size & 0xff);
        } else {
            self.explicit_header_mode();
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

            self.idle();
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

    pub fn packet_rssi(&self) -> u8 {
        let freq = self.frequency.get() as f64;
        let rssi_base;
        if freq < 868E6 {
            rssi_base = 164;
        } else {
            rssi_base = 157;
        }
        self.register_return(RegMap::RegPktRssiValue) - rssi_base
    }

    pub fn packet_snr(&self) -> f32 {
        self.register_return(RegMap::RegPktSnrValue) as f32 * 0.25
    }

    #[allow(arithmetic_overflow)]
    pub fn packet_frequency_error(&self) -> i64 {
        let mut freq_error;
        freq_error = self.register_return(RegMap::RegFreqErrorMsb) & 0b111;
        freq_error <<= 8;
        freq_error += self.register_return(RegMap::RegFreqErrorMid);
        freq_error <<= 8;
        freq_error += self.register_return(RegMap::RegFreqErrorLsb);

        if self.register_return(RegMap::RegFreqErrorMsb) & 0b1000 != 0 { // Sign bit is on
             //freq_error 24288; // B1000'0000'0000'0000'0000
        }

        let f_xtal = 32E6 as f64; // Fxosc: crystal oscillator (Xtal) frequency (2.5. Chip Specification, p. 14)
        let f_error =
            ((freq_error << 24) as f64 / f_xtal) * (self.get_signal_bandwidth() as f64 / 500000.0); // p. 37

        return f_error as i64;
    }

    //
    // FIFO functions
    //
    pub fn available(&self) -> bool {
        return self.register_return(RegMap::RegRxNbBytes) > self.packet_index.get();
    }

    pub fn peek(&self) -> u8 {
        if !self.available() {
            return 0;
        }

        // store current Fifo address
        let current_address = self.register_return(RegMap::RegFifoAddrPtr) as u8;

        // read
        let b = self.register_return(RegMap::RegFifo) as u8;

        // restore Fifo address
        self.register_write(RegMap::RegFifoAddrPtr, current_address);

        return b;
    }

    // read value at FIFO
    pub fn read(&self) -> u8 {
        if !self.available() {
            return 0;
        }

        self.packet_index.set(self.packet_index.get() + 1);

        self.register_return(RegMap::RegFifo)
    }

    // n-byte writes
    pub fn write(&self, buf: &[u8], mut size: u8) {
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

    //
    // Init functions
    //
    pub fn begin(&self, frequency: u64) -> ReturnCode {
        //if self.reset_pin.read() {
        //  return ReturnCode::FAIL;
        //}
        //
        //let version = self.register_return(RegMap::RegVersion) as u8;
        //if version != 0x12 {
        //  return ReturnCode::FAIL;
        //}

        self.sleep();

        self.set_frequency(frequency);

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

        self.idle();

        ReturnCode::SUCCESS
    }

    pub fn end(&self) -> ReturnCode {
        self.sleep()
    }

    //
    // Transmission functions
    //
    pub fn receive(&self, size: u8) {
        self.register_write(RegMap::RegDioMapping1, 0x00); // Dio0 => Rxdone

        if size > 0 {
            self.implicit_header_mode();

            self.register_write(RegMap::RegPayloadLength, size & 0xff);
        } else {
            self.explicit_header_mode();
        }

        self.register_write(
            RegMap::RegOpMode,
            Mode::ModeLongRangeMode as u8 | Mode::ModeRxContinuous as u8,
        );
    }

    pub fn set_power(&self, mut level: i8, output_pin: u8) {
        if output_pin == 0 {
            // Rfo
            if level < 0 {
                level = 0;
            } else if level > 14 {
                level = 14;
            }

            self.register_write(RegMap::RegPaConfig, 0x70 | level as u8);
        } else {
            // Pa Boost
            if level > 17 {
                if level > 20 {
                    level = 20;
                }

                // subtract 3 from level, so 18 - 20 maps to 15 - 17
                level -= 3;

                // High Power +20 dBm Operation (Semtech Sx1276/77/78/79 5.4.3.)
                self.register_write(RegMap::RegPaDac, 0x87);
                self.set_ocp(140);
            } else {
                if level < 2 {
                    level = 2;
                }
                //Default value PaHf/Lf or +17dBm
                self.register_write(RegMap::RegPaDac, 0x84);
                self.set_ocp(100);
            }

            self.register_write(RegMap::RegPaConfig, PA_BOOST | (level as u8 - 2));
        }
    }

    pub fn set_frequency(&self, frequency: u64) {
        self.frequency.set(frequency);

        let frf = (frequency << 19) / 32000000;

        self.register_write(RegMap::RegFrfMsb, (frf >> 16) as u8);
        self.register_write(RegMap::RegFrfMid, (frf >> 8) as u8);
        self.register_write(RegMap::RegFrfLsb, (frf >> 0) as u8);
    }

    pub fn get_spreading_factor(&self) -> u8 {
        self.register_return(RegMap::RegModemConfig2) >> 4
    }

    pub fn set_spreading_factor(&self, mut sf: u8) {
        if sf < 6 {
            sf = 6;
        } else if sf > 12 {
            sf = 12;
        }

        if sf == 6 {
            self.register_write(RegMap::RegDetectionOptimize, 0xc5);
            self.register_write(RegMap::RegDetectionThreshold, 0x0c);
        } else {
            self.register_write(RegMap::RegDetectionOptimize, 0xc3);
            self.register_write(RegMap::RegDetectionThreshold, 0x0a);
        }

        self.register_write(
            RegMap::RegModemConfig2,
            (self.register_return(RegMap::RegModemConfig2) as u8 & 0x0f) | ((sf << 4) & 0xf0),
        );
        self.set_ldo_flag();
    }

    pub fn get_signal_bandwidth(&self) -> f64 {
        let bw = (self.register_return(RegMap::RegModemConfig1) >> 4) as u8;

        match bw {
            0 => return 7.8E3,
            1 => return 10.4E3,
            2 => return 15.6E3,
            3 => return 20.8E3,
            4 => return 31.25E3,
            5 => return 41.7E3,
            6 => return 62.5E3,
            7 => return 125E3,
            8 => return 250E3,
            _ => return 500E3,
        }
    }

    pub fn set_signal_bandwidth(&self, sbw: f64) {
        let bw: u8;

        if sbw <= 7.8E3 {
            bw = 0;
        } else if sbw <= 10.4E3 {
            bw = 1;
        } else if sbw <= 15.6E3 {
            bw = 2;
        } else if sbw <= 20.8E3 {
            bw = 3;
        } else if sbw <= 31.25E3 {
            bw = 4;
        } else if sbw <= 41.7E3 {
            bw = 5;
        } else if sbw <= 62.5E3 {
            bw = 6;
        } else if sbw <= 125E3 {
            bw = 7;
        } else if sbw <= 250E3 {
            bw = 8;
        } else
        /*if sbw <= 250E3*/
        {
            bw = 9;
        }

        self.register_write(
            RegMap::RegModemConfig1,
            (self.register_return(RegMap::RegModemConfig1) & 0x0f) as u8 | (bw << 4),
        );
        self.set_ldo_flag();
    }

    pub fn set_ldo_flag(&self) {
        // Section 4.1.1.5
        let symbol_duration =
            1000 / (self.get_signal_bandwidth() / (1 << self.get_spreading_factor()) as f64) as i64;

        // Section 4.1.1.6
        let ldo_on: bool = symbol_duration > 16;

        let config3: u8;
        if ldo_on {
            config3 = self.register_return(RegMap::RegModemConfig3) | 0b1000;
        } else {
            config3 = self.register_return(RegMap::RegModemConfig3);
        }
        self.register_write(RegMap::RegModemConfig3, config3);
    }

    pub fn set_coding_rate4(&self, mut denominator: u8) {
        if denominator < 5 {
            denominator = 5;
        } else if denominator > 8 {
            denominator = 8;
        }

        let cr = denominator - 4 as u8;

        self.register_write(
            RegMap::RegModemConfig1,
            (self.register_return(RegMap::RegModemConfig1) as u8 & 0xf1) | (cr << 1),
        );
    }

    pub fn set_preamble_length(&self, length: i64) {
        self.register_write(RegMap::RegPreambleMsb, (length >> 8) as u8);
        self.register_write(RegMap::RegPreambleLsb, (length >> 0) as u8);
    }

    pub fn set_sync_word(&self, sw: u8) {
        self.register_write(RegMap::RegSyncWord, sw);
    }

    pub fn enable_crc(&self) {
        self.register_write(
            RegMap::RegModemConfig2,
            self.register_return(RegMap::RegModemConfig2) as u8 | 0x04,
        );
    }

    pub fn disable_crc(&self) {
        self.register_write(
            RegMap::RegModemConfig2,
            self.register_return(RegMap::RegModemConfig2) as u8 & 0xfb,
        );
    }

    pub fn enable_invert_iq(&self) {
        self.register_write(RegMap::RegInvertiq, 0x66);
        self.register_write(RegMap::RegInvertiq2, 0x19);
    }

    pub fn disable_invert_iq(&self) {
        self.register_write(RegMap::RegInvertiq, 0x27);
        self.register_write(RegMap::RegInvertiq2, 0x1d);
    }

    pub fn set_ocp(&self, ma: u8) {
        let mut ocp_trim = 27 as u8;

        if ma <= 120 {
            ocp_trim = (ma - 45) / 5;
        } else if ma <= 240 {
            ocp_trim = (ma + 30) / 10;
        }

        self.register_write(RegMap::RegOcp, 0x20 | (0x1F & ocp_trim));
    }

    pub fn random(&self) -> u8 {
        self.register_return(RegMap::RegRssiWideband)
    }

    // Radio operations for handling LoRa IRQ
    pub fn handle_lora_irq(&self) -> ReturnCode {
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
