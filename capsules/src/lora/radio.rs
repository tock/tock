
use core::cell::Cell;
use kernel::common::cells::{TakeCell};
use kernel::hil::gpio;
use kernel::hil::spi;
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lora as usize;

// registers
enum InternalState {
  RxOn,
  RxOff
}

enum RegMap {
  RegFifo                 = 0x00,
  RegOpMode               = 0x01,
  RegFrfMsb               = 0x06,
  RegFrfMid               = 0x07,
  RegFrfLsb               = 0x08,
  RegPaConfig             = 0x09,
  RegOcp                  = 0x0b,
  RegLna                  = 0x0c,
  RegFifoAddrPtr          = 0x0d,
  RegFifoTxBaseAddr       = 0x0e,
  RegFifoRxBaseAddr       = 0x0f,
  RegFifoRxCurrentAddr    = 0x10,
  RegIrqFlags             = 0x12,
  RegRxNbBytes            = 0x13,
  RegPktSnrValue          = 0x19,
  RegPktRssiValue         = 0x1a,
  RegModemConfig1         = 0x1d,
  RegModemConfig2         = 0x1e,
  RegPreambleMsb          = 0x20,
  RegPreambleLsb          = 0x21,
  RegPayloadLength        = 0x22,
  RegModemConfig3         = 0x26,
  RegFreqErrorMsb         = 0x28,
  RegFreqErrorMid         = 0x29,
  RegFreqErrorLsb         = 0x2a,
  RegRssiWideband         = 0x2c,
  RegDetectionOptimize    = 0x31,
  RegInvertiq             = 0x33,
  RegDetectionThreshold   = 0x37,
  RegSyncWord             = 0x39,
  RegInvertiq2            = 0x3b,
  RegDioMapping1         = 0x40,
  RegVersion              = 0x42,
  RegPaDac                = 0x4d
}

// modes
enum Mode {
  ModeLongRangeMode       = 0x80,
  ModeSleep               = 0x00,
  ModeStdby               = 0x01,
  ModeTx                  = 0x03,
  ModeRxContinuous        = 0x05,
  ModeRxSingle            = 0x06
}


// Irq masks
enum Irq {
  IrqTxDoneMask           = 0x08,
  IrqPayloadCrcErrorMask  = 0x20,
  IrqRxDoneMask           = 0x40
}

// Other config
const PaBoost: u8           = 0x80;
const MaxPktLength: u8      = 255;

// The modem
pub struct Radio<'a, S: spi::SpiMasterDevice> {
  spi: &'a S,
  spi_rx: TakeCell<'static, [u8]>,
  spi_tx: TakeCell<'static, [u8]>,
  //Pins
  //ss_pin: &'a dyn gpio::Pin,
  reset_pin: &'a dyn gpio::Pin,
  irq_pin: &'a dyn gpio::InterruptPin,
  //State
  spi_busy: Cell<bool>,
  interrupt_handling: Cell<bool>,
  interrupt_pending: Cell<bool>,
  state: Cell<InternalState>,
  //LoRa params
  frequency: u64,
  packet_index: u8,
  implicit_header: bool,
  tx_done: bool,
  rx_done: bool,
}

impl<S: spi::SpiMasterDevice> Radio<'a, S> {
    pub fn new(
        spi: &'a S,
        //ss: &'a dyn gpio::Pin,
        reset: &'a dyn gpio::Pin,
        irq: &'a dyn gpio::InterruptPin,
    ) -> Radio<'a, S> {
        Radio {
            spi: spi,
            spi_rx: TakeCell::empty(),
            spi_tx: TakeCell::empty(),
            spi_busy: Cell::new(false),
            //ss_pin: ss,
            reset_pin: reset,
            irq_pin: irq,
            state: Cell::new(InternalState::RxOff),
            interrupt_handling: Cell::new(false),
            interrupt_pending: Cell::new(false),
            frequency: 0,
            packet_index: 0,
            implicit_header: false,
            tx_done: false,
            rx_done: false,
        }
    }
  // SPI handle for LoRa interrupts
  fn handle_interrupt(&mut self) {
      // In most cases, the first thing the driver does on handling an interrupt is
      // read the IRQ status; this pushes most logic to the SPI handler.
      // The one exception is when the radio receives a packet; to prevent this
      // packet from being overwritten before reading it from the radio,
      // the driver needs to disable reception. This has to be done in the first
      // SPI operation.
      //if self.spi_busy.get() == false {
      //    if self.state.get() == InternalState::RxOn {
      //        // We've received a complete frame; need to disable
      //        // reception until we've read it out from RAM,
      //        // otherwise subsequent packets may corrupt it.
      //       self.state.set(InternalState::RxOff);
      //    } else {
      //        self.interrupt_handling.set(true);
      //    }
      //} else {
      //    self.interrupt_pending.set(true);
      //}
  }

  fn register_write(&self, reg: RegMap, val: u8) -> ReturnCode {
      //digitalWrite(ss_pin, Low);
      if self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none() {
          return ReturnCode::EBUSY;
      }
      let wbuf = self.spi_tx.take().unwrap();
      let rbuf = self.spi_rx.take().unwrap();
      wbuf[0] = (reg as u8) | 0x80;
      wbuf[1] = val;
      self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
      self.spi_busy.set(true);
      //digitalWrite(ss_pin, High);
      ReturnCode::SUCCESS
  }

  fn register_read(&self, reg: RegMap) -> ReturnCode {
      //digitalWrite(ss_pin, Low);
      if self.spi_busy.get() || self.spi_tx.is_none() || self.spi_rx.is_none() {
          return ReturnCode::EBUSY;
      }
      let wbuf = self.spi_tx.take().unwrap();
      let rbuf = self.spi_rx.take().unwrap();
      wbuf[0] = (reg as u8) | 0x7f;
      wbuf[1] = 0;
      self.spi.read_write_bytes(wbuf, Some(rbuf), 2);
      self.spi_busy.set(true);
      //digitalWrite(ss_pin, High);
      ReturnCode::SUCCESS
  }

  fn register_return(&self, reg: RegMap) -> u8 {
    if self.register_read(reg) == ReturnCode::SUCCESS {
      self.spi_rx.take().unwrap()[1] as u8
    } else {
      return 0;
    }
  }

  // Based on https://github.com/sandeepmistry/arduino-LoRa

  //
  // Config functions
  //
  fn idle(&self) {
    self.register_write(RegMap::RegOpMode,Mode::ModeLongRangeMode as u8 | Mode::ModeStdby as u8);
  }

  fn sleep(&self) {
    self.register_write(RegMap::RegOpMode,Mode::ModeLongRangeMode as u8 | Mode::ModeSleep as u8);
  }

  fn explicitHeaderMode(&mut self) {
    self.implicit_header = false; 
    self.register_write(RegMap::RegModemConfig1,self.register_return(RegMap::RegModemConfig1) as u8 & 0xfe);
  }

  fn implicitHeaderMode(&mut self) {
    self.implicit_header = true;
    self.register_write(RegMap::RegModemConfig1,self.register_return(RegMap::RegModemConfig1) as u8 | 0x01);
  }

  //
  // Packet functions
  //
  fn beginPacket(&mut self, implicitHeader: bool) -> ReturnCode {
    if self.isTransmitting() {
      return ReturnCode::SUCCESS;
    }
  
    self.idle();
  
    if implicitHeader {
      self.implicitHeaderMode();
    } else {
      self.explicitHeaderMode();
    }
  
    // reset Fifo address and paload length
    self.register_write(RegMap::RegFifoAddrPtr,0);
    self.register_write(RegMap::RegPayloadLength,0);
  
    ReturnCode::FAIL
  }

  fn endPacket(&mut self, asyn: bool) -> ReturnCode {
    if (asyn) && (self.tx_done) {
        self.register_write(RegMap::RegDioMapping1,0x40); // Dio0 => Txdone
    }
    // put in Tx mode
    self.register_write(RegMap::RegOpMode,Mode::ModeLongRangeMode as u8 | Mode::ModeTx as u8);
  
    if !asyn {
      // wait for Tx done
      while self.register_return(RegMap::RegIrqFlags) & (Irq::IrqTxDoneMask as u8) == 0 {
        //yield;
      }
      // clear Irq's
      self.register_write(RegMap::RegIrqFlags,Irq::IrqTxDoneMask as u8);
    }
  
    ReturnCode::SUCCESS
  }

  fn isTransmitting(&mut self) -> bool {
    if (self.register_return(RegMap::RegOpMode) & Mode::ModeTx as u8) == Mode::ModeTx as u8 {
      return true;
    }
  
    if (self.register_return(RegMap::RegIrqFlags) & Irq::IrqTxDoneMask as u8) != 0 {
      // clear Irq's
      self.register_write(RegMap::RegIrqFlags,Irq::IrqTxDoneMask as u8);
    }
  
    return false;
  }

  fn parsePacket(&mut self, size: u8) -> u8 {
    let mut packetLength = 0 as u8;
    let mut irqFlags = self.register_return(RegMap::RegIrqFlags) as u8;
  
    if size > 0 {
      self.implicitHeaderMode(); 
      self.register_write(RegMap::RegPayloadLength,size & 0xff);
    } else {
      self.explicitHeaderMode();
    }
  
    // clear Irq's
    self.register_write(RegMap::RegIrqFlags,irqFlags);
  
    if (irqFlags & Irq::IrqRxDoneMask as u8 != 0) && (irqFlags & Irq::IrqPayloadCrcErrorMask as u8 != 0) {
      // received a packet
      self.packet_index = 0;
  
      // read packet length
      if self.implicit_header {
        packetLength = self.register_return(RegMap::RegPayloadLength) as u8;
      } else {
        packetLength = self.register_return(RegMap::RegRxNbBytes) as u8;
      }
  
      // set Fifo address to current Rx address
      self.register_write(RegMap::RegFifoAddrPtr,self.register_return(RegMap::RegFifoRxCurrentAddr));
  
      self.idle();
  
    } else if self.register_return(RegMap::RegOpMode) != (Mode::ModeLongRangeMode as u8 | Mode::ModeRxSingle as u8) {
      // not currently in Rx mode
  
      // reset Fifo address
      self.register_write(RegMap::RegFifoAddrPtr,0);
  
      // put in single Rx mode
      self.register_write(RegMap::RegOpMode,Mode::ModeLongRangeMode as u8 | Mode::ModeRxSingle as u8);
    }
  
    return packetLength;
  }

  fn packetRssi(&mut self) -> u8 {
    let mut freq = self.frequency as f64;
    let mut rssiBase;
    if freq < 868E6 {
      rssiBase = 164;
    } else {
      rssiBase = 157;
    }
    self.register_return(RegMap::RegPktRssiValue) - rssiBase
  }
  
  fn packetSnr(&mut self) -> f32 {
    self.register_return(RegMap::RegPktSnrValue) as f32 * 0.25
  }
  
  fn packetFrequencyError(&mut self) -> i64 {
    let mut freqError;
    freqError = self.register_return(RegMap::RegFreqErrorMsb) & 0b111;
    freqError <<= 8;
    freqError += self.register_return(RegMap::RegFreqErrorMid);
    freqError <<= 8;
    freqError += self.register_return(RegMap::RegFreqErrorLsb);
  
    if self.register_return(RegMap::RegFreqErrorMsb) & 0b1000 != 0 { // Sign bit is on
       //freqError -= 524288; // B1000'0000'0000'0000'0000
    }
  
    let mut fXtal = 32E6 as f64; // Fxosc: crystal oscillator (Xtal) frequency (2.5. Chip Specification, p. 14)
    let mut fError = ((freqError << 24) as f64 / fXtal) * (self.getSignalBandwidth() as f64 / 500000.0); // p. 37
  
    return fError as i64;
  }

  //
  // FIFO functions
  //
  fn available(&mut self) -> bool {
    return self.register_return(RegMap::RegRxNbBytes) > self.packet_index;
  }

  fn peek(&mut self) -> u8 {
    if !self.available() {
      return 0;
    }
  
    // store current Fifo address
    let mut currentAddress = self.register_return(RegMap::RegFifoAddrPtr) as u8;
  
    // read
    let mut b = self.register_return(RegMap::RegFifo) as u8;
  
    // restore Fifo address
    self.register_write(RegMap::RegFifoAddrPtr,currentAddress);
  
    return b;
  }

  // read value at FIFO
  fn read(&mut self) -> u8 {
    if !self.available() {
      return 0;
    }
  
    self.packet_index += 1;
  
    self.register_return(RegMap::RegFifo)
  }
  
  // n-byte writes
  fn write(&mut self, buf: &[u8], mut size: u8) {
    let mut currentLength = self.register_return(RegMap::RegPayloadLength) as u8;
  
    // check size
    if (currentLength + size) > MaxPktLength {
      size = MaxPktLength - currentLength;
    }
  
    // write data
    for i in 0..size {
      self.register_write(RegMap::RegFifo,buf[i as usize]);
    }
  
    // update length
    self.register_write(RegMap::RegPayloadLength,currentLength + size);
  }

  //
  // Init functions
  //
  pub fn begin(&self, frequency: u64) -> ReturnCode {
    //self.ss_pin.set();
  
    if self.reset_pin.read() {
      return ReturnCode::SUCCESS;
    }
  
    // check version
    let mut version = self.register_return(RegMap::RegVersion) as u8;
    if version != 0x12 {
      return ReturnCode::SUCCESS;
    }
  
    self.sleep();
  
    self.setFrequency(frequency);
  
    // set base addresses
    self.register_write(RegMap::RegFifoTxBaseAddr,0);
    self.register_write(RegMap::RegFifoRxBaseAddr,0);
  
    // set Lna boost
    self.register_write(RegMap::RegLna,self.register_return(RegMap::RegLna) as u8 | 0x03);
  
    // set auto Agc
    self.register_write(RegMap::RegModemConfig3,0x04);
  
    // set output power to 17 dBm
    self.setPower(17, 0);
  
    self.idle();
  
    ReturnCode::FAIL
  }

  fn end(&mut self) {
    self.sleep();
  }

  //
  // Transmission functions
  //
  fn receive(&mut self, size: u8) {
  
    self.register_write(RegMap::RegDioMapping1,0x00); // Dio0 => Rxdone
  
    if size > 0 {
      self.implicitHeaderMode();
  
      self.register_write(RegMap::RegPayloadLength,size & 0xff);
    } else {
      self.explicitHeaderMode();
    }
  
    self.register_write(RegMap::RegOpMode,Mode::ModeLongRangeMode as u8 | Mode::ModeRxContinuous as u8);
  }

  fn setPower(&self, mut level: u8, outputPin: u8) {
    if outputPin == 0 {
      // Rfo
      if level < 0 {
        level = 0;
      } else if level > 14 {
        level = 14;
      }
  
      self.register_write(RegMap::RegPaConfig,0x70 | level);
    } else {
      // Pa Boost
      if level > 17 {
        if level > 20 {
          level = 20;
        }
  
        // subtract 3 from level, so 18 - 20 maps to 15 - 17
        level -= 3;
  
        // High Power +20 dBm Operation (Semtech Sx1276/77/78/79 5.4.3.)
        self.register_write(RegMap::RegPaDac,0x87);
        self.setOcp(140);
      } else {
        if level < 2 {
          level = 2;
        }
        //Default value PaHf/Lf or +17dBm
        self.register_write(RegMap::RegPaDac,0x84);
        self.setOcp(100);
      }
  
      self.register_write(RegMap::RegPaConfig,PaBoost | (level - 2));
    }
  }

  fn setFrequency(&self, frequency: u64) {
    //self.frequency = frequency;
  
    let mut frf = (frequency << 19) / 32000000;
  
    self.register_write(RegMap::RegFrfMsb,(frf >> 16) as u8);
    self.register_write(RegMap::RegFrfMid,(frf >> 8) as u8);
    self.register_write(RegMap::RegFrfLsb,(frf >> 0) as u8);
  }

  fn getSpreadingFactor(&mut self) -> u8 {
    self.register_return(RegMap::RegModemConfig2) >> 4
  }

  fn setSpreadingFactor(&mut self, mut sf: u8) {
    if sf < 6 {
      sf = 6;
    } else if sf > 12 {
      sf = 12;
    }
  
    if sf == 6 {
      self.register_write(RegMap::RegDetectionOptimize,0xc5);
      self.register_write(RegMap::RegDetectionThreshold,0x0c);
    } else {
      self.register_write(RegMap::RegDetectionOptimize,0xc3);
      self.register_write(RegMap::RegDetectionThreshold,0x0a);
    }
  
    self.register_write(RegMap::RegModemConfig2,(self.register_return(RegMap::RegModemConfig2) as u8 & 0x0f) | ((sf << 4) & 0xf0));
    self.setLdoFlag();
  }

  fn getSignalBandwidth(&mut self) -> f64 {
    let mut bw = (self.register_return(RegMap::RegModemConfig1) >> 4) as u8;
  
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

  fn setSignalBandwidth(&mut self, sbw: f64) {
    let mut bw: u8;
  
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
    } else /*if sbw <= 250E3*/ {
      bw = 9;
    }
  
    self.register_write(RegMap::RegModemConfig1,(self.register_return(RegMap::RegModemConfig1) & 0x0f) as u8 | (bw << 4));
    self.setLdoFlag();
  }

  fn setLdoFlag(&mut self) {
  // Section 4.1.1.5
    let mut symbolDuration = 1000 / ( self.getSignalBandwidth() / (1 << self.getSpreadingFactor()) as f64 )  as i64;
  
    // Section 4.1.1.6
    let mut ldoOn: bool = symbolDuration > 16;
  
    let mut config3 = self.register_return(RegMap::RegModemConfig3) as u8;
    if ldoOn {
      //config3 |= 0x1000;
    }
    self.register_write(RegMap::RegModemConfig3,config3);
  }

  fn setCodingRate4(&mut self, mut denominator: u8) {
    if denominator < 5 {
      denominator = 5;
    } else if denominator > 8 {
      denominator = 8;
    }
  
    let mut cr = denominator - 4 as u8;
  
    self.register_write(RegMap::RegModemConfig1,(self.register_return(RegMap::RegModemConfig1) as u8 & 0xf1) | (cr << 1));
  }

  fn setPreambleLength(&mut self, length: i64) {
    self.register_write(RegMap::RegPreambleMsb,(length >> 8) as u8);
    self.register_write(RegMap::RegPreambleLsb,(length >> 0) as u8);
  }

  fn setSyncWord(&mut self, sw: u8) {
    self.register_write(RegMap::RegSyncWord,sw);
  }

  fn enableCrc(&mut self) {
    self.register_write(RegMap::RegModemConfig2,self.register_return(RegMap::RegModemConfig2) as u8 | 0x04);
  }

  fn disableCrc(&mut self) {
    self.register_write(RegMap::RegModemConfig2,self.register_return(RegMap::RegModemConfig2) as u8 & 0xfb);
  }

  fn enableInvertIq(&mut self) {
    self.register_write(RegMap::RegInvertiq,0x66);
    self.register_write(RegMap::RegInvertiq2,0x19);
  }

  fn disableInvertIq(&mut self) {
    self.register_write(RegMap::RegInvertiq,0x27);
    self.register_write(RegMap::RegInvertiq2,0x1d);
  }

  fn setOcp(&self, mA: u8) {
    let mut ocpTrim = 27 as u8;
  
    if mA <= 120 {
      ocpTrim = (mA - 45) / 5;
    } else if mA <=240 {
      ocpTrim = (mA + 30) / 10;
    }
  
    self.register_write(RegMap::RegOcp,0x20 | (0x1F & ocpTrim));
  }

  fn random(&mut self) -> u8 {
    self.register_return(RegMap::RegRssiWideband)
  }

  // handle LoRa interrupt
  fn handleLoraIrq(&mut self) {
    let mut irqFlags = self.register_return(RegMap::RegIrqFlags) as u8;
  
    // clear Irq's
    self.register_write(RegMap::RegIrqFlags,irqFlags);
  
    if (irqFlags & Irq::IrqPayloadCrcErrorMask as u8) == 0 {
  
      if (irqFlags & Irq::IrqRxDoneMask as u8) != 0 {
        // received a packet
        self.packet_index = 0;
  
        // read packet length
        let mut packetLength;
        if self.implicit_header {
          packetLength = self.register_return(RegMap::RegPayloadLength) as u8;
        } else {
          packetLength = self.register_return(RegMap::RegRxNbBytes) as u8;
        }
  
        // set Fifo address to current Rx address
        self.register_write(RegMap::RegFifoAddrPtr,self.register_return(RegMap::RegFifoRxCurrentAddr));
  
        if self.rx_done && packetLength != 0 {
          self.handle_interrupt();
        }
  
        // reset Fifo address
        self.register_write(RegMap::RegFifoAddrPtr,0);
      }
      else if (irqFlags & Irq::IrqTxDoneMask as u8) != 0 {
        if self.tx_done {
          self.handle_interrupt();
        }
      }
    }
  }
}

//
// DRIVER
//

pub struct App {
  callback: Option<Callback>,
}

pub struct RadioDriver<'a, S: spi::SpiMasterDevice> {
  /// Underlying physical device
  device: &'a Radio<'a, S>,

  /// Grant of apps that use this radio driver.
  apps: Grant<App>,
}

impl Default for App {
    fn default() -> Self {
        App {
            callback: None,
        }
    }
}

impl<S: spi::SpiMasterDevice> RadioDriver<'a, S> {
    pub fn new(
      device: &'a Radio<'a, S>,
      grant: Grant<App>,
    ) -> RadioDriver<'a, S> {
      RadioDriver {
        device: device,
        apps: grant,
    }
  }
}


impl<S: spi::SpiMasterDevice> Driver for RadioDriver<'a, S> {
  /// Command interface.
  ///
  /// ### `command_num`
  ///
  /// - `0`: Return SUCCESS if this driver is included on the platform.
  /// - `1`: Start the radio.
  fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
    match command_num {
      0 => ReturnCode::SUCCESS,

      1 => self.device.begin(0),

      _ => ReturnCode::ENOSUPPORT,
    }
  }
}


