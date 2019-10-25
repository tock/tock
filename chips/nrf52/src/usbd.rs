//! Universal Serial Bus Device with EasyDMA (USBD)

// TODO: implement the USB HIL and remove this unused warning.
#![allow(dead_code)]

use kernel::common::cells::VolatileCell;
use kernel::common::registers::{register_bitfields, ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;

const USBD_BASE: StaticRef<UsbdRegisters> =
    unsafe { StaticRef::new(0x40027000 as *const UsbdRegisters) };

const NUM_ENDPOINTS: usize = 8;

#[repr(C)]
struct ChipInfoRegisters {
    /// Undocumented register indicating the model of the chip
    /// - Address: 0x000 - 0x004
    chip_model: ReadOnly<u32, ChipModel::Register>,
    /// Undocumented register indicating the revision of the chip
    /// - Address: 0x004 - 0x008
    chip_revision: ReadOnly<u32, ChipRevision::Register>,
}

const CHIPINFO_BASE: StaticRef<ChipInfoRegisters> =
    unsafe { StaticRef::new(0x10000130 as *const ChipInfoRegisters) };

#[repr(C)]
struct UsbErrataRegisters {
    /// Undocumented register - Errata 171
    /// - Address: 0x000 - 0x004
    reg0: ReadWrite<u32>,
    _reserved1: [u32; 4],
    /// Undocumented register - Errata 171
    /// - Address: 0x014 - 0x018
    reg14: WriteOnly<u32>,
    _reserved2: [u32; 63],
    /// Undocumented register - Errata 187
    /// - Address: 0x114 - 0x118
    reg114: WriteOnly<u32>,
}

const USBERRATA_BASE: StaticRef<UsbErrataRegisters> =
    unsafe { StaticRef::new(0x4006EC00 as *const UsbErrataRegisters) };

#[repr(C)]
struct UsbdRegisters {
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
    epin: [detail::EndpointRegisters; NUM_ENDPOINTS],
    /// - Address: 0x6A0 - 0x6B4
    isoin: detail::EndpointRegisters,
    _reserved13: [u32; 19],
    /// - Address: 0x700 - 0x7A0
    epout: [detail::EndpointRegisters; NUM_ENDPOINTS],
    /// - Address: 0x7A0 - 0x7B4
    isoout: detail::EndpointRegisters,
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
    use kernel::common::cells::VolatileCell;
    use kernel::common::registers::{ReadOnly, ReadWrite};

    #[repr(C)]
    pub struct EndpointRegisters {
        ptr: VolatileCell<*const u8>,
        maxcnt: ReadWrite<u32, Count::Register>,
        amount: ReadOnly<u32, Amount::Register>,
        // padding
        _reserved: [u32; 2],
    }

    impl EndpointRegisters {
        pub fn set_buffer<'a>(&'a self, slice: &'a [VolatileCell<u8>]) {
            self.ptr.set(slice.as_ptr() as *const u8);
            self.maxcnt.write(Count::MAXCNT.val(slice.len() as u32));
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

pub struct Usbd {
    registers: StaticRef<UsbdRegisters>,
    // Stub for the USB device controller state.
}

impl Usbd {
    const fn new() -> Self {
        Usbd {
            registers: USBD_BASE,
        }
    }
}

pub static mut USBD: Usbd = Usbd::new();
