//! USB Client driver.

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, VolatileCell};
use kernel::common::registers::{
    register_bitfields, register_structs, LocalRegisterCopy, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::hil::usb::TransferType;

pub const N_ENDPOINTS: usize = 12;
pub const N_BUFFERS: usize = 32;

register_structs! {
    pub UsbRegisters {
        (0x000 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x004 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x008 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x00c => usbctrl: ReadWrite<u32, USBCTRL::Register>),
        (0x010 => usbstat: ReadOnly<u32, USBSTAT::Register>),
        (0x014 => avbuffer: WriteOnly<u32, AVBUFFER::Register>),
        (0x018 => rxfifo: ReadOnly<u32, RXFIFO::Register>),
        (0x01c => rxenable_setup: ReadWrite<u32, RXENABLE_SETUP::Register>),
        (0x020 => rxenable_out: ReadWrite<u32, RXENABLE_OUT::Register>),
        (0x024 => in_sent: ReadWrite<u32, IN_SENT::Register>),
        (0x028 => stall: ReadWrite<u32, STALL::Register>),
        (0x02c => configin: [ReadWrite<u32, CONFIGIN::Register>; N_ENDPOINTS]),
        (0x05c => iso: ReadWrite<u32, ISO::Register>),
        (0x060 => data_toggle_clear: WriteOnly<u32, DATA_TOGGLE_CLEAR::Register>),
        (0x064 => phy_config: ReadWrite<u32, PHY_CONFIG::Register>),
        (0x068 => _reserved0),
        (0x800 => buffer: [ReadWrite<u64, BUFFER::Register>; N_BUFFERS]),
        (0x900 => @END),
    }
}

register_bitfields![u32,
    INTR [
        PKT_RECEIVED OFFSET(0) NUMBITS(1) [],
        PKT_SENT OFFSET(1) NUMBITS(1) [],
        DISCONNECTED OFFSET(2) NUMBITS(1) [],
        HOST_LOST OFFSET(3) NUMBITS(1) [],
        LINK_RESET OFFSET(4) NUMBITS(1) [],
        LINK_SUSPEND OFFSET(5) NUMBITS(1) [],
        LINK_RESUME OFFSET(6) NUMBITS(1) [],
        AV_EMPTY OFFSET(7) NUMBITS(1) [],
        RX_FULL OFFSET(8) NUMBITS(1) [],
        AV_OVERFLOW OFFSET(9) NUMBITS(1) [],
        LINK_IN_ERR OFFSET(10) NUMBITS(1) [],
        RX_CRC_ERR OFFSET(11) NUMBITS(1) [],
        RX_PID_ERR OFFSET(12) NUMBITS(1) [],
        RX_BITSTUFF_ERR OFFSET(13) NUMBITS(1) [],
        FRAME OFFSET(14) NUMBITS(1) [],
        CONNECTED OFFSET(15) NUMBITS(1) []
    ],
    USBCTRL [
        ENABLE OFFSET(0) NUMBITS(1) [],
        DEVICE_ADDRESS OFFSET(16) NUMBITS(6) []
    ],
    USBSTAT [
        FRAME OFFSET(0) NUMBITS(10) [],
        HOST_LOST OFFSET(11) NUMBITS(1) [],
        LINK_STATE OFFSET(12) NUMBITS(2) [],
        SENSE OFFSET(15) NUMBITS(1) [],
        AV_DEPTH OFFSET(16) NUMBITS(2) [],
        AV_FULL OFFSET(23) NUMBITS(1) [],
        RX_DEPTH OFFSET(24) NUMBITS(2) [],
        RX_EMPTY OFFSET(31) NUMBITS(1) []
    ],
    AVBUFFER [
        BUFFER OFFSET(0) NUMBITS(4) []
    ],
    RXFIFO [
        BUFFER OFFSET(0) NUMBITS(4) [],
        SIZE OFFSET(8) NUMBITS(6) [],
        SETUP OFFSET(19) NUMBITS(1) [],
        EP OFFSET(20) NUMBITS(3) []
    ],
    RXENABLE_SETUP [
        SETUP0 OFFSET(0) NUMBITS(1) [],
        SETUP1 OFFSET(1) NUMBITS(1) [],
        SETUP2 OFFSET(2) NUMBITS(1) [],
        SETUP3 OFFSET(3) NUMBITS(1) [],
        SETUP4 OFFSET(4) NUMBITS(1) [],
        SETUP5 OFFSET(5) NUMBITS(1) [],
        SETUP6 OFFSET(6) NUMBITS(1) [],
        SETUP7 OFFSET(7) NUMBITS(1) [],
        SETUP8 OFFSET(8) NUMBITS(1) [],
        SETUP9 OFFSET(9) NUMBITS(1) [],
        SETUP10 OFFSET(10) NUMBITS(1) [],
        SETUP11 OFFSET(11) NUMBITS(1) []
    ],
    RXENABLE_OUT [
        OUT0 OFFSET(0) NUMBITS(1) [],
        OUT1 OFFSET(1) NUMBITS(1) [],
        OUT2 OFFSET(2) NUMBITS(1) [],
        OUT3 OFFSET(3) NUMBITS(1) [],
        OUT4 OFFSET(4) NUMBITS(1) [],
        OUT5 OFFSET(5) NUMBITS(1) [],
        OUT6 OFFSET(6) NUMBITS(1) [],
        OUT7 OFFSET(7) NUMBITS(1) [],
        OUT8 OFFSET(8) NUMBITS(1) [],
        OUT9 OFFSET(9) NUMBITS(1) [],
        OUT10 OFFSET(10) NUMBITS(1) [],
        OUT11 OFFSET(11) NUMBITS(1) []
    ],
    IN_SENT [
        SENT0 OFFSET(0) NUMBITS(1) [],
        SENT1 OFFSET(1) NUMBITS(1) [],
        SENT2 OFFSET(2) NUMBITS(1) [],
        SENT3 OFFSET(3) NUMBITS(1) [],
        SENT4 OFFSET(4) NUMBITS(1) [],
        SENT5 OFFSET(5) NUMBITS(1) [],
        SENT6 OFFSET(6) NUMBITS(1) [],
        SENT7 OFFSET(7) NUMBITS(1) [],
        SENT8 OFFSET(8) NUMBITS(1) [],
        SENT9 OFFSET(9) NUMBITS(1) [],
        SENT10 OFFSET(10) NUMBITS(1) [],
        SENT11 OFFSET(11) NUMBITS(1) []
    ],
    STALL [
        STALL0 OFFSET(0) NUMBITS(1) [],
        STALL1 OFFSET(1) NUMBITS(1) [],
        STALL2 OFFSET(2) NUMBITS(1) [],
        STALL3 OFFSET(3) NUMBITS(1) [],
        STALL4 OFFSET(4) NUMBITS(1) [],
        STALL5 OFFSET(5) NUMBITS(1) [],
        STALL6 OFFSET(6) NUMBITS(1) [],
        STALL7 OFFSET(7) NUMBITS(1) [],
        STALL8 OFFSET(8) NUMBITS(1) [],
        STALL9 OFFSET(9) NUMBITS(1) [],
        STALL10 OFFSET(10) NUMBITS(1) [],
        STALL11 OFFSET(11) NUMBITS(1) []
    ],
    CONFIGIN [
        BUFFER OFFSET(0) NUMBITS(4) [],
        SIZE OFFSET(8) NUMBITS(6) [],
        PEND OFFSET(30) NUMBITS(1) [],
        RDY OFFSET(31) NUMBITS(1) []
    ],
    ISO [
        ISO0 OFFSET(0) NUMBITS(1) [],
        ISO1 OFFSET(1) NUMBITS(1) [],
        ISO2 OFFSET(2) NUMBITS(1) [],
        ISO3 OFFSET(3) NUMBITS(1) [],
        ISO4 OFFSET(4) NUMBITS(1) [],
        ISO5 OFFSET(5) NUMBITS(1) [],
        ISO6 OFFSET(6) NUMBITS(1) [],
        ISO7 OFFSET(7) NUMBITS(1) [],
        ISO8 OFFSET(8) NUMBITS(1) [],
        ISO9 OFFSET(9) NUMBITS(1) [],
        ISO10 OFFSET(10) NUMBITS(1) [],
        ISO11 OFFSET(11) NUMBITS(1) []
    ],
    DATA_TOGGLE_CLEAR [
        CLEAR0 OFFSET(0) NUMBITS(1) [],
        CLEAR1 OFFSET(1) NUMBITS(1) [],
        CLEAR2 OFFSET(2) NUMBITS(1) [],
        CLEAR3 OFFSET(3) NUMBITS(1) [],
        CLEAR4 OFFSET(4) NUMBITS(1) [],
        CLEAR5 OFFSET(5) NUMBITS(1) [],
        CLEAR6 OFFSET(6) NUMBITS(1) [],
        CLEAR7 OFFSET(7) NUMBITS(1) [],
        CLEAR8 OFFSET(8) NUMBITS(1) [],
        CLEAR9 OFFSET(9) NUMBITS(1) [],
        CLEAR10 OFFSET(10) NUMBITS(1) [],
        CLEAR11 OFFSET(11) NUMBITS(1) []
    ],
    PHY_CONFIG [
        RX_DIFFERENTIAL_MODE OFFSET(0) NUMBITS(1) [],
        TX_DIFFERENTIAL_MODE OFFSET(1) NUMBITS(1) [],
        EOP_SINGLE_BIT OFFSET(2) NUMBITS(1) [],
        OVERRIDE_PWR_SENSE_EN OFFSET(3) NUMBITS(1) [],
        OVERRIDE_PWR_SENSE_VAL OFFSET(4) NUMBITS(1) [],
        PINFLIP OFFSET(5) NUMBITS(1) [],
        USB_REF_DISABLE OFFSET(6) NUMBITS(1) []
    ]
];

register_bitfields![u64,
    BUFFER [
        REQUEST_TYPE OFFSET(0) NUMBITS(8) [],
        REQUEST OFFSET(8) NUMBITS(8) [],
        VALUE OFFSET(16) NUMBITS(16) [],
        INDEX OFFSET(32) NUMBITS(16) [],
        LENGTH OFFSET(48) NUMBITS(16) []
    ]
];

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CtrlState {
    Init,
    ReadIn,
    ReadStatus,
    WriteOut,
    WriteStatus,
    WriteStatusWait,
    InDelay,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkInState {
    Init,
    Delay,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkOutState {
    Init,
    Delay,
}

#[derive(Copy, Clone, Debug)]
pub enum EndpointState {
    Disabled,
    Ctrl(CtrlState),
    BulkIn(BulkInState),
    BulkOut(BulkOutState),
    Iso,
}

type EndpointConfigValue = LocalRegisterCopy<u32, CONFIGIN::Register>;

#[derive(Copy, Clone, Debug, Default)]
pub struct DeviceConfig {
    pub endpoint_configs: [Option<EndpointConfigValue>; N_ENDPOINTS],
}

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Host,
    Device {
        speed: hil::usb::DeviceSpeed,
        config: DeviceConfig,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    // Controller disabled
    Reset,

    // Controller enabled, detached from bus
    // (We may go to this state when the Host
    // controller suspends the bus.)
    Idle(Mode),

    // Controller enabled, attached to bus
    Active(Mode),
}

pub enum BankIndex {
    Bank0,
    Bank1,
}

impl From<BankIndex> for usize {
    fn from(bi: BankIndex) -> usize {
        match bi {
            BankIndex::Bank0 => 0,
            BankIndex::Bank1 => 1,
        }
    }
}

#[repr(C)]
pub struct Endpoint<'a> {
    slice_in: OptionalCell<&'a [VolatileCell<u8>]>,
    slice_out: OptionalCell<&'a [VolatileCell<u8>]>,
    state: Cell<EndpointState>,

    _reserved: u32,
}

impl Endpoint<'_> {
    pub const fn new() -> Self {
        Endpoint {
            slice_in: OptionalCell::empty(),
            slice_out: OptionalCell::empty(),
            state: Cell::new(EndpointState::Disabled),
            _reserved: 0,
        }
    }
}

#[derive(Copy, Clone)]
struct Buffer {
    id: usize,
    free: bool,
}

impl Buffer {
    pub const fn new(id: usize) -> Self {
        Buffer { id, free: true }
    }
}

pub struct Usb<'a> {
    registers: StaticRef<UsbRegisters>,
    descriptors: [Endpoint<'a>; N_ENDPOINTS],
    client: OptionalCell<&'a dyn hil::usb::Client<'a>>,
    state: OptionalCell<State>,
    bufs: Cell<[Buffer; N_BUFFERS]>,
}

impl<'a> Usb<'a> {
    pub const fn new(base: StaticRef<UsbRegisters>) -> Self {
        Usb {
            registers: base,
            descriptors: [
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
            ],
            client: OptionalCell::empty(),
            state: OptionalCell::new(State::Reset),
            bufs: Cell::new([
                Buffer::new(0),
                Buffer::new(1),
                Buffer::new(2),
                Buffer::new(3),
                Buffer::new(4),
                Buffer::new(5),
                Buffer::new(6),
                Buffer::new(7),
                Buffer::new(8),
                Buffer::new(9),
                Buffer::new(10),
                Buffer::new(11),
                Buffer::new(12),
                Buffer::new(13),
                Buffer::new(14),
                Buffer::new(15),
                Buffer::new(16),
                Buffer::new(17),
                Buffer::new(18),
                Buffer::new(19),
                Buffer::new(20),
                Buffer::new(21),
                Buffer::new(22),
                Buffer::new(23),
                Buffer::new(24),
                Buffer::new(25),
                Buffer::new(26),
                Buffer::new(27),
                Buffer::new(28),
                Buffer::new(29),
                Buffer::new(30),
                Buffer::new(31),
            ]),
        }
    }

    fn get_state(&self) -> State {
        self.state.expect("get_state: state value is in use")
    }

    fn set_state(&self, state: State) {
        self.state.set(state);
    }

    pub fn handle_interrupt(&self) {
        debug!("USB IRQ");
    }

    /// Provide a buffer for transfers in and out of the given endpoint
    /// (The controller need not be enabled before calling this method.)
    fn endpoint_bank_set_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        self.descriptors[endpoint].slice_in.set(buf);
        self.descriptors[endpoint].slice_out.set(buf);
    }

    /// Enable the controller's clocks and interrupt and transition to Idle state
    fn _enable(&self, mode: Mode) {
        let regs = self.registers;

        match self.get_state() {
            State::Reset => {
                regs.rxenable_setup.write(RXENABLE_SETUP::SETUP0::SET);
                regs.rxenable_out.write(RXENABLE_OUT::OUT0::SET);

                regs.usbctrl.write(USBCTRL::ENABLE::SET);

                self.set_state(State::Idle(mode));
            }
            _ => panic!("Already enabled"),
        }
    }
}

impl<'a> hil::usb::UsbController<'a> for Usb<'a> {
    fn set_client(&self, client: &'a dyn hil::usb::Client<'a>) {
        self.client.set(client);
    }

    fn endpoint_set_ctrl_buffer(&self, buf: &'a [VolatileCell<u8>]) {
        self.endpoint_bank_set_buffer(0, buf);
    }

    fn endpoint_set_in_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        self.endpoint_bank_set_buffer(endpoint, buf);
    }

    fn endpoint_set_out_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        self.endpoint_bank_set_buffer(endpoint, buf);
    }

    fn enable_as_device(&self, speed: hil::usb::DeviceSpeed) {
        match self.get_state() {
            State::Reset => self._enable(Mode::Device {
                speed: speed,
                config: DeviceConfig::default(),
            }),
            _ => debug!("Already enabled"),
        }
    }

    fn attach(&self) {
        let regs = self.registers;

        match self.get_state() {
            State::Reset => unreachable!("Not enabled"),
            State::Active(_) => unreachable!("Already attached"),
            State::Idle(mode) => {
                regs.rxenable_setup.write(RXENABLE_SETUP::SETUP10::SET);
                regs.rxenable_out.write(RXENABLE_OUT::OUT0::SET);

                regs.usbctrl.write(USBCTRL::ENABLE::SET);

                self.set_state(State::Active(mode));
            }
        }
    }

    fn detach(&self) {
        unimplemented!()
    }

    fn set_address(&self, _addr: u16) {
        unimplemented!()
    }

    fn enable_address(&self) {
        unimplemented!()
    }

    fn endpoint_in_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::Ctrl(CtrlState::Init));
            }
            TransferType::Bulk => {
                // How is this different to control?
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::BulkIn(BulkInState::Init));
            }
            TransferType::Interrupt => unimplemented!(),
            TransferType::Isochronous => {
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.registers.iso.set(1 << endpoint);
                self.descriptors[endpoint].state.set(EndpointState::Iso);
            }
        };
    }

    fn endpoint_out_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.registers
                    .rxenable_out
                    .set(1 << endpoint | self.registers.rxenable_out.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::Ctrl(CtrlState::Init));
            }
            TransferType::Bulk => {
                // How is this different to control?
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.registers
                    .rxenable_out
                    .set(1 << endpoint | self.registers.rxenable_out.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::BulkOut(BulkOutState::Init));
            }
            TransferType::Interrupt => unimplemented!(),
            TransferType::Isochronous => {
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.registers
                    .rxenable_out
                    .set(1 << endpoint | self.registers.rxenable_out.get());
                self.registers.iso.set(1 << endpoint);
                self.descriptors[endpoint].state.set(EndpointState::Iso);
            }
        };
    }

    fn endpoint_in_out_enable(&self, _transfer_type: TransferType, _endpoint: usize) {
        unimplemented!()
    }

    fn endpoint_resume_in(&self, _endpoint: usize) {
        unimplemented!()
    }

    fn endpoint_resume_out(&self, _endpoint: usize) {
        unimplemented!()
    }
}
