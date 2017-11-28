//! Implementation of the SAM4L flash controller.
//!
//! This implementation of the flash controller for the SAM4L uses interrupts to
//! handle main tasks of a flash -- write, reads, and erases. If modifying this
//! file, you should check whether the flash commands (issued via issue_command)
//! generates an interrupt and design a higher level function based off of that.
//!
//! Although the datasheet says that when the FRDY interrupt is on, an interrupt
//! will be generated after a command is complete, it doesn't appear to occur
//! for some commands.
//!
//! A clean interface for reading from flash, writing pages and erasing pages is
//! defined below and should be used to handle the complexity of these tasks.
//!
//! The driver should be `configure()`'d before use, and a Client should be set
//! to enable a callback after a command is completed.
//!
//! Almost all of the flash controller functionality is implemented (except for
//! general purpose fuse bits, and more granular control of the cache).
//!
//! - Author:  Kevin Baichoo <kbaichoo@cs.stanford.edu>
//! - Date: July 27, 2016

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use helpers::{DeferredCall, Task};
use kernel::ReturnCode;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use pm;

/// Struct of the FLASHCALW registers. Section 14.10 of the datasheet.
#[repr(C, packed)]
struct FlashcalwRegisters {
    fcr: VolatileCell<u32>,
    fcmd: VolatileCell<u32>,
    fsr: VolatileCell<u32>,
    fpr: VolatileCell<u32>,
    fvr: VolatileCell<u32>,
    fgpfrhi: VolatileCell<u32>,
    fgpfrlo: VolatileCell<u32>,
    _reserved1: [VolatileCell<u32>; 251],
    ctrl: VolatileCell<u32>,
    sr: VolatileCell<u32>,
    _reserved2: [VolatileCell<u32>; 4],
    maint0: VolatileCell<u32>,
    maint1: VolatileCell<u32>,
    mcfg: VolatileCell<u32>,
    men: VolatileCell<u32>,
    mctrl: VolatileCell<u32>,
    msr: VolatileCell<u32>,
    _reserved3: [VolatileCell<u32>; 49],
    pvr: VolatileCell<u32>,
}
const FLASHCALW_BASE_ADDRS: usize = 0x400A0000;

#[allow(dead_code)]
enum RegKey {
    CONTROL,
    COMMAND,
    STATUS,
    PARAMETER,
    VERSION,
    GPFRHI,
    GPFRLO,
}

static DEFERRED_CALL: DeferredCall = unsafe { DeferredCall::new(Task::Flashcalw) };

/// There are 18 recognized commands for the flash. These are "bare-bones"
/// commands and values that are written to the Flash's command register to
/// inform the flash what to do. Table 14-5.
#[derive(Clone, Copy, PartialEq)]
pub enum FlashCMD {
    NOP,
    WP,
    EP,
    CPB,
    LP,
    UP,
    EA,
    WGPB,
    EGPB,
    SSB,
    PGPFB,
    EAGPF,
    QPR,
    WUP,
    EUP,
    QPRUP,
    HSEN,
    HSDIS,
}

/// The two Flash speeds.
#[derive(Clone, Copy)]
pub enum Speed {
    Standard,
    HighSpeed,
}

/// FlashState is used to track the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    Unconfigured, //                 Flash is unconfigured, call configure().
    Ready, //                        Flash is ready to complete a command.
    Read, //                         Performing a read operation.
    WriteUnlocking { page: i32 }, // Started a write operation.
    WriteErasing { page: i32 }, //   Waiting on the page to erase.
    WriteWriting, //                 Waiting on the page to actually be written.
    EraseUnlocking { page: i32 }, // Started an erase operation.
    EraseErasing, //                 Waiting on the erase to finish.
}

/// This is a wrapper around a u8 array that is sized to a single page for the
/// SAM4L. Users of this module must pass an object of this type to use the
/// `hil::flash::Flash` interface.
///
/// An example looks like:
///
/// ```
/// static mut PAGEBUFFER: Sam4lPage = Sam4lPage::new();
/// ```
pub struct Sam4lPage(pub [u8; PAGE_SIZE as usize]);

impl Sam4lPage {
    pub const fn new() -> Sam4lPage {
        Sam4lPage([0; PAGE_SIZE as usize])
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<usize> for Sam4lPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Sam4lPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for Sam4lPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// The FLASHCALW controller
pub struct FLASHCALW {
    registers: *mut FlashcalwRegisters,
    ahb_clock: pm::Clock,
    hramc1_clock: pm::Clock,
    pb_clock: pm::Clock,
    client: Cell<Option<&'static hil::flash::Client<FLASHCALW>>>,
    current_state: Cell<FlashState>,
    buffer: TakeCell<'static, Sam4lPage>,
}

// static instance for the board. Only one FLASHCALW on chip.
pub static mut FLASH_CONTROLLER: FLASHCALW = FLASHCALW::new(FLASHCALW_BASE_ADDRS,
                                                            pm::HSBClock::FLASHCALW,
                                                            pm::HSBClock::FLASHCALWP,
                                                            pm::PBBClock::FLASHCALW);


// Few constants relating to module configuration.
const PAGE_SIZE: u32 = 512;
const NB_OF_REGIONS: u32 = 16;
const FLASHCALW_CMD_KEY: u32 = 0xA5 << 24;

#[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE)]
const FREQ_PS1_FWS_1_FWU_MAX_FREQ: u32 = 12000000;
#[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE)]
const FREQ_PS0_FWS_0_MAX_FREQ: u32 = 18000000;
#[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE)]
const FREQ_PS0_FWS_1_MAX_FREQ: u32 = 36000000;
#[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE)]
const FREQ_PS1_FWS_0_MAX_FREQ: u32 = 8000000;


#[cfg(not(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE))]
const FREQ_PS2_FWS_0_MAX_FREQ: u32 = 24000000;

// Macros for getting the i-th bit.
macro_rules! bit {
    ($w:expr) => (0x1u32 << $w);
}

impl FLASHCALW {
    const fn new(base_addr: usize,
                 ahb_clk: pm::HSBClock,
                 hramc1_clk: pm::HSBClock,
                 pb_clk: pm::PBBClock)
                 -> FLASHCALW {
        FLASHCALW {
            registers: base_addr as *mut FlashcalwRegisters,
            ahb_clock: pm::Clock::HSB(ahb_clk),
            hramc1_clock: pm::Clock::HSB(hramc1_clk),
            pb_clock: pm::Clock::PBB(pb_clk),
            client: Cell::new(None),
            current_state: Cell::new(FlashState::Unconfigured),
            buffer: TakeCell::empty(),
        }
    }


    /// Cache controlling functionality.

    //  Flush the cache. Should be called after every write!
    fn invalidate_cache(&self) {
        let registers: &FlashcalwRegisters = unsafe { &*self.registers };
        registers.maint0.set(0x1);
    }

    pub fn enable_picocache(&self, enable: bool) {
        let registers: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            registers.ctrl.set(0x1);
        } else {
            registers.ctrl.set(0x0);
        }
    }

    /// Enable HCACHE
    pub fn enable_cache(&self) {

        // enable appropriate clocks
        unsafe {
            pm::enable_clock(pm::Clock::HSB(pm::HSBClock::FLASHCALWP));
            pm::enable_clock(pm::Clock::PBB(pm::PBBClock::HRAMC1));
        }

        // enable and wait for it to be ready
        self.enable_picocache(true);
        while !self.pico_enabled() {}
    }

    pub fn pico_enabled(&self) -> bool {
        let registers: &FlashcalwRegisters = unsafe { &*self.registers };
        registers.sr.get() & 0x1 != 0
    }

    // Helper to read a flashcalw register (espically if your function is doing so once)
    fn read_register(&self, key: RegKey) -> u32 {
        let registers: &FlashcalwRegisters = unsafe { &*self.registers };

        match key {
            RegKey::CONTROL => registers.fcr.get(),
            RegKey::COMMAND => registers.fcmd.get(),
            RegKey::STATUS => registers.fsr.get(),
            RegKey::PARAMETER => registers.fpr.get(),
            RegKey::VERSION => registers.fvr.get(),
            RegKey::GPFRHI => registers.fgpfrhi.get(),
            RegKey::GPFRLO => registers.fgpfrlo.get(),
        }
    }


    pub fn handle_interrupt(&self) {
        //  disable the interrupt line for flash
        self.enable_ready_int(false);

        let error_status = self.get_error_status();

        // Since the only interrupt on is FRDY, a command should have
        // either completed or failed at this point.

        // Check for errors and report to Client if there are any
        if error_status != 0 {
            let attempted_operation = self.current_state.get();

            // Reset state now that we are ready to do a new operation.
            self.current_state.set(FlashState::Ready);

            self.client.get().map(|client| match attempted_operation {
                FlashState::Read => {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, hil::flash::Error::FlashError);
                    });
                }
                FlashState::WriteUnlocking { .. } |
                FlashState::WriteErasing { .. } |
                FlashState::WriteWriting => {
                    self.buffer.take().map(|buffer| {
                        client.write_complete(buffer, hil::flash::Error::FlashError);
                    });
                }
                FlashState::EraseUnlocking { .. } |
                FlashState::EraseErasing => {
                    client.erase_complete(hil::flash::Error::FlashError);
                }
                _ => {}
            });
        }

        // Part of a command succeeded -- continue onto next steps.
        match self.current_state.get() {
            FlashState::Read => {
                self.current_state.set(FlashState::Ready);

                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, hil::flash::Error::CommandComplete);
                    });
                });
            }
            FlashState::WriteUnlocking { page } => {
                self.current_state.set(FlashState::WriteErasing { page: page });
                self.flashcalw_erase_page(page);
            }
            FlashState::WriteErasing { page } => {
                //  Write page buffer isn't really a command, and
                //  clear page buffer doesn't trigger an interrupt thus
                //  I'm combining these with an actual command, write_page,
                //  which generates and interrupt and saves the page.
                self.clear_page_buffer();
                self.write_to_page_buffer(page as usize * PAGE_SIZE as usize);

                self.current_state.set(FlashState::WriteWriting);
                self.flashcalw_write_page(page);
            }
            FlashState::WriteWriting => {
                // Flush the cache
                self.invalidate_cache();

                self.current_state.set(FlashState::Ready);

                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.write_complete(buffer, hil::flash::Error::CommandComplete);
                    });
                });
            }
            FlashState::EraseUnlocking { page } => {
                self.current_state.set(FlashState::EraseErasing);
                self.flashcalw_erase_page(page);
            }
            FlashState::EraseErasing => {
                self.current_state.set(FlashState::Ready);

                self.client
                    .get()
                    .map(|client| { client.erase_complete(hil::flash::Error::CommandComplete); });
            }
            _ => {
                self.current_state.set(FlashState::Ready);
            }

        }
    }

    /// FLASH properties.
    pub fn get_flash_size(&self) -> u32 {
        let flash_sizes = [4, 8, 16, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 2048];
        // get the FSZ number and lookup in the table for the size.
        flash_sizes[self.read_register(RegKey::PARAMETER) as usize & 0xf] << 10
    }

    pub fn get_page_count(&self) -> u32 {
        self.get_flash_size() / PAGE_SIZE
    }

    pub fn get_page_count_per_region(&self) -> u32 {
        self.get_page_count() / NB_OF_REGIONS
    }


    pub fn get_page_region(&self, page_number: i32) -> u32 {
        (if page_number >= 0 {
            page_number as u32
        } else {
            self.get_page_number()
        } / self.get_page_count_per_region())
    }

    pub fn get_region_first_page_number(&self, region: u32) -> u32 {
        region * self.get_page_count_per_region()
    }


    /// FLASHC Control
    #[allow(dead_code)]
    fn get_wait_state(&self) -> u32 {
        if self.read_register(RegKey::CONTROL) & bit!(6) == 0 {
            0
        } else {
            1
        }
    }

    fn set_wait_state(&self, wait_state: u32) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if wait_state == 1 {
            regs.fcr.set(regs.fcr.get() | bit!(6));
        } else {
            regs.fcr.set(regs.fcr.get() & !bit!(6));
        }
    }

    fn enable_ws1_read_opt(&mut self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.fcr.set(regs.fcr.get() | bit!(7));
        } else {
            regs.fcr.set(regs.fcr.get() | !bit!(7));
        }
    }

    //  By default, we are going with High Speed Enable (based on our device running
    //  in PS2).
    #[cfg(not(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE))]
    fn set_flash_waitstate_and_readmode(&mut self,
                                        cpu_freq: u32,
                                        _ps_val: u32,
                                        _is_fwu_enabled: bool) {
        // ps_val and is_fwu_enabled not used in this implementation.
        if cpu_freq > FREQ_PS2_FWS_0_MAX_FREQ {
            self.set_wait_state(1);
        } else {
            self.set_wait_state(0);
        }

        self.issue_command(FlashCMD::HSEN, -1);
    }


    #[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE)]
    fn set_flash_waitstate_and_readmode(&mut self,
                                        cpu_freq: u32,
                                        ps_val: u32,
                                        is_fwu_enabled: bool) {
        if ps_val == 0 {
            if cpu_freq > FREQ_PS0_FWS_0_MAX_FREQ {
                self.set_wait_state(1);
                if cpu_freq <= FREQ_PS0_FWS_1_MAX_FREQ {
                    self.issue_command(FlashCMD::HSDIS, -1);
                } else {
                    self.issue_command(FlashCMD::HSEN, -1);
                }
            } else {
                if is_fwu_enabled && cpu_freq <= FREQ_PS1_FWS_1_FWU_MAX_FREQ {
                    self.set_wait_state(1);
                    self.issue_command(FlashCMD::HSDIS, -1);
                } else {
                    self.set_wait_state(0);
                    self.issue_command(FlashCMD::HSDIS, -1);
                }
            }

        } else {
            // ps_val == 1
            if cpu_freq > FREQ_PS1_FWS_0_MAX_FREQ {
                self.set_wait_state(1);
            } else {
                self.set_wait_state(0);
            }
            self.issue_command(FlashCMD::HSDIS, -1);
        }
    }

    /// Configure high-speed flash mode. This is taken from the ASF code
    pub fn enable_high_speed_flash(&self) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };

        // Since we are running at a fast speed we have to set a clock delay
        // for flash, as well as enable fast flash mode.
        let flashcalw_fcr = regs.fcr.get();
        regs.fcr.set(flashcalw_fcr | (1 << 6));

        // Enable high speed mode for flash
        let flashcalw_fcmd = regs.fcmd.get();
        let flashcalw_fcmd_new1 = flashcalw_fcmd & (!(0x3F << 0));
        let flashcalw_fcmd_new2 = flashcalw_fcmd_new1 | (0xA5 << 24) | (0x10 << 0);
        regs.fcmd.set(flashcalw_fcmd_new2);

        // And wait for the flash to be ready
        while regs.fsr.get() & (1 << 0) == 0 {}
    }

    #[allow(dead_code)]
    fn is_ready_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(0)) != 0
    }

    fn enable_ready_int(&self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.fcr.set(regs.fcr.get() | bit!(0));
        } else {
            regs.fcr.set(regs.fcr.get() & !bit!(0));
        }
    }

    #[allow(dead_code)]
    fn is_lock_error_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(2)) != 0
    }

    fn enable_lock_error_int(&self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.fcr.set(regs.fcr.get() | bit!(2));
        } else {
            regs.fcr.set(regs.fcr.get() & !bit!(2));
        }
    }

    #[allow(dead_code)]
    fn is_prog_error_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(3)) != 0
    }

    fn enable_prog_error_int(&self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.fcr.set(regs.fcr.get() | bit!(3));
        } else {
            regs.fcr.set(regs.fcr.get() & !bit!(3));
        }
    }

    #[allow(dead_code)]
    fn is_ecc_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(4)) != 0
    }

    fn enable_ecc_int(&self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.fcr.set(regs.fcr.get() | bit!(4));
        } else {
            regs.fcr.set(regs.fcr.get() & !bit!(4));
        }
    }

    /// Flashcalw status

    pub fn is_ready(&self) -> bool {
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        self.read_register(RegKey::STATUS) & bit!(0) != 0
    }

    fn get_error_status(&self) -> u32 {
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        self.read_register(RegKey::STATUS) & (bit!(3) | bit!(2))
    }

    #[allow(dead_code)]
    fn is_lock_error(&self) -> bool {
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        self.read_register(RegKey::STATUS) & bit!(2) != 0
    }

    #[allow(dead_code)]
    fn is_programming_error(&self) -> bool {
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        self.read_register(RegKey::STATUS) & bit!(3) != 0
    }

    /// Flashcalw command control
    fn get_page_number(&self) -> u32 {
        // create a mask for the page number field
        let mut page_mask: u32 = bit!(8) - 1;
        page_mask |= page_mask << 24;
        page_mask = !page_mask;

        (self.read_register(RegKey::COMMAND) & page_mask) >> 8
    }

    pub fn issue_command(&self, command: FlashCMD, page_number: i32) {
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        if command != FlashCMD::QPRUP && command != FlashCMD::QPR && command != FlashCMD::CPB &&
           command != FlashCMD::HSEN {
            // Enable ready interrupt.
            self.enable_ready_int(true);
        }

        let cmd_regs: &FlashcalwRegisters = unsafe { &*self.registers };
        let mut reg_val: u32 = cmd_regs.fcmd.get();

        let clear_cmd_mask: u32 = !(bit!(6) - 1);
        reg_val &= clear_cmd_mask;

        // craft the command
        if page_number >= 0 {
            reg_val = FLASHCALW_CMD_KEY | (page_number as u32) << 8 | command as u32;
        } else {
            reg_val |= FLASHCALW_CMD_KEY | command as u32;
        }

        cmd_regs.fcmd.set(reg_val); // write the cmd

        // Since we don't enable interrupts for these commands, spin wait
        // until they are finished. In particular, QPR and QPRUP will not issue
        // interrupts (see datasheet 14.6 paragraph 2).
        if command == FlashCMD::QPRUP || command == FlashCMD::QPR || command == FlashCMD::CPB ||
           command == FlashCMD::HSEN {
            while (cmd_regs.fsr.get() & 0x01) != 0x01 {}
        }
    }


    /// Flashcalw global commands
    pub fn no_operation(&self) {
        self.issue_command(FlashCMD::NOP, -1);
    }

    pub fn erase_all(&self) {
        self.issue_command(FlashCMD::EA, -1);
    }

    /// FLASHCALW Protection Mechanisms
    #[allow(dead_code)]
    fn is_security_bit_active(&self) -> bool {
        (self.read_register(RegKey::STATUS) & bit!(4)) != 0
    }

    #[allow(dead_code)]
    fn set_security_bit(&self) {
        self.issue_command(FlashCMD::SSB, -1);
    }

    pub fn is_page_region_locked(&self, page_number: u32) -> bool {
        self.is_region_locked(self.get_page_region(page_number as i32))
    }

    pub fn is_region_locked(&self, region: u32) -> bool {
        (self.read_register(RegKey::STATUS) & bit!(region + 16)) != 0
    }

    pub fn lock_page_region(&self, page_number: i32, lock: bool) {
        if lock {
            self.issue_command(FlashCMD::LP, page_number);
        } else {
            self.issue_command(FlashCMD::UP, page_number);
        }
    }

    #[allow(dead_code)]
    fn lock_region(&self, region: u32, lock: bool) {
        let first_page: i32 = self.get_region_first_page_number(region) as i32;
        self.lock_page_region(first_page, lock);
    }

    /// Flashcalw Access to Flash Pages
    fn clear_page_buffer(&self) {
        self.issue_command(FlashCMD::CPB, -1);
    }

    fn is_page_erased(&self) -> bool {
        let registers: &FlashcalwRegisters = unsafe { &*self.registers };
        let status = registers.fsr.get();

        (status & bit!(5)) != 0
    }

    #[allow(dead_code)]
    fn quick_page_read(&self, page_number: i32) -> bool {
        self.issue_command(FlashCMD::QPR, page_number);
        self.is_page_erased()
    }

    fn flashcalw_erase_page(&self, page_number: i32) {
        self.issue_command(FlashCMD::EP, page_number);
    }

    fn flashcalw_write_page(&self, page_number: i32) {
        self.issue_command(FlashCMD::WP, page_number);
    }

    /// There's a user_page that isn't contiguous with the rest of the flash.
    /// Currently it's not being used.
    #[allow(dead_code)]
    fn quick_user_page_read(&self) -> bool {
        self.issue_command(FlashCMD::QPRUP, -1);
        self.is_page_erased()
    }

    #[allow(dead_code)]
    fn erase_user_page(&self, check: bool) -> bool {
        self.issue_command(FlashCMD::EUP, -1);
        if check {
            self.quick_user_page_read()
        } else {
            true
        }
    }

    #[allow(dead_code)]
    fn write_user_page(&self) {
        self.issue_command(FlashCMD::WUP, -1);
    }

    // Instead of having several memset/memcpy functions as Atmel's ASF
    // implementation will only have one to write to the page buffer.
    fn write_to_page_buffer(&self, pg_buff_addr: usize) {
        let mut page_buffer: *mut u8 = pg_buff_addr as *mut u8;

        // Errata 45.1.7 - Need to write a 64-bit all one word for every write
        // to the page buffer.
        let cleared_double_word: [u8; 8] = [255; 8];
        let clr_ptr: *const u8 = &cleared_double_word[0] as *const u8;

        self.buffer.map(|buffer| {
            unsafe {
                use core::ptr;

                let mut start_buffer: *const u8 = &buffer[0] as *const u8;
                let mut data_transfered: u32 = 0;
                while data_transfered < PAGE_SIZE {
                    // errata copy..
                    ptr::copy(clr_ptr, page_buffer, 8);

                    // real copy
                    ptr::copy(start_buffer, page_buffer, 8);
                    page_buffer = page_buffer.offset(8);
                    start_buffer = start_buffer.offset(8);
                    data_transfered += 8;
                }
            }
        });
    }
}

// Implementation of high level calls using the low-lv functions.
impl FLASHCALW {
    pub fn configure(&mut self) {
        // Enable all clocks (if they aren't on already...).
        unsafe {
            pm::enable_clock(self.ahb_clock);
            pm::enable_clock(self.hramc1_clock);
            pm::enable_clock(self.pb_clock);

        }

        // Configure all other interrupts explicitly. Note the issue_command
        // function turns this on when need be.
        self.enable_ready_int(false);
        self.enable_lock_error_int(false);
        self.enable_prog_error_int(false);
        self.enable_ecc_int(false);

        // Enable wait state 1 optimization.
        self.enable_ws1_read_opt(true);
        // Change speed mode.
        self.set_flash_waitstate_and_readmode(48_000_000, 0, false);

        // By default the picocache ( a cache only for the flash) is turned off.
        // However the bootloader turns it on. I will explicitly turn it on
        // here. So if the bootloader changes, nothing breaks.
        self.enable_picocache(true);

        self.current_state.set(FlashState::Ready);
    }

    pub fn get_page_size(&self) -> u32 {
        PAGE_SIZE
    }

    pub fn get_number_pages(&self) -> u32 {
        // Check clock and enable just in case.
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        self.get_page_count()
    }

    // Address is some raw address in flash that you want to read.
    pub fn read_range(&self,
                      address: usize,
                      size: usize,
                      buffer: &'static mut Sam4lPage)
                      -> ReturnCode {
        // Enable clock in case it's off.
        unsafe {
            pm::enable_clock(self.ahb_clock);
        }

        // Check that address makes sense and buffer has room.
        if address > (self.get_flash_size() as usize) ||
           address + size > (self.get_flash_size() as usize) ||
           address + size < size || buffer.len() < size {
            // invalid flash address
            return ReturnCode::EINVAL;
        }

        // Actually do a copy from flash into the buffer.
        let mut byte: *const u8 = address as *const u8;
        unsafe {
            for i in 0..size {
                buffer[i] = *byte;
                byte = byte.offset(1);
            }
        }

        self.current_state.set(FlashState::Read);
        // Hold on to the buffer for the callback.
        self.buffer.replace(buffer);

        // This is kind of strange, but because read() in this case is
        // synchronous, we still need to schedule as if we had an interrupt so
        // we can allow this function to return and then call the callback.
        DEFERRED_CALL.set();

        ReturnCode::SUCCESS
    }

    pub fn write_page(&self, page_num: i32, data: &'static mut Sam4lPage) -> ReturnCode {
        // Enable clock in case it's off.
        unsafe {
            pm::enable_clock(self.ahb_clock);
        }

        // If we're not ready don't take the command.
        if self.current_state.get() != FlashState::Ready {
            return ReturnCode::EBUSY;
        }

        // Save the buffer for the future write.
        self.buffer.replace(data);

        self.current_state.set(FlashState::WriteUnlocking { page: page_num });
        self.lock_page_region(page_num, false);
        ReturnCode::SUCCESS
    }

    pub fn erase_page(&self, page_num: i32) -> ReturnCode {
        // Enable AHB clock (in case it was off).
        unsafe {
            pm::enable_clock(self.ahb_clock);
        }
        if self.current_state.get() != FlashState::Ready {
            return ReturnCode::EBUSY;
        }

        self.current_state.set(FlashState::EraseUnlocking { page: page_num });
        self.lock_page_region(page_num, false);
        ReturnCode::SUCCESS
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for FLASHCALW {
    fn set_client(&self, client: &'static C) {
        self.client.set(Some(client));
    }
}

impl hil::flash::Flash for FLASHCALW {
    type Page = Sam4lPage;

    fn read_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode {
        self.read_range(page_number * (PAGE_SIZE as usize), buf.len(), buf)
    }

    fn write_page(&self, page_number: usize, buf: &'static mut Self::Page) -> ReturnCode {
        self.write_page(page_number as i32, buf)
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.erase_page(page_number as i32)
    }
}
