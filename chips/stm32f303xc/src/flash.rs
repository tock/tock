use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::registers::register_bitfields;
use kernel::common::registers::{ReadOnly, WriteOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x8000_0000) as *const FlashRegisters };

#[repr(C)]
struct FlashRegisters {
    /// Flash access control register
    /// Address offset 0x00
    pub acr: ReadWrite<u32, AccessControl::Register>,
    /// Flash key register
    /// Address offset 0x04
    pub kr: WriteOnly<u32, Key::Register>,
    /// Flash option key register
    /// Address offset 0x08
    pub okr: WriteOnly<u32, Key::Register>,
    /// Flash status register
    /// Address offset 0x0C
    pub sr: ReadWrite<u32, Status::Register>,
    /// Flash control register
    /// Address offset 0x10
    pub cr: ReadWrite<u32, Control::Register>,
    /// Flash address register
    /// Address offset 0x14
    pub ar: WriteOnly<u32, Adress::Register>,
    /// Reserved
    _reserved: u32,
    /// Flash option byte register
    /// Address offset 0x1C
    pub obr: ReadOnly<u32, OptionByte::Register>,
    /// Flash write protection register
    /// Address offset 0x20
    pub wrpr: ReadOnly<u32, WriteProtect::register>,
}

register_bitfields! [u32,
    AccessControl [
        /// Prefetch buffer status
        PRFTBS OFFSET(5) NUMBITS(1) [],
        /// Prefetch buffer enable
        PRFTBE OFFSET(4) NUMBITS(1) [],
        /// Flash half cycle access enable
        HLFCYA OFFSET(3) NUMBITS(1) [],
        /// Represents the ratio of the HCLK period to the Flash access time
        LATENCY OFFSET(0) NUMBITS(3) [
            /// If 0 < HCLK <= 24MHz
            ZeroWaitState = 0,
            /// If 24MHz < HCLK <= 48MHz
            OneWaitState = 1,
            /// If 48MHz < HCLK <= 72MHz
            TwoWaitState = 2
        ]
    ],
    Key [
        /// Flash or option byte key
        /// Represents the keys to unlock the flash or the option 
        /// bytes write enable
        KEYR OFFSET(0) NUMBITS(32) []
    ],
    OptionKey [
        /// Option byte key
        /// Represents the keys to unlock the option bytes write enable
        OPTKEYR OFFSET(0) NUMBITS(32) []
    ],
    Status [
        /// End of operation
        /// Set by the hardware when a flash operation (programming or erase)
        /// is completed.
        EOP OFFSET(5) NUMBITS(1) [],
        /// Write protection error 
        /// Set by the hardware when programming a write-protected 
        /// address of the flash memory.
        WRPRTERR OFFSET(4) NUMBITS(1) [],
        /// Programming error
        /// Set by the hardware when an address to be programmed contains a 
        /// value different from 0xFFFF before programming.
        /// Note that the STRT bit in Control register should be reset when 
        /// the operation finishes or and error occurs.
        PGERR OFFSET(2) NUMBITS(1) [],
        /// Busy 
        /// Indicates that a flash operation is in progress. This is set on
        /// the beginning of a Flash operation and reset when the operation
        /// finishes or an error occurs.
        BSY OFFSET(0) NUMBITS(1) []
    ],
    Control [
        /// Force option byte loading
        /// When set, this bit forces the option byte reloading.
        /// This generates a system reset.
        OBL_LAUNCH OFFSET(13) NUMBITS(1) [],
        /// End of operation interrupt enable
        /// This enables the interrupt generation when the EOP bit in the 
        /// Status register is set.
        EOPIE OFFSET(12) NUMBITS(1) [],
        /// Error interrupt enable
        /// This bit enables the interrupt generation on an errror when PGERR
        /// or WRPRTERR are set in the Status register
        ERRIE OFFSET(10) NUMBITS(1) [],
        /// Option bytes write enable
        /// When set, the option bytes can be programmed. This bit is set on 
        /// on writing the correct key sequence to the OptionKey register.
        OPTWRE OFFSET(9) NUMBITS(1) [],
        /// When set, it indicates that the Flash is locked. This bit is reset
        /// by hardware after detecting the unlock sequence.
        LOCK OFFSET(7) NUMBITS(1) [],
        /// This bit triggers and ERASE operation when set. This bit is only 
        /// set by software and reset when the BSY bit is reset.
        STRT OFFSET(6) NUMBITS(1) [],
        /// Option byte erase chosen
        OPTER OFFSET(5) NUMBITS(1) [],
        /// Option byte programming chosen
        OPTPG OFFSET(4) NUMBITS(1) [],
        /// Mass erase of all user pages chosen
        MER OFFSET(2) NUMBITS(1) [],
        /// Page erase chosen
        PER OFFSET(1) NUMBITS(1) [],
        /// Flash programming chosen
        PG OFFSET(0) NUMBITS(1) [],
    ],
    Address [
        /// Flash address
        /// Chooses the address to program when programming is selected 
        /// or a page to erase when Page Erase is selected.
        /// Note that write access to this register is blocked when the 
        /// BSY bit in the Status register is set.
        FAR OFFSET(0) NUMBITS(32) []
    ],
    OptionByte [
        DATA1 OFFSET(24) NUMBITS(8) [],
        DATA0 OFFSET(16) NUMBITS(8) [],
        /// This allows the user to enable the SRAM hardware parity check.
        /// Disabled by default.
        SRAMPE OFFSET(14) NUMBITS(1) [
            /// Parity check enabled
            ENABLED = 0,
            /// Parity check diasbled
            DISABLED = 1
        ],
        /// This bit selects the analog monitoring on the VDDA power source
        VDDAMONITOR OFFSET(13) NUMBITS(1) [
            /// VDDA power supply supervisor disabled
            DISABLED = 0,
            /// VDDA power supply supervisor enabled
            ENABLED = 1
        ],
        /// Together with the BOOT0, this bit selects Boot mode from the main
        /// Flash memory, SRAM or System memory
        NBOOT1 OFFSET(12) NUMBITS(1) [],
        NRSTSTDBY OFFSET(10) NUMBITS(1) [
            /// Reset generated when entering Standby mode
            RST = 0,
            /// No reset generated
            NRST = 1
        ],
        NRSTSTOP OFFSET(9) NUMBITS(1) [
            /// Reset generated when entering Stop mode
            RST = 0,
            /// No reset generated
            NRST = 1
        ],
        /// Chooses watchdog type
        WDGSW OFFSET(8) NUMBITS(1) [
            /// Hardware watchdog
            HARDWARE = 0,
            /// Software watchdog
            SOFTWARE = 1
        ],
        /// Read protection Level status
        RDPRT OFFSET(1) NUMBITS(2) [
            /// Read protection level 0 (ST production setup)
            LVL0 = 0,
            /// Read protection level 1
            LVL1 = 1,
            /// Read protection level 2
            LVL2 = 3
        ],
        /// Option byte Load error
        /// When set, this indicates that the loaded option byte and its 
        /// complement do not match. The corresponding byte and its complement
        /// are read as 0xFF in the OptionByte or WriteProtect register
        OPTERR OFFSET(1) NUMBITS(1) []
    ],
    WriteProtect [
        /// Write protect
        /// This register contains the write-protection option 
        /// bytes loaded by the OBL
        WRP OFFSET(0) NUMBITS(32) []
    ]
];

const PAGE_SIZE: usize = 2048;

pub struct StmF303Page(pub [u8; PAGE_SIZE as usize]);

impl Default for StmF303Page {
    fn default() -> Self {
        0: [0; PAGE_SIZE as usize],
    }
}

impl StmF303Page {
    fn len(&self) -> usize {
        self.0.len();
    }
}

impl Index<usize> for StmF303Page {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &mut self.0[idx]
    }
}

impl IndexMut<usize> for StmF303Page {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for StmF303Page {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// TODO: Verify if there should be other states
pub enum FlashState {
    Ready,
    Read,
    Write,
    Erase,
}

pub static mut FLASH: Flash = Flash::new();

// TODO: Choose proper Cell for struct fields
pub struct Flash {
    registers: StaticRef<FlashRegisters>,
    client: OptionalCell<&'static dyn hil::flash::Client<Flash>>,
    buffer: TakeCell<'static, StmF303Page>
    state: Cell<FlashState>,
}

impl Flash {
    pub const fn new() -> Flash {
        Flash {
            registers: FLASH_BASE,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            state: Cell::new(FlashState::Ready),
        }
    }
}

/// TODO
impl hil::flash::Flash for Flash {
    type Page = StmF303Page;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.read(page_number, buf)
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.write(page_number, buf)
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.erase(buf)
    }
}
