use kernel::common::VolatileCell;
use nrf5x;

pub const UARTE_BASE: u32 = 0x40002000;
#[repr(C)]
pub struct UARTE {
    pub task_startrx: VolatileCell<u32>,              // 0x000-0x004
    pub task_stoprx: VolatileCell<u32>,               // 0x004-0x008
    pub task_starttx: VolatileCell<u32>,              // 0x008-0x00c
    pub task_stoptx: VolatileCell<u32>,               // 0x00c-0x010
    _reserved1: [u32; 7],                             // 0x010-0x02c
    pub task_flush_rx: VolatileCell<u32>,             // 0x02c-0x030
    _reserved2: [u32; 52],                            // 0x030-0x100
    pub event_cts: VolatileCell<u32>,                 // 0x100-0x104
    pub event_ncts: VolatileCell<u32>,                // 0x104-0x108
    _reserved3: [u32; 2],                             // 0x108-0x110
    pub event_endrx: VolatileCell<u32>,               // 0x110-0x114
    _reserved4: [u32; 3],                             // 0x114-0x120
    pub event_endtx: VolatileCell<u32>,               // 0x120-0x124
    pub event_error: VolatileCell<u32>,               // 0x124-0x128
    _reserved6: [u32; 7],                             // 0x128-0x144
    pub event_rxto: VolatileCell<u32>,                // 0x144-0x148
    _reserved7: [u32; 1],                             // 0x148-0x14C
    pub event_rxstarted: VolatileCell<u32>,           // 0x14C-0x150
    pub event_txstarted: VolatileCell<u32>,           // 0x150-0x154
    _reserved8: [u32; 1],                             // 0x154-0x158
    pub event_txstopped: VolatileCell<u32>,           // 0x158-0x15c
    _reserved9: [u32; 41],                            // 0x15c-0x200
    pub shorts: VolatileCell<u32>,                    // 0x200-0x204
    _reserved10: [u32; 64],                           // 0x204-0x304
    pub intenset: VolatileCell<u32>,                  // 0x304-0x308
    pub intenclr: VolatileCell<u32>,                  // 0x308-0x30C
    _reserved11: [u32; 93],                           // 0x30C-0x480
    pub errorsrc: VolatileCell<u32>,                  // 0x480-0x484
    _reserved12: [u32; 31],                           // 0x484-0x500
    pub enable: VolatileCell<u32>,                    // 0x500-0x504
    _reserved13: [u32; 1],                            // 0x504-0x508
    pub pselrts: VolatileCell<nrf5x::pinmux::Pinmux>, // 0x508-0x50c
    pub pseltxd: VolatileCell<nrf5x::pinmux::Pinmux>, // 0x50c-0x510
    pub pselcts: VolatileCell<nrf5x::pinmux::Pinmux>, // 0x510-0x514
    pub pselrxd: VolatileCell<nrf5x::pinmux::Pinmux>, // 0x514-0x518
    _reserved14: [u32; 3],                            // 0x518-0x524
    pub baudrate: VolatileCell<u32>,                  // 0x524-0x528
    _reserved15: [u32; 3],                            // 0x528-0x534
    pub rxd_ptr: VolatileCell<u32>,                   // 0x534-0x538
    pub rxd_maxcnt: VolatileCell<u32>,                // 0x538-0x53c
    pub rxd_amount: VolatileCell<u32>,                // 0x53c-0x540
    _reserved16: [u32; 1],                            // 0x540-0x544
    pub txd_ptr: VolatileCell<u32>,                   // 0x544-0x548
    pub txd_maxcnt: VolatileCell<u32>,                // 0x548-0x54c
    pub txd_amount: VolatileCell<u32>,                // 0x54c-0x550
    _reserved17: [u32; 7],                            // 0x550-0x56C
    pub config: VolatileCell<u32>,                    // 0x56C-0x570
}

pub const UICR_BASE: usize = 0x10001200;
#[repr(C)]
pub struct UICR {
    pub pselreset0: VolatileCell<u32>, // 0x200 - 0x204
    pub pselreset1: VolatileCell<u32>, // 0x204 - 0x208
    pub approtect: VolatileCell<u32>,  // 0x208 - 0x20c
    pub nfcpins: VolatileCell<u32>,    // 0x20c - 0x210
}

pub const NVMC_BASE: usize = 0x4001E400;
#[repr(C)]
pub struct NVMC {
    pub ready: VolatileCell<u32>,        // 0x400-0x404
    _reserved1: [VolatileCell<u32>; 64], // 0x404-0x504
    pub config: VolatileCell<u32>,       //0x504-0x508
    pub erasepage: VolatileCell<u32>,    //0x508-0x50c
    pub erasepcr0: VolatileCell<u32>,    // 0x50c-0x510
    pub eraseuicr: VolatileCell<u32>,    // 0x510-0x514
    _reserved2: [VolatileCell<u32>; 11], // 0x514-0x540
    pub icachecnf: VolatileCell<u32>,    //0x540-0x544
    _reserved3: [VolatileCell<u32>; 1],  //0x544-0x548
    pub ihit: VolatileCell<u32>,         //0x548-0x54c
    pub imiss: VolatileCell<u32>,        //0x54c-0x550
}

pub const RADIO_BASE: usize = 0x40001000;
#[allow(non_snake_case)]
#[repr(C)]
pub struct RADIO {
    pub task_txen: VolatileCell<u32>,      // 0x000 - 0x004
    pub task_rxen: VolatileCell<u32>,      // 0x004 - 0x008
    pub task_start: VolatileCell<u32>,     // 0x008 - 0x00c
    pub task_stop: VolatileCell<u32>,      // 0x00c - 0x010
    pub task_disable: VolatileCell<u32>,   // 0x010 - 0x014
    pub task_rssistart: VolatileCell<u32>, // 0x014 - 0x018
    pub task_rssistop: VolatileCell<u32>,  // 0x018 - 0x01c
    pub task_bcstart: VolatileCell<u32>,   // 0x01c - 0x020
    pub task_bcstop: VolatileCell<u32>,    // 0x020 - 0x024
    _reserved1: [u32; 55],                 // 0x024 - 0x100
    pub event_ready: VolatileCell<u32>,    // 0x100 - 0x104
    pub event_address: VolatileCell<u32>,  // 0x104 - 0x108
    pub event_payload: VolatileCell<u32>,  // 0x108 - 0x10c
    pub event_end: VolatileCell<u32>,      // 0x10c - 0x110
    pub event_disabled: VolatileCell<u32>, // 0x110 - 0x114
    pub event_devmatch: VolatileCell<u32>, // 0x114 - 0x118
    pub event_devmiss: VolatileCell<u32>,  // 0x118 - 0x11c
    pub event_rssiend: VolatileCell<u32>,  // 0x11c - 0x120
    _reserved2: [u32; 2],                  // 0x120 - 0x128
    pub bcmatch: VolatileCell<u32>,        // 0x128 - 0x12c
    _reserved3: [u32; 1],                  // 0x12c - 0x130
    pub crcok: VolatileCell<u32>,          // 0x130 - 0x134
    pub crcerror: VolatileCell<u32>,       // 0x134 - 0x138
    _reserved4: [u32; 50],                 // 0x138 - 0x200
    pub shorts: VolatileCell<u32>,         // 0x200 - 0x204
    _reserved5: [u32; 64],                 // 0x204 - 0x304
    pub intenset: VolatileCell<u32>,       // 0x304 - 0x308
    pub intenclr: VolatileCell<u32>,       // 0x308 - 0x30c
    _reserved6: [u32; 61],                 // 0x30c - 0x400
    pub crcstatus: VolatileCell<u32>,      // 0x400 - 0x404
    _reserved7: [u32; 1],                  // 0x404 - 0x408
    pub rxmatch: VolatileCell<u32>,        // 0x408 - 0x40c
    pub rxcrc: VolatileCell<u32>,          // 0x40c - 0x410
    pub dai: VolatileCell<u32>,            // 0x410 - 0x414
    _reserved8: [u32; 60],                 // 0x414 - 0x504
    pub packetptr: VolatileCell<u32>,      // 0x504 - 0x508
    pub frequency: VolatileCell<u32>,      // 0x508 - 0x50c
    pub txpower: VolatileCell<u32>,        // 0x50c - 0x510
    pub mode: VolatileCell<u32>,           // 0x510 - 0x514
    pub pcnf0: VolatileCell<u32>,          // 0x514 - 0x518
    pub pcnf1: VolatileCell<u32>,          // 0x518 - 0x51c
    pub base0: VolatileCell<u32>,          // 0x51c - 0x520
    pub base1: VolatileCell<u32>,          // 0x520 - 0x524
    pub prefix0: VolatileCell<u32>,        // 0x524 - 0x528
    pub prefix1: VolatileCell<u32>,        // 0x528 - 0x52c
    pub txaddress: VolatileCell<u32>,      // 0x52c - 0x530
    pub rxaddresses: VolatileCell<u32>,    // 0x530 - 0x534
    pub crccnf: VolatileCell<u32>,         // 0x534 - 0x538
    pub crcpoly: VolatileCell<u32>,        // 0x538 - 0x53c
    pub crcinit: VolatileCell<u32>,        // 0x53c - 0x540
    _reserved9: [u32; 1],                  // 0x540 - 0x544
    pub tifs: VolatileCell<u32>,           // 0x544 - 0x548
    pub rssisample: VolatileCell<u32>,     // 0x548 - 0x54c
    _reserved10: [u32; 1],                 // 0x54c - 0x550
    pub state: VolatileCell<u32>,          // 0x550 - 0x554
    pub datawhiteiv: VolatileCell<u32>,    // 0x554 - 0x558
    _reserved11: [u32; 2],                 // 0x558 - 0x560
    pub bcc: VolatileCell<u32>,            // 0x560 - 0x564
    _reserved12: [u32; 39],                // 0x564 - 0x600
    pub dab: [VolatileCell<u32>; 8],       // 0x600 - 0x620
    pub dap: [VolatileCell<u32>; 8],       // 0x620 - 0x640
    pub dacnf: VolatileCell<u32>,          // 0x640 - 0x644
    _reserved13: [u32; 3],                 // 0x644 - 0x650
    pub modecnf0: VolatileCell<u32>,       // 0x650 - 0x654
    _reserved14: [u32; 618],               // 0x654 - 0xFFC
    pub power: VolatileCell<u32>,          // 0xFFC - 0x1000
}
