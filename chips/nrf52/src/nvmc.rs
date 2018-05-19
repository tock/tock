// Non-Volatile Memory Controller
// Used in order read and write to internal flash
// Minimal implementation to support activation of the reset button

use kernel::common::regs::{ReadOnly, ReadWrite};

pub const NVMC_BASE: usize = 0x4001E400;
#[repr(C)]
struct NvmcRegisters {
    /// Ready flag
    /// Address 0x400 - 0x404
    pub ready: ReadOnly<u32, Ready::Register>,
    /// Reserved
    _reserved1: [u32; 64],
    /// Configuration register
    /// Address: 0x504 - 0x508
    pub config: ReadWrite<u32, Configuration::Register>,
    /// Register for erasing a page in Code area
    /// Address: 0x508 - 0x50C
    pub erasepage: ReadWrite<u32, ErasePage::Register>,
    /// Register for erasing all non-volatile user memory
    /// Address: 0x50C - 0x510
    pub eraseall: ReadWrite<u32, EraseAll::Register>,
    _reserved2: u32,
    /// Register for erasing User Information Configuration Registers
    /// Address: 0x514 - 0x518
    pub eraseuicr: ReadWrite<u32, EraseUicr::Register>,
    /// Reserved
    _reserved3: [u32; 10],
    /// Configuration register
    /// Address: 0x540 - 0x544
    pub icachecnf: ReadWrite<u32, CacheConfiguration::Register>,
    /// Reserved
    _reserved4: u32,
    /// Configuration register
    /// Address: 0x548 - 0x54c
    pub ihit: ReadWrite<u32, CacheHit::Register>,
    /// Configuration register
    /// Address: 0x54C - 0x550
    pub imiss: ReadWrite<u32, CacheMiss::Register>,
}

register_bitfields! [u32,
    /// Ready flag
    Ready [
        /// NVMC is ready or busy
        READY OFFSET(0) NUMBITS(1) [
            /// NVMC is busy (on-going write or erase operation)
            BUSY = 0,
            /// NVMC is ready
            READY = 1
        ]
    ],
    /// Configuration register
    Configuration [
        /// Program memory access mode. It is strongly recommended
        /// to only activate erase and write modes when they are actively
        /// used. Enabling write or erase will invalidate the cache and keep
        /// it invalidated.
        WEN OFFSET(0) NUMBITS(2) [
            /// Read only access
            REN = 0,
            /// Write Enabled
            WEN = 1,
            /// Erase enabled
            EEN = 2
        ]
    ],
    /// Register for erasing a page in Code area
    ErasePage [
        /// Register for starting erase of a page in Code area
        ERASEPAGE OFFSET(0) NUMBITS(32) []
    ],
    /// Register for erasing all non-volatile user memory
    EraseAll [
        /// Erase all non-volatile memory including UICR registers. Note
        /// that code erase has to be enabled by CONFIG.EEN before the
        /// UICR can be erased
        ERASEALL OFFSET(0) NUMBITS(1) [
            /// No operation
            NOOPERATION = 0,
            /// Start chip erase
            ERASE = 1
        ]
    ],
    /// Register for erasing User Information Configuration Registers
    EraseUicr [
        /// Register starting erase of all User Information Configuratio Registers.
        /// Note that code erase has to be enabled by CONFIG.EEN before the UICR can be erased
        ERASEUICR OFFSET(0) NUMBITS(1) [
            /// No operation
            NOOPERATION = 0,
            /// Start erase of UICR
            ERASE = 1
        ]
    ],
    /// I-Code cache configuration register
    CacheConfiguration [
        /// Cache enabled
        CACHEEN OFFSET(0) NUMBITS(1) [
            /// Disable cache. Invalidates all cache entries
            DISABLED = 0,
            /// Enable cache
            ENABLED = 1
        ],
        /// Cache profiling enable
        CACHEPROFEN OFFSET(8) NUMBITS(1) [
            /// Disable cache profiling
            DISABLED = 0,
            /// Enable cache profiling
            ENABLED = 1
        ]
    ],
    /// I-Code cache hit counter
    CacheHit [
        /// Number of cache hits
        HITS OFFSET(0) NUMBITS(32) []
    ],
    /// I-Code cache miss counter
    CacheMiss [
        /// Number of cache misses
        MISSES OFFSET(0) NUMBITS(32) []
    ]
];

pub struct Nvmc {
    regs: *const NvmcRegisters,
}

impl Nvmc {
    pub const fn new() -> Nvmc {
        Nvmc {
            regs: NVMC_BASE as *const NvmcRegisters,
        }
    }

    pub fn configure_writeable(&self) {
        let regs = unsafe { &*self.regs };
        regs.config.write(Configuration::WEN::WEN);
    }

    pub fn configure_eraseable(&self) {
        let regs = unsafe { &*self.regs };
        regs.config.write(Configuration::WEN::EEN);
    }

    pub fn erase_uicr(&self) {
        let regs = unsafe { &*self.regs };
        regs.config.write(Configuration::WEN::EEN);
        while !self.is_ready() {}
        regs.erasepage.write(ErasePage::ERASEPAGE.val(0x10001000));
        while !self.is_ready() {}
    }

    pub fn is_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.ready.is_set(Ready::READY)
    }
}
