// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Radio driver, Bluetooth Low Energy, nRF5x-family, unsafe code

use core::ptr::{addr_of, addr_of_mut};
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_bitfields};

#[repr(C)]
pub struct RadioRegisters {
    /// Enable Radio in TX mode
    /// - Address: 0x000 - 0x004
    pub task_txen: WriteOnly<u32, Task::Register>,
    /// Enable Radio in RX mode
    /// - Address: 0x004 - 0x008
    pub task_rxen: WriteOnly<u32, Task::Register>,
    /// Start Radio
    /// - Address: 0x008 - 0x00c
    pub task_start: WriteOnly<u32, Task::Register>,
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
    _reserved1: [u32; 55],
    /// Radio has ramped up and is ready to be started
    /// - Address: 0x100 - 0x104
    pub event_ready: ReadWrite<u32, Event::Register>,
    /// Address sent or received
    /// - Address: 0x104 - 0x108
    pub event_address: ReadWrite<u32, Event::Register>,
    /// Packet payload sent or received
    /// - Address: 0x108 - 0x10c
    pub event_payload: ReadWrite<u32, Event::Register>,
    /// Packet sent or received
    /// - Address: 0x10c - 0x110
    pub event_end: ReadWrite<u32, Event::Register>,
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
    _reserved2: [u32; 2],
    /// Bit counter reached bit count value
    /// - Address: 0x128 - 0x12c
    event_bcmatch: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved3: [u32; 1],
    /// Packet received with CRC ok
    /// - Address: 0x130 - 0x134
    event_crcok: ReadWrite<u32, Event::Register>,
    /// Packet received with CRC error
    /// - Address: 0x134 - 0x138
    crcerror: ReadWrite<u32, Event::Register>,
    /// Reserved
    _reserved4: [u32; 50],
    /// Shortcut register
    /// - Address: 0x200 - 0x204
    shorts: ReadWrite<u32, Shortcut::Register>,
    /// Reserved
    _reserved5: [u32; 64],
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    pub intenset: ReadWrite<u32, Interrupt::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30c
    pub intenclr: ReadWrite<u32, Interrupt::Register>,
    /// Reserved
    _reserved6: [u32; 61],
    /// CRC status
    /// - Address: 0x400 - 0x404
    pub crcstatus: ReadOnly<u32, Event::Register>,
    /// Reserved
    _reserved7: [u32; 1],
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
    _reserved8: [u32; 60],
    /// Packet pointer
    /// - Address: 0x504 - 0x508
    packetptr: ReadWrite<u32, PacketPointer::Register>,
    /// Frequency
    /// - Address: 0x508 - 0x50c
    pub frequency: ReadWrite<u32, Frequency::Register>,
    /// Output power
    /// - Address: 0x50c - 0x510
    pub txpower: ReadWrite<u32, TransmitPower::Register>,
    /// Data rate and modulation
    /// - Address: 0x510 - 0x514
    pub mode: ReadWrite<u32, Mode::Register>,
    /// Packet configuration register 0
    /// - Address 0x514 - 0x518
    pub pcnf0: ReadWrite<u32, PacketConfiguration0::Register>,
    /// Packet configuration register 1
    /// - Address: 0x518 - 0x51c
    pub pcnf1: ReadWrite<u32, PacketConfiguration1::Register>,
    /// Base address 0
    /// - Address: 0x51c - 0x520
    pub base0: ReadWrite<u32, BaseAddress::Register>,
    /// Base address 1
    /// - Address: 0x520 - 0x524
    base1: ReadWrite<u32, BaseAddress::Register>,
    /// Prefix bytes for logical addresses 0-3
    /// - Address: 0x524 - 0x528
    pub prefix0: ReadWrite<u32, Prefix0::Register>,
    /// Prefix bytes for logical addresses 4-7
    /// - Address: 0x528 - 0x52c
    prefix1: ReadWrite<u32, Prefix1::Register>,
    /// Transmit address select
    /// - Address: 0x52c - 0x530
    pub txaddress: ReadWrite<u32, TransmitAddress::Register>,
    /// Receive address select
    /// - Address: 0x530 - 0x534
    pub rxaddresses: ReadWrite<u32, ReceiveAddresses::Register>,
    /// CRC configuration
    /// - Address: 0x534 - 0x538
    pub crccnf: ReadWrite<u32, CrcConfiguration::Register>,
    /// CRC polynomial
    /// - Address: 0x538 - 0x53c
    pub crcpoly: ReadWrite<u32, CrcPolynomial::Register>,
    /// CRC initial value
    /// - Address: 0x53c - 0x540
    pub crcinit: ReadWrite<u32, CrcInitialValue::Register>,
    /// Reserved
    _reserved9: [u32; 1],
    /// Interframe spacing in microseconds
    /// - Address: 0x544 - 0x548
    tifs: ReadWrite<u32, InterFrameSpacing::Register>,
    /// RSSI sample
    /// - Address: 0x548 - 0x54c
    rssisample: ReadWrite<u32, RssiSample::Register>,
    /// Reserved
    _reserved10: [u32; 1],
    /// Current radio state
    /// - Address: 0x550 - 0x554
    pub state: ReadOnly<u32, State::Register>,
    /// Data whitening initial value
    /// - Address: 0x554 - 0x558
    pub datawhiteiv: ReadWrite<u32, DataWhiteIv::Register>,
    /// Reserved
    _reserved11: [u32; 2],
    /// Bit counter compare
    /// - Address: 0x560 - 0x564
    bcc: ReadWrite<u32, BitCounterCompare::Register>,
    /// Reserved
    _reserved12: [u32; 9],
    /// Mode trim, nRF5340 network core only (erratum 117)
    /// - Address: 0x588 - 0x58c
    pub errata_117: ReadWrite<u32>,
    /// Reserved
    _reserved13: [u32; 29],
    /// Device address base segments
    /// - Address: 0x600 - 0x620
    dab: [ReadWrite<u32, DeviceAddressBase::Register>; 8],
    /// Device address prefix
    /// - Address: 0x620 - 0x640
    dap: [ReadWrite<u32, DeviceAddressPrefix::Register>; 8],
    /// Device address match configuration
    /// - Address: 0x640 - 0x644
    dacnf: ReadWrite<u32, DeviceAddressMatch::Register>,
    /// Reserved
    _reserved14: [u32; 3],
    /// Radio mode configuration register
    /// - Address: 0x650 - 0x654
    modecnf0: ReadWrite<u32, RadioModeConfig::Register>,
    /// Reserved
    _reserved15: [u32; 618],
    /// Peripheral power control
    /// - Address: 0xFFC - 0x1000
    pub power: ReadWrite<u32, Task::Register>,
}

register_bitfields! [u32,
    /// Task register
    pub Task [
        /// Enable task
        ENABLE OFFSET(0) NUMBITS(1)
    ],
    /// Event register
    pub Event [
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
        DISABLED_RSSISTOP OFFSET(8) NUMBITS(1)
    ],
    /// Interrupt register
    pub Interrupt [
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
        CRCERROR OFFSET(13) NUMBITS(1)
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
    pub Frequency [
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
    pub Mode [
        /// Radio data rate and modulation setting.
        /// The radio supports Frequency-shift Keying (FSK) modulation
        MODE OFFSET(0) NUMBITS(4) [
            NRF_1MBIT = 0,
            NRF_2MBIT = 1,
            NRF_250KBIT = 2,
            BLE_1MBIT = 3
        ]
    ],
    /// Packet configuration register 0
    pub PacketConfiguration0 [
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
        PLEN OFFSET(24) NUMBITS(1) [
            EIGHT = 0,
            SIXTEEN = 1
        ]
    ],
    /// Packet configuration register 1
    pub PacketConfiguration1 [
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
    pub TransmitAddress [
        /// Logical address to be used when transmitting a packet
        ADDRESS OFFSET(0) NUMBITS(3)
    ],
    /// Receive addresses register
    pub ReceiveAddresses [
        /// Enable or disable reception on logical address 0-7
        ADDRESS OFFSET(0) NUMBITS(8)
    ],
    /// CRC configuration register
    pub CrcConfiguration [
        /// CRC length in bytes
        LEN OFFSET(0) NUMBITS(2) [
            DISABLED = 0,
            ONE = 1,
            TWO = 2,
            THREE = 3
        ],
        /// Include or exclude packet field from CRC calculation
        SKIPADDR OFFSET(8) NUMBITS(1) [
            INCLUDE = 0,
            EXCLUDE = 1
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

/// The radio's packet buffer. The radio stores S0 and LENGTH ahead of up to
/// MAXLEN (255) payload bytes.
static mut PAYLOAD: [u8; 255 + 2] = [0x00; 255 + 2];

pub struct RadioRegistersManager {
    pub registers: StaticRef<RadioRegisters>,
}

impl RadioRegistersManager {
    /// Create a DMA-enabled register manager for the radio.
    ///
    /// # Safety
    ///
    /// This controls DMA hardware and owns the radio's packet buffer. As such,
    /// it must be unique. This requires:
    ///
    /// - This constructor must be called at most once.
    /// - There must not be any other code that accesses the packet pointer
    ///   register.
    pub unsafe fn new(registers: StaticRef<RadioRegisters>) -> Self {
        Self { registers }
    }

    /// Point the radio's DMA at the packet buffer.
    pub fn set_dma_ptr(&self) {
        self.registers.packetptr.set(addr_of!(PAYLOAD) as u32);
    }

    /// Power the radio off, which stops its DMA, and copy `buf` into the
    /// packet buffer.
    pub fn set_payload(&self, buf: &[u8]) {
        self.registers.power.write(Task::ENABLE::CLEAR);

        // # Safety
        //
        // The radio is off, so the hardware does not access the buffer, and
        // this reference ends with this function.
        let payload = unsafe { &mut *addr_of_mut!(PAYLOAD) };
        payload[..buf.len()].copy_from_slice(buf);
    }

    /// Power the radio off, which stops its DMA, and copy the received packet
    /// into `buf`. Returns the packet's LENGTH field.
    pub fn copy_rx_payload(&self, buf: &mut [u8]) -> u8 {
        self.registers.power.write(Task::ENABLE::CLEAR);

        // # Safety
        //
        // The radio is off, so the hardware does not access the buffer, and
        // this reference ends with this function.
        let payload = unsafe { &*addr_of!(PAYLOAD) };
        let n = buf.len().min(payload.len());
        buf[..n].copy_from_slice(&payload[..n]);
        payload[1]
    }
}
