Kernel Serial Peripheral Interface (SPI) HIL
============================================

**TRD:** 104 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Pat Pannuto <br/>
**Draft-Created:** Feb 13, 2017<br/>
**Draft-Modified:** Feb 13, 2017<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL) for
Serial Peripheral Interface (SPI) bus in the Tock operating system kernel.  It
describes the Rust traits and other definitions for this service as well as the
reasoning behind them. This document is in full compliance with [TRD1].

1. Introduction
-------------------------------

The Serial Peripheral Interface (SPI) bus, the Inter-Integrated Circuit (I2C)
bus, and Universal Asynchronous Reciever Transmitters (UART) are the most common
means used to connect chips in modern embedded systems. Defining features of the
SPI bus are a shared synchronous clock (SCLK) and full-duplex interface
(MOSI/MISO), and a per-chip point-to-point chip select (CS) signal.

SPI is designed as a star topology, with a central _master_ that has one or more
_slave_ devices attached. Less commonly, SPI may also be configured with a
daisy-chain of peripheral sharing the same chip select (CS) signal, but this
topology difference does not affect this interface design.  The SPI master is
responsible for generating the bus clock (SCLK). As a full duplex bus, the
master is always sending data to the slave on the Master Out Slave In (MOSI)
net and receiving data on the Master In Slave Out (MISO) net. Finally, the SPI
master is responsible for selecting active slave devices using dedicated chip
select (CS) lines that run to each slave device.

The SPI bus has several possible configurations for how data is transmitted,
which the HIL exposes as enumerations in hil::spi as follows:

   * `kernel::hil::spi::DataOrder`: Whether data is sent MSB or LSB first
   * `kernel::hil::spi::ClockPolarity`: Whether SCLK idle low or high
   * `kernel::hil::spi::ClockPhase`: Sample on the rising or falling edge of SCLK

The SPI HIL provides traits for both SPI masters and SPI slaves.

   * `kernel::hil::spi::SpiMasterClient`: callback trait
   * `kernel::hil::spi::SpiMaster`:
   * `kernel::hil::spi::SpiMasterDevice`:

The rest of this document discusses each in turn.



5. Example Implementation
---------------------------------

6. Authors' Address
---------------------------------

email - pat.pannuto@gmail.com
