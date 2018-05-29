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
use deferred_call_tasks::Task;
use kernel::common::cells::TakeCell;
use kernel::common::deferred_call::DeferredCall;
use kernel::common::regs::{ReadOnly, ReadWrite, WriteOnly};
use kernel::hil;
use kernel::ReturnCode;
use pm;

/// Struct of the FLASHCALW registers. Section 14.10 of the datasheet.
#[repr(C)]
struct FlashcalwRegisters {
    fcr: ReadWrite<u32, FlashControl::Register>,
    fcmd: ReadWrite<u32, FlashCommand::Register>,
    fsr: ReadOnly<u32, FlashStatus::Register>,
    fpr: ReadOnly<u32, FlashParameter::Register>,
    fvr: ReadOnly<u32, FlashVersion::Register>,
    fgpfrhi: ReadOnly<u32, FlashGeneralPurposeFuseHigh::Register>,
    fgpfrlo: ReadOnly<u32, FlashGeneralPurposeFuseLow::Register>,
    _reserved1: [u32; 251],
    ctrl: WriteOnly<u32, PicoCacheControl::Register>,
    sr: ReadWrite<u32, PicoCacheStatus::Register>,
    _reserved2: [u32; 4],
    maint0: WriteOnly<u32, PicoCacheMaintenance0::Register>,
    maint1: WriteOnly<u32, PicoCacheMaintenance1::Register>,
    mcfg: ReadWrite<u32, PicoCacheMonitorConfiguration::Register>,
    men: ReadWrite<u32, PicoCacheMonitorEnable::Register>,
    mctrl: WriteOnly<u32, PicoCacheMonitorStatus::Register>,
    msr: ReadOnly<u32, PicoCacheMonitorStatus::Register>,
}

register_bitfields![u32,
    FlashControl [
        /// Wait State 1 Optimization
        WS1OPT OFFSET(7) NUMBITS(1) [
            NoOptimize = 0,
            Optimize = 1
        ],
        /// Flash Wait State
        FWS OFFSET(6) NUMBITS(1) [
            ZeroWaitStates = 0,
            OneWaitState = 1
        ],
        /// ECC Error Interrupt Enable
        ECCE OFFSET(4) NUMBITS(1) [],
        /// Programming Error Interrupt Enable
        PROGE OFFSET(3) NUMBITS(1) [],
        /// Lock Error Interrupt Enable
        LOCKE OFFSET(2) NUMBITS(1) [],
        /// Flash Ready Interrupt Enable
        FRDY OFFSET(0) NUMBITS(1) []
    ],

    FlashCommand [
        /// Write protection key
        KEY OFFSET(24) NUMBITS(8) [],
        /// Page number
        PAGEN OFFSET(8) NUMBITS(16) [],
        /// Command
        CMD OFFSET(0) NUMBITS(6) [
            NOP = 0,
            WP = 1,
            EP = 2,
            CPB = 3,
            LP = 4,
            UP = 5,
            EA = 6,
            WGPB = 7,
            EGPB = 8,
            SSB = 9,
            PGPFB = 10,
            EAGPF = 11,
            QPR = 12,
            WUP = 13,
            EUP = 14,
            QPRUP = 15,
            HSEN = 16,
            HSDIS = 17
        ]
    ],

    FlashStatus [
        /// Lock region x Lock Status
        LOCK15 31,
        LOCK14 30,
        LOCK13 29,
        LOCK12 28,
        LOCK11 27,
        LOCK10 26,
        LOCK9 25,
        LOCK8 24,
        LOCK7 23,
        LOCK6 22,
        LOCK5 21,
        LOCK4 20,
        LOCK3 19,
        LOCK2 18,
        LOCK1 17,
        LOCK0 16,
        ///ECC Error Status
        ///
        /// WARNING! Datasheet has this bit listed in two places...
        ECCERR 9,
        /// High Speed Mode
        HSMODE 6,
        /// Quick Page Read Result
        QPRR 5,
        /// Security Fuses Status
        SECURITY 4,
        /// Programming Error Status
        PROGE 3,
        /// Lock Error Status
        LOCKE 2,
        /// Flash Ready Status
        FRDY 0
    ],

    FlashParameter [
        /// Page Size
        PSZ OFFSET(8) NUMBITS(3) [
            Bytes32 = 0,
            Bytes64 = 1,
            Bytes128 = 2,
            Bytes256 = 3,
            Bytes512 = 4,
            Bytes1024 = 5,
            Bytes2048 = 6,
            Bytes4096 = 7
        ],
        /// Flash Size
        FSZ OFFSET(0) NUMBITS(4) [
            KBytes4 = 0,
            KBytes8 = 1,
            KBytes16 = 2,
            KBytes32 = 3,
            KBytes48 = 4,
            KBytes64 = 5,
            KBytes96 = 6,
            KBytes128 = 7,
            KBytes192 = 8,
            KBytes256 = 9,
            KBytes384 = 10,
            KBytes512 = 11,
            KBytes768 = 12,
            KBytes1024 = 13,
            KBytes2048 = 14
        ]
    ],

    FlashVersion [
        /// Variant Number
        VARIANT OFFSET(16) NUMBITS(4) [],
        /// Version Number
        VERSION OFFSET(0) NUMBITS(12) []
    ],

    FlashGeneralPurposeFuseHigh [
        /// General Purpose Fuse
        GPF63 31,
        GPF62 30,
        GPF61 29,
        GPF60 28,
        GPF59 27,
        GPF58 26,
        GPF57 25,
        GPF56 24,
        GPF55 23,
        GPF54 22,
        GPF53 21,
        GPF52 20,
        GPF51 19,
        GPF50 18,
        GPF49 17,
        GPF48 16,
        GPF47 15,
        GPF46 14,
        GPF45 13,
        GPF44 12,
        GPF43 11,
        GPF42 10,
        GPF41 9,
        GPF40 8,
        GPF39 7,
        GPF38 6,
        GPF37 5,
        GPF36 4,
        GPF35 3,
        GPF34 2,
        GPF33 1,
        GPF32 0
    ],

    FlashGeneralPurposeFuseLow [
        GPF31 31,
        GPF30 30,
        GPF29 29,
        GPF28 28,
        GPF27 27,
        GPF26 26,
        GPF25 25,
        GPF24 24,
        GPF23 23,
        GPF22 22,
        GPF21 21,
        GPF20 20,
        GPF19 19,
        GPF18 18,
        GPF17 17,
        GPF16 16,
        GPF15 15,
        GPF14 14,
        GPF13 13,
        GPF12 12,
        GPF11 11,
        GPF10 10,
        GPF9 9,
        GPF8 8,
        GPF7 7,
        GPF6 6,
        GPF5 5,
        GPF4 4,
        GPF3 3,
        GPF2 2,
        GPF1 1,
        GPF0 0
    ],

    PicoCacheControl [
        /// Cache Enable
        CEN OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    PicoCacheStatus [
        /// Cache Controller Status
        CSTS OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ],

    PicoCacheMaintenance0 [
        /// Cache Controller Invalidate All
        INVALL 0
    ],

    PicoCacheMaintenance1 [
        /// Invalidate Index
        INDEX OFFSET(4) NUMBITS(4) []
    ],

    PicoCacheMonitorConfiguration [
        /// Cache Controller Monitor Counter Mode
        MODE OFFSET(0) NUMBITS(1) [
            CycleCount = 0,
            IhitCount = 1,
            DhitCount = 2
        ]
    ],

    PicoCacheMonitorEnable [
        /// Monitor Enable
        MENABLE OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    PicoCacheMonitorControl [
        /// Monitor Software Reset
        SWRST 0
    ],

    PicoCacheMonitorStatus [
        /// Monitor Event Counter
        EVENTCNT OFFSET(0) NUMBITS(32) []
    ]
];

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

static DEFERRED_CALL: DeferredCall<Task> = unsafe { DeferredCall::new(Task::Flashcalw) };

/// There are 18 recognized commands for the flash. These are "bare-bones"
/// commands and values that are written to the Flash's command register to
/// inform the flash what to do. Table 14-5.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum FlashCMD {
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

/// FlashState is used to track the current state and command of the flash.
#[derive(Clone, Copy, PartialEq)]
enum FlashState {
    Unconfigured,                 // Flash is unconfigured, call configure().
    Ready,                        // Flash is ready to complete a command.
    Read,                         // Performing a read operation.
    WriteUnlocking { page: i32 }, // Started a write operation.
    WriteErasing { page: i32 },   // Waiting on the page to erase.
    WriteWriting,                 // Waiting on the page to actually be written.
    EraseUnlocking { page: i32 }, // Started an erase operation.
    EraseErasing,                 // Waiting on the erase to finish.
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
pub static mut FLASH_CONTROLLER: FLASHCALW = FLASHCALW::new(
    FLASHCALW_BASE_ADDRS,
    pm::HSBClock::FLASHCALW,
    pm::HSBClock::FLASHCALWP,
    pm::PBBClock::FLASHCALW,
);

// Few constants relating to module configuration.
const PAGE_SIZE: u32 = 512;

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

impl FLASHCALW {
    const fn new(
        base_addr: usize,
        ahb_clk: pm::HSBClock,
        hramc1_clk: pm::HSBClock,
        pb_clk: pm::PBBClock,
    ) -> FLASHCALW {
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
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        regs.maint0.write(PicoCacheMaintenance0::INVALL::SET);
    }

    fn enable_picocache(&self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.ctrl.write(PicoCacheControl::CEN::Enable);
        } else {
            regs.ctrl.write(PicoCacheControl::CEN::Disable);
        }
    }

    /// Enable HCACHE
    pub fn enable_cache(&self) {
        // enable appropriate clocks
        pm::enable_clock(pm::Clock::HSB(pm::HSBClock::FLASHCALWP));
        pm::enable_clock(pm::Clock::PBB(pm::PBBClock::HRAMC1));

        // enable and wait for it to be ready
        self.enable_picocache(true);
        while !self.pico_enabled() {}
    }

    fn pico_enabled(&self) -> bool {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        regs.sr.is_set(PicoCacheStatus::CSTS)
    }

    pub fn handle_interrupt(&self) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };

        // Disable the interrupt line for flash
        regs.fcr.modify(FlashControl::FRDY::CLEAR);

        // Since the only interrupt on is FRDY, a command should have
        // either completed or failed at this point.

        // Check for errors and report to Client if there are any
        if self.is_error() {
            let attempted_operation = self.current_state.get();

            // Reset state now that we are ready to do a new operation.
            self.current_state.set(FlashState::Ready);

            self.client.get().map(|client| match attempted_operation {
                FlashState::Read => {
                    self.buffer.take().map(|buffer| {
                        client.read_complete(buffer, hil::flash::Error::FlashError);
                    });
                }
                FlashState::WriteUnlocking { .. }
                | FlashState::WriteErasing { .. }
                | FlashState::WriteWriting => {
                    self.buffer.take().map(|buffer| {
                        client.write_complete(buffer, hil::flash::Error::FlashError);
                    });
                }
                FlashState::EraseUnlocking { .. } | FlashState::EraseErasing => {
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
                self.current_state
                    .set(FlashState::WriteErasing { page: page });
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

                self.client.get().map(|client| {
                    client.erase_complete(hil::flash::Error::CommandComplete);
                });
            }
            _ => {
                self.current_state.set(FlashState::Ready);
            }
        }
    }

    /// FLASH properties.
    fn get_flash_size(&self) -> u32 {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        let flash_sizes = [
            4, 8, 16, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 2048,
        ];
        // get the FSZ number and lookup in the table for the size.
        flash_sizes[regs.fpr.read(FlashParameter::FSZ) as usize] << 10
    }

    /// FLASHC Control
    pub fn set_wait_state(&self, wait_state: u32) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        regs.fcr.modify(FlashControl::FWS.val(wait_state));
    }

    fn enable_ws1_read_opt(&mut self, enable: bool) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        if enable {
            regs.fcr.modify(FlashControl::WS1OPT::Optimize);
        } else {
            regs.fcr.modify(FlashControl::WS1OPT::NoOptimize);
        }
    }

    //  By default, we are going with High Speed Enable (based on our device running
    //  in PS2).
    #[cfg(not(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE))]
    fn set_flash_waitstate_and_readmode(
        &mut self,
        cpu_freq: u32,
        _ps_val: u32,
        _is_fwu_enabled: bool,
    ) {
        // ps_val and is_fwu_enabled not used in this implementation.
        if cpu_freq > FREQ_PS2_FWS_0_MAX_FREQ {
            self.set_wait_state(1);
        } else {
            self.set_wait_state(0);
        }

        self.issue_command(FlashCMD::HSEN, -1);
    }

    #[cfg(CONFIG_FLASH_READ_MODE_HIGH_SPEED_DISABLE)]
    fn set_flash_waitstate_and_readmode(
        &mut self,
        cpu_freq: u32,
        ps_val: u32,
        is_fwu_enabled: bool,
    ) {
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
        regs.fcr.modify(FlashControl::FWS::OneWaitState);

        // Enable high speed mode for flash
        regs.fcmd
            .modify(FlashCommand::KEY.val(0xA5) + FlashCommand::CMD::HSEN);

        // And wait for the flash to be ready
        while !regs.fsr.is_set(FlashStatus::FRDY) {}
    }

    /// Flashcalw status
    fn is_error(&self) -> bool {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        pm::enable_clock(self.pb_clock);
        regs.fsr.is_set(FlashStatus::LOCKE) | regs.fsr.is_set(FlashStatus::PROGE)
    }

    /// Flashcalw command control
    fn issue_command(&self, command: FlashCMD, page_number: i32) {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        pm::enable_clock(self.pb_clock);
        // For most commands we wait for the interrupt, for some certain
        // fast/rarely used commands or commands that don't generate interrupts
        // it is better to wait (or at least that is how this driver was
        // originally implemented).
        if command != FlashCMD::QPRUP && command != FlashCMD::QPR && command != FlashCMD::CPB
            && command != FlashCMD::HSEN
        {
            // Enable ready interrupt.
            regs.fcr.modify(FlashControl::FRDY::SET);
        }

        // Setup the command register to run this command.
        let mut cmd = FlashCommand::KEY.val(0xA5) + FlashCommand::CMD.val(command as u32);

        // If this command relies on using a certain page, we need to add that
        // in as well.
        if page_number >= 0 {
            cmd += FlashCommand::PAGEN.val(page_number as u32);
        }

        regs.fcmd.write(cmd);

        // Since we don't enable interrupts for these commands, spin wait
        // until they are finished. In particular, QPR and QPRUP will not issue
        // interrupts (see datasheet 14.6 paragraph 2).
        if command == FlashCMD::QPRUP || command == FlashCMD::QPR || command == FlashCMD::CPB
            || command == FlashCMD::HSEN
        {
            while !regs.fsr.is_set(FlashStatus::FRDY) {}
        }
    }

    /// Flashcalw global commands
    fn lock_page_region(&self, page_number: i32, lock: bool) {
        if lock {
            self.issue_command(FlashCMD::LP, page_number);
        } else {
            self.issue_command(FlashCMD::UP, page_number);
        }
    }

    /// Flashcalw Access to Flash Pages
    fn clear_page_buffer(&self) {
        self.issue_command(FlashCMD::CPB, -1);
    }

    fn is_page_erased(&self) -> bool {
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };
        regs.fsr.is_set(FlashStatus::QPRR)
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
        let regs: &FlashcalwRegisters = unsafe { &*self.registers };

        // Enable all clocks (if they aren't on already...).
        pm::enable_clock(self.ahb_clock);
        pm::enable_clock(self.hramc1_clock);
        pm::enable_clock(self.pb_clock);

        // Configure all other interrupts explicitly. Note the issue_command
        // function turns this on when need be.
        regs.fcr.modify(
            FlashControl::FRDY::CLEAR + FlashControl::LOCKE::CLEAR + FlashControl::PROGE::CLEAR
                + FlashControl::ECCE::CLEAR,
        );

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

    // Address is some raw address in flash that you want to read.
    fn read_range(
        &self,
        address: usize,
        size: usize,
        buffer: &'static mut Sam4lPage,
    ) -> ReturnCode {
        if self.current_state.get() == FlashState::Unconfigured {
            return ReturnCode::FAIL;
        }

        // Enable clock in case it's off.
        pm::enable_clock(self.ahb_clock);

        // Check that address makes sense and buffer has room.
        if address > (self.get_flash_size() as usize)
            || address + size > (self.get_flash_size() as usize) || address + size < size
            || buffer.len() < size
        {
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

    fn write_page(&self, page_num: i32, data: &'static mut Sam4lPage) -> ReturnCode {
        // Enable clock in case it's off.
        pm::enable_clock(self.ahb_clock);

        match self.current_state.get() {
            FlashState::Unconfigured => return ReturnCode::FAIL,
            FlashState::Ready => {}
            // If we're not ready don't take the command
            _ => return ReturnCode::EBUSY,
        }

        // Save the buffer for the future write.
        self.buffer.replace(data);

        self.current_state
            .set(FlashState::WriteUnlocking { page: page_num });
        self.lock_page_region(page_num, false);
        ReturnCode::SUCCESS
    }

    fn erase_page(&self, page_num: i32) -> ReturnCode {
        // Enable AHB clock (in case it was off).
        pm::enable_clock(self.ahb_clock);
        if self.current_state.get() != FlashState::Ready {
            return ReturnCode::EBUSY;
        }

        self.current_state
            .set(FlashState::EraseUnlocking { page: page_num });
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
