//! USB Client driver.

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, VolatileCell};
use kernel::common::registers::{
    register_bitfields, register_structs, LocalRegisterCopy, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
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
        (0x800 => buffer: [ReadWrite<u64, BUFFER::Register>; N_BUFFERS * 8]),
        (0x1000 => @END),
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
        DEVICE_ADDRESS OFFSET(16) NUMBITS(7) []
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
        SIZE OFFSET(8) NUMBITS(7) [],
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

// This is only useful for decoding the data from the buffer
// Don't use this to write.
register_bitfields![u64,
    BUFFER [
        REQUEST_TYPE OFFSET(0) NUMBITS(8) [],
        REQUEST OFFSET(8) NUMBITS(8) [],
        VALUE OFFSET(16) NUMBITS(16) [],
        INDEX OFFSET(32) NUMBITS(16) [],
        LENGTH OFFSET(48) NUMBITS(16) []
    ]
];

enum SetupRequest {
    GetStatus = 0,
    ClearFeature = 1,
    SetFeature = 3,
    SetAddress = 5,
    GetDescriptor = 6,
    SetDescriptor = 7,
    GetConfiguration = 8,
    SetConfiguration = 9,
    GetInterface = 10,
    SetInterface = 11,
    SynchFrame = 12,
    Unsupported = 100,
}

impl From<u32> for SetupRequest {
    fn from(num: u32) -> Self {
        match num {
            0 => SetupRequest::GetStatus,
            1 => SetupRequest::ClearFeature,
            3 => SetupRequest::SetFeature,
            5 => SetupRequest::SetAddress,
            6 => SetupRequest::GetDescriptor,
            7 => SetupRequest::SetDescriptor,
            8 => SetupRequest::GetConfiguration,
            9 => SetupRequest::SetConfiguration,
            10 => SetupRequest::GetInterface,
            11 => SetupRequest::SetInterface,
            12 => SetupRequest::SynchFrame,
            _ => SetupRequest::Unsupported,
        }
    }
}

/// State of the control endpoint (endpoint 0).
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CtrlState {
    /// Control endpoint is idle, and waiting for a command from the host.
    Init,
    /// Control endpoint has started an IN transfer.
    ReadIn,
    /// Control endpoint has moved to the status phase.
    ReadStatus,
    /// Control endpoint is handling a control write (OUT) transfer.
    WriteOut,
    /// Control endpoint needs to set the address in hardware
    SetAddress,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkInState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending IN packet transfer on this endpoint.
    In(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkOutState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending OUT packet in this endpoint's buffer, to be read by
    // the client application.
    OutDelay,
    // There is a pending EPDATA to reply to.
    OutData(usize),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InterruptState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending IN packet transfer on this endpoint.
    In(usize),
}

#[derive(Copy, Clone, Debug)]
pub enum EndpointState {
    Disabled,
    Ctrl(CtrlState),
    Bulk(Option<BulkInState>, Option<BulkOutState>),
    Interrupt(u32, InterruptState),
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
    addr: Cell<u16>,
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
            addr: Cell::new(0),
        }
    }

    fn get_state(&self) -> State {
        self.state.expect("get_state: state value is in use")
    }

    fn set_state(&self, state: State) {
        self.state.set(state);
    }

    fn get_transfer_type(&self, ep: usize) -> TransferType {
        match self.descriptors[ep].state.get() {
            EndpointState::Bulk(_, _) => TransferType::Bulk,
            EndpointState::Iso => TransferType::Isochronous,
            EndpointState::Interrupt(_, _) => TransferType::Interrupt,
            EndpointState::Ctrl(_) => TransferType::Control,
            EndpointState::Disabled => unreachable!(),
        }
    }

    fn disable_interrupts(&self) {
        self.registers.intr_enable.write(
            INTR::PKT_RECEIVED::CLEAR
                + INTR::PKT_SENT::CLEAR
                + INTR::DISCONNECTED::CLEAR
                + INTR::HOST_LOST::CLEAR
                + INTR::LINK_RESET::CLEAR
                + INTR::LINK_SUSPEND::CLEAR
                + INTR::LINK_RESUME::CLEAR
                + INTR::AV_EMPTY::CLEAR
                + INTR::RX_FULL::CLEAR
                + INTR::AV_OVERFLOW::CLEAR
                + INTR::LINK_IN_ERR::CLEAR
                + INTR::RX_CRC_ERR::CLEAR
                + INTR::RX_PID_ERR::CLEAR
                + INTR::RX_BITSTUFF_ERR::CLEAR
                + INTR::FRAME::CLEAR
                + INTR::CONNECTED::CLEAR,
        );
        self.registers.intr_state.set(0xFFFF_FFFF);
    }

    fn enable_interrupts(&self) {
        self.registers.intr_enable.write(
            INTR::PKT_RECEIVED::SET
                + INTR::PKT_SENT::SET
                + INTR::DISCONNECTED::SET
                + INTR::HOST_LOST::SET
                + INTR::LINK_RESET::SET
                + INTR::LINK_SUSPEND::SET
                + INTR::LINK_RESUME::SET
                + INTR::AV_EMPTY::SET
                + INTR::RX_FULL::SET
                + INTR::AV_OVERFLOW::SET
                + INTR::LINK_IN_ERR::SET
                + INTR::RX_CRC_ERR::SET
                + INTR::RX_PID_ERR::SET
                + INTR::RX_BITSTUFF_ERR::SET
                + INTR::FRAME::CLEAR
                + INTR::CONNECTED::SET,
        );
    }

    fn free_buffer(&self, buf_id: usize) {
        let mut bufs = self.bufs.get();

        for buf in bufs.iter_mut() {
            if buf.id == buf_id {
                buf.free = true;
                break;
            }
        }

        self.bufs.set(bufs);
    }

    fn stall(&self, endpoint: usize) {
        self.registers
            .stall
            .set(1 << endpoint | self.registers.stall.get());
    }

    fn copy_slice_out_to_hw(&self, ep: usize, buf_id: usize, size: usize) {
        // Get the slice
        let slice = self.descriptors[ep]
            .slice_out
            .expect("No OUT slice set for this descriptor");

        let mut slice_start = 0;

        for offset in 0..(size / 8) {
            let slice_end = (offset + 1) * 8;

            let mut to_write: u64 = 0;
            for (i, buf) in slice[slice_start..slice_end].iter().enumerate() {
                to_write |= (buf.get() as u64) << (i * 8);
            }

            // Write the data
            self.registers.buffer[(buf_id * 8) + offset].set(to_write);

            // Prepare for next loop
            slice_start = slice_end;
        }

        // Check if there is any remainder less then 8
        if slice_start < size {
            let mut to_write: u64 = 0;
            for (i, buf) in slice[slice_start..size].iter().enumerate() {
                to_write |= (buf.get() as u64) << (i * 8);
            }

            // Write the data
            self.registers.buffer[(buf_id * 8) + (slice_start / 8)].set(to_write);
        }

        self.registers.configin[ep].write(
            CONFIGIN::BUFFER.val(buf_id as u32)
                + CONFIGIN::SIZE.val(size as u32)
                + CONFIGIN::RDY::SET,
        );
    }

    fn copy_from_hw(&self, ep: usize, buf_id: usize, size: usize) {
        // Get the slice
        let slice = self.descriptors[ep]
            .slice_in
            .expect("No IN slice set for this descriptor");

        // Read the date to the buffer
        // TODO: Handle long packets
        for offset in 0..(size / 8) {
            let data = self.registers.buffer[(buf_id * 8) + offset].get();

            for (i, d) in data.to_ne_bytes().iter().enumerate() {
                slice[(offset * 8) + i].set(*d);
            }
        }
    }

    fn complete_ctrl_status(&self) {
        let endpoint = 0;

        self.client.map(|client| {
            client.ctrl_status(endpoint);
            client.ctrl_status_complete(endpoint);
            self.descriptors[endpoint]
                .state
                .set(EndpointState::Ctrl(CtrlState::Init));
        });
    }

    fn control_ep_receive(&self, ep: usize, buf_id: usize, size: u32, setup: u32) {
        let hw_buf = self.registers.buffer[buf_id * 8].extract();

        match self.descriptors[ep].state.get() {
            EndpointState::Disabled => unimplemented!(),
            EndpointState::Ctrl(state) => {
                let request_type = hw_buf.read(BUFFER::REQUEST_TYPE);
                let length = hw_buf.read(BUFFER::LENGTH);

                let ep_buf = &self.descriptors[ep].slice_out;
                let ep_buf = ep_buf.expect("No OUT slice set for this descriptor");
                if ep_buf.len() < 8 {
                    panic!("EP0 DMA buffer length < 8");
                }

                // Re-construct the SETUP packet from various registers. The
                // client's ctrl_setup() will parse it as a SetupData
                // descriptor.
                for (i, buf) in hw_buf.get().to_ne_bytes().iter().enumerate() {
                    ep_buf[i].set(*buf);
                }

                match state {
                    CtrlState::Init => {
                        if setup != 0 && size == 8 {
                            self.client.map(|client| {
                                // Notify the client that the ctrl setup event has occurred.
                                // Allow it to configure any data we need to send back.
                                match client.ctrl_setup(ep) {
                                    hil::usb::CtrlSetupResult::OkSetAddress => {
                                        self.descriptors[ep]
                                            .state
                                            .set(EndpointState::Ctrl(CtrlState::SetAddress));
                                    }
                                    hil::usb::CtrlSetupResult::Ok => {
                                        if length == 0 {
                                            self.copy_slice_out_to_hw(0, 0, 0);
                                            self.complete_ctrl_status();
                                        } else {
                                            let to_host = request_type & (1 << 7) != (1 << 7);
                                            if to_host {
                                                match client.ctrl_out(ep, hw_buf.get() as u32) {
                                                    hil::usb::CtrlOutResult::Ok => {
                                                        self.descriptors[ep].state.set(
                                                            EndpointState::Ctrl(
                                                                CtrlState::ReadStatus,
                                                            ),
                                                        );
                                                        self.copy_slice_out_to_hw(ep, buf_id, 0);
                                                    }
                                                    hil::usb::CtrlOutResult::Delay => {
                                                        unimplemented!()
                                                    }
                                                    hil::usb::CtrlOutResult::Halted => {
                                                        unimplemented!()
                                                    }
                                                }
                                            } else {
                                                match client.ctrl_in(ep) {
                                                    hil::usb::CtrlInResult::Packet(size, last) => {
                                                        if size == 0 {
                                                            panic!("Empty ctrl packet?");
                                                        }

                                                        self.copy_slice_out_to_hw(ep, buf_id, size);

                                                        if last {
                                                            self.descriptors[ep].state.set(
                                                                EndpointState::Ctrl(
                                                                    CtrlState::ReadStatus,
                                                                ),
                                                            );
                                                        } else {
                                                            self.descriptors[ep].state.set(
                                                                EndpointState::Ctrl(
                                                                    CtrlState::WriteOut,
                                                                ),
                                                            );
                                                        }
                                                    }
                                                    hil::usb::CtrlInResult::Delay => {
                                                        unimplemented!()
                                                    }
                                                    hil::usb::CtrlInResult::Error => unreachable!(),
                                                }
                                            }
                                        }
                                    }
                                    _err => {
                                        self.stall(ep);
                                        self.free_buffer(buf_id);
                                        self.descriptors[ep]
                                            .state
                                            .set(EndpointState::Ctrl(CtrlState::Init));
                                    }
                                };
                            });
                        }
                    }
                    CtrlState::ReadIn => {
                        self.copy_from_hw(ep, buf_id, size as usize);
                    }
                    CtrlState::ReadStatus => {
                        self.complete_ctrl_status();
                    }
                    CtrlState::WriteOut => unreachable!(),
                    CtrlState::SetAddress => unreachable!(),
                }
            }
            EndpointState::Bulk(_in_state, _out_state) => unimplemented!(),
            EndpointState::Iso => unimplemented!(),
            EndpointState::Interrupt(_, _) => unimplemented!(),
        }
    }

    fn ep_receive(&self, ep: usize, buf_id: usize, size: u32, _setup: u32) {
        let ep_buf = &self.descriptors[ep].slice_out;
        let ep_buf = ep_buf.expect("No OUT slice set for this descriptor");
        if ep_buf.len() < 8 {
            panic!("EP0 DMA buffer length < 8");
        }

        self.client.map(|client| {
            self.copy_from_hw(ep, buf_id, size as usize);
            let result = client.packet_out(self.get_transfer_type(ep), ep as usize, size);
            match self.descriptors[ep].state.get() {
                EndpointState::Disabled => unimplemented!(),
                EndpointState::Ctrl(_state) => unimplemented!(),
                EndpointState::Bulk(_in_state, _out_state) => {
                    let new_out_state = match result {
                        hil::usb::OutResult::Ok => BulkOutState::Init,

                        hil::usb::OutResult::Delay => BulkOutState::OutDelay,

                        hil::usb::OutResult::Error => BulkOutState::Init,
                    };
                    self.descriptors[ep]
                        .state
                        .set(EndpointState::Bulk(None, Some(new_out_state)));
                }
                EndpointState::Iso => unimplemented!(),
                EndpointState::Interrupt(_, _) => {}
            }
        });
    }

    pub fn handle_interrupt(&self) {
        let irqs = self.registers.intr_state.extract();

        // Disable interrupts
        self.disable_interrupts();

        if !self.registers.usbstat.is_set(USBSTAT::AV_FULL) {
            let mut bufs = self.bufs.get();

            for buf in bufs.iter_mut() {
                if !buf.free {
                    continue;
                }

                if self.registers.usbstat.is_set(USBSTAT::AV_FULL) {
                    break;
                }

                self.registers.avbuffer.set(buf.id as u32);
                buf.free = false;
            }

            self.bufs.set(bufs);
        }

        if irqs.is_set(INTR::FRAME) {
            for (ep, desc) in self.descriptors.iter().enumerate() {
                match desc.state.get() {
                    EndpointState::Disabled => {}
                    EndpointState::Ctrl(_) => {}
                    EndpointState::Bulk(_in_state, _out_state) => {}
                    EndpointState::Iso => {}
                    EndpointState::Interrupt(packet_size, state) => match state {
                        InterruptState::Init => {}
                        InterruptState::In(send_size) => {
                            let buf = self.registers.configin[ep as usize].read(CONFIGIN::BUFFER);
                            self.free_buffer(buf as usize);

                            self.client.map(|client| {
                                match client.packet_in(TransferType::Interrupt, ep as usize) {
                                    hil::usb::InResult::Packet(size) => {
                                        if size == 0 {
                                            panic!("Empty ctrl packet?");
                                        }

                                        self.copy_slice_out_to_hw(ep as usize, buf as usize, size);

                                        if send_size == size {
                                            self.descriptors[ep as usize].state.set(
                                                EndpointState::Interrupt(
                                                    packet_size,
                                                    InterruptState::Init,
                                                ),
                                            );
                                        } else {
                                            self.descriptors[ep as usize].state.set(
                                                EndpointState::Interrupt(
                                                    packet_size,
                                                    InterruptState::In(send_size - size),
                                                ),
                                            );
                                        }
                                    }
                                    hil::usb::InResult::Delay => unimplemented!(),
                                    hil::usb::InResult::Error => unreachable!(),
                                };
                            });
                        }
                    },
                }
            }
        }

        if irqs.is_set(INTR::PKT_SENT) {
            let mut in_sent = self.registers.in_sent.get();

            while in_sent != 0 {
                let ep = in_sent.trailing_zeros();

                // We are handling this case, clear it
                self.registers.in_sent.set(1 << ep);
                in_sent = in_sent & !(1 << ep);

                let buf = self.registers.configin[ep as usize].read(CONFIGIN::BUFFER);

                self.free_buffer(buf as usize);

                self.client.map(|client| {
                    client.packet_transmitted(ep as usize);
                });

                match self.descriptors[ep as usize].state.get() {
                    EndpointState::Disabled => unimplemented!(),
                    EndpointState::Ctrl(state) => match state {
                        CtrlState::Init => {}
                        CtrlState::ReadIn => {
                            unimplemented!();
                        }
                        CtrlState::ReadStatus => {
                            self.complete_ctrl_status();
                        }
                        CtrlState::WriteOut => {
                            self.client.map(|client| {
                                match client.ctrl_in(ep as usize) {
                                    hil::usb::CtrlInResult::Packet(size, last) => {
                                        if size == 0 {
                                            panic!("Empty ctrl packet?");
                                        }

                                        self.copy_slice_out_to_hw(ep as usize, buf as usize, size);

                                        if last {
                                            self.descriptors[ep as usize]
                                                .state
                                                .set(EndpointState::Ctrl(CtrlState::ReadStatus));
                                        } else {
                                            self.descriptors[ep as usize]
                                                .state
                                                .set(EndpointState::Ctrl(CtrlState::WriteOut));
                                        }
                                    }
                                    hil::usb::CtrlInResult::Delay => unimplemented!(),
                                    hil::usb::CtrlInResult::Error => unreachable!(),
                                };
                            });
                        }
                        CtrlState::SetAddress => {
                            self.registers
                                .usbctrl
                                .modify(USBCTRL::DEVICE_ADDRESS.val(self.addr.get() as u32));
                            self.descriptors[ep as usize]
                                .state
                                .set(EndpointState::Ctrl(CtrlState::Init));
                        }
                    },
                    EndpointState::Bulk(_in_state, _out_state) => {}
                    EndpointState::Iso => unimplemented!(),
                    EndpointState::Interrupt(_, _) => {}
                }
            }
        }

        if irqs.is_set(INTR::PKT_RECEIVED) {
            while !self.registers.usbstat.is_set(USBSTAT::RX_EMPTY) {
                let rxinfo = self.registers.rxfifo.extract();
                let buf = rxinfo.read(RXFIFO::BUFFER);
                let size = rxinfo.read(RXFIFO::SIZE);
                let ep = rxinfo.read(RXFIFO::EP);
                let setup = rxinfo.read(RXFIFO::SETUP);

                // Check if it's the control endpoint
                match ep {
                    0 => {
                        self.control_ep_receive(ep as usize, buf as usize, size, setup);
                        self.free_buffer(buf as usize);
                        break;
                    }
                    1..=7 => {
                        let receive_size = match self.descriptors[ep as usize].state.get() {
                            EndpointState::Disabled => size,
                            EndpointState::Ctrl(_state) => size,
                            EndpointState::Bulk(_in_state, _out_state) => size,
                            EndpointState::Iso => size,
                            EndpointState::Interrupt(packet_size, _state) => packet_size,
                        };
                        self.ep_receive(ep as usize, buf as usize, receive_size, setup);
                        self.free_buffer(buf as usize);
                        break;
                    }
                    8 => unimplemented!("isochronous endpoint"),
                    _ => unimplemented!(),
                }
            }
        }

        if irqs.is_set(INTR::LINK_RESET) {
            // The link was reset

            self.descriptors[0]
                .state
                .set(EndpointState::Ctrl(CtrlState::Init));
        }

        self.enable_interrupts();
    }

    fn transmit_in(&self, ep: usize) {
        self.client.map(|client| {
            let result = client.packet_in(self.get_transfer_type(ep), ep);

            let new_in_state = match result {
                hil::usb::InResult::Packet(size) => {
                    let mut buf_id = None;
                    let mut bufs = self.bufs.get();

                    for buf in bufs.iter_mut() {
                        if !buf.free {
                            continue;
                        }

                        self.registers.avbuffer.set(buf.id as u32);
                        buf.free = false;
                        buf_id = Some(buf.id);
                        break;
                    }

                    self.bufs.set(bufs);

                    if buf_id.is_some() {
                        self.copy_slice_out_to_hw(ep, buf_id.unwrap(), size)
                    } else {
                        panic!("No free bufs");
                    }
                    match self.descriptors[ep as usize].state.get() {
                        EndpointState::Disabled => unreachable!(),
                        EndpointState::Ctrl(_state) => unreachable!(),
                        EndpointState::Bulk(_in_state, _out_state) => {
                            EndpointState::Bulk(Some(BulkInState::In(size)), None)
                        }
                        EndpointState::Iso => unreachable!(),
                        EndpointState::Interrupt(packet_size, _state) => {
                            EndpointState::Interrupt(packet_size, InterruptState::In(size))
                        }
                    }
                }

                hil::usb::InResult::Delay => {
                    // No packet to send now. Wait for a resume call from the client.
                    match self.descriptors[ep as usize].state.get() {
                        EndpointState::Disabled => unreachable!(),
                        EndpointState::Ctrl(_state) => unreachable!(),
                        EndpointState::Bulk(_in_state, _out_state) => {
                            EndpointState::Bulk(Some(BulkInState::Init), None)
                        }
                        EndpointState::Iso => unreachable!(),
                        EndpointState::Interrupt(packet_size, _state) => {
                            EndpointState::Interrupt(packet_size, InterruptState::Init)
                        }
                    }
                }

                hil::usb::InResult::Error => {
                    self.stall(ep);
                    match self.descriptors[ep as usize].state.get() {
                        EndpointState::Disabled => unreachable!(),
                        EndpointState::Ctrl(_state) => unreachable!(),
                        EndpointState::Bulk(_in_state, _out_state) => {
                            EndpointState::Bulk(Some(BulkInState::Init), None)
                        }
                        EndpointState::Iso => unreachable!(),
                        EndpointState::Interrupt(packet_size, _state) => {
                            EndpointState::Interrupt(packet_size, InterruptState::Init)
                        }
                    }
                }
            };

            self.descriptors[ep].state.set(new_in_state);
        });
    }

    /// Provide a buffer for transfers in and out of the given endpoint
    /// (The controller need not be enabled before calling this method.)
    fn endpoint_bank_set_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        self.descriptors[endpoint].slice_in.set(buf);
        self.descriptors[endpoint].slice_out.set(buf);
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
            State::Reset => {
                self.registers.phy_config.write(
                    PHY_CONFIG::PINFLIP::CLEAR
                        + PHY_CONFIG::RX_DIFFERENTIAL_MODE::CLEAR
                        + PHY_CONFIG::TX_DIFFERENTIAL_MODE::CLEAR
                        + PHY_CONFIG::EOP_SINGLE_BIT::SET,
                );

                self.set_state(State::Idle(Mode::Device {
                    speed: speed,
                    config: DeviceConfig::default(),
                }))
            }
            _ => panic!("Already enabled"),
        }
    }

    fn attach(&self) {
        match self.get_state() {
            State::Reset => unreachable!("Not enabled"),
            State::Active(_) => unreachable!("Already attached"),
            State::Idle(mode) => {
                self.registers.usbctrl.write(USBCTRL::ENABLE::SET);

                self.enable_interrupts();

                self.set_state(State::Active(mode));
            }
        }
    }

    fn detach(&self) {
        unimplemented!()
    }

    fn set_address(&self, addr: u16) {
        self.addr.set(addr);

        self.copy_slice_out_to_hw(0, 0, 0);
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
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::Bulk(Some(BulkInState::Init), None));
            }
            TransferType::Interrupt => {
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::Interrupt(64, InterruptState::Init));
            }
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
                self.registers
                    .rxenable_setup
                    .set(1 << endpoint | self.registers.rxenable_setup.get());
                self.registers
                    .rxenable_out
                    .set(1 << endpoint | self.registers.rxenable_out.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::Bulk(None, Some(BulkOutState::Init)));
            }
            TransferType::Interrupt => {
                self.registers
                    .rxenable_out
                    .set(1 << endpoint | self.registers.rxenable_out.get());
                self.descriptors[endpoint]
                    .state
                    .set(EndpointState::Interrupt(64, InterruptState::Init));
            }
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

    fn endpoint_resume_in(&self, endpoint: usize) {
        self.transmit_in(endpoint)
    }

    fn endpoint_resume_out(&self, _endpoint: usize) {
        unimplemented!()
    }
}
