//! Embedded Flash Memory Controller
//!
//! Used for reading, writing and erasing the flash and the option bytes.
//! Erase and write operations have hardware interrupt support, while read
//! operations use pseudo interrupts in the form of deferred calls. The
//! programming interface only allows halfword(u16) writes.
//!
//! Option bytes should be used with caution, especially those concerning
//! read and write protection. For example, erasing the option bytes
//! enables by default readout protection.

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::deferred_call::DeferredCall;
use kernel::common::registers::register_bitfields;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

use crate::deferred_call_tasks::DeferredCallTask;

const FLASH_BASE: StaticRef<FlashRegisters> =
    unsafe { StaticRef::new(0x40022000 as *const FlashRegisters) };

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
    pub ar: WriteOnly<u32, Address::Register>,
    /// Reserved
    _reserved: u32,
    /// Flash option byte register
    /// Address offset 0x1C
    pub obr: ReadOnly<u32, OptionByte::Register>,
    /// Flash write protection register
    /// Address offset 0x20
    pub wrpr: ReadOnly<u32, WriteProtect::Register>,
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
        OBLLAUNCH OFFSET(13) NUMBITS(1) [],
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
        PG OFFSET(0) NUMBITS(1) []
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

static DEFERRED_CALL: DeferredCall<DeferredCallTask> =
    unsafe { DeferredCall::new(DeferredCallTask::Flash) };

const PAGE_SIZE: usize = 2048;

/// Address of the first flash page.
const PAGE_START: usize = 0x08000000;

/// Address of the first option byte.
const OPT_START: usize = 0x1FFFF800;

/// Used for unlocking the flash or the option bytes.
const KEY1: u32 = 0x45670123;
const KEY2: u32 = 0xCDEF89AB;

/// This is a wrapper around a u8 array that is sized to a single page for the
/// stm32f303xc. Users of this module must pass an object of this type to use the
/// `hil::flash::Flash` interface.
///
/// An example looks like:
///
/// ```rust
/// # extern crate stm32f303xc;
/// # use stm32f303xc::flash::StmF303Page;
/// # use kernel::static_init;
///
/// let pagebuffer = unsafe { static_init!(StmF303Page, StmF303Page::default()) };
/// ```
pub struct StmF303Page(pub [u8; PAGE_SIZE as usize]);

impl Default for StmF303Page {
    fn default() -> Self {
        Self {
            0: [0; PAGE_SIZE as usize],
        }
    }
}

impl StmF303Page {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for StmF303Page {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
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

#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    Ready,       // Entry state.
    Read,        // Read procedure.
    Write,       // Programming procedure.
    Erase,       // Erase procedure.
    WriteOption, // Option bytes programming procedure.
    EraseOption, // Option bytes erase procedure.
}

pub static mut FLASH: Flash = Flash::new();

pub struct Flash {
    registers: StaticRef<FlashRegisters>,
    client: OptionalCell<&'static dyn hil::flash::Client<Flash>>,
    buffer: TakeCell<'static, StmF303Page>,
    state: Cell<FlashState>,
    write_counter: Cell<usize>,
    page_number: Cell<usize>,
}

impl Flash {
    pub const fn new() -> Flash {
        Flash {
            registers: FLASH_BASE,
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            state: Cell::new(FlashState::Ready),
            write_counter: Cell::new(0),
            page_number: Cell::new(0),
        }
    }

    /// Enables hardware interrupts.
    pub fn enable(&self) {
        self.registers.cr.modify(Control::EOPIE::SET);
        self.registers.cr.modify(Control::ERRIE::SET);
    }

    pub fn is_locked(&self) -> bool {
        self.registers.cr.is_set(Control::LOCK)
    }

    pub fn unlock(&self) {
        self.registers.kr.write(Key::KEYR.val(KEY1));
        self.registers.kr.write(Key::KEYR.val(KEY2));
    }

    pub fn lock(&self) {
        self.registers.cr.modify(Control::LOCK::SET);
    }

    pub fn unlock_option(&self) {
        self.registers.okr.write(Key::KEYR.val(KEY1));
        self.registers.okr.write(Key::KEYR.val(KEY2));
    }

    pub fn lock_option(&self) {
        self.registers.cr.modify(Control::OPTWRE::CLEAR);
    }

    /// Forces option byte reloading. Also generates a system reset.
    pub fn load_option(&self) {
        self.registers.cr.modify(Control::OBLLAUNCH::SET);
    }

    pub fn handle_interrupt(&self) {
        if self.registers.sr.is_set(Status::EOP) {
            // Reset by writing a 1.
            self.registers.sr.modify(Status::EOP::SET);

            match self.state.get() {
                FlashState::Write => {
                    self.write_counter.set(self.write_counter.get() + 2);

                    if self.write_counter.get() == PAGE_SIZE {
                        self.registers.cr.modify(Control::PG::CLEAR);
                        self.state.set(FlashState::Ready);
                        self.write_counter.set(0);

                        self.client.map(|client| {
                            self.buffer.take().map(|buffer| {
                                client.write_complete(buffer, hil::flash::Error::CommandComplete);
                            });
                        });
                    } else {
                        self.program_halfword();
                    }
                }
                FlashState::Erase => {
                    if self.registers.cr.is_set(Control::PER) {
                        self.registers.cr.modify(Control::PER::CLEAR);
                    }

                    if self.registers.cr.is_set(Control::MER) {
                        self.registers.cr.modify(Control::MER::CLEAR);
                    }

                    self.state.set(FlashState::Ready);
                    self.client.map(|client| {
                        client.erase_complete(hil::flash::Error::CommandComplete);
                    });
                }
                FlashState::WriteOption => {
                    self.registers.cr.modify(Control::OPTPG::CLEAR);
                    self.state.set(FlashState::Ready);
                }
                FlashState::EraseOption => {
                    self.registers.cr.modify(Control::OPTER::CLEAR);
                    self.state.set(FlashState::Ready);
                }
                _ => {}
            }
        }

        if self.state.get() == FlashState::Read {
            self.state.set(FlashState::Ready);
            self.client.map(|client| {
                self.buffer.take().map(|buffer| {
                    client.read_complete(buffer, hil::flash::Error::CommandComplete);
                });
            });
        }

        if self.registers.sr.is_set(Status::WRPRTERR) {
            if self.registers.cr.is_set(Control::PG) {
                self.registers.cr.modify(Control::PG::CLEAR);
            }

            if self.registers.cr.is_set(Control::OPTPG) {
                self.registers.cr.modify(Control::OPTPG::CLEAR);
            }

            // Reset by writing a 1.
            self.registers.sr.modify(Status::WRPRTERR::SET);
            panic!("WRPRTERR: programming a write-protected address.");
        }

        if self.registers.sr.is_set(Status::PGERR) {
            if self.registers.cr.is_set(Control::PG) {
                self.registers.cr.modify(Control::PG::CLEAR);
            }

            if self.registers.cr.is_set(Control::OPTPG) {
                self.registers.cr.modify(Control::OPTPG::CLEAR);
            }

            // Reset by writing a 1.
            self.registers.sr.modify(Status::PGERR::SET);
            panic!("PGERR: address contains a value different from 0xFFFF before programming.");
        }
    }

    pub fn program_halfword(&self) {
        self.buffer.take().map(|buffer| {
            let i = self.write_counter.get();

            let halfword: u16 = (buffer[i] as u16) << 0 | (buffer[i + 1] as u16) << 8;
            let page_addr = PAGE_START + self.page_number.get() * PAGE_SIZE;
            let address = page_addr + i;
            let location = unsafe { &*(address as *const VolatileCell<u16>) };
            location.set(halfword);

            self.buffer.replace(buffer);
        });
    }

    pub fn erase_page(&self, page_number: usize) -> ReturnCode {
        if page_number > 128 {
            return ReturnCode::EINVAL;
        }

        while self.registers.sr.is_set(Status::BSY) {}

        if self.is_locked() {
            self.unlock();
        }

        self.enable();
        self.state.set(FlashState::Erase);

        // Choose page erase mode.
        self.registers.cr.modify(Control::PER::SET);
        self.registers
            .ar
            .write(Address::FAR.val((PAGE_START + page_number * PAGE_SIZE) as u32));
        self.registers.cr.modify(Control::STRT::SET);

        ReturnCode::SUCCESS
    }

    pub fn erase_all(&self) -> ReturnCode {
        while self.registers.sr.is_set(Status::BSY) {}

        if self.is_locked() {
            self.unlock();
        }

        self.enable();
        self.state.set(FlashState::Erase);

        // Choose mass erase mode.
        self.registers.cr.modify(Control::MER::SET);
        self.registers.cr.modify(Control::STRT::SET);

        ReturnCode::SUCCESS
    }

    pub fn write_page(
        &self,
        page_number: usize,
        buffer: &'static mut StmF303Page,
    ) -> Result<(), (ReturnCode, &'static mut StmF303Page)> {
        if page_number > 128 {
            return Err((ReturnCode::EINVAL, buffer));
        }

        while self.registers.sr.is_set(Status::BSY) {}

        if self.is_locked() {
            self.unlock();
        }

        self.enable();
        self.state.set(FlashState::Write);

        // Choose programming mode.
        self.registers.cr.modify(Control::PG::SET);

        self.buffer.replace(buffer);
        self.page_number.set(page_number);
        self.program_halfword();

        Ok(())
    }

    pub fn read_page(
        &self,
        page_number: usize,
        buffer: &'static mut StmF303Page,
    ) -> Result<(), (ReturnCode, &'static mut StmF303Page)> {
        if page_number > 128 {
            return Err((ReturnCode::EINVAL, buffer));
        }

        while self.registers.sr.is_set(Status::BSY) {}

        let mut byte: *const u8 = (PAGE_START + page_number * PAGE_SIZE) as *const u8;
        unsafe {
            for i in 0..buffer.len() {
                buffer[i] = *byte;
                byte = byte.offset(1);
            }
        }

        self.buffer.replace(buffer);
        self.state.set(FlashState::Read);
        DEFERRED_CALL.set();

        Ok(())
    }

    /// Allows programming the 8 option bytes:
    /// 0: RDP, 1: USER, 2: DATA0, 3:DATA1, 4. WRP0, 5: WRP1, 6.WRP2, 7. WRP3
    pub fn write_option(&self, byte_number: usize, value: u8) -> ReturnCode {
        if byte_number > 7 {
            return ReturnCode::EINVAL;
        }

        while self.registers.sr.is_set(Status::BSY) {}
        if self.is_locked() {
            self.unlock();
        }

        self.unlock_option();
        self.enable();
        self.state.set(FlashState::WriteOption);

        // Choose option byte programming mode.
        self.registers.cr.modify(Control::OPTPG::SET);

        let address = OPT_START + byte_number * 2;
        let location = unsafe { &*(address as *const VolatileCell<u16>) };
        let halfword: u16 = value as u16;
        location.set(halfword);

        ReturnCode::SUCCESS
    }

    pub fn erase_option(&self) -> ReturnCode {
        while self.registers.sr.is_set(Status::BSY) {}
        if self.is_locked() {
            self.unlock();
        }

        self.unlock_option();
        self.enable();
        self.state.set(FlashState::EraseOption);

        // Choose option byte erase mode.
        self.registers.cr.modify(Control::OPTER::SET);
        self.registers.cr.modify(Control::STRT::SET);

        ReturnCode::SUCCESS
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for Flash {
    fn set_client(&self, client: &'static C) {
        self.client.set(client);
    }
}

impl hil::flash::Flash for Flash {
    type Page = StmF303Page;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.read_page(page_number, buf)
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.write_page(page_number, buf)
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.erase_page(page_number)
    }
}
