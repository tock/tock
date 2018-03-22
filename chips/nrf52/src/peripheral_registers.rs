use kernel::common::VolatileCell;

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
