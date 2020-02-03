
// Based on https://github.com/sandeepmistry/arduino-LoRa

use base;
use spi;

impl<S: spi::SpiMasterDevice> RF233<'a, S> {
  //
  // Config functions
  //
  fn idle() {
    self.register_write(RegOpMode,ModeLongRangeMode | ModeStdby);
  }

  fn sleep() {
    self.register_write(RegOpMode,ModeLongRangeMode | ModeSleep);
  }

  fn explicitHeaderMode() {
    implicit_header = 0; 
    self.register_write(RegModemConfig_1,self.register_read(RegModemConfig_1) & 0xfe);
  }

  fn implicitHeaderMode() {
    implicit_header = 1;
    self.register_write(RegModemConfig_1,self.register_read(RegModemConfig_1) | 0x01);
  }

  //
  // Packet functions
  //
  fn beginPacket(implicitHeader: i32) {
    if isTransmitting() {
      return 0;
    }

    idle();

    if implicitHeader {
      implicitHeaderMode();
    } else {
      explicitHeaderMode();
    }

    // reset Fifo address and paload length
    self.register_write(RegFifoAddrPtr,0);
    self.register_write(RegPayloadLength,0);

    return 1;
  }

  fn endPacket(bool async) {
    if (async) && (_onTxDone)
        self.register_write(RegDioMapping_1,0x40); // Dio0 => Txdone

    // put in Tx mode
    self.register_write(RegOpMode,ModeLongRangeMode | ModeTx);

    if !async {
      // wait for Tx done
      while ((self.register_read(RegIrqFlags) & IrqTxDoneMask) == 0) {
        yield();
      }
      // clear Irq's
      self.register_write(RegIrqFlags,IrqTxDoneMask);
    }

    return 1;
  }

  fn isTransmitting() {
    if (self.register_read(RegOpMode) & ModeTx) == ModeTx {
      return true;
    }

    if self.register_read(RegIrqFlags) & IrqTxDoneMask {
      // clear Irq's
      self.register_write(RegIrqFlags,IrqTxDoneMask);
    }

    return false;
  }

  fn parsePacket(size: i32) {
    let packetLength: i32 = 0;
    let irqFlags: i32 = self.register_read(RegIrqFlags);

    if size > 0 {
      implicitHeaderMode(); 
      self.register_write(RegPayloadLength,size & 0xff);
    } else {
      explicitHeaderMode();
    }

    // clear Irq's
    self.register_write(RegIrqFlags,irqFlags);

    if (irqFlags & IrqRxDoneMask) && (irqFlags & IrqPayloadCrcErrorMask) == 0 {
      // received a packet
      packet_index = 0;

      // read packet length
      if implicit_header {
        packetLength = self.register_read(RegPayloadLength);
      } else {
        packetLength = self.register_read(RegRxNbBytes);
      }

      // set Fifo address to current Rx address
      self.register_write(RegFifoAddrPtr,self.register_read(RegFifoRxCurrentAddr));

      idle();

    } else if self.register_read(RegOpMode) != (ModeLongRangeMode | ModeRxSingle) {
      // not currently in Rx mode

      // reset Fifo address
      self.register_write(RegFifoAddrPtr,0);

      // put in single Rx mode
      self.register_write(RegOpMode,ModeLongRangeMode | ModeRxSingle);
    }

    return packetLength;
  }

  fn packetRssi() {
    return (self.register_read(RegPktRssiValue) - (_frequency < 868E6 ? 164 : 157));
  }

  fn packetSnr() {
    return ((u8)self.register_read(RegPktSnrValue)) * 0.25;
  }

  fn packetFrequencyError() {
    let freqError: i32 = 0;
    freqError = static_cast<u32>(self.register_read(RegFreqErrorMsb) & B111);
    freqError <<= 8L;
    freqError += static_cast<u32>(self.register_read(RegFreqErrorMid));
    freqError <<= 8L;
    freqError += static_cast<u32>(self.register_read(RegFreqErrorLsb));

    if self.register_read(RegFreqErrorMsb) & B1000 { // Sign bit is on
       freqError -= 524288; // B1000'0000'0000'0000'0000
    }

    const float fXtal = 32E6; // Fxosc: crystal oscillator (Xtal) frequency (2.5. Chip Specification, p. 14)
    const float fError = ((static_cast<float>(freqError) * (1L << 24)) / fXtal) * (getSignalBandwidth() / 500000.0f); // p. 37

    return static_cast<long>(fError);
  }

  //
  // FIFO functions
  //
  fn available() {
    return (self.register_read(RegRxNbBytes) - packet_index);
  }

  fn peek() {
    if !available() {
      return -1;
    }

    // store current Fifo address
    let currentAddress: i32 = self.register_read(RegFifoAddrPtr);

    // read
    let b: u8 = self.register_read(RegFifo);

    // restore Fifo address
    self.register_write(RegFifoAddrPtr,currentAddress);

    return b;
  }

  fn read() {
    if !available() {
      return -1;
    }

    packet_index++;

    return self.register_read(RegFifo);
  }

  fn write(byte: u8) {
    return write(&byte, sizeof(byte));
  }

  fn write(const size: u8 *buffer, size) {
    let currentLength: i32 = self.register_read(RegPayloadLength);

    // check size
    if (currentLength + size) > MaxPktLength {
      size = MaxPktLength - currentLength;
    }

    // write data
    for (size_t i = 0; i < size; i++) {
      self.register_write(RegFifo,buffer[i]);
    }

    // update length
    self.register_write(RegPayloadLength,currentLength + size);

    return size;
  }

  //
  // Init functions
  //
  fn begin(frequency: i64) {
    //pinMode(_ss, Output);
    //digitalWrite(_ss, High);

    //if _reset != -1 {
    //  pinMode(_reset, Output);
    //  digitalWrite(_reset, Low);
    //  delay(10);
    //  digitalWrite(_reset, High);
    //  delay(10);
    //}

    // check version
    let version: u8 = self.register_read(RegVersion);
    if version != 0x12 {
      return 0;
    }

    sleep();

    setFrequency(frequency);

    // set base addresses
    self.register_write(RegFifoTxBaseAddr,0);
    self.register_write(RegFifoRxBaseAddr,0);

    // set Lna boost
    self.register_write(RegLna,self.register_read(RegLna) | 0x03);

    // set auto Agc
    self.register_write(RegModemConfig_3,0x04);

    // set output power to 17 dBm
    setTxPower(17);

    idle();

    return 1;
  }

  fn end() {
    sleep();
  }

  //
  // Transmission functions
  //
  fn receive(size: i32) {

    self.register_write(RegDioMapping_1,0x00); // Dio0 => Rxdone

    if size > 0 {
      implicitHeaderMode();

      self.register_write(RegPayloadLength,size & 0xff);
    } else {
      explicitHeaderMode();
    }

    self.register_write(RegOpMode,ModeLongRangeMode | ModeRxContinuous);
  }

  fn setTxPower(level: i32, int outputPin) {
    if PaOutputRfoPin == outputPin {
      // Rfo
      if level < 0 {
        level = 0;
      } else if level > 14 {
        level = 14;
      }

      self.register_write(RegPaConfig,0x70 | level);
    } else {
      // Pa Boost
      if level > 17 {
        if level > 20 {
          level = 20;
        }

        // subtract 3 from level, so 18 - 20 maps to 15 - 17
        level -= 3;

        // High Power +20 dBm Operation (Semtech Sx1276/77/78/79 5.4.3.)
        self.register_write(RegPaDac,0x87);
        setOcp(140);
      } else {
        if level < 2 {
          level = 2;
        }
        //Default value PaHf/Lf or +17dBm
        self.register_write(RegPaDac,0x84);
        setOcp(100);
      }

      self.register_write(RegPaConfig,PaBoost | (level - 2));
    }
  }

  fn setFrequency(frequency: i64) {
    _frequency = frequency;

    let frf: u64 = ((u64)frequency << 19) / 32000000;

    self.register_write(RegFrfMsb,(u8)(frf >> 16));
    self.register_write(RegFrfMid,(u8)(frf >> 8));
    self.register_write(RegFrfLsb,(u8)(frf >> 0));
  }

  fn getSpreadingFactor() {
    return self.register_read(RegModemConfig_2) >> 4;
  }

  fn setSpreadingFactor(sf: i32) {
    if sf < 6 {
      sf = 6;
    } else if sf > 12 {
      sf = 12;
    }

    if sf == 6 {
      self.register_write(RegDetectionOptimize,0xc5);
      self.register_write(RegDetectionThreshold,0x0c);
    } else {
      self.register_write(RegDetectionOptimize,0xc3);
      self.register_write(RegDetectionThreshold,0x0a);
    }

    self.register_write(RegModemConfig_2,(self.register_read(RegModemConfig_2) & 0x0f) | ((sf << 4) & 0xf0));
    setLdoFlag();
  }

  fn getSignalBandwidth() {
    byte bw = (self.register_read(RegModemConfig_1) >> 4);

    match bw {
      0 => return 7.8E3;
      1 => return 10.4E3;
      2 => return 15.6E3;
      3 => return 20.8E3;
      4 => return 31.25E3;
      5 => return 41.7E3;
      6 => return 62.5E3;
      7 => return 125E3;
      8 => return 250E3;
      9 => return 500E3;
    }

    return -1;
  }

  fn setSignalBandwidth(sbw: i64) {
    bw: i32;

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

    self.register_write(RegModemConfig_1,(self.register_read(RegModemConfig_1) & 0x0f) | (bw << 4));
    setLdoFlag();
  }

  fn setLdoFlag() {
    // Section 4.1.1.5
    let symbolDuration: i64 = 1000 / ( getSignalBandwidth() / (1L << getSpreadingFactor()) ) ;

    // Section 4.1.1.6
    ldoOn: bool = symbolDuration > 16;

    let config3: u8 = self.register_read(RegModemConfig_3);
    bitWrite(config3, 3, ldoOn);
    self.register_write(RegModemConfig_3,config3);
  }

  void setCodingRate4(denominator: i32) {
    if denominator < 5 {
      denominator = 5;
    } else if denominator > 8 {
      denominator = 8;
    }

    let cr: i32 = denominator - 4;

    self.register_write(RegModemConfig_1,(self.register_read(RegModemConfig_1) & 0xf1) | (cr << 1));
  }

  fn setPreambleLength(length: i64) {
    self.register_write(RegPreambleMsb,(u8)(length >> 8));
    self.register_write(RegPreambleLsb,(u8)(length >> 0));
  }

  fn setSyncWord(sw: i32) {
    self.register_write(RegSyncWord,sw);
  }

  fn enableCrc() {
    self.register_write(RegModemConfig_2,self.register_read(RegModemConfig_2) | 0x04);
  }

  fn disableCrc() {
    self.register_write(RegModemConfig_2,self.register_read(RegModemConfig_2) & 0xfb);
  }

  fn enableInvertIq() {
    self.register_write(RegInvertiq,0x66);
    self.register_write(RegInvertiq2,0x19);
  }

  fn disableInvertIq() {
    self.register_write(RegInvertiq,0x27);
    self.register_write(RegInvertiq2,0x1d);
  }

  fn setOcp(mA: u8) {
    let ocpTrim: u8 = 27;

    if mA <= 120 {
      ocpTrim = (mA - 45) / 5;
    } else if mA <=240 {
      ocpTrim = (mA + 30) / 10;
    }

    self.register_write(RegOcp,0x20 | (0x1F & ocpTrim));
  }

  fn random() {
    return self.register_read(RegRssiWideband);
  }

  void dumpRegisters(Stream& out) {
    for (let i: i32 = 0; i < 128; i++) {
      println!("0x");
      println!(i, Hex);
      println!(": 0x");
      println!ln(i).read(Hex);
    }
  }

//
// Interrupt functions
//
//  void handleDio0Rise() {
//    let irqFlags: i32 = self.register_read(RegIrqFlags);
//
//    // clear Irq's
//    self.register_write(RegIrqFlags,irqFlags);
//
//    if (irqFlags & IrqPayloadCrcErrorMask) == 0 {
//
//      if (irqFlags & IrqRxDoneMask) != 0 {
//        // received a packet
//        packet_index = 0;
//
//        // read packet length
//        let packetLength: i32 = implicit_header ? self.register_read(RegPayloadLength) : self.register_read(RegRxNbBytes);
//
//        // set Fifo address to current Rx address
//        self.register_write(RegFifoAddrPtr,self.register_read(RegFifoRxCurrentAddr));
//
//        if _onReceive {
//          _onReceive(packetLength);
//        }
//
//        // reset Fifo address
//        self.register_write(RegFifoAddrPtr,0);
//      }
//      else if (irqFlags & IrqTxDoneMask) != 0 {
//        if _onTxDone {
//          _onTxDone();
//        }
//      }
//    }
//  }
//
//  void onDio0Rise() {
//    LoRa.handleDio0Rise();
//  }
//
//  void onReceive(void(*callback)(int)) {
//    _onReceive = callback;
//
//    if callback {
//      pinMode(_dio0, Input);
//      Spi.usingInterrupt(digitalPinToInterrupt(_dio0));
//      attachInterrupt(digitalPinToInterrupt(_dio0), onDio0Rise, Rising);
//    } else {
//      detachInterrupt(digitalPinToInterrupt(_dio0));
//      Spi.notUsingInterrupt(digitalPinToInterrupt(_dio0));
//    }
//  }
//
//  void onTxDone(void(*callback)()) {
//    _onTxDone = callback;
//
//    if callback {
//      pinMode(_dio0, Input);
//      Spi.usingInterrupt(digitalPinToInterrupt(_dio0));
//      attachInterrupt(digitalPinToInterrupt(_dio0), onDio0Rise, Rising);
//    } else {
//      detachInterrupt(digitalPinToInterrupt(_dio0));
//      Spi.notUsingInterrupt(digitalPinToInterrupt(_dio0));
//    }
//  }
//}
