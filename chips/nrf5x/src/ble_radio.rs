// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Radio driver, Bluetooth Low Energy, NRF52
//!
//! The generic radio configuration i.e., not specific to Bluetooth are
//! functions and similar which do not start with `ble`. Moreover, Bluetooth
//! Low Energy specific radio configuration starts with `ble`
//!
//! For more readability the Bluetooth specific configuration may be moved to
//! separate trait
//!
//! ### Author
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: July 18, 2017
//!
//! ### Packet Configuration
//! ```txt
//! +----------+------+--------+----+--------+----+---------+-----+
//! | Preamble | Base | Prefix | S0 | Length | S1 | Payload | CRC |
//! +----------+------+--------+----+--------+----+---------+-----+
//! ```
//!
//! * Preamble - 1 byte
//!
//! * Base and prefix forms together the access address
//!
//! * S0, an optional parameter that is configured to indicate how many bytes of
//!   the payload is the PDU Type. Configured as 1 byte!
//!
//! * Length, an optional parameter that is configured to indicate how many bits
//!   of the payload is the length field. Configured as 8 bits!
//!
//! * S1, Not used
//!
//! * Payload - 2 to 255 bytes
//!
//! * CRC - 3 bytes

use core::cell::Cell;
use kernel::ErrorCode;
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use nrf5x_unsafe::ble_radio::{
    CrcConfiguration, Event, Frequency, Interrupt, Mode, PacketConfiguration0,
    PacketConfiguration1, ReceiveAddresses, Task, TransmitAddress,
};

use crate::constants::TxPower;

pub use nrf5x_unsafe::ble_radio::{RadioRegisters, RadioRegistersManager};

pub struct Radio<'a> {
    registers: RadioRegistersManager,
    tx_power: Cell<TxPower>,
    rx_client: OptionalCell<&'a dyn ble_advertising::RxClient>,
    tx_client: OptionalCell<&'a dyn ble_advertising::TxClient>,
    buffer: TakeCell<'static, [u8]>,
    /// FICR trim the nRF5340 network core radio needs after each power-up
    /// (erratum 117). `None` on nRF52, whose radio needs no trim.
    nrf5340_trim: Option<u32>,
}

impl<'a> Radio<'a> {
    pub const fn new(registers: RadioRegistersManager, nrf5340_trim: Option<u32>) -> Radio<'a> {
        Radio {
            registers,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            nrf5340_trim,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.registers
            .registers
            .mode
            .matches_all(Mode::MODE::BLE_1MBIT)
    }

    fn tx(&self) {
        self.registers
            .registers
            .event_ready
            .write(Event::READY::CLEAR);
        self.registers.registers.task_txen.write(Task::ENABLE::SET);
    }

    fn rx(&self) {
        self.registers
            .registers
            .event_ready
            .write(Event::READY::CLEAR);
        self.registers.registers.task_rxen.write(Task::ENABLE::SET);
    }

    fn set_rx_address(&self) {
        self.registers
            .registers
            .rxaddresses
            .write(ReceiveAddresses::ADDRESS.val(1));
    }

    fn set_tx_address(&self) {
        self.registers
            .registers
            .txaddress
            .write(TransmitAddress::ADDRESS.val(0));
    }

    fn radio_on(&self) {
        // reset and enable power
        self.registers.registers.power.write(Task::ENABLE::CLEAR);
        self.registers.registers.power.write(Task::ENABLE::SET);
    }

    fn radio_off(&self) {
        self.registers.registers.power.write(Task::ENABLE::CLEAR);
    }

    fn set_tx_power(&self) {
        self.registers
            .registers
            .txpower
            .set(self.tx_power.get() as u32);
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        self.disable_all_interrupts();

        if self.registers.registers.event_ready.is_set(Event::READY) {
            self.registers
                .registers
                .event_ready
                .write(Event::READY::CLEAR);
            self.registers
                .registers
                .event_end
                .write(Event::READY::CLEAR);
            self.registers.registers.task_start.write(Task::ENABLE::SET);
        }

        if self.registers.registers.event_address.is_set(Event::READY) {
            self.registers
                .registers
                .event_address
                .write(Event::READY::CLEAR);
        }
        if self.registers.registers.event_payload.is_set(Event::READY) {
            self.registers
                .registers
                .event_payload
                .write(Event::READY::CLEAR);
        }

        // tx or rx finished!
        if self.registers.registers.event_end.is_set(Event::READY) {
            self.registers
                .registers
                .event_end
                .write(Event::READY::CLEAR);

            let result = if self.registers.registers.crcstatus.is_set(Event::READY) {
                Ok(())
            } else {
                Err(ErrorCode::FAIL)
            };

            match self.registers.registers.state.get() {
                crate::constants::RADIO_STATE_TXRU
                | crate::constants::RADIO_STATE_TXIDLE
                | crate::constants::RADIO_STATE_TXDISABLE
                | crate::constants::RADIO_STATE_TX => {
                    self.radio_off();
                    self.tx_client
                        .map(|client| client.transmit_event(self.buffer.take().unwrap(), result));
                }
                crate::constants::RADIO_STATE_RXRU
                | crate::constants::RADIO_STATE_RXIDLE
                | crate::constants::RADIO_STATE_RXDISABLE
                | crate::constants::RADIO_STATE_RX => {
                    let payload = self.registers.take_rx_payload();
                    self.rx_client.map(|client| {
                        // Length is: S0 (1 Byte) + Length (1 Byte) + S1 (0 Bytes) + Payload
                        // And because the length field is directly read from the packet
                        // We need to add 2 to length to get the total length
                        let len = payload[1] + 2;
                        client.receive_event(payload, len, result)
                    });
                }
                // Radio state - Disabled
                _ => (),
            }
        }
        self.enable_interrupts();
    }

    pub fn enable_interrupts(&self) {
        self.registers.registers.intenset.write(
            Interrupt::READY::SET
                + Interrupt::ADDRESS::SET
                + Interrupt::PAYLOAD::SET
                + Interrupt::END::SET,
        );
    }

    pub fn enable_interrupt(&self, intr: u32) {
        self.registers.registers.intenset.set(intr);
    }

    pub fn clear_interrupt(&self, intr: u32) {
        self.registers.registers.intenclr.set(intr);
    }

    pub fn disable_all_interrupts(&self) {
        // disable all possible interrupts
        self.registers.registers.intenclr.set(0xffffffff);
    }

    fn replace_radio_buffer(&self, buf: &'static mut [u8]) -> &'static mut [u8] {
        self.registers.set_payload(buf);
        buf
    }

    fn ble_initialize(&self, channel: RadioChannel) {
        self.radio_on();

        self.ble_set_tx_power();

        self.ble_set_channel_rate();

        self.ble_set_channel_freq(channel);
        self.ble_set_data_whitening(channel);

        self.set_tx_address();
        self.set_rx_address();

        self.ble_set_packet_config();
        self.ble_set_advertising_access_address();

        self.ble_set_crc_config();

        self.registers.set_dma_ptr();
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.1.1 CRC Generation
    fn ble_set_crc_config(&self) {
        self.registers
            .registers
            .crccnf
            .write(CrcConfiguration::LEN::THREE + CrcConfiguration::SKIPADDR::EXCLUDE);
        self.registers
            .registers
            .crcinit
            .set(crate::constants::RADIO_CRCINIT_BLE);
        self.registers
            .registers
            .crcpoly
            .set(crate::constants::RADIO_CRCPOLY_BLE);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.1.2 Access Address
    // Set access address to 0x8E89BED6
    fn ble_set_advertising_access_address(&self) {
        self.registers.registers.prefix0.set(0x0000008e);
        self.registers.registers.base0.set(0x89bed600);
    }

    // Packet configuration
    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.1 Packet Format
    //
    // LSB                                                      MSB
    // +----------+   +----------------+   +---------------+   +------------+
    // | Preamble | - | Access Address | - | PDU           | - | CRC        |
    // | (1 byte) |   | (4 bytes)      |   | (2-255 bytes) |   | (3 bytes)  |
    // +----------+   +----------------+   +---------------+   +------------+
    //
    fn ble_set_packet_config(&self) {
        // sets the header of PDU TYPE to 1 byte
        // sets the header length to 1 byte
        self.registers.registers.pcnf0.write(
            PacketConfiguration0::LFLEN.val(8)
                + PacketConfiguration0::S0LEN.val(1)
                + PacketConfiguration0::S1LEN::CLEAR
                + PacketConfiguration0::S1INCL::CLEAR
                + PacketConfiguration0::PLEN::EIGHT,
        );

        self.registers.registers.pcnf1.write(
            PacketConfiguration1::WHITEEN::ENABLED
                + PacketConfiguration1::ENDIAN::LITTLE
                + PacketConfiguration1::BALEN.val(3)
                + PacketConfiguration1::STATLEN::CLEAR
                + PacketConfiguration1::MAXLEN.val(255),
        );
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part A], 4.6 REFERENCE SIGNAL DEFINITION
    // Bit Rate = 1 Mb/s ±1 ppm
    fn ble_set_channel_rate(&self) {
        if let Some(trim) = self.nrf5340_trim {
            self.registers.registers.errata_117.set(trim);
        }
        self.registers.registers.mode.write(Mode::MODE::BLE_1MBIT);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.2 Data Whitening
    // Configure channel index to the LFSR and the hardware solves the rest
    fn ble_set_data_whitening(&self, channel: RadioChannel) {
        self.registers
            .registers
            .datawhiteiv
            .set(channel.get_channel_index());
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 1.4.1
    // RF Channels:     0 - 39
    // Data:            0 - 36
    // Advertising:     37, 38, 39
    fn ble_set_channel_freq(&self, channel: RadioChannel) {
        self.registers
            .registers
            .frequency
            .write(Frequency::FREQUENCY.val(channel as u32));
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3 TRANSMITTER CHARACTERISTICS
    // Minimum Output Power : -20dBm
    // Maximum Output Power : +10dBm
    //
    // no check is required because the BleConfig::set_tx_power() method ensures that only
    // valid tranmitting power is configured!
    fn ble_set_tx_power(&self) {
        self.set_tx_power();
    }
}

impl<'a> ble_advertising::BleAdvertisementDriver<'a> for Radio<'a> {
    fn transmit_advertisement(&self, buf: &'static mut [u8], _len: usize, channel: RadioChannel) {
        let res = self.replace_radio_buffer(buf);
        self.buffer.replace(res);
        self.ble_initialize(channel);
        self.tx();
        self.enable_interrupts();
    }

    fn receive_advertisement(&self, channel: RadioChannel) {
        self.ble_initialize(channel);
        self.rx();
        self.enable_interrupts();
    }

    fn set_receive_client(&self, client: &'a dyn ble_advertising::RxClient) {
        self.rx_client.set(client);
    }

    fn set_transmit_client(&self, client: &'a dyn ble_advertising::TxClient) {
        self.tx_client.set(client);
    }
}

impl ble_advertising::BleConfig for Radio<'_> {
    // The BLE Advertising Driver validates that the `tx_power` is between -20 to 10 dBm but then
    // underlying chip must validate if the current `tx_power` is supported as well
    fn set_tx_power(&self, tx_power: u8) -> Result<(), ErrorCode> {
        // Convert u8 to TxPower
        match crate::constants::TxPower::try_from(tx_power) {
            // Invalid transmitting power, propogate error
            Err(()) => Err(ErrorCode::NOSUPPORT),
            // The nRF5340 network core radio tops out at 0 dBm
            Ok(TxPower::Positive4dBM | TxPower::Positive3dBM) if self.nrf5340_trim.is_some() => {
                Err(ErrorCode::NOSUPPORT)
            }
            // Valid transmitting power, propogate success
            Ok(res) => {
                self.tx_power.set(res);
                Ok(())
            }
        }
    }
}
