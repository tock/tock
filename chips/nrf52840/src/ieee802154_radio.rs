// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! IEEE 802.15.4 radio driver for nRF52
//!
//! This driver implements a subset of 802.15.4 sending and receiving for the
//! nRF52840 chip per the nRF52840_PS_v1.0 spec. Upon calling the initialization
//! function, the chip is powered on and configured to the fields of the Radio
//! struct. This driver maintains a state machine between receiving,
//! transmitting, and sending acknowledgements. Because the nRF52840 15.4 radio
//! chip does not possess hardware support for ACK, this driver implements
//! software support for sending ACK when a received packet requests to be
//! acknowledged. The driver currently lacks support to listen for requested ACK
//! on packets the radio has sent. As of 8/14/23, the driver is able to send and
//! receive 15.4 packets as used in the basic 15.4 libtock-c apps.
//!
//! ## Driver State Machine
//!
//! To aid in future implementations, this describes a simplified and concise
//! version of the nrf52840 radio state machine specification and the state
//! machine this driver separately maintains.
//!
//! To interact with the radio, tasks are issued to the radio which in turn
//! trigger interrupt events. To receive, the radio must first "ramp up". The
//! RXRU state is entered by issuing a RXEN task. Once the radio has ramped up
//! successfully, it is now in the RXIDLE state and triggers a READY interrupt
//! event. To optimize the radio's operation, this driver enables hardware
//! shortcuts such that upon receiving the READY event, the radio chip
//! immediately triggers a START task. The START task notifies the radio to begin
//! officially "listening for packets" (RX state). Upon completing receiving the
//! packet, the radio issues an END event. The driver then determines if the
//! received packet has requested to be acknowledged (bit flag) and sends an ACK
//! accordingly. Finally, the received packet buffer and accompanying fields are
//! passed to the registered radio client. This marks the end of a receive cycle
//! and a new READY event is issued to once again begin listening for packets.
//!
//! When a registered radio client wishes to send a packet. The transmit(...)
//! method is called. To transmit a packet, the radio must first ramp up for
//! receiving and then perform a clear channel assessment by listening for a
//! specified period of time to determine if there is "traffic". If traffic is
//! detected, the radio sets an alarm and waits to perform another CCA after this
//! backoff. If the channel is determined to be clear, the radio then begins a TX
//! ramp up, enters a TX state and then sends the packet. To progress through
//! these states, hardware shortcuts are once again enabled in this driver. The
//! driver first issues a DISABLE task. A hardware shortcut is enabled so that
//! upon receipt of the disable task, the radio automatically issues a RXEN task
//! to enter the RXRU state. Additionally, a shortcut is enabled such that when
//! the RXREADY event is received, the radio automatically issues a CCA_START
//! task. Finally, a shortcut is also enabled such that upon receiving a CCAIDLE
//! event the radio automatically issues a TXEN event to ramp up the radio. The
//! driver then handles receiving the READY interrupt event and triggers the
//! START task to begin sending the packet. Upon completing the sending of the
//! packet, the radio issues an END event, to which the driver then returns the
//! radio to a receiving mode as described above. (For a more complete
//! explanation of the radio's operation, refer to nRF52840_PS_v1.0)
//!
//! This radio state machine provides nine possible states the radio can exist
//! in. For ease of implementation and clarity, this driver also maintains a
//! simplified state machine. These states consist of the radio being off (OFF),
//! receiving (RX), transmitting (TX), or acknowledging (ACK).

// Author: Tyler Potyondy
// 8/21/23

use crate::timer::TimerAlarm;
use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::radio::{self, PowerClient, RadioChannel, RadioConfig, RadioData};
use kernel::hil::time::{Alarm, AlarmClient, Time};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use nrf52::constants::TxPower;

const RADIO_BASE: StaticRef<RadioRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const RadioRegisters) };

const ACK_FLAG: u8 = 0b00100000;

pub const IEEE802154_PAYLOAD_LENGTH: usize = 255;
pub const IEEE802154_BACKOFF_PERIOD: usize = 320; //microseconds = 20 symbols
pub const IEEE802154_ACK_TIME: usize = 512; //microseconds = 32 symbols
pub const IEEE802154_MAX_POLLING_ATTEMPTS: u8 = 4;
pub const IEEE802154_MIN_BE: u8 = 3;
pub const IEEE802154_MAX_BE: u8 = 5;

// ACK Requires MHR and MFR fields. More explicitly this is composed of:
// | Frame Control (2 bytes) | Sequence Number (1 byte) | MFR (2 bytes) |.
// In total the ACK frame is 5 bytes long + 2 PSDU bytes (7 bytes total).
const SEQ_NUM_LEN: usize = 1;
pub const ACK_BUF_SIZE: usize =
    radio::SPI_HEADER_SIZE + radio::PHR_SIZE + radio::MHR_FC_SIZE + SEQ_NUM_LEN + radio::MFR_SIZE;

/// Where the 15.4 packet from the radio is stored in the buffer. The HIL
/// reserves one byte at the beginning of the buffer for use by the
/// capsule/hardware. We have no use for this, but the upper layers expect it so
/// we skip over it.
// We can't just drop the byte from the buffer because then it would be lost
// forever when we tried to return the frame buffer.
const BUF_PREFIX_SIZE: u32 = 1;

#[repr(C)]
struct RadioRegisters {
    /// Enable Radio in TX mode
    /// - Address: 0x000 - 0x004
    task_txen: WriteOnly<u32, Task::Register>,
    /// Enable Radio in RX mode
    /// - Address: 0x004 - 0x008
    task_rxen: WriteOnly<u32, Task::Register>,
    /// Start Radio
    /// - Address: 0x008 - 0x00c
    task_start: WriteOnly<u32, Task::Register>,
    /// Stop Radio
    /// - Address: 0x00c - 0x010
    task_stop: WriteOnly<u32, Task::Register>,
    /// Disable Radio
    /// - Address: 0x010 - 0x014
    task_disable: WriteOnly<u32, Task::Register>,
    /// Start the RSSI and take one single sample of the receive signal strength
    /// - Address: 0x014- 0x018
    task_rssistart: WriteOnly<u32, Task::Register>,
    /// Stop the RSSI measurement
    /// - Address: 0x018 - 0x01c
    task_rssistop: WriteOnly<u32, Task::Register>,
    /// Start the bit counter
    /// - Address: 0x01c - 0x020
    task_bcstart: WriteOnly<u32, Task::Register>,
    /// Stop the bit counter
    /// - Address: 0x020 - 0x024
    task_bcstop: WriteOnly<u32, Task::Register>,
    /// Reserved
    _reserved1: [u32; 2],
    /// Stop the bit counter
    /// - Address: 0x02c - 0x030
    task_ccastart: WriteOnly<u32, Task::Register>,
    /// Stop the bit counter
    /// - Address: 0x030 - 0x034
    task_ccastop: WriteOnly<u32, Task::Register>,
    /// Reserved
    _reserved2: [u32; 51],
    /// Radio has ramped up and is ready to be started
    /// - Address: 0x100 - 0x104
    event_ready: ReadWrite<u32, Event::Register>,
    /// Address sent or received
    /// - Address: 0x104 - 0x108
    event_address: ReadWrite<u32, Event::Register>,
    /// Packet payload sent or received
    /// - Address: 0x108 - 0x10c
    event_payload: ReadWrite<u32, Event::Register>,
    /// Packet sent or received
    /// - Address: 0x10c - 0x110
    event_end: ReadWrite<u32, Event::Register>,
    /// Radio has been disabled
    /// - Address: 0x110 - 0x114
    event_disabled: ReadWrite<u32, Event::Register>,
    /// A device address match occurred on the last received packet
    /// - Address: 0x114 - 0x118
    event_devmatch: ReadWrite<u32>,
    /// No device address match occurred on the last received packet
    /// - Address: 0x118 - 0x11c
    event_devmiss: ReadWrite<u32, Event::Register>,
    /// Sampling of receive signal strength complete
    /// - Address: 0x11c - 0x120
    event_rssiend: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved3: [u32; 2],
    /// Bit counter reached bit count value
    /// - Address: 0x128 - 0x12c
    event_bcmatch: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved4: [u32; 1],
    /// Packet received with CRC ok
    /// - Address: 0x130 - 0x134
    event_crcok: ReadWrite<u32, Event::Register>,
    /// Packet received with CRC error
    /// - Address: 0x134 - 0x138
    crcerror: ReadWrite<u32, Event::Register>,
    /// IEEE 802.15.4 length field received
    /// - Address: 0x138 - 0x13c
    event_framestart: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved5: [u32; 2],
    /// Wireless medium in idle - clear to send
    /// - Address: 0x144-0x148
    event_ccaidle: ReadWrite<u32, Event::Register>,
    /// Wireless medium busy - do not send
    /// - Address: 0x148-0x14c
    event_ccabusy: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved6: [u32; 45],
    /// Shortcut register
    /// - Address: 0x200 - 0x204
    shorts: ReadWrite<u32, Shortcut::Register>,
    /// Reserved
    _reserved7: [u32; 64],
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    intenset: ReadWrite<u32, Interrupt::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30c
    intenclr: ReadWrite<u32, Interrupt::Register>,
    /// Reserved
    _reserved8: [u32; 61],
    /// CRC status
    /// - Address: 0x400 - 0x404
    crcstatus: ReadOnly<u32, Event::Register>,
    /// Reserved
    _reserved9: [u32; 1],
    /// Received address
    /// - Address: 0x408 - 0x40c
    rxmatch: ReadOnly<u32, ReceiveMatch::Register>,
    /// CRC field of previously received packet
    /// - Address: 0x40c - 0x410
    rxcrc: ReadOnly<u32, ReceiveCrc::Register>,
    /// Device address match index
    /// - Address: 0x410 - 0x414
    dai: ReadOnly<u32, DeviceAddressIndex::Register>,
    /// Reserved
    _reserved10: [u32; 60],
    /// Packet pointer
    /// - Address: 0x504 - 0x508
    packetptr: ReadWrite<u32, PacketPointer::Register>,
    /// Frequency
    /// - Address: 0x508 - 0x50c
    frequency: ReadWrite<u32, Frequency::Register>,
    /// Output power
    /// - Address: 0x50c - 0x510
    txpower: ReadWrite<u32, TransmitPower::Register>,
    /// Data rate and modulation
    /// - Address: 0x510 - 0x514
    mode: ReadWrite<u32, Mode::Register>,
    /// Packet configuration register 0
    /// - Address 0x514 - 0x518
    pcnf0: ReadWrite<u32, PacketConfiguration0::Register>,
    /// Packet configuration register 1
    /// - Address: 0x518 - 0x51c
    pcnf1: ReadWrite<u32, PacketConfiguration1::Register>,
    /// Base address 0
    /// - Address: 0x51c - 0x520
    base0: ReadWrite<u32, BaseAddress::Register>,
    /// Base address 1
    /// - Address: 0x520 - 0x524
    base1: ReadWrite<u32, BaseAddress::Register>,
    /// Prefix bytes for logical addresses 0-3
    /// - Address: 0x524 - 0x528
    prefix0: ReadWrite<u32, Prefix0::Register>,
    /// Prefix bytes for logical addresses 4-7
    /// - Address: 0x528 - 0x52c
    prefix1: ReadWrite<u32, Prefix1::Register>,
    /// Transmit address select
    /// - Address: 0x52c - 0x530
    txaddress: ReadWrite<u32, TransmitAddress::Register>,
    /// Receive address select
    /// - Address: 0x530 - 0x534
    rxaddresses: ReadWrite<u32, ReceiveAddresses::Register>,
    /// CRC configuration
    /// - Address: 0x534 - 0x538
    crccnf: ReadWrite<u32, CrcConfiguration::Register>,
    /// CRC polynomial
    /// - Address: 0x538 - 0x53c
    crcpoly: ReadWrite<u32, CrcPolynomial::Register>,
    /// CRC initial value
    /// - Address: 0x53c - 0x540
    crcinit: ReadWrite<u32, CrcInitialValue::Register>,
    /// Reserved
    _reserved11: [u32; 1],
    /// Interframe spacing in microseconds
    /// - Address: 0x544 - 0x548
    tifs: ReadWrite<u32, InterFrameSpacing::Register>,
    /// RSSI sample
    /// - Address: 0x548 - 0x54c
    rssisample: ReadWrite<u32, RssiSample::Register>,
    /// Reserved
    _reserved12: [u32; 1],
    /// Current radio state
    /// - Address: 0x550 - 0x554
    state: ReadOnly<u32, State::Register>,
    /// Data whitening initial value
    /// - Address: 0x554 - 0x558
    datawhiteiv: ReadWrite<u32, DataWhiteIv::Register>,
    /// Reserved
    _reserved13: [u32; 2],
    /// Bit counter compare
    /// - Address: 0x560 - 0x564
    bcc: ReadWrite<u32, BitCounterCompare::Register>,
    /// Reserved
    _reserved14: [u32; 39],
    /// Device address base segments
    /// - Address: 0x600 - 0x620
    dab: [ReadWrite<u32, DeviceAddressBase::Register>; 8],
    /// Device address prefix
    /// - Address: 0x620 - 0x640
    dap: [ReadWrite<u32, DeviceAddressPrefix::Register>; 8],
    /// Device address match configuration
    /// - Address: 0x640 - 0x644
    dacnf: ReadWrite<u32, DeviceAddressMatch::Register>,
    /// MAC header Search Pattern Configuration
    /// - Address: 0x644 - 0x648
    mhrmatchconf: ReadWrite<u32, MACHeaderSearch::Register>,
    /// MAC Header Search Pattern Mask
    /// - Address: 0x648 - 0x64C
    mhrmatchmas: ReadWrite<u32, MACHeaderMask::Register>,
    /// Reserved
    _reserved15: [u32; 1],
    /// Radio mode configuration register
    /// - Address: 0x650 - 0x654
    modecnf0: ReadWrite<u32, RadioModeConfig::Register>,
    /// Reserved
    _reserved16: [u32; 6],
    /// Clear Channel Assesment (CCA) control register
    /// - Address: 0x66C - 0x670
    ccactrl: ReadWrite<u32, CCAControl::Register>,
    /// Reserved
    _reserved17: [u32; 611],
    /// Peripheral power control
    /// - Address: 0xFFC - 0x1000
    power: ReadWrite<u32, Task::Register>,
}

register_bitfields! [u32,
    /// Task register
    Task [
        /// Enable task
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    /// Event register
    Event [
        /// Ready event
        READY OFFSET(0) NUMBITS(1)
    ],
    /// Shortcut register
    Shortcut [
        /// Shortcut between READY event and START task
        READY_START OFFSET(0) NUMBITS(1),
        /// Shortcut between END event and DISABLE task
        END_DISABLE OFFSET(1) NUMBITS(1),
        /// Shortcut between DISABLED event and TXEN task
        DISABLED_TXEN OFFSET(2) NUMBITS(1),
        /// Shortcut between DISABLED event and RXEN task
        DISABLED_RXEN OFFSET(3) NUMBITS(1),
        /// Shortcut between ADDRESS event and RSSISTART task
        ADDRESS_RSSISTART OFFSET(4) NUMBITS(1),
        /// Shortcut between END event and START task
        END_START OFFSET(5) NUMBITS(1),
        /// Shortcut between ADDRESS event and BCSTART task
        ADDRESS_BCSTART OFFSET(6) NUMBITS(1),
        /// Shortcut between DISABLED event and RSSISTOP task
        DISABLED_RSSISTOP OFFSET(8) NUMBITS(1),
        /// Shortcut between CCAIDLE_TXEN
        CCAIDLE_TXEN OFFSET(12) NUMBITS(1),
        /// Shortcut between RXREADY_CCASTART
        RXREADY_CCASTART OFFSET(11) NUMBITS(1),
        /// Shortcut between TXREADY event and START task
        TXREADY_START OFFSET(19) NUMBITS(1),

    ],
    /// Interrupt register
    Interrupt [
        /// READY event
        READY OFFSET(0) NUMBITS(1),
        /// ADDRESS event
        ADDRESS OFFSET(1) NUMBITS(1),
        /// PAYLOAD event
        PAYLOAD OFFSET(2) NUMBITS(1),
        /// END event
        END OFFSET(3) NUMBITS(1),
        /// DISABLED event
        DISABLED OFFSET(4) NUMBITS(1),
        /// DEVMATCH event
        DEVMATCH OFFSET(5) NUMBITS(1),
        /// DEVMISS event
        DEVMISS OFFSET(6) NUMBITS(1),
        /// RSSIEND event
        RSSIEND OFFSET(7) NUMBITS(1),
        /// BCMATCH event
        BCMATCH OFFSET(10) NUMBITS(1),
        /// CRCOK event
        CRCOK OFFSET(12) NUMBITS(1),
        /// CRCERROR event
        CRCERROR OFFSET(13) NUMBITS(1),
        /// CCAIDLE event
        FRAMESTART OFFSET(14) NUMBITS(1),
        /// CCAIDLE event
        CCAIDLE OFFSET(17) NUMBITS(1),
        /// CCABUSY event
        CCABUSY OFFSET(18) NUMBITS(1),
        /// TXREADY event
        TXREADY OFFSET(21) NUMBITS(1),
        /// RXREADY event
        RXREADY OFFSET(22) NUMBITS(1),
    ],
    /// Receive match register
    ReceiveMatch [
        /// Logical address of which previous packet was received
        MATCH OFFSET(0) NUMBITS(3)
    ],
    /// Received CRC register
    ReceiveCrc [
        /// CRC field of previously received packet
        CRC OFFSET(0) NUMBITS(24)
    ],
    /// Device address match index register
    DeviceAddressIndex [
        /// Device address match index
        /// Index (n) of device address, see DAB\[n\] and DAP\[n\], that got an
        /// address match
        INDEX OFFSET(0) NUMBITS(3)
    ],
    /// Packet pointer register
    PacketPointer [
        /// Packet address to be used for the next transmission or reception. When transmitting, the packet pointed to by this
        /// address will be transmitted and when receiving, the received packet will be written to this address. This address is a byte
        /// aligned ram address.
        POINTER OFFSET(0) NUMBITS(32)
    ],
    /// Frequency register
    Frequency [
        /// Radio channel frequency
        /// Frequency = 2400 + FREQUENCY (MHz)
        FREQUENCY OFFSET(0) NUMBITS(7) [],
        /// Channel map selection.
        /// Channel map between 2400 MHZ .. 2500 MHZ
        MAP OFFSET(8) NUMBITS(1) [
            DEFAULT = 0,
            LOW = 1
        ]
    ],
    /// Transmitting power register
    TransmitPower [
        /// Radio output power
        POWER OFFSET(0) NUMBITS(8) [
            POS4DBM = 4,
            POS3DBM = 3,
            ODBM = 0,
            NEG4DBM = 0xfc,
            NEG8DBM = 0xf8,
            NEG12DBM = 0xf4,
            NEG16DBM = 0xf0,
            NEG20DBM = 0xec,
            NEG40DBM = 0xd8
        ]
    ],
    /// Data rate and modulation register
    Mode [
        /// Radio data rate and modulation setting.
        /// The radio supports Frequency-shift Keying (FSK) modulation
        MODE OFFSET(0) NUMBITS(4) [
            NRF_1MBIT = 0,
            NRF_2MBIT = 1,
            NRF_250KBIT = 2,
            BLE_1MBIT = 3,
            BLE_2MBIT = 4,
            BLE_LR125KBIT = 5,
            BLE_LR500KBIT = 6,
            IEEE802154_250KBIT = 15
        ]
    ],
    /// Packet configuration register 0
    PacketConfiguration0 [
        /// Length on air of LENGTH field in number of bits
        LFLEN OFFSET(0) NUMBITS(4) [],
        /// Length on air of S0 field in number of bytes
        S0LEN OFFSET(8) NUMBITS(1) [],
        /// Length on air of S1 field in number of bits.
        S1LEN OFFSET(16) NUMBITS(4) [],
        /// Include or exclude S1 field in RAM
        S1INCL OFFSET(20) NUMBITS(1) [
            AUTOMATIC = 0,
            INCLUDE = 1
        ],
        /// Length of preamble on air. Decision point: TASKS_START task
        PLEN OFFSET(24) NUMBITS(2) [
            EIGHT = 0,
            SIXTEEN = 1,
            THIRTYTWOZEROS = 2,
            LONGRANGE = 3
        ],
        CRCINC OFFSET(26) NUMBITS(1) [
            EXCLUDE = 0,
            INCLUDE = 1
        ]
    ],
    /// Packet configuration register 1
    PacketConfiguration1 [
        /// Maximum length of packet payload
        MAXLEN OFFSET(0) NUMBITS(8) [],
        /// Static length in number of bytes
        STATLEN OFFSET(8) NUMBITS(8) [],
        /// Base address length in number of bytes
        BALEN OFFSET(16) NUMBITS(3) [],
        /// On air endianness
        ENDIAN OFFSET(24) NUMBITS(1) [
            LITTLE = 0,
            BIG = 1
        ],
        /// Enable or disable packet whitening
        WHITEEN OFFSET(25) NUMBITS(1) [
            DISABLED = 0,
            ENABLED = 1
        ]
    ],
    /// Radio base address register
    BaseAddress [
        /// BASE0 or BASE1
        BASE OFFSET(0) NUMBITS(32)
    ],
    /// Radio prefix0 registers
    Prefix0 [
        /// Address prefix 0
        AP0 OFFSET(0) NUMBITS(8),
        /// Address prefix 1
        AP1 OFFSET(8) NUMBITS(8),
        /// Address prefix 2
        AP2 OFFSET(16) NUMBITS(8),
        /// Address prefix 3
        AP3 OFFSET(24) NUMBITS(8)
    ],
    /// Radio prefix0 registers
    Prefix1 [
        /// Address prefix 4
        AP4 OFFSET(0) NUMBITS(8),
        /// Address prefix 5
        AP5 OFFSET(8) NUMBITS(8),
        /// Address prefix 6
        AP6 OFFSET(16) NUMBITS(8),
        /// Address prefix 7
        AP7 OFFSET(24) NUMBITS(8)
    ],
    /// Transmit address register
    TransmitAddress [
        /// Logical address to be used when transmitting a packet
        ADDRESS OFFSET(0) NUMBITS(3)
    ],
    /// Receive addresses register
    ReceiveAddresses [
        /// Enable or disable reception on logical address 0-7
        ADDRESS OFFSET(0) NUMBITS(8)
    ],
    /// CRC configuration register
    CrcConfiguration [
        /// CRC length in bytes
        LEN OFFSET(0) NUMBITS(2) [
            DISABLED = 0,
            ONE = 1,
            TWO = 2,
            THREE = 3
        ],
        /// Include or exclude packet field from CRC calculation
        SKIPADDR OFFSET(8) NUMBITS(2) [
            INCLUDE = 0,
            EXCLUDE = 1,
            IEEE802154 = 2
        ]
    ],
    /// CRC polynomial register
    CrcPolynomial [
        /// CRC polynomial
        CRCPOLY OFFSET(0) NUMBITS(24)
    ],
    /// CRC initial value register
    CrcInitialValue [
       /// Initial value for CRC calculation
       CRCINIT OFFSET(0) NUMBITS(24)
    ],
    /// Inter Frame Spacing in us register
    InterFrameSpacing [
        /// Inter Frame Spacing in us
        /// Inter frame space is the time interval between two consecutive packets. It is defined as the time, in micro seconds, from the
        /// end of the last bit of the previous packet to the start of the first bit of the subsequent packet
        TIFS OFFSET(0) NUMBITS(8)
    ],
    /// RSSI sample register
    RssiSample [
        /// RSSI sample result
        RSSISAMPLE OFFSET(0) NUMBITS(7)
    ],
    /// Radio state register
    State [
        /// Current radio state
        STATE OFFSET(0) NUMBITS(4) [
            DISABLED = 0,
            RXRU = 1,
            RXIDLE = 2,
            RX = 3,
            RXDISABLED = 4,
            TXRU = 9,
            TXIDLE = 10,
            TX = 11,
            TXDISABLED = 12
        ]
    ],
    /// Data whitening initial value register
    DataWhiteIv [
        /// Data whitening initial value. Bit 6 is hard-wired to '1', writing '0'
        /// to it has no effect, and it will always be read back and used by the device as '1'
        DATEWHITEIV OFFSET(0) NUMBITS(7)
    ],
    /// Bit counter compare register
    BitCounterCompare [
        /// Bit counter compare
        BCC OFFSET(0) NUMBITS(32)
    ],
    /// Device address base register
    DeviceAddressBase [
        /// Device address base 0-7
        DAB OFFSET(0) NUMBITS(32)
    ],
    /// Device address prefix register
    DeviceAddressPrefix [
        /// Device address prefix 0-7
        DAP OFFSET(0) NUMBITS(32)
    ],
    /// Device address match configuration register
    DeviceAddressMatch [
        /// Enable or disable device address matching on 0-7
        ENA OFFSET(0) NUMBITS(8),
        /// TxAdd for device address 0-7
        TXADD OFFSET(8) NUMBITS(8)
    ],
    MACHeaderSearch [
        CONFIG OFFSET(0) NUMBITS(32)
    ],
    MACHeaderMask [
        PATTERN OFFSET(0) NUMBITS(32)
    ],
    CCAControl [
        CCAMODE OFFSET(0) NUMBITS(3) [
            ED_MODE = 0,
            CARRIER_MODE = 1,
            CARRIER_AND_ED_MODE = 2,
            CARRIER_OR_ED_MODE = 3,
            ED_MODE_TEST_1 = 4
        ],
        CCAEDTHRESH OFFSET(8) NUMBITS(8) [],
        CCACORRTHRESH OFFSET(16) NUMBITS(8) [],
        CCACORRCNT OFFSET(24) NUMBITS(8) []
    ],
    /// Radio mode configuration register
    RadioModeConfig [
        /// Radio ramp-up time
        RU OFFSET(0) NUMBITS(1) [
            DEFAULT = 0,
            FAST = 1
        ],
        /// Default TX value
        /// Specifies what the RADIO will transmit when it is not started, i.e. between:
        /// RADIO.EVENTS_READY and RADIO.TASKS_START
        /// RADIO.EVENTS_END and RADIO.TASKS_START
        DTX OFFSET(8) NUMBITS(2) [
            B1 = 0,
            B0 = 1,
            CENTER = 2
        ]
    ]
];

/// Operating mode of the radio.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RadioState {
    /// Radio peripheral is off.
    OFF,
    /// Currently transmitting a packet.
    TX,
    /// Default state when radio is on. Radio is configured to be in RX mode
    /// when the radio is turned on but not transmitting.
    RX,
    /// Transmitting an acknowledgement packet.
    ACK,
}

/// We use a single deferred call for two operations: triggering config clients
/// and power change clients. This allows us to track which operation we need to
/// perform when we get the deferred call callback.
#[derive(Debug, Clone, Copy)]
enum DeferredOperation {
    /// Waiting to notify that the configuration operation is complete.
    ConfigClientCallback,
    /// Waiting to notify that the power state of the radio changed (ie it
    /// turned on or off).
    PowerClientCallback,
}

pub struct Radio<'a> {
    registers: StaticRef<RadioRegisters>,
    rx_client: OptionalCell<&'a dyn radio::RxClient>,
    tx_client: OptionalCell<&'a dyn radio::TxClient>,
    config_client: OptionalCell<&'a dyn radio::ConfigClient>,
    power_client: OptionalCell<&'a dyn radio::PowerClient>,
    tx_power: Cell<TxPower>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    ack_buf: TakeCell<'static, [u8]>,
    addr: Cell<u16>,
    addr_long: Cell<[u8; 8]>,
    pan: Cell<u16>,
    cca_count: Cell<u8>,
    cca_be: Cell<u8>,
    random_nonce: Cell<u32>,
    channel: Cell<RadioChannel>,
    timer0: OptionalCell<&'a TimerAlarm<'a>>,
    state: Cell<RadioState>,
    deferred_call: DeferredCall,
    deferred_call_operation: OptionalCell<DeferredOperation>,
}

impl<'a> AlarmClient for Radio<'a> {
    fn alarm(&self) {
        // This alarm function is the callback for when the CCA backoff alarm completes
        // Attempt a new CCA period by issuing CCASTART task
        self.registers.task_ccastart.write(Task::ENABLE::SET);
    }
}

impl<'a> Radio<'a> {
    pub fn new(ack_buf: &'static mut [u8; ACK_BUF_SIZE]) -> Self {
        Self {
            registers: RADIO_BASE,
            rx_client: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            config_client: OptionalCell::empty(),
            power_client: OptionalCell::empty(),
            tx_power: Cell::new(TxPower::ZerodBm),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            ack_buf: TakeCell::new(ack_buf),
            addr: Cell::new(0),
            addr_long: Cell::new([0x00; 8]),
            pan: Cell::new(0),
            cca_count: Cell::new(0),
            cca_be: Cell::new(0),
            random_nonce: Cell::new(0xDEADBEEF),
            channel: Cell::new(RadioChannel::Channel26),
            timer0: OptionalCell::empty(),
            state: Cell::new(RadioState::OFF),
            deferred_call: DeferredCall::new(),
            deferred_call_operation: OptionalCell::empty(),
        }
    }

    pub fn set_timer_ref(&self, timer: &'a crate::timer::TimerAlarm<'a>) {
        self.timer0.set(timer);
    }

    pub fn is_enabled(&self) -> bool {
        self.registers
            .mode
            .matches_all(Mode::MODE::IEEE802154_250KBIT)
    }

    fn rx(&self) {
        self.state.set(RadioState::RX);

        // Unwrap fail = Radio RX Buffer is missing (may be due to receive client not replacing in receive(...) method,
        // or some instance in  driver taking buffer without properly replacing).
        let rbuf = self.rx_buf.take().unwrap();
        self.rx_buf.replace(self.set_dma_ptr(rbuf));

        // Instruct radio hardware to automatically progress from RXIDLE to RX
        // state upon receipt of internal `READY` signal after radio ramp-up completes.
        self.registers.shorts.write(Shortcut::READY_START::SET);

        self.registers.task_rxen.write(Task::ENABLE::SET);
    }

    fn radio_on(&self) {
        // reset and enable power
        self.registers.power.write(Task::ENABLE::CLEAR);
        self.registers.power.write(Task::ENABLE::SET);
    }

    fn radio_off(&self) {
        self.state.set(RadioState::OFF);

        self.registers.power.write(Task::ENABLE::CLEAR);
    }

    fn radio_is_on(&self) -> bool {
        self.registers.power.is_set(Task::ENABLE)
    }

    fn set_dma_ptr(&self, buffer: &'static mut [u8]) -> &'static mut [u8] {
        self.registers
            .packetptr
            .set(buffer.as_ptr() as u32 + BUF_PREFIX_SIZE);
        buffer
    }

    fn crc_check(&self) -> Result<(), ErrorCode> {
        if self.registers.crcstatus.is_set(Event::READY) {
            Ok(())
        } else {
            Err(ErrorCode::FAIL)
        }
    }

    // TODO: RECEIVING ACK FOR A SENT TX IS NOT IMPLEMENTED
    //
    // As a general note for the interrupt handler, event registers must still
    // be cleared even when hardware shortcuts are enabled.
    #[inline(never)]
    pub fn handle_interrupt(&self) {
        self.disable_all_interrupts();

        let mut start_task = false;
        let mut rx_init = false;

        match self.state.get() {
            // It should not be possible to receive an interrupt while the
            // tracked radio state is OFF.
            RadioState::OFF => {
                kernel::debug!("[ERROR]--15.4 state machine");
                kernel::debug!("Received interrupt while off");
            }
            RadioState::RX => {
                ////////////////////////////////////////////////////////////////
                // NOTE: This state machine assumes that the READY_START
                // shortcut is enabled at this point in time. If the READY_START
                // shortcut is not enabled, the state machine/driver will likely
                // exhibit undefined behavior.
                ////////////////////////////////////////////////////////////////

                // Since READY_START shortcut enabled, always clear READY event
                self.registers.event_ready.write(Event::READY::CLEAR);

                // Completed receiving a packet, now determine if we need to send ACK
                if self.registers.event_end.is_set(Event::READY) {
                    self.registers.event_end.write(Event::READY::CLEAR);
                    let crc = self.crc_check();

                    // Unwrap fail = Radio RX Buffer is missing (may be due to
                    // receive client not replacing in receive(...) method, or
                    // some instance in driver taking buffer without properly
                    // replacing).
                    let rbuf = self.rx_buf.take().unwrap();

                    // Data buffer format: | PREFIX | PHR | PSDU | LQI |
                    //
                    // Retrieve the length of the PSDU (actual frame). The frame
                    // length is only 7 bits, but of course the field is a byte.
                    // The nRF52840 product specification says this about the
                    // PHR byte (Version 1.8, section 6.20.12.1):
                    //
                    // > The most significant bit is reserved and is set to zero
                    // > for frames that are standard compliant. The radio
                    // > module will report all eight bits and it can
                    // > potentially be used to carry some information.
                    //
                    // We are not using that for any information so we just
                    // force it to zero. This ensures that `data_len` will not
                    // be longer than our buffer.
                    let data_len = (rbuf[radio::PHR_OFFSET] & 0x7F) as usize;

                    // LQI is found just after the data received.
                    let lqi = rbuf[data_len];

                    // We drop the CRC bytes (the MFR) from our frame.
                    let frame_len = data_len - radio::MFR_SIZE;

                    // 6th bit in the first byte of the MAC header determines if
                    // sender requested ACK. If so send ACK first before handing
                    // packet reception. This optimizes the time taken to send
                    // an ACK. If we call the receive function here, there is a
                    // non deterministic time required to complete the function
                    // as it may be passed up the entirety of the networking
                    // stack (leading to ACK timeout being exceeded).
                    if rbuf[radio::PSDU_OFFSET] & ACK_FLAG != 0 && crc.is_ok() {
                        self.ack_buf
                            .take()
                            .map_or(Err(ErrorCode::NOMEM), |ack_buf| {
                                // Entered ACK state //
                                self.state.set(RadioState::ACK);

                                // 4th byte of received packet is the 15.4
                                // sequence number.
                                let sequence_number = rbuf[radio::PSDU_OFFSET + radio::MHR_FC_SIZE];

                                // The frame control field is hardcoded for now;
                                // this is the only possible type of ACK
                                // currently supported so it is reasonable to
                                // hardcode this.
                                ack_buf[radio::PSDU_OFFSET] = 2;
                                ack_buf[radio::PSDU_OFFSET + 1] = 0;
                                ack_buf[radio::PSDU_OFFSET + radio::MHR_FC_SIZE] = sequence_number;

                                // Ensure we replace our RX buffer for the time
                                // being.
                                self.rx_buf.replace(rbuf);

                                // If the transmit function fails, replace the
                                // buffer and return an error.
                                if let Err((_, ret_buf)) = self.transmit(ack_buf, 3) {
                                    self.ack_buf.replace(ret_buf);
                                    Err(ErrorCode::FAIL)
                                } else {
                                    Ok(())
                                }
                            })
                            .unwrap_or_else(|err| {
                                // The ACK was not sent; we do not need to drop
                                // the packet, but print msg for debugging
                                // purposes, notify receive client of packet,
                                // and reset radio to receiving.
                                self.rx_client.map(|client| {
                                    start_task = true;
                                    client.receive(
                                        self.rx_buf.take().unwrap(),
                                        frame_len,
                                        lqi,
                                        crc.is_ok(),
                                        Err(err),
                                    );
                                });

                                kernel::debug!(
                                    "[ACKFail] Failed sending ACK in response to received packet."
                                );
                            });
                    } else {
                        // Packet received that does not require an ACK. Pass
                        // received packet to client and return radio to general
                        // receiving state to listen for new packets.
                        self.rx_client.map(|client| {
                            start_task = true;
                            client.receive(rbuf, frame_len, lqi, crc.is_ok(), Ok(()));
                        });
                    }
                }
            }
            RadioState::TX => {
                ////////////////////////////////////////////////////////////////
                // NOTE: This state machine assumes that the DISABLED_RXEN,
                // CCAIDLE_TXEN, RXREADY_CCASTART shortcuts are enabled at this
                // point in time. If the READY_START shortcut is not enabled,
                // the state machine/driver will likely exhibit undefined
                // behavior.
                ////////////////////////////////////////////////////////////////

                // Handle Event_ready interrupt. The TX path performs both a TX
                // ramp up and an RX ramp up. This means that there are two
                // potential cases we must handle. The ready event due to the Rx
                // Ramp up shortcuts to the CCASTART while the ready event due
                // to the Tx ramp up requires we issue a start task in response
                // to progress the state machine.
                if self.registers.event_ready.is_set(Event::READY) {
                    // In both cases, we must clear event
                    self.registers.event_ready.write(Event::READY::CLEAR);

                    // Ready event from Tx ramp up will be in radio internal
                    // TXIDLE state
                    if self.registers.state.get() == nrf52::constants::RADIO_STATE_TXIDLE {
                        start_task = true;
                    }
                }

                // Handle CCA related interrupts.
                if self.registers.event_ccabusy.is_set(Event::READY) {
                    self.registers.event_ccabusy.write(Event::READY::CLEAR);

                    // Need to back off for a period of time outlined in the
                    // IEEE 802.15.4 standard (see Figure 69 in section 7.5.1.4
                    // The CSMA-CA algorithm of the standard).
                    if self.cca_count.get() < IEEE802154_MAX_POLLING_ATTEMPTS {
                        self.cca_count.set(self.cca_count.get() + 1);
                        self.cca_be.set(self.cca_be.get() + 1);
                        let backoff_periods = self.random_nonce() & ((1 << self.cca_be.get()) - 1);
                        let current_time = self.timer0.unwrap_or_panic().now();
                        self.timer0
                            .unwrap_or_panic() // Unwrap fail = Missing timer reference for CSMA
                            .set_alarm(
                                current_time,
                                kernel::hil::time::Ticks32::from(
                                    backoff_periods * (IEEE802154_BACKOFF_PERIOD as u32),
                                ),
                            );
                    } else {
                        // We have exceeded the IEEE802154_MAX_POLLING_ATTEMPTS
                        // and should fail the transmission/return buffer to
                        // sending client.

                        let result = Err(ErrorCode::BUSY);
                        self.tx_client.map(|client| {
                            // Unwrap fail = TX Buffer is missing and was
                            // mistakenly not replaced after completion of
                            // set_dma_ptr(...)
                            let tbuf = self.tx_buf.take().unwrap();
                            client.send_done(tbuf, false, result);
                        });
                        rx_init = true;
                    }
                }

                // End event received; The TX is now finished and we need to
                // notify the sending client.
                if self.registers.event_end.is_set(Event::READY) {
                    self.registers.event_end.write(Event::READY::CLEAR);
                    let result = Ok(());

                    // TODO: Acked is hardcoded to always return false; add
                    // support to receive tx ACK.
                    self.tx_client.map(|client| {
                        // Unwrap fail = TX Buffer is missing and was mistakenly
                        // not replaced after completion of set_dma_ptr(...)
                        let tbuf = self.tx_buf.take().unwrap();
                        client.send_done(tbuf, false, result);
                    });
                    rx_init = true;
                }
            }
            RadioState::ACK => {
                ////////////////////////////////////////////////////////////////
                // NOTE: This state machine assumes that the READY_START
                // shortcut is enabled at this point in time. If the READY_START
                // shortcut is not enabled, the state machine/driver will likely
                // exhibit undefined behavior.
                ////////////////////////////////////////////////////////////////

                // Since READY_START shortcut enabled, always clear READY event
                self.registers.event_ready.write(Event::READY::CLEAR);

                // Completed sending ACK
                if self.registers.event_end.is_set(Event::READY) {
                    self.registers.event_end.write(Event::READY::CLEAR);

                    // Unwrap fail = TX Buffer is missing and was mistakenly not
                    // replaced after completion of set_dma_ptr(...)
                    let tbuf = self.tx_buf.take().unwrap();

                    // We must replace the ACK buffer that was passed to tx_buf
                    self.ack_buf.replace(tbuf);

                    // Reset radio to proper receiving state
                    rx_init = true;

                    // Notify receive client of packet that triggered the ACK.
                    self.rx_client.map(|client| {
                        // Unwrap fail = Radio RX Buffer is missing (may be due
                        // to receive client not replacing in receive(...)
                        // method, or some instance in  driver taking buffer
                        // without properly replacing).
                        let rbuf = self.rx_buf.take().unwrap();

                        // Data buffer format: | PREFIX | PHR | PSDU | LQI |
                        //
                        // See the RX case above for how these values are set.
                        let data_len = (rbuf[radio::PHR_OFFSET] & 0x7F) as usize;
                        let lqi = rbuf[data_len];
                        let frame_len = data_len - radio::MFR_SIZE;

                        // We know the CRC passed because otherwise we would not
                        // have transmitted an ACK.
                        client.receive(rbuf, frame_len, lqi, true, Ok(()));
                    });
                }
            }
        }

        // Enabling hardware shortcuts allows for a much faster operation.
        // However, this can also lead to race conditions and strange edge
        // cases. Namely, if a task_start or rx_en is set while interrupts are
        // disabled, the event_end interrupt can be "missed" and the interrupt
        // handler will not be called. If the event is missed, the state machine
        // is unable to progress and the driver enters a deadlock.
        self.enable_interrupts();
        if rx_init {
            self.rx();
        }
        if start_task {
            self.registers.task_start.write(Task::ENABLE::SET);
        }
    }

    pub fn enable_interrupts(&self) {
        self.registers
            .intenset
            .write(Interrupt::READY::SET + Interrupt::CCABUSY::SET + Interrupt::END::SET);
    }

    pub fn enable_interrupt(&self, intr: u32) {
        self.registers.intenset.set(intr);
    }

    pub fn clear_interrupt(&self, intr: u32) {
        self.registers.intenclr.set(intr);
    }

    pub fn disable_all_interrupts(&self) {
        // disable all possible interrupts
        self.registers.intenclr.set(0xffffffff);
    }

    pub fn set_ack_buffer(&self, buffer: &'static mut [u8]) {
        self.ack_buf.replace(buffer);
    }

    fn radio_initialize(&self) {
        self.radio_on();

        // CONFIGURE RADIO //
        self.ieee802154_set_channel_rate();

        self.ieee802154_set_packet_config();

        self.ieee802154_set_crc_config();

        self.ieee802154_set_rampup_mode();

        self.ieee802154_set_cca_config();

        self.ieee802154_set_tx_power();

        self.ieee802154_set_channel_freq();

        // Begin receiving procedure
        self.enable_interrupts();
        self.rx();
    }

    // IEEE802.15.4 SPECIFICATION Section 6.20.12.5 of the NRF52840 Datasheet
    fn ieee802154_set_crc_config(&self) {
        self.registers
            .crccnf
            .write(CrcConfiguration::LEN::TWO + CrcConfiguration::SKIPADDR::IEEE802154);
        self.registers
            .crcinit
            .set(nrf52::constants::RADIO_CRCINIT_IEEE802154);
        self.registers
            .crcpoly
            .set(nrf52::constants::RADIO_CRCPOLY_IEEE802154);
    }

    fn ieee802154_set_rampup_mode(&self) {
        self.registers
            .modecnf0
            .write(RadioModeConfig::RU::FAST + RadioModeConfig::DTX::CENTER);
    }

    fn ieee802154_set_cca_config(&self) {
        self.registers.ccactrl.write(
            CCAControl::CCAMODE.val(nrf52::constants::IEEE802154_CCA_MODE)
                + CCAControl::CCAEDTHRESH.val(nrf52::constants::IEEE802154_CCA_ED_THRESH)
                + CCAControl::CCACORRTHRESH.val(nrf52::constants::IEEE802154_CCA_CORR_THRESH)
                + CCAControl::CCACORRCNT.val(nrf52::constants::IEEE802154_CCA_CORR_CNT),
        );
    }

    // Packet configuration
    // Settings taken from RiotOS nrf52840 15.4 driver
    fn ieee802154_set_packet_config(&self) {
        self.registers.pcnf0.write(
            PacketConfiguration0::LFLEN.val(8)
                + PacketConfiguration0::PLEN::THIRTYTWOZEROS
                + PacketConfiguration0::CRCINC::INCLUDE,
        );

        self.registers
            .pcnf1
            .write(PacketConfiguration1::MAXLEN.val(nrf52::constants::RADIO_PAYLOAD_LENGTH as u32));
    }

    fn ieee802154_set_channel_rate(&self) {
        self.registers.mode.write(Mode::MODE::IEEE802154_250KBIT);
    }

    fn ieee802154_set_channel_freq(&self) {
        let channel = self.channel.get();
        self.registers
            .frequency
            .write(Frequency::FREQUENCY.val(channel as u32));
    }

    fn ieee802154_set_tx_power(&self) {
        self.registers.txpower.set(self.tx_power.get() as u32);
    }

    pub fn startup(&self) -> Result<(), ErrorCode> {
        self.radio_initialize();
        Ok(())
    }

    // Returns a new pseudo-random number and updates the randomness state.
    //
    // Uses the [Xorshift](https://en.wikipedia.org/wiki/Xorshift) algorithm to
    // produce pseudo-random numbers. Uses the `random_nonce` field to keep
    // state.
    fn random_nonce(&self) -> u32 {
        let mut next_nonce = ::core::num::Wrapping(self.random_nonce.get());
        next_nonce ^= next_nonce << 13;
        next_nonce ^= next_nonce >> 17;
        next_nonce ^= next_nonce << 5;
        self.random_nonce.set(next_nonce.0);
        self.random_nonce.get()
    }
}

impl<'a> kernel::hil::radio::RadioConfig<'a> for Radio<'a> {
    fn initialize(&self) -> Result<(), ErrorCode> {
        Ok(())
    }

    fn set_power_client(&self, client: &'a dyn PowerClient) {
        self.power_client.set(client);
    }

    fn reset(&self) -> Result<(), ErrorCode> {
        self.radio_initialize();
        Ok(())
    }

    fn start(&self) -> Result<(), ErrorCode> {
        self.radio_initialize();

        // Configure deferred call to trigger callback.
        self.deferred_call_operation
            .set(DeferredOperation::PowerClientCallback);
        self.deferred_call.set();

        Ok(())
    }

    fn stop(&self) -> Result<(), ErrorCode> {
        self.radio_off();

        // Configure deferred call to trigger callback.
        self.deferred_call_operation
            .set(DeferredOperation::PowerClientCallback);
        self.deferred_call.set();

        Ok(())
    }

    fn is_on(&self) -> bool {
        self.radio_is_on()
    }

    fn busy(&self) -> bool {
        // `tx_buf` is only occupied when a transmission is underway.
        self.tx_buf.is_some()
    }

    fn set_config_client(&self, client: &'a dyn radio::ConfigClient) {
        self.config_client.set(client);
    }

    /// Commit the config calls to hardware, changing (in theory):
    ///
    /// - the RX address
    /// - PAN ID
    /// - TX power
    /// - channel
    ///
    /// to the specified values. **However**, the nRF52840 IEEE 802.15.4 radio
    /// does not support hardware-level address filtering (see
    /// [here](https://devzone.nordicsemi.com/f/nordic-q-a/19320/using-nrf52840-for-802-15-4)).
    /// So setting the addresses and PAN ID have no meaning for this chip and
    /// any filtering must be done in higher layers in software.
    ///
    /// Issues a callback to the config client when done.
    fn config_commit(&self) {
        // All we can configure is TX power and channel frequency.
        self.ieee802154_set_tx_power();
        self.ieee802154_set_channel_freq();

        // Enable deferred call so we can generate a `ConfigClient` callback.
        self.deferred_call_operation
            .set(DeferredOperation::ConfigClientCallback);
        self.deferred_call.set();
    }

    //#################################################
    /// Accessors
    //#################################################

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
        self.tx_power.get() as i8
    }

    /// The 802.15.4 channel
    fn get_channel(&self) -> u8 {
        self.channel.get().get_channel_number()
    }

    //#################################################
    /// Mutators
    //#################################################

    fn set_address(&self, addr: u16) {
        self.addr.set(addr);
    }

    fn set_address_long(&self, addr: [u8; 8]) {
        self.addr_long.set(addr);
    }

    fn set_pan(&self, id: u16) {
        self.pan.set(id);
    }

    fn set_channel(&self, chan: RadioChannel) {
        self.channel.set(chan);
    }

    fn set_tx_power(&self, tx_power: i8) -> Result<(), ErrorCode> {
        // Convert u8 to TxPower
        match nrf52::constants::TxPower::try_from(tx_power as u8) {
            // Invalid transmitting power, propagate error
            Err(()) => Err(ErrorCode::NOSUPPORT),
            // Valid transmitting power, propagate success
            Ok(res) => {
                self.tx_power.set(res);
                Ok(())
            }
        }
    }
}

impl<'a> kernel::hil::radio::RadioData<'a> for Radio<'a> {
    fn set_receive_client(&self, client: &'a dyn radio::RxClient) {
        self.rx_client.set(client);
    }

    fn set_receive_buffer(&self, buffer: &'static mut [u8]) {
        self.rx_buf.replace(buffer);
    }

    fn set_transmit_client(&self, client: &'a dyn radio::TxClient) {
        self.tx_client.set(client);
    }

    fn transmit(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.state.get() == RadioState::OFF {
            return Err((ErrorCode::OFF, buf));
        } else if self.busy() {
            return Err((ErrorCode::BUSY, buf));
        } else if buf.len() < radio::PSDU_OFFSET + frame_len + radio::MFR_SIZE {
            // Not enough room for CRC or PHR or reserved byte
            return Err((ErrorCode::SIZE, buf));
        }

        // Insert the PHR which is the PDSU length.
        buf[radio::PHR_OFFSET] = (frame_len + radio::MFR_SIZE) as u8;

        // The tx_buf does not possess static memory. This buffer only
        // temporarily holds a reference to another buffer passed as a function
        // argument. The tx_buf holds ownership of this buffer until it is
        // returned through the send_done(...) function.
        self.tx_buf.replace(self.set_dma_ptr(buf));

        // The transmit function handles sending both ACK and standard packets
        if let RadioState::ACK = self.state.get() {
            self.registers.task_txen.write(Task::ENABLE::SET);
        } else {
            // Configure radio for standard packet TX
            self.state.set(RadioState::TX);

            // Instruct radio hardware to automatically progress from:
            // - RXDISABLE to RXRU state upon receipt of internal disabled event
            // - RXIDLE to RX state upon receipt of ready event and radio ramp
            //   up completed, begin CCA backoff
            // - RX to TXRU state upon internal receipt CCA completion event
            //   (clear to begin transmitting)
            self.registers.shorts.write(
                Shortcut::DISABLED_RXEN::SET
                    + Shortcut::RXREADY_CCASTART::SET
                    + Shortcut::CCAIDLE_TXEN::SET,
            );

            // Radio is in proper shortcut state, disable and begin TX sequence
            self.registers.task_disable.write(Task::ENABLE::SET);
        }

        Ok(())
    }
}

impl DeferredCallClient for Radio<'_> {
    fn handle_deferred_call(&self) {
        // On deferred call we trigger the config or power callbacks. The
        // `.take()` ensures we clear what is pending.
        self.deferred_call_operation.take().map(|op| match op {
            DeferredOperation::ConfigClientCallback => {
                self.config_client.map(|client| {
                    client.config_done(Ok(()));
                });
            }
            DeferredOperation::PowerClientCallback => {
                self.power_client.map(|client| {
                    client.changed(self.radio_is_on());
                });
            }
        });
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
