//! Near Field Communication Tag (NFCT)
//!
//! Author
//! -------------------
//!
//! * Jean-Michel Picod <jmichel@google.com>
//! * Mirna Al-Shetairy <mshetairy@google.com>

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{
    register_bitfields, register_structs, InMemoryRegister, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::hil::nfc::NfcFieldState;
use kernel::ReturnCode;

const NFCT_BASE: StaticRef<NfctRegisters> =
    unsafe { StaticRef::new(0x40005000 as *const NfctRegisters) };

register_structs! {
    NfctRegisters {
        /// Activate NFCT peripheral for incoming and outgoing frames, change
        /// state to activated.
        (0x000 => task_activate: WriteOnly<u32, Task::Register>),
        /// Disable NFCT peripheral
        (0x004 => task_disable: WriteOnly<u32, Task::Register>),
        /// Enable NFC sense field mode, change state to sense mode
        (0x008 => task_sense: WriteOnly<u32, Task::Register>),
        /// Start transmission of an outgoing frame, change state to transmit
        (0x00C => task_starttx: WriteOnly<u32, Task::Register>),
        (0x010 => _reserved1),
        /// Initialized the EasyDMA for receive
        (0x01C => task_enablerxdata: WriteOnly<u32, Task::Register>),
        (0x020 => _reserved2),
        /// Force state machine to IDLE state
        (0x024 => task_goidle: WriteOnly<u32, Task::Register>),
        /// Force state machine to SLEEP_A state
        (0x028 => task_gosleep: WriteOnly<u32, Task::Register>),
        (0x02C => _reserved3),
        /// The NFCT peripheral is ready to receive and send frames
        (0x100 => event_ready: ReadWrite<u32, Event::Register>),
        /// Remote NFC field detected
        (0x104 => event_fielddetected: ReadWrite<u32, Event::Register>),
        /// Remote NFC field lost
        (0x108 => event_fieldlost: ReadWrite<u32, Event::Register>),
        /// Marks the start of the first symbol of a transmitted frame
        (0x10C => event_txframestart: ReadWrite<u32, Event::Register>),
        /// Marks the end of the last transmitted on-air symbol of a frame
        (0x110 => event_txframeend: ReadWrite<u32, Event::Register>),
        /// Marks the end of the first symbol of a received frame
        (0x114 => event_rxframestart: ReadWrite<u32, Event::Register>),
        /// Received data has been checked (CRC, parity) and transferred to
        /// RAM, and EasyDMA has ended accessing the RX buffer
        (0x118 => event_rxframeend: ReadWrite<u32, Event::Register>),
        /// NFC error reported. The ERRORSTATUS register contains details on
        /// the source of error
        (0x11C => event_error: ReadWrite<u32, Event::Register>),
        (0x120 => _reserved4),
        /// NFC RX frame error reported. The FRAMESTATUS.RX register contains
        /// details on the source of error
        (0x128 => event_rxerror: ReadWrite<u32, Event::Register>),
        /// RX buffer (as defined by PACKETPTR and MAXLEN) in data RAM full
        (0x12C => event_endrx: ReadWrite<u32, Event::Register>),
        /// Transmission of data in RAM has ended, and EasyDMA has ended
        /// accessing the TX buffer
        (0x130 => event_endtx: ReadWrite<u32, Event::Register>),
        (0x134 => _reserved5),
        /// Auto collision resolution process has started
        (0x138 => event_autocolresstarted: ReadWrite<u32, Event::Register>),
        (0x13C => _reserved6),
        /// NFC auto collision resolution error reported
        (0x148 => event_collision: ReadWrite<u32, Event::Register>),
        /// NFC auto collision resolution successfully completed
        (0x14C => event_selected: ReadWrite<u32, Event::Register>),
        /// EasyDMA is ready to receive or send frames
        (0x150 => event_started: ReadWrite<u32, Event::Register>),
        (0x154 => _reserved7),
        /// Shortcuts between local events and tasks
        (0x200 => shorts: ReadWrite<u32, Shorts::Register>),
        (0x204 => _reserved9),
        /// Enable or disable interrupt
        (0x300 => inten: ReadWrite<u32, Interrupt::Register>),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, Interrupt::Register>),
        /// Disable interrupt
        (0x308 => intenclr: ReadWrite<u32, Interrupt::Register>),
        (0x30C => _reserved10),
        /// NFC Error Status register
        (0x404 => errorstatus: ReadWrite<u32, ErrorStatus::Register>),
        (0x408 => _reserved11),
        /// Result of last incoming frame
        (0x40C => framestatus_rx: ReadWrite<u32, FrameStatus::Register>),
        /// NfcTag State register
        (0x410 => nfctagstate: ReadOnly<u32, NfcTagState::Register>),
        (0x414 => _reserved12),
        /// Sleep state during automatic collision resolution
        (0x420 => sleepstate: ReadOnly<u32, SleepState::Register>),
        (0x424 => _reserved13),
        /// Indicates the presence or not of a valid field
        (0x43C => fieldpresent: ReadOnly<u32, FieldPresent::Register>),
        (0x440 => _reserved14),
        /// Minimum frame delay
        (0x504 => framedelay_min: ReadWrite<u32, FrameDelayMin::Register>),
        /// Maximum frame delay
        (0x508 => framedelay_max: ReadWrite<u32, FrameDelayMax::Register>),
        /// Configuration register for the Frame Delay Timer
        (0x50C => framedelay_mode: ReadWrite<u32, FrameDelayMode::Register>),
        /// Packet pointer for TXD and RXD data storage in Data RAM
        (0x510 => packetptr: ReadWrite<u32, Pointer::Register>),
        /// Size of the RAM buffer allocated to TXD and RXD data storage each
        (0x514 => maxlen: ReadWrite<u32, MaxLen::Register>),
        /// Configuration of outgoing frames
        (0x518 => txd_frameconfig: ReadWrite<u32, TxdFrameConfig::Register>),
        /// Size of outgoing frame
        (0x51C => txd_amount: ReadWrite<u32, Amount::Register>),
        /// Configuration of incoming frames
        (0x520 => rxd_frameconfig: ReadWrite<u32, RxdFrameConfig::Register>),
        /// Size of incoming frame
        (0x524 => rxd_amount: ReadOnly<u32, Amount::Register>),
        (0x528 => _reserved15),
        /// Last NFCID1 part (4, 7 or 10 bytes ID)
        (0x590 => nfcid1_last: ReadWrite<u32, NfcIdPart3::Register>),
        /// Second last NFCID1 part (7 or 10 bytes ID)
        (0x594 => nfcid1_2nd_last: ReadWrite<u32, NfcIdPart2::Register>),
        /// Third last NFCID1 part (10 bytes ID)
        (0x598 => nfcid1_3rd_last: ReadWrite<u32, NfcIdPart1::Register>),
        /// Controls the auto collision resolution function.
        /// This setting must be done before the NFCT peripheral is enabled
        (0x59C => autocolresconfig: ReadWrite<u32, AutoColConfig::Register>),
        /// NFC-A SENS_RES auto-response settings
        (0x5A0 => sensres: ReadWrite<u32, SensRes::Register>),
        /// NFC-A SEL_RES auto-response settings
        (0x5A4 => selres: ReadWrite<u32, SelRes::Register>),
        (0x5A8 => _reserved16),
        /// Errata 98: NFCT: Not able to communicate with the peer
        /// Undocumented register
        (0x68C => errata98: WriteOnly<u32>),
        (0x690 => @END),
    }
}

register_bitfields![u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1) [
            Trigger = 1
        ]
    ],

    /// Read event
    Event [
        GENERATED OFFSET(0) NUMBITS(1) [
            NotGenerated = 0,
            Generated = 1
        ]
    ],

    /// Shortcuts
    Shorts [
        // Shortcut between event FIELDDETECTED and task ACTIVATE
        FIELDDETECTED_ACTIVATE OFFSET(0) NUMBITS(1),
        // Shortcut between event FIELDLOST and task SENSE
        FIELDLOST_SENSE OFFSET(1) NUMBITS(1),
        // Shortcut between event TXFRAMEEND and task ENABLERXDATA
        TXFRAMEEND_ENABLERXDATA OFFSET(5) NUMBITS(1)
    ],

    // NFC Interrupts
    Interrupt [
        READY OFFSET(0) NUMBITS(1),
        FIELDDETECTED OFFSET(1) NUMBITS(1),
        FIELDLOST OFFSET(2) NUMBITS(1),
        TXFRAMESTART OFFSET(3) NUMBITS(1),
        TXFRAMEEND OFFSET(4) NUMBITS(1),
        RXFRAMESTART OFFSET(5) NUMBITS(1),
        RXFRAMEEND OFFSET(6) NUMBITS(1),
        ERROR OFFSET(7) NUMBITS(1),
        RXERROR OFFSET(10) NUMBITS(1),
        ENDRX OFFSET(11) NUMBITS(1),
        ENDTX OFFSET(12) NUMBITS(1),
        AUTOCOLRESSTARTED OFFSET(14) NUMBITS(1),
        COLLISION OFFSET(18) NUMBITS(1),
        SELECTED OFFSET(19) NUMBITS(1),
        STARTED OFFSET(20) NUMBITS(1)
    ],

    ErrorStatus [
        FRAMEDELAYTIMEOUT OFFSET(0) NUMBITS(1)
    ],

    FrameStatus [
        CRCERROR OFFSET(0) NUMBITS(1) [
            CRCCorrect = 0,
            CRCError = 1
        ],
        PARITYSTATUS OFFSET(2) NUMBITS(1) [
            ParityOk = 0,
            ParityError = 1
        ],
        OVERRUN OFFSET(3) NUMBITS(1) [
            NoOverrun = 0,
            Overrun = 1
        ]
    ],

    NfcTagState [
        NFCTAGSTATE OFFSET(0) NUMBITS(3) [
            Disabled = 0,
            RampUp = 2,
            Idle = 3,
            Receive = 4,
            FrameDelay = 5,
            Transmit = 6
        ]
    ],

    SleepState [
        SLEEPSTATE OFFSET(0) NUMBITS(1) [
            Idle = 0,
            SleepA = 1
        ]
    ],

    FieldPresent [
        FIELDPRESENT OFFSET(0) NUMBITS(1) [
            NoField = 0,
            FieldPresent = 1
        ],
        LOCKDETECT OFFSET(1) NUMBITS(1) [
            NotLocked = 0,
            Locked = 1
        ]
    ],

    FrameDelayMin [
        FRAMEDELAYMIN OFFSET(0) NUMBITS(16)
    ],

    FrameDelayMax [
        FRAMEDELAYMAX OFFSET(0) NUMBITS(20)
    ],

    FrameDelayMode [
        FRAMEDELAYMODE OFFSET(0) NUMBITS(2) [
            FreeRun = 0,
            Window = 1,
            ExactVal = 2,
            WindowGrid = 3
        ]
    ],

    Pointer [
        POINTER OFFSET(0) NUMBITS(32)
    ],

    MaxLen [
        LEN OFFSET(0) NUMBITS(9)
    ],

    TxdFrameConfig [
        PARITY OFFSET(0) NUMBITS(1) [
            NoParity = 0,
            Parity = 1
        ],
        DISCARDMODE OFFSET(1) NUMBITS(1) [
            DiscardEnd = 0,
            DiscardStart = 1
        ],
        SOF OFFSET(2) NUMBITS(1) [
            NoSoF = 0,
            SoF = 1
        ],
        CRCMODETX OFFSET(4) NUMBITS(1) [
            NoCRCTX = 0,
            CRC16TX = 1
        ]
    ],

    Amount [
        DATABITS OFFSET(0) NUMBITS(3),
        DATABYTES OFFSET(3) NUMBITS(9)
    ],

    RxdFrameConfig [
        PARITY OFFSET(0) NUMBITS(1) [
            NoParity = 0,
            Parity = 1
        ],
        SOF OFFSET(2) NUMBITS(1) [
            NoSoF = 0,
            SoF = 1
        ],
        CRCMODERX OFFSET(4) NUMBITS(1) [
            NoCRCRX = 0,
            CRC16RX = 1
        ]
    ],

    NfcIdPart3 [
        Z OFFSET(0) NUMBITS(8),
        Y OFFSET(8) NUMBITS(8),
        X OFFSET(16) NUMBITS(8),
        W OFFSET(24) NUMBITS(8)
    ],

    NfcIdPart2 [
        V OFFSET(0) NUMBITS(8),
        U OFFSET(8) NUMBITS(8),
        T OFFSET(16) NUMBITS(8)
    ],

    NfcIdPart1 [
        S OFFSET(0) NUMBITS(8),
        R OFFSET(8) NUMBITS(8),
        Q OFFSET(16) NUMBITS(8)
    ],

    AutoColConfig [
        MODE OFFSET(0) NUMBITS(1) [
            Enabled = 0,
            Disabled = 1
        ]
    ],

    SensRes [
        BITFRAMESDD OFFSET(0) NUMBITS(5) [
            SDD00000 = 0,
            SDD00001 = 1,
            SDD00010 = 2,
            SDD00100 = 4,
            SDD01000 = 8,
            SDD10000 = 16
        ],
        NFCIDSIZE OFFSET(6) NUMBITS(2) [
            NFCID1Single = 0,
            NFCID1Double = 1,
            NFCID1Triple = 2
        ],
        PLATFCONFIG OFFSET(8) NUMBITS(4) []
    ],

    SelRes [
        CASCADE OFFSET(2) NUMBITS(1) [],
        PROTOCOL OFFSET(5) NUMBITS(2) [
            Type2 = 0,
            Type4A = 1,
            NfcDep = 2,
            NfcDepAndType4A = 3
        ]
    ]
];

pub static mut NFCT: NfcTag = NfcTag::new();

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TagType {
    Type1,
    Type2,
    Type3,
    Type4,
    Type5,
    Unknown,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NfcState {
    Disabled,
    Initialized,
    Activated,
    Transmitting,
    Transmitted,
    Receiving,
    Received,
}

pub struct NfcTag<'a> {
    registers: StaticRef<NfctRegisters>,
    client: OptionalCell<&'a dyn hil::nfc::Client<'a>>,
    // To keep additional code-related states
    state: Cell<NfcState>,
    // For storing the buffers' references.
    tx_buffer: TakeCell<'static, [u8]>,
    rx_buffer: TakeCell<'static, [u8]>,
    tag_type: Cell<TagType>,
}

impl<'a> NfcTag<'a> {
    const fn new() -> Self {
        NfcTag {
            registers: NFCT_BASE,
            client: OptionalCell::empty(),
            state: Cell::new(NfcState::Disabled),
            tx_buffer: TakeCell::empty(),
            rx_buffer: TakeCell::empty(),
            tag_type: Cell::new(TagType::Unknown),
        }
    }

    /// Helper function to mask false bits
    fn mask_rx_bits(&self, buf: &mut [u8]) {
        let amount = self.registers.rxd_amount.read(Amount::DATABYTES) as usize;
        let bits = self.registers.rxd_amount.read(Amount::DATABITS);
        if bits == 0 {
            return;
        }
        if amount < buf.len() {
            // bit_and with 2^bits - 1
            buf[amount] &= (1 << bits) - 1;
        }
    }

    fn get_state(&self) -> NfcState {
        self.state.get()
    }

    fn get_rx_amount(&self) -> u32 {
        let rx_amount = self.registers.rxd_amount.read(Amount::DATABYTES)
            + match self.registers.rxd_amount.read(Amount::DATABITS) {
                0 => 0,
                _ => 1,
            };
        // If CRC bytes are counted
        if rx_amount > 2 {
            return rx_amount - 2;
        }
        rx_amount
    }

    fn field_check(&self) -> bool {
        if !self
            .registers
            .fieldpresent
            .is_set(FieldPresent::FIELDPRESENT)
            && !self.registers.fieldpresent.is_set(FieldPresent::LOCKDETECT)
        {
            // No active field
            return false;
        }
        true
    }

    fn handle_selected(&self) {
        self.state.set(NfcState::Activated);
        self.client.map(|client| client.tag_selected());
    }

    fn handle_rxend(&self) {
        self.state.set(NfcState::Received);
        // Return the buffer to the capsule
        self.client.map(|client| {
            self.rx_buffer.take().map(|rx_buffer| {
                let returncode = if self.registers.framestatus_rx.is_set(FrameStatus::OVERRUN) {
                    ReturnCode::ENOMEM
                } else if self.registers.framestatus_rx.is_set(FrameStatus::CRCERROR)
                    || self
                        .registers
                        .framestatus_rx
                        .is_set(FrameStatus::PARITYSTATUS)
                {
                    ReturnCode::FAIL
                } else {
                    ReturnCode::SUCCESS
                };
                let rx_amount = self.get_rx_amount() as usize;
                self.mask_rx_bits(rx_buffer);
                client.frame_received(rx_buffer, rx_amount, returncode);
            });
        });
        self.clear_errors();
    }

    fn handle_txend(&self) {
        self.state.set(NfcState::Transmitted);
        // Return the buffer to the capsule
        self.client.map(|client| {
            self.tx_buffer.take().map(|tx_buffer| {
                let returncode = if self
                    .registers
                    .errorstatus
                    .is_set(ErrorStatus::FRAMEDELAYTIMEOUT)
                {
                    ReturnCode::FAIL
                } else {
                    ReturnCode::SUCCESS
                };
                client.frame_transmitted(tx_buffer, returncode);
            });
        });
        self.clear_errors();
    }

    fn handle_field(&self, mut field_state: NfcFieldState) {
        if field_state == NfcFieldState::Unknown {
            field_state = if self.field_check() {
                NfcFieldState::On
            } else {
                NfcFieldState::Off
            };
        }

        match field_state {
            NfcFieldState::On => self.client.map(|client| client.field_detected()).unwrap(),
            NfcFieldState::Off => self
                .client
                .map(|client| client.field_lost(self.rx_buffer.take(), self.tx_buffer.take()))
                .unwrap(),
            _ => (),
        }
    }

    /// Helper function that clears TX/RX errors related registers.
    fn clear_errors(&self) {
        self.registers
            .errorstatus
            .set(self.registers.errorstatus.get());
        self.registers
            .framestatus_rx
            .set(self.registers.framestatus_rx.get());
    }

    pub fn handle_interrupt(&self) {
        let mut current_field = NfcFieldState::None;
        let saved_inter = self.registers.intenset.extract();
        self.disable_all_interrupts();

        let active_events = self.active_events();
        let events_to_process = saved_inter.bitand(active_events.get());
        if events_to_process.is_set(Interrupt::FIELDDETECTED) {
            current_field = NfcFieldState::On;
        }
        if events_to_process.is_set(Interrupt::FIELDLOST) {
            current_field = match current_field {
                NfcFieldState::None => NfcFieldState::Off,
                _ => NfcFieldState::Unknown,
            }
        }
        if current_field != NfcFieldState::None {
            self.handle_field(current_field);
        }

        if events_to_process.is_set(Interrupt::RXFRAMEEND) {
            self.handle_rxend();
        }
        if events_to_process.is_set(Interrupt::TXFRAMEEND) {
            self.handle_txend();
        }
        if events_to_process.is_set(Interrupt::SELECTED) {
            self.handle_selected();
        }
        // Ensure there are no spurious errors.
        self.clear_errors();
        self.enable_interrupts();
    }

    fn active_events(&self) -> InMemoryRegister<u32, Interrupt::Register> {
        let result = InMemoryRegister::new(0);
        if NfcTag::take_event(&self.registers.event_ready) {
            result.modify(Interrupt::READY::SET);
        }
        if NfcTag::take_event(&self.registers.event_fielddetected) {
            result.modify(Interrupt::FIELDDETECTED::SET);
        }
        if NfcTag::take_event(&self.registers.event_fieldlost) {
            result.modify(Interrupt::FIELDLOST::SET);
        }
        if NfcTag::take_event(&self.registers.event_txframestart) {
            result.modify(Interrupt::TXFRAMESTART::SET);
        }
        if NfcTag::take_event(&self.registers.event_txframeend) {
            result.modify(Interrupt::TXFRAMEEND::SET);
        }
        if NfcTag::take_event(&self.registers.event_rxframestart) {
            result.modify(Interrupt::RXFRAMESTART::SET);
        }
        if NfcTag::take_event(&self.registers.event_rxframeend) {
            result.modify(Interrupt::RXFRAMEEND::SET);
        }
        if NfcTag::take_event(&self.registers.event_error) {
            result.modify(Interrupt::ERROR::SET);
        }
        if NfcTag::take_event(&self.registers.event_rxerror) {
            result.modify(Interrupt::RXERROR::SET);
        }
        if NfcTag::take_event(&self.registers.event_endrx) {
            result.modify(Interrupt::ENDRX::SET);
        }
        if NfcTag::take_event(&self.registers.event_endtx) {
            result.modify(Interrupt::ENDTX::SET);
        }
        if NfcTag::take_event(&self.registers.event_autocolresstarted) {
            result.modify(Interrupt::AUTOCOLRESSTARTED::SET);
        }
        if NfcTag::take_event(&self.registers.event_collision) {
            result.modify(Interrupt::COLLISION::SET);
        }
        if NfcTag::take_event(&self.registers.event_selected) {
            result.modify(Interrupt::SELECTED::SET);
        }
        if NfcTag::take_event(&self.registers.event_started) {
            result.modify(Interrupt::STARTED::SET);
        }
        result
    }

    // Reads the status of an Event register and clears the register.
    // Returns the READY status.
    fn take_event(event: &ReadWrite<u32, Event::Register>) -> bool {
        let result = event.is_set(Event::GENERATED);
        if result {
            event.write(Event::GENERATED::CLEAR);
        }
        result
    }

    fn disable_all_interrupts(&self) {
        self.registers.intenclr.set(0xffffffff);
    }

    /// Enable the main event interrupts
    fn enable_interrupts(&self) {
        self.registers.intenset.write(
            Interrupt::SELECTED::SET + Interrupt::FIELDLOST::SET + Interrupt::FIELDDETECTED::SET,
        );
    }

    fn configure(&self) {
        self.clear_errors();
        self.registers
            .sensres
            .modify(SensRes::BITFRAMESDD::SDD00100);
        match self.tag_type.get() {
            TagType::Type4 => self.registers.selres.modify(SelRes::PROTOCOL::Type4A),
            _ => (),
        }
    }

    fn enable(&self) {
        self.registers.errata98.set(0x38148);
        self.registers
            .framedelay_mode
            .write(FrameDelayMode::FRAMEDELAYMODE::WindowGrid);
        self.registers.framedelay_max.set(0x1000);
        // TODO: Remove TASKS_ACTIVATE and Enable TASKS_SENSE instead.
        self.registers.task_activate.write(Task::ENABLE::Trigger);
        self.enable_interrupts();
        self.state.set(NfcState::Initialized);
    }

    fn disable(&self) {
        self.state.set(NfcState::Disabled);
        self.disable_all_interrupts();
        self.registers.task_disable.write(Task::ENABLE::Trigger);
    }

    pub fn set_dma_registers(&self, buffer: &[u8]) {
        let len: u32 = buffer.len() as u32;
        self.registers.packetptr.set(buffer.as_ptr() as u32);
        self.registers.maxlen.write(MaxLen::LEN.val(len));
        if self.get_state() == NfcState::Transmitting {
            self.registers
                .txd_amount
                .write(Amount::DATABYTES.val(len) + Amount::DATABITS::CLEAR);
        }
    }
}

impl<'a> hil::nfc::NfcTag<'a> for NfcTag<'a> {
    fn set_client(&self, client: &'a dyn hil::nfc::Client<'a>) {
        self.client.set(client);
    }

    fn enable(&self) {
        self.enable();
    }

    fn activate(&self) {
        self.enable();
        self.registers.task_activate.write(Task::ENABLE::Trigger);
    }

    fn deactivate(&self) {
        self.client.map(|client| client.tag_deactivated());
        self.disable();
        // TODO: Enable task sense when it's correctly configured.
    }

    fn transmit_buffer(
        &self,
        buf: &'static mut [u8],
        amount: usize,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])> {
        self.state.set(NfcState::Transmitting);
        self.set_dma_registers(&buf[..amount]);
        self.tx_buffer.replace(buf);
        self.registers.intenset.modify(Interrupt::TXFRAMEEND::SET);
        self.clear_errors();
        self.registers.task_starttx.write(Task::ENABLE::Trigger);
        Ok(amount)
    }

    fn receive_buffer(
        &self,
        buf: &'static mut [u8],
    ) -> Result<(), (ReturnCode, &'static mut [u8])> {
        self.state.set(NfcState::Receiving);
        self.set_dma_registers(buf);
        self.rx_buffer.replace(buf);
        self.registers.intenset.modify(Interrupt::RXFRAMEEND::SET);
        self.registers
            .task_enablerxdata
            .write(Task::ENABLE::Trigger);
        Ok(())
    }

    fn configure(&self, tag_type: u8) -> ReturnCode {
        match tag_type {
            4 => self.tag_type.set(TagType::Type4),
            _ => {
                debug!("No implementation for this tag type.");
                return ReturnCode::ENOSUPPORT;
            }
        }
        self.configure();
        ReturnCode::SUCCESS
    }

    fn set_framedelaymax(&self, max_delay: u32) {
        self.registers.framedelay_max.set(max_delay);
    }

    fn unmask_select(&self) {
        self.registers.intenset.write(Interrupt::SELECTED::SET);
    }
}
