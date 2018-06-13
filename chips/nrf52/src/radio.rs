//! Radio driver, Bluetooth Low Energy, NRF52
//!
//! The generic radio configuration i.e., not specific to Bluetooth are functions and similar which
//! do not start with `ble`. Moreover, Bluetooth Low Energy specific radio configuration
//! starts with `ble`
//!
//! For more readability the Bluetooth specific configuration may be moved to separate trait
//!
//! ### Author
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Date: July 18, 2017
//!
//! ### Packet Configuration
//! ```
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
//! the payload is the PDU Type. Configured as 1 byte!
//!
//! * Length, an optional parameter that is configured to indicate how many bits of the
//! payload is the length field. Configured as 8 bits!
//!
//! * S1, Not used
//!
//! * Payload - 2 to 255 bytes
//!
//! * CRC - 3 bytes

use core::cell::Cell;
use core::convert::TryFrom;
use kernel;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;
use kernel::ReturnCode;
use nrf5x;
use nrf5x::constants::TxPower;

const RADIO_BASE: usize = 0x40001000;

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
    _reserved1: [u32; 55],
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
    intenset: ReadWrite<u32, Interrupt::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30c
    intenclr: ReadWrite<u32, Interrupt::Register>,
    /// Reserved
    _reserved6: [u32; 61],
    /// CRC status
    /// - Address: 0x400 - 0x404
    crcstatus: ReadOnly<u32, Event::Register>,
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
    state: ReadOnly<u32, State::Register>,
    /// Data whitening initial value
    /// - Address: 0x554 - 0x558
    datawhiteiv: ReadWrite<u32, DataWhiteIv::Register>,
    /// Reserved
    _reserved11: [u32; 2],
    /// Bit counter compare
    /// - Address: 0x560 - 0x564
    bcc: ReadWrite<u32, BitCounterCompare::Register>,
    /// Reserved
    _reserved12: [u32; 39],
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
    _reserved13: [u32; 3],
    /// Radio mode configuration register
    /// - Address: 0x650 - 0x654
    modecnf0: ReadWrite<u32, RadioModeConfig::Register>,
    /// Reserved
    _reserved14: [u32; 618],
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
        DISABLED_RSSISTOP OFFSET(8) NUMBITS(1)
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
        /// Index (n) of device address, see DAB[n] and DAP[n], that got an
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
            BLE_1MBIT = 3
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
        PLEN OFFSET(24) NUMBITS(1) [
            EIGHT = 0,
            SIXTEEN = 1
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

static mut PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

pub struct Radio {
    regs: *const RadioRegisters,
    tx_power: Cell<TxPower>,
    rx_client: Cell<Option<&'static ble_advertising::RxClient>>,
    tx_client: Cell<Option<&'static ble_advertising::TxClient>>,
}

pub static mut RADIO: Radio = Radio::new();

impl Radio {
    pub const fn new() -> Radio {
        Radio {
            regs: RADIO_BASE as *const RadioRegisters,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: Cell::new(None),
            tx_client: Cell::new(None),
        }
    }

    fn tx(&self) {
        let regs = unsafe { &*self.regs };
        regs.event_ready.write(Event::READY::CLEAR);
        regs.task_txen.write(Task::ENABLE::SET);
    }

    fn rx(&self) {
        let regs = unsafe { &*self.regs };
        regs.event_ready.write(Event::READY::CLEAR);
        regs.task_rxen.write(Task::ENABLE::SET);
    }

    fn set_rx_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.rxaddresses.write(ReceiveAddresses::ADDRESS.val(1));
    }

    fn set_tx_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.txaddress.write(TransmitAddress::ADDRESS.val(0));
    }

    fn radio_on(&self) {
        let regs = unsafe { &*self.regs };
        // reset and enable power
        regs.power.write(Task::ENABLE::CLEAR);
        regs.power.write(Task::ENABLE::SET);
    }

    fn radio_off(&self) {
        let regs = unsafe { &*self.regs };
        regs.power.write(Task::ENABLE::CLEAR);
    }

    fn set_tx_power(&self) {
        let regs = unsafe { &*self.regs };
        regs.txpower.set(self.tx_power.get() as u32);
    }

    fn set_dma_ptr(&self) {
        let regs = unsafe { &*self.regs };
        unsafe {
            regs.packetptr.set(PAYLOAD.as_ptr() as u32);
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = unsafe { &*self.regs };
        self.disable_all_interrupts();

        if regs.event_ready.is_set(Event::READY) {
            regs.event_ready.write(Event::READY::CLEAR);
            regs.event_end.write(Event::READY::CLEAR);
            regs.task_start.write(Task::ENABLE::SET);
        }

        if regs.event_address.is_set(Event::READY) {
            regs.event_address.write(Event::READY::CLEAR);
        }
        if regs.event_payload.is_set(Event::READY) {
            regs.event_payload.write(Event::READY::CLEAR);
        }

        // tx or rx finished!
        if regs.event_end.is_set(Event::READY) {
            regs.event_end.write(Event::READY::CLEAR);

            let result = if regs.crcstatus.is_set(Event::READY) {
                ReturnCode::SUCCESS
            } else {
                ReturnCode::FAIL
            };

            match regs.state.get() {
                nrf5x::constants::RADIO_STATE_TXRU
                | nrf5x::constants::RADIO_STATE_TXIDLE
                | nrf5x::constants::RADIO_STATE_TXDISABLE
                | nrf5x::constants::RADIO_STATE_TX => {
                    self.radio_off();
                    self.tx_client
                        .get()
                        .map(|client| client.transmit_event(result));
                }
                nrf5x::constants::RADIO_STATE_RXRU
                | nrf5x::constants::RADIO_STATE_RXIDLE
                | nrf5x::constants::RADIO_STATE_RXDISABLE
                | nrf5x::constants::RADIO_STATE_RX => {
                    self.radio_off();
                    unsafe {
                        self.rx_client.get().map(|client| {
                            // Length is: S0 (1 Byte) + Length (1 Byte) + S1 (0 Bytes) + Payload
                            // And because the length field is directly read from the packet
                            // We need to add 2 to length to get the total length
                            client.receive_event(&mut PAYLOAD, PAYLOAD[1] + 2, result)
                        });
                    }
                }
                // Radio state - Disabled
                _ => (),
            }
        }
        self.enable_interrupts();
    }

    pub fn enable_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.write(
            Interrupt::READY::SET + Interrupt::ADDRESS::SET + Interrupt::PAYLOAD::SET
                + Interrupt::END::SET,
        );
    }

    pub fn enable_interrupt(&self, intr: u32) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(intr);
    }

    pub fn clear_interrupt(&self, intr: u32) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(intr);
    }

    pub fn disable_all_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        // disable all possible interrupts
        regs.intenclr.set(0xffffffff);
    }

    fn replace_radio_buffer(&self, buf: &'static mut [u8]) -> &'static mut [u8] {
        // set payload
        for (i, c) in buf.as_ref().iter().enumerate() {
            unsafe {
                PAYLOAD[i] = *c;
            }
        }
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

        self.set_dma_ptr();
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.1.1 CRC Generation
    fn ble_set_crc_config(&self) {
        let regs = unsafe { &*self.regs };
        regs.crccnf
            .write(CrcConfiguration::LEN::THREE + CrcConfiguration::SKIPADDR::EXCLUDE);
        regs.crcinit.set(nrf5x::constants::RADIO_CRCINIT_BLE);
        regs.crcpoly.set(nrf5x::constants::RADIO_CRCPOLY_BLE);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.1.2 Access Address
    // Set access address to 0x8E89BED6
    fn ble_set_advertising_access_address(&self) {
        let regs = unsafe { &*self.regs };
        regs.prefix0.set(0x0000008e);
        regs.base0.set(0x89bed600);
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
        let regs = unsafe { &*self.regs };

        // sets the header of PDU TYPE to 1 byte
        // sets the header length to 1 byte
        regs.pcnf0.write(
            PacketConfiguration0::LFLEN.val(8) + PacketConfiguration0::S0LEN.val(1)
                + PacketConfiguration0::S1LEN::CLEAR
                + PacketConfiguration0::S1INCL::CLEAR
                + PacketConfiguration0::PLEN::EIGHT,
        );

        regs.pcnf1.write(
            PacketConfiguration1::WHITEEN::ENABLED + PacketConfiguration1::ENDIAN::LITTLE
                + PacketConfiguration1::BALEN.val(3)
                + PacketConfiguration1::STATLEN::CLEAR
                + PacketConfiguration1::MAXLEN.val(255),
        );
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part A], 4.6 REFERENCE SIGNAL DEFINITION
    // Bit Rate = 1 Mb/s ±1 ppm
    fn ble_set_channel_rate(&self) {
        let regs = unsafe { &*self.regs };
        regs.mode.write(Mode::MODE::BLE_1MBIT);
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 3.2 Data Whitening
    // Configure channel index to the LFSR and the hardware solves the rest
    fn ble_set_data_whitening(&self, channel: RadioChannel) {
        let regs = unsafe { &*self.regs };
        regs.datawhiteiv.set(channel.get_channel_index());
    }

    // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 1.4.1
    // RF Channels:     0 - 39
    // Data:            0 - 36
    // Advertising:     37, 38, 39
    fn ble_set_channel_freq(&self, channel: RadioChannel) {
        let regs = unsafe { &*self.regs };
        regs.frequency
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

impl ble_advertising::BleAdvertisementDriver for Radio {
    fn transmit_advertisement(
        &self,
        buf: &'static mut [u8],
        _len: usize,
        channel: RadioChannel,
    ) -> &'static mut [u8] {
        let res = self.replace_radio_buffer(buf);
        self.ble_initialize(channel);
        self.tx();
        self.enable_interrupts();
        res
    }

    fn receive_advertisement(&self, channel: RadioChannel) {
        self.ble_initialize(channel);
        self.rx();
        self.enable_interrupts();
    }

    fn set_receive_client(&self, client: &'static ble_advertising::RxClient) {
        self.rx_client.set(Some(client));
    }

    fn set_transmit_client(&self, client: &'static ble_advertising::TxClient) {
        self.tx_client.set(Some(client));
    }
}

impl ble_advertising::BleConfig for Radio {
    // The BLE Advertising Driver validates that the `tx_power` is between -20 to 10 dBm but then
    // underlying chip must validate if the current `tx_power` is supported as well
    fn set_tx_power(&self, tx_power: u8) -> kernel::ReturnCode {
        // Convert u8 to TxPower
        match nrf5x::constants::TxPower::try_from(tx_power) {
            // Invalid transmitting power, propogate error
            Err(_) => kernel::ReturnCode::ENOSUPPORT,
            // Valid transmitting power, propogate success
            Ok(res) => {
                self.tx_power.set(res);
                kernel::ReturnCode::SUCCESS
            }
        }
    }
}
