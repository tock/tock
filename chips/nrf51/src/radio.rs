//! Radio driver, Bluetooth Low Energy, nRF51
//!
//! Sending Bluetooth Low Energy advertisement packets with payloads up to 31 bytes
//!
//! Currently all fields in PAYLOAD array are configurable from user-space
//! except the PDU_TYPE.
//!
//! ### Authors
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: June 22, 2017

use core::cell::Cell;
use core::convert::TryFrom;
use kernel;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;
use kernel::ReturnCode;
use nrf5x;
use nrf5x::constants::TxPower;

pub static mut RADIO: Radio = Radio::new();

static mut PAYLOAD: [u8; nrf5x::constants::RADIO_PAYLOAD_LENGTH] =
    [0x00; nrf5x::constants::RADIO_PAYLOAD_LENGTH];

#[repr(C)]
struct RadioRegisters {
    txen: WriteOnly<u32, Task::Register>,        // 0x000 ---> 0x004
    rxen: WriteOnly<u32, Task::Register>,        // 0x004 ---> 0x008
    start: WriteOnly<u32, Task::Register>,       // 0x008 ---> 0x00c
    stop: WriteOnly<u32, Task::Register>,        // 0x00c ---> 0x010
    disable: WriteOnly<u32, Task::Register>,     // 0x010 ---> 0x014
    rssistart: WriteOnly<u32, Task::Register>,   // 0x014 ---> 0x018
    rssistop: WriteOnly<u32, Task::Register>,    // 0x018 ---> 0x01c
    bcstart: WriteOnly<u32, Task::Register>,     // 0x01c ---> 0x020
    bcstop: WriteOnly<u32, Task::Register>,      // 0x020 ---> 0x024
    _reserved1: [u32; 55],                       // 0x024 ---> 0x100
    ready: ReadWrite<u32, Event::Register>,      // 0x100 ---> 0x104
    address: ReadWrite<u32, Event::Register>,    // 0x104 ---> 0x108
    payload: ReadWrite<u32, Event::Register>,    // 0x108 ---> 0x10c
    end: ReadWrite<u32, Event::Register>,        // 0x10c ---> 0x110
    disabled: ReadWrite<u32, Event::Register>,   // 0x110 ---> 0x114
    devmatch: ReadWrite<u32, Event::Register>,   // 0x114 ---> 0x118
    devmiss: ReadWrite<u32, Event::Register>,    // 0x118 ---> 0x11c
    rssiend: ReadWrite<u32, Event::Register>,    // 0x11c -->  0x120
    _reserved2: [u32; 2],                        // 0x120 ---> 0x128
    bcmatch: ReadWrite<u32, Event::Register>,    // 0x128 ---> 0x12c
    _reserved3: [u32; 53],                       // 0x12c ---> 0x200
    shorts: ReadWrite<u32, Shortcuts::Register>, // 0x200 ---> 0x204
    _reserved4: [u32; 64],                       // 0x204 ---> 0x304
    intenset: ReadWrite<u32, Interrupt::Register>, // 0x304 ---> 0x308
    intenclr: ReadWrite<u32, Interrupt::Register>, // 0x308 ---> 0x30c
    _reserved5: [u32; 61],                       // 0x30c ---> 0x400
    crcstatus: ReadOnly<u32, CrcStatus::Register>, // 0x400 - 0x404
    _reserved6: [u32; 1],                        // 0x404 - 0x408
    rxmatch: ReadOnly<u32, RxMatch::Register>,   // 0x408 - 0x40c
    rxcrc: ReadOnly<u32, RxCrc::Register>,       // 0x40c - 0x410
    dai: ReadOnly<u32, DeviceAddressIndex::Register>, // 0x410 - 0x414
    _reserved7: [u32; 60],                       // 0x414 - 0x504
    packetptr: ReadWrite<u32, PacketPointer::Register>, // 0x504 - 0x508
    frequency: ReadWrite<u32, Frequency::Register>, // 0x508 - 0x50c
    txpower: ReadWrite<u32, TransmitPower::Register>, // 0x50c - 0x510
    mode: ReadWrite<u32, Mode::Register>,        // 0x510 - 0x514
    pcnf0: ReadWrite<u32, Pcnf0::Register>,      // 0x514 - 0x518
    pcnf1: ReadWrite<u32, Pcnf1::Register>,      // 0x518 - 0x51c
    base0: ReadWrite<u32, Base::Register>,       // 0x51c - 0x520
    base1: ReadWrite<u32, Base::Register>,       // 0x520 - 0x524
    prefix0: ReadWrite<u32, Prefix0::Register>,  // 0x524 - 0x528
    prefix1: ReadWrite<u32, Prefix1::Register>,  // 0x528 - 0x52c
    txaddress: ReadWrite<u32, TransmitAddress::Register>, // 0x52c - 0x530
    rxaddresses: ReadWrite<u32, ReceiveAddress::Register>, // 0x530 - 0x534
    crccnf: ReadWrite<u32, CrcCnf::Register>,    // 0x534 - 0x538
    crcpoly: ReadWrite<u32, CrcPolynomial::Register>, // 0x538 - 0x53c
    crcinit: ReadWrite<u32, CrcInitialValue::Register>, // 0x53c - 0x540
    test: ReadWrite<u32, Test::Register>,        // 0x540 - 0x544
    tifs: ReadWrite<u32, TimeInterframeSpacing::Register>, // 0x544 - 0x548
    rssisample: ReadOnly<u32, RssiSampleResult::Register>, // 0x548 - 0x54c
    _reserved8: [u32; 1],                        // 0x54c - 0x550
    state: ReadOnly<u32, State::Register>,       // 0x550 - 0x554
    datawhiteiv: ReadWrite<u32, DataWhiteningIV::Register>, // 0x554 - 0x558
    _reserved9: [u32; 2],                        // 0x558 - 0x560
    bcc: ReadWrite<u32, BitCounterCompare::Register>, // 0x560 - 0x564
    _reserved10: [u32; 39],                      // 0x560 - 0x600
    dab0: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x600 - 0x604
    dab1: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x604 - 0x608
    dab2: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x608 - 0x60c
    dab3: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x60c - 0x610
    dab4: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x610 - 0x614
    dab5: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x614 - 0x618
    dab6: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x618 - 0x61c
    dab7: ReadWrite<u32, DeviceAddressBaseSegment::Register>, // 0x61c - 0x620
    dap0: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x620 - 0x624
    dap1: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x624 - 0x628
    dap2: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x628 - 0x62c
    dap3: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x62c - 0x630
    dap4: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x630 - 0x634
    dap5: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x634 - 0x638
    dap6: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x638 - 0x63c
    dap7: ReadWrite<u32, DeviceAddressPrefix::Register>, // 0x63c - 0x640
    dacnf: ReadWrite<u32, Dacnf::Register>,      // 0x640 - 0x644
    _reserved11: [u32; 56],                      // 0x644 - 0x724
    override0: ReadWrite<u32, TrimOverrideN::Register>, // 0x724 - 0x728
    override1: ReadWrite<u32, TrimOverrideN::Register>, // 0x728 - 0x72c
    override2: ReadWrite<u32, TrimOverrideN::Register>, // 0x72c - 0x730
    override3: ReadWrite<u32, TrimOverrideN::Register>, // 0x730 - 0x734
    override4: ReadWrite<u32, TrimOverride4::Register>, // 0x734 - 0x738
    _reserved12: [u32; 561],                     // 0x738 - 0x724
    power: ReadWrite<u32, Power::Register>,      // 0xFFC - 0x1000
}

register_bitfields![u32,
    /// Tasks
    Task [
        EXECUTE 0
    ],

    /// Events.
    Event [
        READY 0
    ],

    Shortcuts [
        READY_START 0,
        END_DISABLE 1,
        DISABLED_TXEN 2,
        DISABLED_RXEN 3,
        ADDRESS_RSSISTART 4,
        END_START 5,
        ADDRESS_BCSTART 6,
        DISABLED_RSSISTOP 8
    ],

    Interrupt [
        READY 0,
        ADDRESS 1,
        PAYLOAD 2,
        END 3,
        DISABLED 4,
        DEVMATCH 5,
        DEVMISS 6,
        RSSIEND 7,
        BCMATCH 10
    ],

    CrcStatus [
        CRCSTATUS OFFSET(0) NUMBITS(1) [
            CRCError = 0,
            CRCOk = 1
        ]
    ],

    RxMatch [
        /// Logical address of which previous packet was received.
        RXMATCH OFFSET(0) NUMBITS(3)
    ],

    RxCrc [
        /// CRC field of previously received packet.
        RXCRC OFFSET(0) NUMBITS(24)
    ],

    DeviceAddressIndex [
        /// Index (n) of device address, see DAB[n] and DAP[n],
        /// that got an address match.
        DAI OFFSET(0) NUMBITS(3)
    ],

    PacketPointer [
        /// Packet address to be used for the next transmission
        /// or reception. When transmitting, the packet pointed
        /// to by this address will be transmitted and when
        /// receiving, the received packet will be written to
        /// this address. This address is a byte aligned ram
        /// address.
        ///
        /// Decision point: START task.
        PACKETPTR OFFSET(0) NUMBITS(32)
    ],

    Frequency [
        /// Radio channel frequency.
        ///
        /// Frequency = 2400 + FREQUENCY (MHz).
        ///
        /// Decision point: TXEN or RXEN
        FREQUENCY OFFSET(0) NUMBITS(7)
    ],

    TransmitPower [
        /// Radio output power.
        ///
        /// Decision point: TXEN task.
        TXPOWER OFFSET(0) NUMBITS(8) [
            Pos4dBm = 0x04,
            ZerodBm = 0x00,
            Neg4dBm = 0xFC,
            Neg8dBm = 0xF8,
            Neg12dBm = 0xF4,
            Neg16dBm = 0xF0,
            Neg20dBm = 0xEC,
            Neg30dBm = 0xD8
        ]
    ],

    Mode [
        /// Radio data rate and modulation setting. The radio
        /// supports Frequency-shift Keying (FSK) modulation.
        MODE OFFSET(0) NUMBITS(2) [
            /// 1 Mbit/s Nordic proprietary radio mode.
            Nrf_1Mbit = 0,
            /// 2 Mbit/s Nordic proprietary radio mode.
            Nrf_2Mbit = 1,
            /// 250 kbit/s Nordic proprietary radio mode.
            Nrf_250Kbit = 2,
            /// 1 Mbit/s Bluetooth Low Energy
            Ble_1Mbit = 3
        ]
    ],

    Pcnf0 [
        /// Length on air of LENGTH field in number of bits.
        /// Decision point: START task.
        LFLEN OFFSET(0) NUMBITS(4),
        /// Length on air of S0 field in number of bits.
        /// Decision point: START task.
        S0LEN OFFSET(8) NUMBITS(1),
        /// Length on air of S1 field in number of bits.
        /// Decision point: START task.
        S1LEN OFFSET(16) NUMBITS(4)
    ],

    Pcnf1 [
        /// Maximum length of packet payload. If the packet
        /// payload is larger than MAXLEN, the radio will
        /// truncate the payload to MAXLEN.
        MAXLEN OFFSET(0) NUMBITS(8),

        /// Static length in number of bytes The static length parameter is
        /// added to the total length of the payload when sending and receiving
        /// packets, e.g. if the static length is set to N the radio will receive
        /// or send N bytes more than what is defined in the LENGTH field of the
        /// packet.
        ///
        /// Decision point: START task.
        STATLEN OFFSET(8) NUMBITS(8),

        /// Base address length in number of bytes The address field is
        /// composed of the base address and the one byte long address prefix,
        /// e.g. set BALEN=2 to get a total address of 3 bytes.
        ///
        ///Decision point: START task.
        BALEN OFFSET(16) NUMBITS(3),

        /// On air endianness of packet, this applies to the S0, LENGTH, S1 and the PAYLOAD fields.
        ///
        /// Little = 0,
        /// Big = 1
        ///
        /// Decision point: START task.
        ENDIAN OFFSET(24) NUMBITS(1),

        /// Enable or diable packet whitening.
        WHITEEN OFFSET(25) NUMBITS(1)
    ],

    Base [
        BASE OFFSET(0) NUMBITS(32)
    ],

    Prefix0 [
        AP0 OFFSET(0) NUMBITS(8),
        AP1 OFFSET(8) NUMBITS(8),
        AP2 OFFSET(16) NUMBITS(8),
        AP3 OFFSET(24) NUMBITS(8)
    ],

    Prefix1 [
        AP4 OFFSET(0) NUMBITS(8),
        AP5 OFFSET(8) NUMBITS(8),
        AP6 OFFSET(16) NUMBITS(8),
        AP7 OFFSET(24) NUMBITS(8)
    ],

    TransmitAddress [
        TXADDRESS OFFSET(0) NUMBITS(3)
    ],

    ReceiveAddress [
        ADDR0 0,
        ADDR1 1,
        ADDR2 2,
        ADDR3 3,
        ADDR4 4,
        ADDR5 5,
        ADDR6 6,
        ADDR7 7
    ],

    CrcCnf [
        LEN OFFSET(0) NUMBITS(2) [
            Disabled = 0,
            One = 1,
            Two = 2,
            Three = 3
        ],
        SKIPADDR OFFSET(8) NUMBITS(1) [
            Include = 0,
            Skip = 1
        ]
    ],

    CrcPolynomial [
        CRCPOLY OFFSET(0) NUMBITS(24)
    ],

    CrcInitialValue [
        CRCINIT OFFSET(0) NUMBITS(24)
    ],

    Test [
        CONSTCARRIER 0,
        PLLLOCK 1
    ],

    TimeInterframeSpacing [
        TIFS OFFSET(0) NUMBITS(8)
    ],

    RssiSampleResult [
        RSSISAMPLE OFFSET(0) NUMBITS(7)
    ],

    State [
        STATE OFFSET(0) NUMBITS(4) [
            Disabled = 0,
            RxRu = 1,
            RxIdle = 2,
            Rx = 3,
            RxDisable = 4,
            TxRu = 9,
            TxIdle = 10,
            Tx = 11,
            TxDiable = 12
        ]
    ],

    DataWhiteningIV [
        /// Bit 0 corresponds to Position 6 of the LSFR, Bit 1 to Position 5, etc
        DATAWHITEIV OFFSET(0) NUMBITS(6),
        /// Always 1 (write has no effect).
        RESERVED OFFSET(6) NUMBITS(1)
    ],

    BitCounterCompare [
        BCC OFFSET(0) NUMBITS(32)
    ],

    DeviceAddressBaseSegment [
        DAB OFFSET(0) NUMBITS(32)
    ],

    DeviceAddressPrefix [
        DAP OFFSET(0) NUMBITS(32)
    ],

    Dacnf [
        ENA0 0,
        ENA1 1,
        ENA2 2,
        ENA3 3,
        ENA4 4,
        ENA5 5,
        ENA6 6,
        ENA7 7,
        TXADD0 8,
        TXADD1 9,
        TXADD2 10,
        TXADD3 11,
        TXADD4 12,
        TXADD5 13,
        TXADD6 14,
        TXADD7 15
    ],

    TrimOverrideN [
        OVERRIDEn OFFSET(0) NUMBITS(32)
    ],

    TrimOverride4 [
        OVERRIDE4 OFFSET(0) NUMBITS(28),
        ENABLE OFFSET(31) NUMBITS(1)
    ],

    Power [
        POWER 0
    ]
];

const RADIO_BASE: StaticRef<RadioRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const RadioRegisters) };

pub struct Radio {
    registers: StaticRef<RadioRegisters>,
    tx_power: Cell<TxPower>,
    rx_client: OptionalCell<&'static ble_advertising::RxClient>,
    tx_client: OptionalCell<&'static ble_advertising::TxClient>,
}

impl Radio {
    pub const fn new() -> Radio {
        Radio {
            registers: RADIO_BASE,
            tx_power: Cell::new(TxPower::ZerodBm),
            rx_client: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
        }
    }

    fn ble_initialize(&self, channel: RadioChannel) {
        let regs = &*self.registers;

        self.radio_on();

        // TX Power acc. to twpower variable in the struct
        self.set_tx_power();

        // BLE MODE
        self.set_channel_rate(nrf5x::constants::RadioMode::Ble1Mbit as u32);

        self.set_channel_freq(channel);
        self.set_data_whitening(channel);

        // Set PREFIX | BASE Address
        regs.prefix0.write(Prefix0::AP0.val(0x8e));
        regs.base0.write(Base::BASE.val(0x89bed600));

        self.set_tx_address(0x00);
        self.set_rx_address(0x01);

        // Set Packet Config
        self.set_packet_config(0x00);

        // CRC Config
        self.set_crc_config();

        // Buffer configuration
        self.set_dma_ptr();
    }

    fn tx(&self) {
        let regs = &*self.registers;
        regs.ready.write(Event::READY::CLEAR);
        regs.txen.write(Task::EXECUTE::SET);
    }

    fn rx(&self) {
        let regs = &*self.registers;
        regs.ready.write(Event::READY::CLEAR);
        regs.rxen.write(Task::EXECUTE::SET);
    }

    fn set_crc_config(&self) {
        let regs = &*self.registers;
        regs.crccnf.set(
            nrf5x::constants::RADIO_CRCCNF_LEN_3BYTES
                | nrf5x::constants::RADIO_CRCCNF_SKIPADDR
                    << nrf5x::constants::RADIO_CRCCNF_SKIPADDR_POS,
        );
        regs.crcinit.set(nrf5x::constants::RADIO_CRCINIT_BLE);
        regs.crcpoly.set(nrf5x::constants::RADIO_CRCPOLY_BLE);
    }

    // Packet configuration
    fn set_packet_config(&self, _: u32) {
        let regs = &*self.registers;
        regs.pcnf0.set(
            (nrf5x::constants::RADIO_PCNF0_S0_LEN_1BYTE << nrf5x::constants::RADIO_PCNF0_S0LEN_POS)
                | (nrf5x::constants::RADIO_PCNF0_LFLEN_1BYTE
                    << nrf5x::constants::RADIO_PCNF0_LFLEN_POS),
        );

        regs.pcnf1.set(
            (nrf5x::constants::RADIO_PCNF1_WHITEEN_ENABLED <<
                nrf5x::constants::RADIO_PCNF1_WHITEEN_POS) |
                 (nrf5x::constants::RADIO_PCNF1_ENDIAN_LITTLE <<
                     nrf5x::constants::RADIO_PCNF1_ENDIAN_POS) |
                 // Total Address is 4 bytes (BASE ADDRESS + PREFIX (1))
                 (nrf5x::constants::RADIO_PCNF1_BALEN_3BYTES <<
                  nrf5x::constants::RADIO_PCNF1_BALEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_STATLEN_DONT_EXTEND
                    << nrf5x::constants::RADIO_PCNF1_STATLEN_POS)
                | (nrf5x::constants::RADIO_PCNF1_MAXLEN_37BYTES
                    << nrf5x::constants::RADIO_PCNF1_MAXLEN_POS),
        );
    }

    fn set_rx_address(&self, _: u32) {
        let regs = &*self.registers;
        regs.rxaddresses.write(ReceiveAddress::ADDR0::SET);
    }

    fn set_tx_address(&self, _: u32) {
        let regs = &*self.registers;
        regs.txaddress.write(TransmitAddress::TXADDRESS.val(0));
    }

    fn set_channel_rate(&self, rate: u32) {
        let regs = &*self.registers;
        // set channel rate,  3 - BLE 1MBIT/s
        regs.mode.set(rate);
    }

    fn set_data_whitening(&self, channel: RadioChannel) {
        let regs = &*self.registers;
        regs.datawhiteiv.set(channel.get_channel_index());
    }

    fn set_channel_freq(&self, channel: RadioChannel) {
        let regs = &*self.registers;
        //37, 38 and 39 for adv.
        regs.frequency.set(channel as u32);
    }

    fn radio_on(&self) {
        let regs = &*self.registers;
        // reset and enable power
        regs.power.write(Power::POWER::CLEAR);
        regs.power.write(Power::POWER::SET);
    }

    fn radio_off(&self) {
        let regs = &*self.registers;
        regs.power.write(Power::POWER::CLEAR);
    }

    // pre-condition validated before arriving here
    fn set_tx_power(&self) {
        let regs = &*self.registers;
        regs.txpower.set(self.tx_power.get() as u32);
    }

    fn set_dma_ptr(&self) {
        let regs = &*self.registers;
        unsafe {
            regs.packetptr.set(PAYLOAD.as_ptr() as u32);
        }
    }

    #[inline(never)]
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        self.disable_interrupts();

        if regs.ready.is_set(Event::READY) {
            regs.ready.write(Event::READY::CLEAR);
            regs.end.write(Event::READY::CLEAR);
            regs.start.write(Task::EXECUTE::SET);
        }

        if regs.payload.is_set(Event::READY) {
            regs.payload.write(Event::READY::CLEAR);
        }

        if regs.address.is_set(Event::READY) {
            regs.address.write(Event::READY::CLEAR);
        }

        if regs.end.is_set(Event::READY) {
            regs.end.write(Event::READY::CLEAR);
            regs.disable.write(Task::EXECUTE::SET);

            let result = if regs.crcstatus.get() == 1 {
                Ok(Success::Success)
            } else {
                Err(Error::FAIL)
            };

            match regs.state.get() {
                nrf5x::constants::RADIO_STATE_TXRU
                | nrf5x::constants::RADIO_STATE_TXIDLE
                | nrf5x::constants::RADIO_STATE_TXDISABLE
                | nrf5x::constants::RADIO_STATE_TX => {
                    self.radio_off();
                    self.tx_client.map(|client| client.transmit_event(result));
                }
                nrf5x::constants::RADIO_STATE_RXRU
                | nrf5x::constants::RADIO_STATE_RXIDLE
                | nrf5x::constants::RADIO_STATE_RXDISABLE
                | nrf5x::constants::RADIO_STATE_RX => {
                    self.radio_off();
                    unsafe {
                        self.rx_client.map(|client| {
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
        let regs = &*self.registers;
        regs.intenset.set(
            nrf5x::constants::RADIO_INTENSET_READY
                | nrf5x::constants::RADIO_INTENSET_ADDRESS
                | nrf5x::constants::RADIO_INTENSET_PAYLOAD
                | nrf5x::constants::RADIO_INTENSET_END,
        );
    }

    pub fn disable_interrupts(&self) {
        let regs = &*self.registers;
        // disable all possible interrupts
        regs.intenclr.set(0xffffffff);
    }

    pub fn replace_radio_buffer(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8] {
        // set payload
        for (i, c) in buf.as_ref()[0..len].iter().enumerate() {
            unsafe {
                PAYLOAD[i] = *c;
            }
        }
        buf
    }
}

impl ble_advertising::BleAdvertisementDriver for Radio {
    fn transmit_advertisement(
        &self,
        buf: &'static mut [u8],
        len: usize,
        channel: RadioChannel,
    ) -> &'static mut [u8] {
        let res = self.replace_radio_buffer(buf, len);
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
        self.rx_client.set(client);
    }

    fn set_transmit_client(&self, client: &'static ble_advertising::TxClient) {
        self.tx_client.set(client);
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
                kernel::Ok(Success::Success)
            }
        }
    }
}
