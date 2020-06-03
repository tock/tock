//! Universal Serial Bus Device with EasyDMA (USBD)

use core::cell::Cell;
use cortexm4::support::atomic;
use kernel::common::cells::{OptionalCell, VolatileCell};
use kernel::common::registers::{
    register_bitfields, register_structs, Field, InMemoryRegister, LocalRegisterCopy, ReadOnly,
    ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::debug;
use kernel::hil;
use kernel::hil::usb::TransferType;

use crate::power;

// The following macros provide some diagnostics and panics(!)
// while this module is experimental and should eventually be removed or
// replaced with better error handling.
macro_rules! debug_events {
    [ $( $arg:expr ),+ ] => {
        {} // debug!($( $arg ),+);
    };
}

macro_rules! debug_tasks {
    [ $( $arg:expr ),+ ] => {
        {} // debug!($( $arg ),+);
    };
}

macro_rules! debug_packets {
    [ $( $arg:expr ),+ ] => {
        {} // debug!($( $arg ),+);
    };
}

macro_rules! debug_info {
    [ $( $arg:expr ),+ ] => {
        debug!($( $arg ),+);
    };
}

macro_rules! internal_warn {
    [ $( $arg:expr ),+ ] => {
        debug!($( $arg ),+);
    };
}

macro_rules! internal_err {
    [ $( $arg:expr ),+ ] => {
        panic!($( $arg ),+);
    };
}

const CHIPINFO_BASE: StaticRef<ChipInfoRegisters> =
    unsafe { StaticRef::new(0x10000130 as *const ChipInfoRegisters) };

const USBD_BASE: StaticRef<UsbdRegisters<'static>> =
    unsafe { StaticRef::new(0x40027000 as *const UsbdRegisters<'static>) };

const USBERRATA_BASE: StaticRef<UsbErrataRegisters> =
    unsafe { StaticRef::new(0x4006E000 as *const UsbErrataRegisters) };

const NUM_ENDPOINTS: usize = 8;

register_structs! {
    ChipInfoRegisters {
        /// Undocumented register indicating the model of the chip
        (0x000 => chip_model: ReadOnly<u32, ChipModel::Register>),
        /// Undocumented register indicating the revision of the chip
        /// - Address: 0x004 - 0x008
        (0x004 => chip_revision: ReadOnly<u32, ChipRevision::Register>),
        (0x008 => @END),
    },

    UsbErrataRegisters {
        (0x000 => _reserved0),
        /// Undocumented register - Errata 171
        (0xC00 => reg_c00: ReadWrite<u32>),
        (0xC04 => _reserved1),
        /// Undocumented register - Errata 171
        (0xC14 => reg_c14: WriteOnly<u32>),
        (0xC18 => _reserved2),
        /// Undocumented register - Errata 187
        (0xD14 => reg_d14: WriteOnly<u32>),
        (0xD18 => @END),
    }
}

#[repr(C)]
struct UsbdRegisters<'a> {
    _reserved1: [u32; 1],
    /// Captures the EPIN\[n\].PTR, EPIN\[n\].MAXCNT and EPIN\[n\].CONFIG
    /// registers values and enables endpoint IN not respond to traffic
    /// from host
    /// - Address: 0x004 - 0x024
    task_startepin: [WriteOnly<u32, Task::Register>; NUM_ENDPOINTS],
    /// Captures the ISOIN.PTR, ISOIN.MAXCNT and ISOIN.CONFIG registers values
    /// and enables sending data on iso endpoint
    /// - Address: 0x024 - 0x028
    task_startisoin: WriteOnly<u32, Task::Register>,
    /// Captures the EPOUT\[n\].PTR, EPOUT\[n\].MAXCNT and EPOUT\[n\].CONFIG
    /// registers values and enables endpoint IN n ot respond to traffic
    ///  from host
    /// - Address: 0x028 - 0x048
    task_startepout: [WriteOnly<u32, Task::Register>; NUM_ENDPOINTS],
    /// Captures the ISOOUT.PTR, ISOOUT.MAXCNT and ISOOUT.CONFIG registers
    /// values and enables receiving data on iso endpoint
    /// - Address: 0x048 - 0x04C
    task_startisoout: WriteOnly<u32, Task::Register>,
    /// Allows OUT data stage on control endpoint 0
    /// - Address: 0x04C - 0x050
    task_ep0rcvout: WriteOnly<u32, Task::Register>,
    /// Allows status stage on control endpoint 0
    /// - Address: 0x050 - 0x054
    task_ep0status: WriteOnly<u32, Task::Register>,
    /// STALLs data and status stage on control endpoint 0
    /// - Address: 0x054 - 0x058
    task_ep0stall: WriteOnly<u32, Task::Register>,
    /// Forces D+ and D-lines to the state defined in the DPDMVALUE register
    /// - Address: 0x058 - 0x05C
    task_dpdmdrive: WriteOnly<u32, Task::Register>,
    /// Stops forcing D+ and D- lines to any state (USB engine takes control)
    /// - Address: 0x05C - 0x060
    task_dpdmnodrive: WriteOnly<u32, Task::Register>,
    _reserved2: [u32; 40],
    /// Signals that a USB reset condition has been detected on the USB lines
    /// - Address: 0x100 - 0x104
    event_usbreset: ReadWrite<u32, Event::Register>,
    /// Confirms that the EPIN\[n\].PTR, EPIN\[n\].MAXCNT, EPIN\[n\].CONFIG,
    /// or EPOUT\[n\].PTR, EPOUT\[n\].MAXCNT and EPOUT\[n\].CONFIG
    /// registers have been captured on all endpoints reported in
    /// the EPSTATUS register
    /// - Address: 0x104 - 0x108
    event_started: ReadWrite<u32, Event::Register>,
    /// The whole EPIN\[n\] buffer has been consumed.
    /// The RAM buffer can be accessed safely by software.
    /// - Address: 0x108 - 0x128
    event_endepin: [ReadWrite<u32, Event::Register>; NUM_ENDPOINTS],
    /// An acknowledged data transfer has taken place on the control endpoint
    /// - Address: 0x128 - 0x12C
    event_ep0datadone: ReadWrite<u32, Event::Register>,
    /// The whole ISOIN buffer has been consumed.
    /// The RAM buffer can be accessed safely by software.
    /// - Address: 0x12C - 0x130
    event_endisoin: ReadWrite<u32, Event::Register>,
    /// The whole EPOUT\[n\] buffer has been consumed.
    /// The RAM buffer can be accessed safely by software.
    /// - Address: 0x130 - 0x150
    event_endepout: [ReadWrite<u32, Event::Register>; NUM_ENDPOINTS],
    /// The whole ISOOUT buffer has been consumed.
    /// The RAM buffer can be accessed safely by software.
    /// - Address: 0x150 - 0x154
    event_endisoout: ReadWrite<u32, Event::Register>,
    /// Signals that a SOF (start of frame) condition has been
    /// detected on the USB lines
    /// - Address: 0x154 - 0x158
    event_sof: ReadWrite<u32, Event::Register>,
    /// An event or an error not covered by specific events has occurred,
    /// check EVENTCAUSE register to find the cause
    /// - Address: 0x158 - 0x15C
    event_usbevent: ReadWrite<u32, Event::Register>,
    /// A valid SETUP token has been received (and acknowledged)
    /// on the control endpoint
    /// - Address: 0x15C - 0x160
    event_ep0setup: ReadWrite<u32, Event::Register>,
    /// A data transfer has occurred on a data endpoint,
    /// indicated by the EPDATASTATUS register
    /// - Address: 0x160 - 0x164
    event_epdata: ReadWrite<u32, Event::Register>,
    _reserved3: [u32; 39],
    /// Shortcut register
    /// - Address: 0x200 - 0x204
    shorts: ReadWrite<u32, Shorts::Register>,
    _reserved4: [u32; 63],
    /// Enable or disable interrupt
    /// - Address: 0x300 - 0x304
    inten: ReadWrite<u32, Interrupt::Register>,
    /// Enable interrupt
    /// - Address: 0x304 - 0x308
    intenset: ReadWrite<u32, Interrupt::Register>,
    /// Disable interrupt
    /// - Address: 0x308 - 0x30C
    intenclr: ReadWrite<u32, Interrupt::Register>,
    _reserved5: [u32; 61],
    /// Details on event that caused the USBEVENT even
    /// - Address: 0x400 - 0x404
    eventcause: ReadWrite<u32, EventCause::Register>,
    _reserved6: [u32; 7],
    /// IN\[n\] endpoint halted status.
    /// Can be used as is as response to a GetStatus() request to endpoint.
    /// - Address: 0x420 - 0x440
    halted_epin: [ReadOnly<u32, Halted::Register>; NUM_ENDPOINTS],
    _reserved7: [u32; 1],
    /// OUT\[n\] endpoint halted status.
    /// Can be used as is as response to a GetStatus() request to endpoint.
    /// - Address: 0x444 - 0x464
    halted_epout: [ReadOnly<u32, Halted::Register>; NUM_ENDPOINTS],
    _reserved8: [u32; 1],
    /// Provides information on which endpoint's EasyDMA
    /// registers have been captured
    /// - Address: 0x468 - 0x46C
    epstatus: ReadWrite<u32, EndpointStatus::Register>,
    /// Provides information on which endpoint(s) an acknowledged data
    /// transfer has occurred (EPDATA event)
    /// - Address: 0x46C - 0x470
    epdatastatus: ReadWrite<u32, EndpointStatus::Register>,
    /// Device USB address
    /// - Address: 0x470 - 0x474
    usbaddr: ReadOnly<u32, UsbAddress::Register>,
    _reserved9: [u32; 3],
    /// SETUP data, byte 0, bmRequestType
    /// - Address: 0x480 - 0x484
    bmrequesttype: ReadOnly<u32, RequestType::Register>,
    /// SETUP data, byte 1, bRequest
    /// - Address: 0x484 - 0x488
    brequest: ReadOnly<u32, Request::Register>,
    /// SETUP data, byte 2, wValue LSB
    /// - Address: 0x488 - 0x48C
    wvaluel: ReadOnly<u32, Byte::Register>,
    /// SETUP data, byte 3, wValue MSB
    /// - Address: 0x48C - 0x490
    wvalueh: ReadOnly<u32, Byte::Register>,
    /// SETUP data, byte 4, wIndex LSB
    /// - Address: 0x490 - 0x494
    windexl: ReadOnly<u32, Byte::Register>,
    /// SETUP data, byte 5, wIndex MSB
    /// - Address: 0x494 - 0x498
    windexh: ReadOnly<u32, Byte::Register>,
    /// SETUP data, byte 6, wLength LSB
    /// - Address: 0x498 - 0x49C
    wlengthl: ReadOnly<u32, Byte::Register>,
    /// SETUP data, byte 7, wLength MSB
    /// - Address: 0x49C - 0x4A0
    wlengthh: ReadOnly<u32, Byte::Register>,
    /// Amount of bytes received last in the data stage of
    /// this OUT\[n\] endpoint
    /// - Address: 0x4A0 - 0x4C0
    size_epout: [ReadWrite<u32, EndpointSize::Register>; NUM_ENDPOINTS],
    /// Amount of bytes received last on this iso OUT data endpoint
    /// - Address: 0x4C0 - 0x4C4
    size_iosout: ReadOnly<u32, IsoEndpointSize::Register>,
    _reserved10: [u32; 15],
    /// Enable USB
    /// - Address: 0x500 - 0x504
    enable: ReadWrite<u32, Usb::Register>,
    /// Control of the USB pull-up
    /// - Address: 0x504 - 0x508
    usbpullup: ReadWrite<u32, UsbPullup::Register>,
    /// State at which the DPDMDRIVE task will force D+ and D-.
    /// The DPDMNODRIVE task reverts the control of the lines
    /// to MAC IP (no forcing).
    /// - Address: 0x508 - 0x50C
    dpdmvalue: ReadWrite<u32, DpDmValue::Register>,
    /// Data toggle control and status
    /// - Address: 0x50C - 0x510
    dtoggle: ReadWrite<u32, Toggle::Register>,
    /// Endpoint IN enable
    /// - Address: 0x510 - 0x514
    epinen: ReadWrite<u32, EndpointEnable::Register>,
    /// Endpoint OUT enable
    /// - Address: 0x514 - 0x518
    epouten: ReadWrite<u32, EndpointEnable::Register>,
    /// STALL endpoints
    /// - Address: 0x518 - 0x51C
    epstall: WriteOnly<u32, EndpointStall::Register>,
    /// Controls the split of ISO buffers
    /// - Address: 0x51C - 0x520
    isosplit: ReadWrite<u32, IsoSplit::Register>,
    /// Returns the current value of the start of frame counter
    /// - Address: 0x520 - 0x524
    framecntr: ReadOnly<u32, FrameCounter::Register>,
    _reserved11: [u32; 2],
    /// Controls USBD peripheral low power mode during USB suspend
    /// - Address: 0x52C - 0x530
    lowpower: ReadWrite<u32, LowPower::Register>,
    /// Controls the response of the ISO IN endpoint to an IN token
    /// when no data is ready to be sent
    /// - Address: 0x530 - 0x534
    isoinconfig: ReadWrite<u32, IsoInConfig::Register>,
    _reserved12: [u32; 51],
    /// - Address: 0x600 - 0x6A0
    epin: [detail::EndpointRegisters<'a>; NUM_ENDPOINTS],
    /// - Address: 0x6A0 - 0x6B4
    isoin: detail::EndpointRegisters<'a>,
    _reserved13: [u32; 19],
    /// - Address: 0x700 - 0x7A0
    epout: [detail::EndpointRegisters<'a>; NUM_ENDPOINTS],
    /// - Address: 0x7A0 - 0x7B4
    isoout: detail::EndpointRegisters<'a>,
    _reserved14: [u32; 19],
    /// Errata 166 related register (ISO double buffering not functional)
    /// - Address: 0x800 - 0x804
    errata166_1: WriteOnly<u32>,
    /// Errata 166 related register (ISO double buffering not functional)
    /// - Address: 0x804 - 0x808
    errata166_2: WriteOnly<u32>,
    _reserved15: [u32; 261],
    /// Errata 199 related register (USBD cannot receive tasks during DMA)
    /// - Address: 0xC1C - 0xC20
    errata199: WriteOnly<u32>,
}

mod detail {
    use super::{Amount, Count};
    use core::marker::PhantomData;
    use kernel::common::cells::VolatileCell;
    use kernel::common::registers::{ReadOnly, ReadWrite};

    #[repr(C)]
    pub struct EndpointRegisters<'a> {
        ptr: VolatileCell<*const u8>,
        maxcnt: ReadWrite<u32, Count::Register>,
        amount: ReadOnly<u32, Amount::Register>,
        // padding
        _reserved: [u32; 2],
        // Lifetime marker.
        _phantom: PhantomData<&'a [u8]>,
    }

    impl<'a> EndpointRegisters<'a> {
        pub fn set_buffer(&self, slice: &'a [VolatileCell<u8>]) {
            self.ptr.set(slice.as_ptr() as *const u8);
            self.maxcnt.write(Count::MAXCNT.val(slice.len() as u32));
        }

        pub fn amount(&self) -> u32 {
            self.amount.get()
        }
    }
}

register_bitfields! [u32,
    /// Start task
    Task [
        ENABLE OFFSET(0) NUMBITS(1)
    ],

    /// Read event
    Event [
        READY OFFSET(0) NUMBITS(1)
    ],

    /// Shortcuts
    Shorts [
        // Shortcut between EP0DATADONE event and STARTEPIN[0] task
        EP0DATADONE_STARTEPIN0 OFFSET(0) NUMBITS(1),
        // Shortcut between EP0DATADONE event and STARTEPOUT[0] task
        EP0DATADONE_STARTEPOUT0 OFFSET(1) NUMBITS(1),
        // Shortcut between EP0DATADONE event and EP0STATUS task
        EP0DATADONE_EP0STATUS OFFSET(2) NUMBITS(1),
        // Shortcut between ENDEPOUT[0] event and EP0STATUS task
        ENDEPOUT0_EP0STATUS OFFSET(3) NUMBITS(1),
        // Shortcut between ENDEPOUT[0] event and EP0RCVOUT task
        ENDEPOUT0_EP0RCVOUT OFFSET(4) NUMBITS(1)
    ],

    /// USB Interrupts
    Interrupt [
        USBRESET OFFSET(0) NUMBITS(1),
        STARTED OFFSET(1) NUMBITS(1),
        ENDEPIN0 OFFSET(2) NUMBITS(1),
        ENDEPIN1 OFFSET(3) NUMBITS(1),
        ENDEPIN2 OFFSET(4) NUMBITS(1),
        ENDEPIN3 OFFSET(5) NUMBITS(1),
        ENDEPIN4 OFFSET(6) NUMBITS(1),
        ENDEPIN5 OFFSET(7) NUMBITS(1),
        ENDEPIN6 OFFSET(8) NUMBITS(1),
        ENDEPIN7 OFFSET(9) NUMBITS(1),
        EP0DATADONE OFFSET(10) NUMBITS(1),
        ENDISOIN OFFSET(11) NUMBITS(1),
        ENDEPOUT0 OFFSET(12) NUMBITS(1),
        ENDEPOUT1 OFFSET(13) NUMBITS(1),
        ENDEPOUT2 OFFSET(14) NUMBITS(1),
        ENDEPOUT3 OFFSET(15) NUMBITS(1),
        ENDEPOUT4 OFFSET(16) NUMBITS(1),
        ENDEPOUT5 OFFSET(17) NUMBITS(1),
        ENDEPOUT6 OFFSET(18) NUMBITS(1),
        ENDEPOUT7 OFFSET(19) NUMBITS(1),
        ENDISOOUT OFFSET(20) NUMBITS(1),
        SOF OFFSET(21) NUMBITS(1),
        USBEVENT OFFSET(22) NUMBITS(1),
        EP0SETUP OFFSET(23) NUMBITS(1),
        EPDATA OFFSET(24) NUMBITS(1)
    ],

    /// Cause of a USBEVENT event
    EventCause [
        ISOOUTCRC OFFSET(0) NUMBITS(1),
        SUSPEND OFFSET(8) NUMBITS(1),
        RESUME OFFSET(9) NUMBITS(1),
        USBWUALLOWED OFFSET(10) NUMBITS(1),
        READY OFFSET(11) NUMBITS(1)
    ],

    Halted [
        GETSTATUS OFFSET(0) NUMBITS(16) [
            NotHalted = 0,
            Halted = 1
        ]
    ],

    EndpointStatus [
        EPIN0 OFFSET(0) NUMBITS(1),
        EPIN1 OFFSET(1) NUMBITS(1),
        EPIN2 OFFSET(2) NUMBITS(1),
        EPIN3 OFFSET(3) NUMBITS(1),
        EPIN4 OFFSET(4) NUMBITS(1),
        EPIN5 OFFSET(5) NUMBITS(1),
        EPIN6 OFFSET(6) NUMBITS(1),
        EPIN7 OFFSET(7) NUMBITS(1),
        EPIN8 OFFSET(8) NUMBITS(1),
        EPOUT0 OFFSET(16) NUMBITS(1),
        EPOUT1 OFFSET(17) NUMBITS(1),
        EPOUT2 OFFSET(18) NUMBITS(1),
        EPOUT3 OFFSET(19) NUMBITS(1),
        EPOUT4 OFFSET(20) NUMBITS(1),
        EPOUT5 OFFSET(21) NUMBITS(1),
        EPOUT6 OFFSET(22) NUMBITS(1),
        EPOUT7 OFFSET(23) NUMBITS(1),
        EPOUT8 OFFSET(24) NUMBITS(1)
    ],

    UsbAddress [
        ADDR OFFSET(0) NUMBITS(7)
    ],

    RequestType [
        RECIPIENT OFFSET(0) NUMBITS(5) [
            Device = 0,
            Interface = 1,
            Endpoint = 2,
            Other = 3
        ],
        TYPE OFFSET(5) NUMBITS(2) [
            Standard = 0,
            Class = 1,
            Vendor = 2
        ],
        DIRECTION OFFSET(7) NUMBITS(1) [
            HostToDevice = 0,
            DeviceToHost = 1
        ]
    ],

    Request [
        BREQUEST OFFSET(0) NUMBITS(8) [
            STD_GET_STATUS = 0,
            STD_CLEAR_FEATURE = 1,
            STD_SET_FEATURE = 3,
            STD_SET_ADDRESS = 5,
            STD_GET_DESCRIPTOR = 6,
            STD_SET_DESCRIPTOR = 7,
            STD_GET_CONFIGURATION = 8,
            STD_SET_CONFIGURATION = 9,
            STD_GET_INTERFACE = 10,
            STD_SET_INTERFACE = 11,
            STD_SYNCH_FRAME = 12
        ]
    ],

    Byte [
        VALUE OFFSET(0) NUMBITS(8)
    ],

    EndpointSize [
        SIZE OFFSET(0) NUMBITS(7)
    ],

    IsoEndpointSize [
        SIZE OFFSET(0) NUMBITS(10),
        ZERO OFFSET(16) NUMBITS(1)
    ],

    /// Enable USB
    Usb [
        ENABLE OFFSET(0) NUMBITS(1) [
            OFF = 0,
            ON = 1
        ]
    ],

    UsbPullup [
        CONNECT OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    DpDmValue [
        STATE OFFSET(0) NUMBITS(5) [
            Resume = 1,
            J = 2,
            K = 4
        ]
    ],

    Toggle [
        EP OFFSET(0) NUMBITS(3) [],
        IO OFFSET(7) NUMBITS(1) [
            Out = 0,
            In = 1
        ],
        VALUE OFFSET(8) NUMBITS(2) [
            Nop = 0,
            Data0 = 1,
            Data1 = 2
        ]
    ],

    EndpointEnable [
        EP0 OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP1 OFFSET(1) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP2 OFFSET(2) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP3 OFFSET(3) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP4 OFFSET(4) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP5 OFFSET(5) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP6 OFFSET(6) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        EP7 OFFSET(7) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ],
        ISO OFFSET(8) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    EndpointStall [
        EP OFFSET(0) NUMBITS(3) [],
        IO OFFSET(7) NUMBITS(1) [
            Out = 0,
            In = 1
        ],
        STALL OFFSET(8) NUMBITS(1) [
            UnStall = 0,
            Stall = 1
        ]
    ],

    IsoSplit [
        SPLIT OFFSET(0) NUMBITS(16) [
            OneDir = 0x0000,
            HalfIN = 0x0080
        ]
    ],

    FrameCounter [
        FRAMECNTR OFFSET(0) NUMBITS(11)
    ],

    LowPower [
        LOWPOWER OFFSET(0) NUMBITS(1) [
            ForceNormal = 0,
            LowPower = 1
        ]
    ],

    IsoInConfig [
        RESPONSE OFFSET(0) NUMBITS(1) [
            NoResp = 0,
            ZeroData = 1
        ]
    ],

    Count [
        // 7 bits for a bulk endpoint but 10 bits for ISO EP
        MAXCNT OFFSET(0) NUMBITS(10)
    ],

    Amount [
        // 7 bits for a bulk endpoint but 10 bits for ISO EP
        AMOUNT OFFSET(0) NUMBITS(10)
    ],

    ChipModel [
        MODEL OFFSET(0) NUMBITS(32) [
            NRF52840 = 8
        ]
    ],

    ChipRevision [
        REV OFFSET(0) NUMBITS(32) [
            REVA = 0,
            REVB = 1,
            REVC = 2,
            REVD = 3
        ]
    ]
];

pub static mut USBD: Usbd<'static> = Usbd::new();

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UsbState {
    Disabled,
    Started,
    Initialized,
    PoweredOn,
    Attached,
    Configured,
}

#[derive(Copy, Clone, Debug)]
pub enum EndpointState {
    Disabled,
    Ctrl(CtrlState),
    Bulk(TransferType, Option<BulkInState>, Option<BulkOutState>),
}

impl EndpointState {
    fn ctrl_state(self) -> CtrlState {
        match self {
            EndpointState::Ctrl(state) => state,
            _ => panic!("Expected EndpointState::Ctrl"),
        }
    }

    fn bulk_state(self) -> (TransferType, Option<BulkInState>, Option<BulkOutState>) {
        match self {
            EndpointState::Bulk(transfer_type, in_state, out_state) => {
                (transfer_type, in_state, out_state)
            }
            _ => panic!("Expected EndpointState::Bulk"),
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
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkInState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending DMA transfer on this IN endpoint.
    InDma,
    // There is a pending IN packet transfer on this endpoint.
    InData,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BulkOutState {
    // The endpoint is ready to perform transactions.
    Init,
    // There is a pending OUT packet in this endpoint's buffer, to be read by
    // the client application.
    OutDelay,
    // There is a pending EPDATA to reply to.
    OutData,
    // There is a pending DMA transfer on this OUT endpoint.
    OutDma,
}

pub struct Endpoint<'a> {
    slice_in: OptionalCell<&'a [VolatileCell<u8>]>,
    slice_out: OptionalCell<&'a [VolatileCell<u8>]>,
    state: Cell<EndpointState>,
    // The USB controller can only process one DMA transfer at a time (over all endpoints). The
    // request_transmit_* bits allow to queue transfers until the DMA becomes available again.
    // Whether a DMA transfer is requested on this IN endpoint.
    request_transmit_in: Cell<bool>,
    // Whether a DMA transfer is requested on this OUT endpoint.
    request_transmit_out: Cell<bool>,
}

impl Endpoint<'_> {
    const fn new() -> Self {
        Endpoint {
            slice_in: OptionalCell::empty(),
            slice_out: OptionalCell::empty(),
            state: Cell::new(EndpointState::Disabled),
            request_transmit_in: Cell::new(false),
            request_transmit_out: Cell::new(false),
        }
    }
}

pub struct Usbd<'a> {
    registers: StaticRef<UsbdRegisters<'a>>,
    state: OptionalCell<UsbState>,
    dma_pending: Cell<bool>,
    client: OptionalCell<&'a dyn hil::usb::Client<'a>>,
    descriptors: [Endpoint<'a>; NUM_ENDPOINTS],
}

impl<'a> Usbd<'a> {
    const fn new() -> Self {
        Usbd {
            registers: USBD_BASE,
            client: OptionalCell::empty(),
            state: OptionalCell::new(UsbState::Disabled),
            dma_pending: Cell::new(false),
            descriptors: [
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
                Endpoint::new(),
            ],
        }
    }

    fn has_errata_166(&self) -> bool {
        true
    }

    fn has_errata_171(&self) -> bool {
        true
    }

    fn has_errata_187(&self) -> bool {
        CHIPINFO_BASE
            .chip_model
            .matches_all(ChipModel::MODEL::NRF52840)
            && match CHIPINFO_BASE.chip_revision.read_as_enum(ChipRevision::REV) {
                Some(ChipRevision::REV::Value::REVB)
                | Some(ChipRevision::REV::Value::REVC)
                | Some(ChipRevision::REV::Value::REVD) => true,
                Some(ChipRevision::REV::Value::REVA) | None => false,
            }
    }

    fn has_errata_199(&self) -> bool {
        true
    }

    /// ISO double buffering not functional
    fn apply_errata_166(&self) {
        if self.has_errata_166() {
            self.registers.errata166_1.set(0x7e3);
            self.registers.errata166_2.set(0x40);
        }
    }

    /// USBD might not reach its active state.
    fn apply_errata_171(&self, val: u32) {
        if self.has_errata_171() {
            unsafe {
                atomic(|| {
                    if USBERRATA_BASE.reg_c00.get() == 0 {
                        USBERRATA_BASE.reg_c00.set(0x9375);
                        USBERRATA_BASE.reg_c14.set(val);
                        USBERRATA_BASE.reg_c00.set(0x9375);
                    } else {
                        USBERRATA_BASE.reg_c14.set(val);
                    }
                });
            }
        }
    }

    /// USB cannot be enabled
    fn apply_errata_187(&self, val: u32) {
        if self.has_errata_187() {
            unsafe {
                atomic(|| {
                    if USBERRATA_BASE.reg_c00.get() == 0 {
                        USBERRATA_BASE.reg_c00.set(0x9375);
                        USBERRATA_BASE.reg_d14.set(val);
                        USBERRATA_BASE.reg_c00.set(0x9375);
                    } else {
                        USBERRATA_BASE.reg_d14.set(val);
                    }
                });
            }
        }
    }

    fn apply_errata_199(&self, val: u32) {
        if self.has_errata_199() {
            self.registers.errata199.set(val);
        }
    }

    pub fn get_state(&self) -> UsbState {
        self.state.expect("get_state: state value is in use")
    }

    // Powers the USB PHY on
    fn enable(&self) {
        if self.get_state() != UsbState::Disabled {
            internal_warn!("USBC is already enabled");
            return;
        }
        self.registers.eventcause.modify(EventCause::READY::CLEAR);
        self.apply_errata_187(3);
        self.apply_errata_171(0xc0);
        self.registers.enable.write(Usb::ENABLE::ON);
        while !self.registers.eventcause.is_set(EventCause::READY) {}
        self.registers.eventcause.modify(EventCause::READY::CLEAR);
        self.apply_errata_171(0);
        self.apply_errata_166();
        self.clear_pending_dma();
        self.state.set(UsbState::Initialized);
        self.apply_errata_187(0);
    }

    // TODO: unused function
    fn _suspend(&self) {
        debug_info!("usbc::suspend()");
        self.ep_abort_all();
        if self.registers.eventcause.is_set(EventCause::RESUME) {
            return;
        }
        self.enable_lowpower();
        if self.registers.eventcause.is_set(EventCause::RESUME) {
            self.disable_lowpower();
        } else {
            self.apply_errata_171(0);
        }
        internal_warn!("suspend() not fully implemented");
    }

    fn disable_all_interrupts(&self) {
        self.registers.intenclr.set(0xffffffff);
    }

    fn enable_interrupts(&self, inter: u32) {
        self.registers.inten.set(inter);
    }

    fn power_ready(&self) {
        match self.get_state() {
            UsbState::Disabled => {
                self.enable();
                self.state.set(UsbState::PoweredOn);
            }
            UsbState::Initialized => self.state.set(UsbState::PoweredOn),
            _ => (),
        }
    }

    fn enable_pullup(&self) {
        debug_info!("enable_pullup() - State={:?}", self.get_state());
        if self.get_state() == UsbState::Started {
            debug_info!("Enabling USB pullups");
            self.registers.usbpullup.write(UsbPullup::CONNECT::Enabled);
        }
        self.state.set(UsbState::Attached);
        debug_info!("New state is {:?}", self.get_state());
    }

    fn disable_pullup(&self) {
        debug_info!("Disabling USB pullup - State={:?}", self.get_state());
        self.registers.usbpullup.write(UsbPullup::CONNECT::Disabled);
        self.state.set(UsbState::Started);
        debug_info!("New state is {:?}", self.get_state());
    }

    // Allows the peripheral to be enumerated by the USB master
    fn start(&self) {
        debug_info!("usbc::start() - State={:?}", self.get_state());

        // Depending on the chip model, there are more or less errata to add to the code. To
        // simplify things, this implementation only includes errata relevant to nRF52840 chips
        // revisions >= C.
        //
        // If your chip isn't one of these, you will be alerted by these panics. You can disable
        // them but will likely need to add the relevant errata to this implementation (errata 104,
        // 154, 200).
        let chip_model = CHIPINFO_BASE.chip_model.get();
        if chip_model != u32::from(ChipModel::MODEL::NRF52840) {
            panic!(
                "USB was only tested on NRF52840. Your chip model is {}.",
                chip_model
            );
        }
        let chip_revision = CHIPINFO_BASE.chip_revision.extract();
        match chip_revision.read_as_enum(ChipRevision::REV) {
            Some(ChipRevision::REV::Value::REVA) | Some(ChipRevision::REV::Value::REVB) => {
                panic!(
                    "Errata for USB on NRF52840 chips revisions A and B are not implemented. Your chip revision is {}.",
                    chip_revision.get()
                );
            }
            Some(ChipRevision::REV::Value::REVC) | Some(ChipRevision::REV::Value::REVD) => {
                debug_info!(
                    "Your chip is NRF52840 revision {}. The USB stack was tested on your chip :)",
                    chip_revision.get()
                );
            }
            None => {
                internal_warn!(
                    "Your chip is NRF52840 revision {} (unknown revision). Although this USB implementation should be compatible, your chip hasn't been tested.",
                    chip_revision.get()
                );
            }
        }
        unsafe {
            if !power::POWER.is_vbus_present() {
                debug_info!("[!] VBUS power is not detected.");
                return;
            }
        }
        if self.get_state() == UsbState::Disabled {
            self.enable();
        }
        if self.get_state() != UsbState::PoweredOn {
            debug_info!("Waiting for power regulators...");
            unsafe { while power::POWER.is_vbus_present() && !power::POWER.is_usb_power_ready() {} }
        }
        debug_info!("usbc::start() - subscribing to interrupts.");
        self.registers.intenset.write(
            Interrupt::USBRESET::SET
                + Interrupt::STARTED::SET
                + Interrupt::ENDEPIN0::SET
                + Interrupt::EP0DATADONE::SET
                + Interrupt::ENDEPOUT0::SET
                + Interrupt::USBEVENT::SET
                + Interrupt::EP0SETUP::SET
                + Interrupt::EPDATA::SET,
        );
        self.state.set(UsbState::Started);
    }

    fn stop(&self) {
        debug_info!("usbc::stop() - State={:?}", self.get_state());
        if self.get_state() != UsbState::Started {
            return;
        }
        self.ep_abort_all();
        self.disable_all_interrupts();
        self.registers.usbpullup.write(UsbPullup::CONNECT::Disabled);
        self.state.set(UsbState::PoweredOn);
    }

    fn disable(&self) {
        debug_info!("usbc::disable() - State={:?}", self.get_state());
        self.stop();
        self.registers.enable.write(Usb::ENABLE::OFF);
        self.state.set(UsbState::Initialized);
        self.clear_pending_dma();
    }

    fn clear_pending_dma(&self) {
        debug_packets!("clear_pending_dma()");
        self.apply_errata_199(0);
        self.dma_pending.set(false);
    }

    fn set_pending_dma(&self) {
        debug_packets!("set_pending_dma()");
        if self.dma_pending.get() {
            internal_err!("Pending DMA already in flight");
        }
        self.apply_errata_199(0x82);
        self.dma_pending.set(true);
    }

    fn enable_in_endpoint_(&self, transfer_type: TransferType, endpoint: usize) {
        debug_info!(
            "enable_in_endpoint_({}), State={:?}",
            endpoint,
            self.get_state()
        );
        self.registers.intenset.write(match endpoint {
            0 => Interrupt::ENDEPIN0::SET,
            1 => Interrupt::ENDEPIN1::SET,
            2 => Interrupt::ENDEPIN2::SET,
            3 => Interrupt::ENDEPIN3::SET,
            4 => Interrupt::ENDEPIN4::SET,
            5 => Interrupt::ENDEPIN5::SET,
            6 => Interrupt::ENDEPIN6::SET,
            7 => Interrupt::ENDEPIN7::SET,
            8 => Interrupt::ENDISOIN::SET,
            _ => unreachable!("unexisting endpoint"),
        });
        self.registers.epinen.modify(match endpoint {
            0 => EndpointEnable::EP0::Enable,
            1 => EndpointEnable::EP1::Enable,
            2 => EndpointEnable::EP2::Enable,
            3 => EndpointEnable::EP2::Enable,
            4 => EndpointEnable::EP2::Enable,
            5 => EndpointEnable::EP2::Enable,
            6 => EndpointEnable::EP2::Enable,
            7 => EndpointEnable::EP2::Enable,
            8 => EndpointEnable::ISO::Enable,
            _ => unreachable!("unexisting endpoint"),
        });
        self.descriptors[endpoint].state.set(match endpoint {
            0 => EndpointState::Ctrl(CtrlState::Init),
            1..=7 => EndpointState::Bulk(transfer_type, Some(BulkInState::Init), None),
            8 => unimplemented!("isochronous endpoint"),
            _ => unreachable!("unexisting endpoint"),
        });
    }

    fn enable_out_endpoint_(&self, transfer_type: TransferType, endpoint: usize) {
        debug_info!(
            "enable_out_endpoint_({}) - State={:?}",
            endpoint,
            self.get_state()
        );
        self.registers.intenset.write(match endpoint {
            0 => Interrupt::ENDEPOUT0::SET,
            1 => Interrupt::ENDEPOUT1::SET,
            2 => Interrupt::ENDEPOUT2::SET,
            3 => Interrupt::ENDEPOUT3::SET,
            4 => Interrupt::ENDEPOUT4::SET,
            5 => Interrupt::ENDEPOUT5::SET,
            6 => Interrupt::ENDEPOUT6::SET,
            7 => Interrupt::ENDEPOUT7::SET,
            8 => Interrupt::ENDISOOUT::SET,
            _ => unreachable!("unexisting endpoint"),
        });
        self.registers.epouten.modify(match endpoint {
            0 => EndpointEnable::EP0::Enable,
            1 => EndpointEnable::EP1::Enable,
            2 => EndpointEnable::EP2::Enable,
            3 => EndpointEnable::EP2::Enable,
            4 => EndpointEnable::EP2::Enable,
            5 => EndpointEnable::EP2::Enable,
            6 => EndpointEnable::EP2::Enable,
            7 => EndpointEnable::EP2::Enable,
            8 => EndpointEnable::ISO::Enable,
            _ => unreachable!("unexisting endpoint"),
        });
        self.descriptors[endpoint].state.set(match endpoint {
            0 => EndpointState::Ctrl(CtrlState::Init),
            1..=7 => EndpointState::Bulk(transfer_type, None, Some(BulkOutState::Init)),
            8 => unimplemented!("isochronous endpoint"),
            _ => unreachable!("unexisting endpoint"),
        });
    }

    fn enable_in_out_endpoint_(&self, transfer_type: TransferType, endpoint: usize) {
        debug_info!(
            "enable_in_out_endpoint_({}) - State={:?}",
            endpoint,
            self.get_state()
        );
        self.registers.intenset.write(match endpoint {
            0 => Interrupt::ENDEPIN0::SET + Interrupt::ENDEPOUT0::SET,
            1 => Interrupt::ENDEPIN1::SET + Interrupt::ENDEPOUT1::SET,
            2 => Interrupt::ENDEPIN2::SET + Interrupt::ENDEPOUT2::SET,
            3 => Interrupt::ENDEPIN3::SET + Interrupt::ENDEPOUT3::SET,
            4 => Interrupt::ENDEPIN4::SET + Interrupt::ENDEPOUT4::SET,
            5 => Interrupt::ENDEPIN5::SET + Interrupt::ENDEPOUT5::SET,
            6 => Interrupt::ENDEPIN6::SET + Interrupt::ENDEPOUT6::SET,
            7 => Interrupt::ENDEPIN7::SET + Interrupt::ENDEPOUT7::SET,
            8 => Interrupt::ENDISOIN::SET + Interrupt::ENDISOOUT::SET,
            _ => unreachable!("unexisting endpoint"),
        });
        self.registers.epinen.modify(match endpoint {
            0 => EndpointEnable::EP0::Enable,
            1 => EndpointEnable::EP1::Enable,
            2 => EndpointEnable::EP2::Enable,
            3 => EndpointEnable::EP2::Enable,
            4 => EndpointEnable::EP2::Enable,
            5 => EndpointEnable::EP2::Enable,
            6 => EndpointEnable::EP2::Enable,
            7 => EndpointEnable::EP2::Enable,
            8 => EndpointEnable::ISO::Enable,
            _ => unreachable!("unexisting endpoint"),
        });
        self.registers.epouten.modify(match endpoint {
            0 => EndpointEnable::EP0::Enable,
            1 => EndpointEnable::EP1::Enable,
            2 => EndpointEnable::EP2::Enable,
            3 => EndpointEnable::EP2::Enable,
            4 => EndpointEnable::EP2::Enable,
            5 => EndpointEnable::EP2::Enable,
            6 => EndpointEnable::EP2::Enable,
            7 => EndpointEnable::EP2::Enable,
            8 => EndpointEnable::ISO::Enable,
            _ => unreachable!("unexisting endpoint"),
        });
        self.descriptors[endpoint].state.set(match endpoint {
            0 => EndpointState::Ctrl(CtrlState::Init),
            1..=7 => EndpointState::Bulk(
                transfer_type,
                Some(BulkInState::Init),
                Some(BulkOutState::Init),
            ),
            8 => unimplemented!("isochronous endpoint"),
            _ => unreachable!("unexisting endpoint"),
        });
    }

    fn ep_abort_all(&self) {
        internal_warn!("ep_abort_all() not implemented");
    }

    pub fn enable_lowpower(&self) {
        internal_warn!("enable_lowpower() not implemented");
    }

    pub fn disable_lowpower(&self) {
        internal_warn!("disable_lowpower() not implemented");
    }

    pub fn set_client(&self, client: &'a dyn hil::usb::Client<'a>) {
        self.client.set(client);
    }

    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;

        // Save then disable all interrupts.
        let saved_inter = regs.intenset.extract();
        self.disable_all_interrupts();

        let active_events = self.active_events(&saved_inter);
        let events_to_process = saved_inter.bitand(active_events.get());

        // The following order in which we test events is important.
        // Interrupts should be processed from bit 0 to bit 31 but EP0SETUP must be last.
        if events_to_process.is_set(Interrupt::USBRESET) {
            self.handle_usbreset();
        }
        if events_to_process.is_set(Interrupt::STARTED) {
            self.handle_started();
        }
        // Note: isochronous endpoint receives a dedicated ENDISOIN interrupt instead.
        for ep in 0..NUM_ENDPOINTS {
            if events_to_process.is_set(inter_endepin(ep)) {
                self.handle_endepin(ep);
            }
        }
        if events_to_process.is_set(Interrupt::EP0DATADONE) {
            self.handle_ep0datadone();
        }
        if events_to_process.is_set(Interrupt::ENDISOIN) {
            self.handle_endisoin();
        }
        // Note: isochronous endpoint receives a dedicated ENDISOOUT interrupt instead.
        for ep in 0..NUM_ENDPOINTS {
            if events_to_process.is_set(inter_endepout(ep)) {
                self.handle_endepout(ep);
            }
        }
        if events_to_process.is_set(Interrupt::ENDISOOUT) {
            self.handle_endisoout();
        }
        if events_to_process.is_set(Interrupt::SOF) {
            self.handle_sof();
        }
        if events_to_process.is_set(Interrupt::USBEVENT) {
            self.handle_usbevent();
        }
        if events_to_process.is_set(Interrupt::EPDATA) {
            self.handle_epdata();
        }

        self.process_dma_requests();

        // Setup packet received.
        // This event must be handled last, even though EPDATA is after.
        if events_to_process.is_set(Interrupt::EP0SETUP) {
            self.handle_ep0setup();
        }

        // Restore interrupts
        self.enable_interrupts(saved_inter.get());
    }

    fn active_events(
        &self,
        _saved_inter: &LocalRegisterCopy<u32, Interrupt::Register>,
    ) -> InMemoryRegister<u32, Interrupt::Register> {
        let regs = &*self.registers;

        let result = InMemoryRegister::new(0);
        if Usbd::take_event(&regs.event_usbreset) {
            debug_events!(
                "- event: usbreset{}",
                ignored_str(_saved_inter, Interrupt::USBRESET)
            );
            result.modify(Interrupt::USBRESET::SET);
        }
        if Usbd::take_event(&regs.event_started) {
            debug_events!(
                "- event: started{}",
                ignored_str(_saved_inter, Interrupt::STARTED)
            );
            result.modify(Interrupt::STARTED::SET);
        }
        for ep in 0..8 {
            if Usbd::take_event(&regs.event_endepin[ep]) {
                debug_events!(
                    "- event: endepin[{}]{}",
                    ep,
                    ignored_str(_saved_inter, inter_endepin(ep))
                );
                result.modify(inter_endepin(ep).val(1));
            }
        }
        if Usbd::take_event(&regs.event_ep0datadone) {
            debug_events!(
                "- event: ep0datadone{}",
                ignored_str(_saved_inter, Interrupt::EP0DATADONE)
            );
            result.modify(Interrupt::EP0DATADONE::SET);
        }
        if Usbd::take_event(&regs.event_endisoin) {
            debug_events!(
                "- event: endisoin{}",
                ignored_str(_saved_inter, Interrupt::ENDISOIN)
            );
            result.modify(Interrupt::ENDISOIN::SET);
        }
        for ep in 0..8 {
            if Usbd::take_event(&regs.event_endepout[ep]) {
                debug_events!(
                    "- event: endepout[{}]{}",
                    ep,
                    ignored_str(_saved_inter, inter_endepout(ep))
                );
                result.modify(inter_endepout(ep).val(1));
            }
        }
        if Usbd::take_event(&regs.event_endisoout) {
            debug_events!(
                "- event: endisoout{}",
                ignored_str(_saved_inter, Interrupt::ENDISOOUT)
            );
            result.modify(Interrupt::ENDISOOUT::SET);
        }
        if Usbd::take_event(&regs.event_sof) {
            debug_events!("- event: sof{}", ignored_str(_saved_inter, Interrupt::SOF));
            result.modify(Interrupt::SOF::SET);
        }
        if Usbd::take_event(&regs.event_usbevent) {
            debug_events!(
                "- event: usbevent{}",
                ignored_str(_saved_inter, Interrupt::USBEVENT)
            );
            result.modify(Interrupt::USBEVENT::SET);
        }
        if Usbd::take_event(&regs.event_ep0setup) {
            debug_events!(
                "- event: ep0setup{}",
                ignored_str(_saved_inter, Interrupt::EP0SETUP)
            );
            result.modify(Interrupt::EP0SETUP::SET);
        }
        if Usbd::take_event(&regs.event_epdata) {
            debug_events!(
                "- event: epdata{}",
                ignored_str(_saved_inter, Interrupt::EPDATA)
            );
            result.modify(Interrupt::EPDATA::SET);
        }
        result
    }

    // Reads the status of an Event register and clears the register.
    // Returns the READY status.
    fn take_event(event: &ReadWrite<u32, Event::Register>) -> bool {
        let result = event.is_set(Event::READY);
        if result {
            event.write(Event::READY::CLEAR);
        }
        result
    }

    fn handle_usbreset(&self) {
        let regs = &*self.registers;

        for (ep, desc) in self.descriptors.iter().enumerate() {
            match desc.state.get() {
                EndpointState::Disabled => {}
                EndpointState::Ctrl(_) => desc.state.set(EndpointState::Ctrl(CtrlState::Init)),
                EndpointState::Bulk(transfer_type, in_state, out_state) => {
                    desc.state.set(EndpointState::Bulk(
                        transfer_type,
                        in_state.map(|_| BulkInState::Init),
                        out_state.map(|_| BulkOutState::Init),
                    ));
                    if out_state.is_some() {
                        // Accept incoming OUT packets.
                        regs.size_epout[ep].set(0);
                    }
                }
            }
            // Clear the DMA status.
            desc.request_transmit_in.set(false);
            desc.request_transmit_out.set(false);
        }

        self.dma_pending.set(false);

        // TODO: reset controller stack
        self.client.map(|client| {
            client.bus_reset();
        });
    }

    fn handle_started(&self) {
        let regs = &*self.registers;

        let epstatus = regs.epstatus.extract();
        // Acknowledge the status by writing ones to the acknowledged bits.
        regs.epstatus.set(epstatus.get());
        debug_events!("epstatus: {:08X}", epstatus.get());

        // Nothing to do here, we just wait for the corresponding ENDEP* event.
    }

    fn handle_endepin(&self, endpoint: usize) {
        // Make DMA available again for other endpoints.
        self.clear_pending_dma();

        match endpoint {
            0 => {}
            1..=7 => {
                let (transfer_type, in_state, out_state) =
                    self.descriptors[endpoint].state.get().bulk_state();
                assert_eq!(in_state, Some(BulkInState::InDma));
                self.descriptors[endpoint].state.set(EndpointState::Bulk(
                    transfer_type,
                    Some(BulkInState::InData),
                    out_state,
                ));
            }
            8 => unimplemented!("isochronous endpoint"),
            _ => unreachable!("unexisting endpoint"),
        }

        // Nothing else to do. Wait for the EPDATA event.
    }

    /// Data has been sent over the USB bus, and the hardware has ACKed it.
    /// This is for the control endpoint only.
    fn handle_ep0datadone(&self) {
        let regs = &*self.registers;

        let endpoint = 0;
        let state = self.descriptors[endpoint].state.get().ctrl_state();
        match state {
            CtrlState::ReadIn => {
                self.transmit_in_ep0();
            }

            CtrlState::ReadStatus => {
                self.complete_ctrl_status();
            }

            CtrlState::WriteOut => {
                // We just completed the Setup stage for a CTRL WRITE transfer,
                // and the DMA has received data. Next step is to signal
                // `startepout[0]` to let the hardware move the data to RAM.
                debug_tasks!("- task: startepout[{}]", endpoint);
                regs.task_startepout[endpoint].write(Task::ENABLE::SET);
            }

            CtrlState::Init => {
                // We shouldn't be there. Let's STALL the endpoint.
                debug_tasks!("- task: ep0stall");
                regs.task_ep0stall.write(Task::ENABLE::SET);
            }
        }
    }

    fn handle_endisoin(&self) {
        unimplemented!("handle_endisoin");
    }

    fn handle_endepout(&self, endpoint: usize) {
        // Make DMA available again for other endpoints.
        self.clear_pending_dma();

        let regs = &*self.registers;

        match endpoint {
            0 => {
                // We got data on the control endpoint during a CTRL WRITE
                // transfer. Let the client handle the data, and then finish up
                // the control write by moving to the status stage.

                // Now we can handle it and pass it to the client to see
                // what the client returns.
                self.client.map(|client| {
                    match client.ctrl_out(endpoint, regs.epout[endpoint].amount()) {
                        hil::usb::CtrlOutResult::Ok => {
                            self.complete_ctrl_status();
                        }
                        hil::usb::CtrlOutResult::Delay => {}
                        _ => {
                            // Respond with STALL to any following transactions
                            // in this request
                            debug_tasks!("- task: ep0stall");
                            regs.task_ep0stall.write(Task::ENABLE::SET);
                            self.descriptors[endpoint]
                                .state
                                .set(EndpointState::Ctrl(CtrlState::Init));
                        }
                    };
                });
            }
            1..=7 => {
                // Notify the client about the new packet.
                let packet_bytes = regs.size_epout[endpoint].get();
                let (transfer_type, in_state, out_state) =
                    self.descriptors[endpoint].state.get().bulk_state();
                assert_eq!(out_state, Some(BulkOutState::OutDma));

                self.debug_out_packet(packet_bytes as usize, endpoint);

                self.client.map(|client| {
                    let result = client.packet_out(transfer_type, endpoint, packet_bytes);
                    debug_packets!("packet_out => {:?}", result);
                    let new_out_state = match result {
                        hil::usb::OutResult::Ok => {
                            // Indicate that the endpoint is ready to receive data again.
                            regs.size_epout[endpoint].set(0);
                            BulkOutState::Init
                        }

                        hil::usb::OutResult::Delay => {
                            // We can't send the packet now. Wait for a resume_out call from the client.
                            BulkOutState::OutDelay
                        }

                        hil::usb::OutResult::Error => {
                            regs.epstall.write(
                                EndpointStall::EP.val(endpoint as u32)
                                    + EndpointStall::IO::Out
                                    + EndpointStall::STALL::Stall,
                            );
                            BulkOutState::Init
                        }
                    };
                    self.descriptors[endpoint].state.set(EndpointState::Bulk(
                        transfer_type,
                        in_state,
                        Some(new_out_state),
                    ));
                });
            }
            8 => unimplemented!("isochronous endpoint"),
            _ => unreachable!("unexisting endpoint"),
        }
    }

    fn handle_endisoout(&self) {
        unimplemented!("handle_endisoout");
    }

    fn handle_sof(&self) {
        unimplemented!("handle_sof");
    }

    fn handle_usbevent(&self) {
        let regs = &*self.registers;

        let eventcause = regs.eventcause.extract();
        // Acknowledge the cause by writing ones to the acknowledged bits.
        regs.eventcause.set(eventcause.get());

        debug_events!("eventcause: {:08x}", eventcause.get());
        if eventcause.is_set(EventCause::ISOOUTCRC) {
            debug_events!("- usbevent: isooutcrc");
            internal_warn!("usbc::isooutcrc not implemented");
        }
        if eventcause.is_set(EventCause::SUSPEND) {
            debug_events!("- usbevent: suspend");
            internal_warn!("usbc::suspend not implemented");
        }
        if eventcause.is_set(EventCause::RESUME) {
            debug_events!("- usbevent: resume");
            internal_warn!("usbc::resume not implemented");
        }
        if eventcause.is_set(EventCause::USBWUALLOWED) {
            debug_events!("- usbevent: usbwuallowed");
            internal_warn!("usbc::usbwuallowed not implemented");
        }
        if eventcause.is_set(EventCause::READY) {
            debug_events!("- usbevent: ready");
            internal_warn!("usbc::ready not implemented");
        }
    }

    fn handle_epdata(&self) {
        let regs = &*self.registers;

        let epdatastatus = regs.epdatastatus.extract();
        // Acknowledge the status by writing ones to the acknowledged bits.
        regs.epdatastatus.set(epdatastatus.get());
        debug_events!("epdatastatus: {:08X}", epdatastatus.get());

        // Endpoint 0 (control) receives an EP0DATADONE event instead.
        // Endpoint 8 (isochronous) doesn't receive any EPDATA event.
        for endpoint in 1..NUM_ENDPOINTS {
            if epdatastatus.is_set(status_epin(endpoint)) {
                let (transfer_type, in_state, out_state) =
                    self.descriptors[endpoint].state.get().bulk_state();
                assert!(in_state.is_some());
                match in_state.unwrap() {
                    BulkInState::InData => {
                        // Totally expected state. Nothing to do.
                    }
                    BulkInState::Init => {
                        internal_warn!(
                            "Received a stale epdata IN in an unexpected state: {:?}",
                            in_state
                        );
                    }
                    BulkInState::InDma => {
                        internal_err!("Unexpected state: {:?}", in_state);
                    }
                }
                self.descriptors[endpoint].state.set(EndpointState::Bulk(
                    transfer_type,
                    Some(BulkInState::Init),
                    out_state,
                ));
                self.client
                    .map(|client| client.packet_transmitted(endpoint));
            }
        }

        // Endpoint 0 (control) receives an EP0DATADONE event instead.
        // Endpoint 8 (isochronous) doesn't receive any EPDATA event.
        for ep in 1..NUM_ENDPOINTS {
            if epdatastatus.is_set(status_epout(ep)) {
                let (transfer_type, in_state, out_state) =
                    self.descriptors[ep].state.get().bulk_state();
                assert!(out_state.is_some());
                match out_state.unwrap() {
                    BulkOutState::Init => {
                        // The endpoint is ready to receive data. Request a transmit_out.
                        self.descriptors[ep].request_transmit_out.set(true);
                    }
                    BulkOutState::OutDelay => {
                        // The endpoint will be resumed later by the client application with transmit_out().
                    }
                    BulkOutState::OutData | BulkOutState::OutDma => {
                        internal_err!("Unexpected state: {:?}", out_state);
                    }
                }
                // Indicate that the endpoint now has data available.
                self.descriptors[ep].state.set(EndpointState::Bulk(
                    transfer_type,
                    in_state,
                    Some(BulkOutState::OutData),
                ));
            }
        }
    }

    /// Handle the first event of a control transfer, the setup stage.
    fn handle_ep0setup(&self) {
        let regs = &*self.registers;

        let endpoint = 0;
        let state = self.descriptors[endpoint].state.get().ctrl_state();
        match state {
            CtrlState::Init => {
                // We are idle, and ready for any control transfer.

                let ep_buf = &self.descriptors[endpoint].slice_out;
                let ep_buf = ep_buf.expect("No OUT slice set for this descriptor");
                if ep_buf.len() < 8 {
                    panic!("EP0 DMA buffer length < 8");
                }

                // Re-construct the SETUP packet from various registers. The
                // client's ctrl_setup() will parse it as a SetupData
                // descriptor.
                ep_buf[0].set((regs.bmrequesttype.get() & 0xff) as u8);
                ep_buf[1].set((regs.brequest.get() & 0xff) as u8);
                ep_buf[2].set(regs.wvaluel.read(Byte::VALUE) as u8);
                ep_buf[3].set(regs.wvalueh.read(Byte::VALUE) as u8);
                ep_buf[4].set(regs.windexl.read(Byte::VALUE) as u8);
                ep_buf[5].set(regs.windexh.read(Byte::VALUE) as u8);
                ep_buf[6].set(regs.wlengthl.read(Byte::VALUE) as u8);
                ep_buf[7].set(regs.wlengthh.read(Byte::VALUE) as u8);
                let size = regs.wlengthl.read(Byte::VALUE) + (regs.wlengthh.read(Byte::VALUE) << 8);

                self.client.map(|client| {
                    // Notify the client that the ctrl setup event has occurred.
                    // Allow it to configure any data we need to send back.
                    match client.ctrl_setup(endpoint) {
                        hil::usb::CtrlSetupResult::OkSetAddress => {}
                        hil::usb::CtrlSetupResult::Ok => {
                            // Setup request is successful.
                            if size == 0 {
                                // Directly handle a 0 length setup request.
                                self.complete_ctrl_status();
                            } else {
                                match regs.bmrequesttype.read_as_enum(RequestType::DIRECTION) {
                                    Some(RequestType::DIRECTION::Value::HostToDevice) => {
                                        // CTRL WRITE transfer with data to
                                        // receive. We first need to setup DMA
                                        // so that the hardware can write the
                                        // data to us.
                                        self.descriptors[endpoint]
                                            .state
                                            .set(EndpointState::Ctrl(CtrlState::WriteOut));
                                        self.transmit_out_ep0();
                                    }
                                    Some(RequestType::DIRECTION::Value::DeviceToHost) => {
                                        self.descriptors[endpoint]
                                            .state
                                            .set(EndpointState::Ctrl(CtrlState::ReadIn));
                                        // Transmit first packet
                                        self.transmit_in_ep0();
                                    }
                                    None => unreachable!(),
                                }
                            }
                        }
                        _err => {
                            // An error occurred, we STALL
                            debug_tasks!("- task: ep0stall");
                            regs.task_ep0stall.write(Task::ENABLE::SET);
                        }
                    }
                });
            }

            CtrlState::ReadIn | CtrlState::ReadStatus | CtrlState::WriteOut => {
                // Unexpected state to receive a SETUP packet. Let's STALL the endpoint.
                internal_warn!("handle_ep0setup - unexpected state = {:?}", state);
                debug_tasks!("- task: ep0stall");
                regs.task_ep0stall.write(Task::ENABLE::SET);
            }
        }
    }

    fn complete_ctrl_status(&self) {
        let regs = &*self.registers;
        let endpoint = 0;

        self.client.map(|client| {
            client.ctrl_status(endpoint);
            debug_tasks!("- task: ep0status");
            regs.task_ep0status.write(Task::ENABLE::SET);
            client.ctrl_status_complete(endpoint);
            self.descriptors[endpoint]
                .state
                .set(EndpointState::Ctrl(CtrlState::Init));
        });
    }

    fn process_dma_requests(&self) {
        if self.dma_pending.get() {
            return;
        }

        for (endpoint, desc) in self.descriptors.iter().enumerate() {
            if desc.request_transmit_in.take() {
                self.transmit_in(endpoint);
                if self.dma_pending.get() {
                    break;
                }
            }
            if desc.request_transmit_out.take() {
                self.transmit_out(endpoint);
                if self.dma_pending.get() {
                    break;
                }
            }
        }
    }

    fn transmit_in_ep0(&self) {
        let regs = &*self.registers;
        let endpoint = 0;

        self.client.map(|client| {
            match client.ctrl_in(endpoint) {
                hil::usb::CtrlInResult::Packet(size, last) => {
                    if size == 0 {
                        internal_err!("Empty ctrl packet?");
                    }
                    self.start_dma_in(endpoint, size);
                    if last {
                        self.descriptors[endpoint]
                            .state
                            .set(EndpointState::Ctrl(CtrlState::ReadStatus));
                    }
                }

                hil::usb::CtrlInResult::Delay => {
                    internal_err!("Unexpected CtrlInResult::Delay");
                    // NAK is automatically sent by the modem.
                }

                hil::usb::CtrlInResult::Error => {
                    // An error occurred, we STALL
                    debug_tasks!("- task: ep0stall");
                    regs.task_ep0stall.write(Task::ENABLE::SET);
                }
            };
        });
    }

    /// Setup a reception for a CTRL WRITE transaction.
    ///
    /// All we have to do is configure DMA for a receive.
    fn transmit_out_ep0(&self) {
        let regs = &*self.registers;
        let endpoint = 0;

        let slice = self.descriptors[endpoint]
            .slice_out
            .expect("No OUT slice set for this descriptor");

        // Start DMA transfer
        self.set_pending_dma();
        regs.epout[endpoint].set_buffer(slice);

        // Run the ep0rcvout to signal to the hardware that the DMA is setup
        // and a buffer is ready.
        debug_tasks!("- task: ep0rcvout");
        regs.task_ep0rcvout.write(Task::ENABLE::SET);
    }

    fn transmit_in(&self, endpoint: usize) {
        debug_events!("transmit_in({})", endpoint);
        let regs = &*self.registers;

        self.client.map(|client| {
            let (transfer_type, in_state, out_state) =
                self.descriptors[endpoint].state.get().bulk_state();
            assert_eq!(in_state, Some(BulkInState::Init));

            let result = client.packet_in(transfer_type, endpoint);
            debug_packets!("packet_in => {:?}", result);
            let new_in_state = match result {
                hil::usb::InResult::Packet(size) => {
                    self.start_dma_in(endpoint, size);
                    BulkInState::InDma
                }

                hil::usb::InResult::Delay => {
                    // No packet to send now. Wait for a resume call from the client.
                    BulkInState::Init
                }

                hil::usb::InResult::Error => {
                    regs.epstall.write(
                        EndpointStall::EP.val(endpoint as u32)
                            + EndpointStall::IO::In
                            + EndpointStall::STALL::Stall,
                    );
                    BulkInState::Init
                }
            };

            self.descriptors[endpoint].state.set(EndpointState::Bulk(
                transfer_type,
                Some(new_in_state),
                out_state,
            ));
        });
    }

    fn transmit_out(&self, endpoint: usize) {
        debug_events!("transmit_out({})", endpoint);

        let (transfer_type, in_state, out_state) =
            self.descriptors[endpoint].state.get().bulk_state();
        // Starting the DMA can only happen in the OutData state, i.e. after an EPDATA event.
        assert_eq!(out_state, Some(BulkOutState::OutData));
        self.start_dma_out(endpoint);

        self.descriptors[endpoint].state.set(EndpointState::Bulk(
            transfer_type,
            in_state,
            Some(BulkOutState::OutDma),
        ));
    }

    fn start_dma_in(&self, endpoint: usize, size: usize) {
        let regs = &*self.registers;

        let slice = self.descriptors[endpoint]
            .slice_in
            .expect("No IN slice set for this descriptor");
        self.debug_in_packet(size, endpoint);

        // Start DMA transfer
        self.set_pending_dma();
        regs.epin[endpoint].set_buffer(&slice[..size]);
        debug_tasks!("- task: startepin[{}]", endpoint);
        regs.task_startepin[endpoint].write(Task::ENABLE::SET);
    }

    fn start_dma_out(&self, endpoint: usize) {
        let regs = &*self.registers;

        let slice = self.descriptors[endpoint]
            .slice_out
            .expect("No OUT slice set for this descriptor");

        // Start DMA transfer
        self.set_pending_dma();
        regs.epout[endpoint].set_buffer(slice);
        debug_tasks!("- task: startepout[{}]", endpoint);
        regs.task_startepout[endpoint].write(Task::ENABLE::SET);
    }

    // Debug-only function
    fn debug_in_packet(&self, size: usize, endpoint: usize) {
        let slice = self.descriptors[endpoint]
            .slice_in
            .expect("No IN slice set for this descriptor");
        if size > slice.len() {
            panic!("Packet is too large: {}", size);
        }

        let mut packet_hex = [0; 128];
        packet_to_hex(slice, &mut packet_hex);
        debug_packets!(
            "in={}",
            core::str::from_utf8(&packet_hex[..(2 * size)]).unwrap()
        );
    }

    // Debug-only function
    fn debug_out_packet(&self, size: usize, endpoint: usize) {
        let slice = self.descriptors[endpoint]
            .slice_out
            .expect("No OUT slice set for this descriptor");
        if size > slice.len() {
            panic!("Packet is too large: {}", size);
        }

        let mut packet_hex = [0; 128];
        packet_to_hex(slice, &mut packet_hex);
        debug_packets!(
            "out={}",
            core::str::from_utf8(&packet_hex[..(2 * size)]).unwrap()
        );
    }
}

impl<'a> power::PowerClient for Usbd<'a> {
    fn handle_power_event(&self, event: power::PowerEvent) {
        match event {
            power::PowerEvent::UsbPluggedIn => self.enable(),
            power::PowerEvent::UsbPluggedOut => self.disable(),
            power::PowerEvent::UsbPowerReady => self.power_ready(),
            _ => internal_warn!("usbc::handle_power_event: unknown power event"),
        }
    }
}

impl<'a> hil::usb::UsbController<'a> for Usbd<'a> {
    fn endpoint_set_ctrl_buffer(&self, buf: &'a [VolatileCell<u8>]) {
        if buf.len() < 8 {
            panic!("Endpoint buffer must be at least 8 bytes");
        }
        if !buf.len().is_power_of_two() {
            panic!("Buffer size must be a power of 2");
        }
        self.descriptors[0].slice_in.set(buf);
        self.descriptors[0].slice_out.set(buf);
    }

    fn endpoint_set_in_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        if buf.len() < 8 {
            panic!("Endpoint buffer must be at least 8 bytes");
        }
        if !buf.len().is_power_of_two() {
            panic!("Buffer size must be a power of 2");
        }
        if endpoint == 0 || endpoint >= NUM_ENDPOINTS {
            panic!("Endpoint number is invalid");
        }
        self.descriptors[endpoint].slice_in.set(buf);
    }

    fn endpoint_set_out_buffer(&self, endpoint: usize, buf: &'a [VolatileCell<u8>]) {
        if buf.len() < 8 {
            panic!("Endpoint buffer must be at least 8 bytes");
        }
        if !buf.len().is_power_of_two() {
            panic!("Buffer size must be a power of 2");
        }
        if endpoint == 0 || endpoint >= NUM_ENDPOINTS {
            panic!("Endpoint number is invalid");
        }
        self.descriptors[endpoint].slice_out.set(buf);
    }

    fn enable_as_device(&self, speed: hil::usb::DeviceSpeed) {
        match speed {
            hil::usb::DeviceSpeed::Low => internal_err!("Low speed is not supported"),
            hil::usb::DeviceSpeed::Full => {}
        }
        self.start();
    }

    fn attach(&self) {
        debug_info!("attach() - State={:?}", self.get_state());
        self.enable_pullup();
    }

    fn detach(&self) {
        debug_info!("detach() - Disabling pull-ups");
        self.disable_pullup();
    }

    fn set_address(&self, _addr: u16) {
        // Nothing to do, it's handled by PHY of nrf52 chip.
        debug_info!("Set Address = {}", _addr);
    }

    fn enable_address(&self) {
        let _regs = &*self.registers;
        debug_info!("Enable Address = {}", _regs.usbaddr.read(UsbAddress::ADDR));
        // Nothing to do, it's handled by PHY of nrf52 chip.
    }

    fn endpoint_in_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                panic!("There is no IN control endpoint");
            }
            TransferType::Bulk | TransferType::Interrupt => {
                if endpoint == 0 || endpoint >= NUM_ENDPOINTS {
                    panic!("Bulk/Interrupt endpoints are endpoints 1 to 7");
                }
                self.enable_in_endpoint_(transfer_type, endpoint);
            }
            TransferType::Isochronous => unimplemented!("isochronous endpoint"),
        }
    }

    fn endpoint_out_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                if endpoint != 0 {
                    panic!("Only endpoint 0 can be a control endpoint");
                }
                self.enable_out_endpoint_(transfer_type, endpoint);
            }
            TransferType::Bulk | TransferType::Interrupt => {
                if endpoint == 0 || endpoint >= NUM_ENDPOINTS {
                    panic!("Bulk/Interrupt endpoints are endpoints 1 to 7");
                }
                self.enable_out_endpoint_(transfer_type, endpoint);
            }
            TransferType::Isochronous => unimplemented!("isochronous endpoint"),
        }
    }

    fn endpoint_in_out_enable(&self, transfer_type: TransferType, endpoint: usize) {
        match transfer_type {
            TransferType::Control => {
                panic!("There is no IN control endpoint");
            }
            TransferType::Bulk | TransferType::Interrupt => {
                if endpoint == 0 || endpoint >= NUM_ENDPOINTS {
                    panic!("Bulk/Interrupt endpoints are endpoints 1 to 7");
                }
                self.enable_in_out_endpoint_(transfer_type, endpoint);
            }
            TransferType::Isochronous => unimplemented!("isochronous endpoint"),
        }
    }

    fn endpoint_resume_in(&self, endpoint: usize) {
        debug_events!("endpoint_resume_in({})", endpoint);

        let (_, in_state, _) = self.descriptors[endpoint].state.get().bulk_state();
        assert!(in_state.is_some());

        if self.dma_pending.get() {
            debug_events!("requesting resume_in[{}]", endpoint);
            // A DMA is already pending. Schedule the resume for later.
            self.descriptors[endpoint].request_transmit_in.set(true);
        } else {
            // Trigger the transaction now.
            self.transmit_in(endpoint);
        }
    }

    fn endpoint_resume_out(&self, endpoint: usize) {
        debug_events!("endpoint_resume_out({})", endpoint);

        let (transfer_type, in_state, out_state) =
            self.descriptors[endpoint].state.get().bulk_state();
        assert!(out_state.is_some());

        match out_state.unwrap() {
            BulkOutState::OutDelay => {
                // The endpoint has now finished processing the last ENDEPOUT. No EPDATA event
                // happened in the meantime, so the state is now back to Init.
                self.descriptors[endpoint].state.set(EndpointState::Bulk(
                    transfer_type,
                    in_state,
                    Some(BulkOutState::Init),
                ));
            }
            BulkOutState::OutData => {
                // Although the client reported a delay before, an EPDATA event has
                // happened in the meantime. This pending transaction will now
                // continue in transmit_out().
                if self.dma_pending.get() {
                    debug_events!("requesting resume_out[{}]", endpoint);
                    // A DMA is already pending. Schedule the resume for later.
                    self.descriptors[endpoint].request_transmit_out.set(true);
                } else {
                    // Trigger the transaction now.
                    self.transmit_out(endpoint);
                }
            }
            BulkOutState::Init | BulkOutState::OutDma => {
                internal_err!("Unexpected state: {:?}", out_state);
            }
        }
    }
}

fn status_epin(ep: usize) -> Field<u32, EndpointStatus::Register> {
    match ep {
        0 => EndpointStatus::EPIN0,
        1 => EndpointStatus::EPIN1,
        2 => EndpointStatus::EPIN2,
        3 => EndpointStatus::EPIN3,
        4 => EndpointStatus::EPIN4,
        5 => EndpointStatus::EPIN5,
        6 => EndpointStatus::EPIN6,
        7 => EndpointStatus::EPIN7,
        8 => EndpointStatus::EPIN8,
        _ => unreachable!(),
    }
}

fn status_epout(ep: usize) -> Field<u32, EndpointStatus::Register> {
    match ep {
        0 => EndpointStatus::EPOUT0,
        1 => EndpointStatus::EPOUT1,
        2 => EndpointStatus::EPOUT2,
        3 => EndpointStatus::EPOUT3,
        4 => EndpointStatus::EPOUT4,
        5 => EndpointStatus::EPOUT5,
        6 => EndpointStatus::EPOUT6,
        7 => EndpointStatus::EPOUT7,
        8 => EndpointStatus::EPOUT8,
        _ => unreachable!(),
    }
}

fn inter_endepin(ep: usize) -> Field<u32, Interrupt::Register> {
    match ep {
        0 => Interrupt::ENDEPIN0,
        1 => Interrupt::ENDEPIN1,
        2 => Interrupt::ENDEPIN2,
        3 => Interrupt::ENDEPIN3,
        4 => Interrupt::ENDEPIN4,
        5 => Interrupt::ENDEPIN5,
        6 => Interrupt::ENDEPIN6,
        7 => Interrupt::ENDEPIN7,
        _ => unreachable!(),
    }
}

fn inter_endepout(ep: usize) -> Field<u32, Interrupt::Register> {
    match ep {
        0 => Interrupt::ENDEPOUT0,
        1 => Interrupt::ENDEPOUT1,
        2 => Interrupt::ENDEPOUT2,
        3 => Interrupt::ENDEPOUT3,
        4 => Interrupt::ENDEPOUT4,
        5 => Interrupt::ENDEPOUT5,
        6 => Interrupt::ENDEPOUT6,
        7 => Interrupt::ENDEPOUT7,
        _ => unreachable!(),
    }
}

// Debugging functions.
fn packet_to_hex(packet: &[VolatileCell<u8>], packet_hex: &mut [u8]) {
    let hex_char = |x: u8| {
        if x < 10 {
            b'0' + x
        } else {
            b'a' + x - 10
        }
    };

    for (i, x) in packet.iter().enumerate() {
        let x = x.get();
        packet_hex[2 * i] = hex_char(x >> 4);
        packet_hex[2 * i + 1] = hex_char(x & 0x0f);
    }
}

#[allow(dead_code)]
fn ignored_str(
    saved_inter: &LocalRegisterCopy<u32, Interrupt::Register>,
    field: Field<u32, Interrupt::Register>,
) -> &'static str {
    if saved_inter.is_set(field) {
        ""
    } else {
        " (ignored)"
    }
}
