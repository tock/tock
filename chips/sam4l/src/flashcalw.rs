//! chips::sam4l::flashcalw -- Implementation of a flash controller.
//!
//! This implementation of the flash controller for at sam4l flash controller
//! uses interrupts to handle main tasks of a flash -- write, reads, and erases.
//! If modifying this file, you should check whether the flash commands (issued
//! via issue_command) generates an interrupt and design a higher level function
//! based off of that.
//!
//! Although the datasheet says that when the FRDY interrupt is on, an interrupt will
//! be generated after a command is complete, it doesn't appear to occur for some
//! commands.
//!
//! A clean interface for reading from flash, writing pages and erasing pages is
//! defined below and should be used to handle the complexity of these tasks.
//!
//! The driver should be configure()'d before use, and a Client should be set to
//! enable a callback after a command is completed.
//!
//! Almost all of the flash controller functionality is implemented (except for
//! general purpose fuse bits, and more granular control of the cache).
//!
//! Author:  Kevin Baichoo <kbaichoo@cs.stanford.edu>
//! Date: July 27, 2016
//!

use core::cell::Cell;
use core::mem;
use kernel::common::VolatileCell;
use kernel::common::take_cell::MapCell;
use nvic;
use pm;

//  These are the registers of the PicoCache -- a cache dedicated to the flash.
#[allow(dead_code)]
struct PicocacheRegisters {
    _reserved_1: [u8; 8],
    control: VolatileCell<u32>,
    status: VolatileCell<u32>,
    _reserved_2: [u8; 16],
    maintenance_register_0: VolatileCell<u32>,
    maintenance_register_1: VolatileCell<u32>,
    montior_configuration: VolatileCell<u32>,
    monitor_enable: VolatileCell<u32>,
    monitor_control: VolatileCell<u32>,
    monitor_status: VolatileCell<u32>,
    _reserved_3: [u8; 196],
    version: VolatileCell<u32>,
}

//  Section 7 (the memory diagram) says the register starts at 0x400A0400
const PICOCACHE_OFFSET: usize = 0x400;


// Struct of the FLASHCALW registers. Section 14.10 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct Registers {
    control: VolatileCell<u32>,
    command: VolatileCell<u32>,
    status: VolatileCell<u32>,
    parameter: VolatileCell<u32>,
    version: VolatileCell<u32>,
    general_purpose_fuse_register_hi: VolatileCell<u32>,
    general_purpose_fuse_register_lo: VolatileCell<u32>,
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

/// Error codes are used to inform the Client if the command completed successfully
/// or whether there was an error and what type of error it was.
pub enum Error {
    CommandComplete, // Command Complete
    LockE, // Lock Error (i.e. tried writing to locked page)
    ProgE, // Program Error (i.e. incorrectly issued flash commands
    LockProgE, // Lock and Program Error
    ECC, // Error Correcting Code Error
}

/// High level commands to issue to the flash. Usually to track the state of
/// a command especially if it's multiple FlashCMDs.
///
/// For example an erase is: 1) Unlock Page  (UP)
///                          2) Erase Page   (EP)
///                          3) Lock Page    (LP)
/// Store what high level command we're doing allows us to track the state and
/// continue the steps of the command in handle_interrupt.
#[derive(Clone, Copy, PartialEq)]
pub enum Command {
    Write { page: i32 },
    Erase { page: i32 },
    None,
}



/// There are 18 recognized commands for the flash. These are 'bare-bones' commands
/// and values that are written to the Flash's command register to inform
/// the flash what to do. Table 14-5.
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

// The two Flash speeds
#[derive(Clone, Copy)]
pub enum Speed {
    Standard,
    HighSpeed,
}

/// FlashState is used to track the current state of the flash in high level
/// command.
///
/// Combined with Command, it defines a unique function the flash is preforming.
#[derive(Clone, Copy, PartialEq)]
pub enum FlashState {
    Locking, // The Flash is locking a region
    Unlocking, // The Flash is unlocking a region
    Writing, // The Flash is writing a page
    Erasing, // The Flash is erasing a page
    Ready, // The Flash is ready to complete a command
    Unconfigured, // The Flash is unconfigured, call configure()
}

// The FLASHCALW controller
pub struct FLASHCALW {
    registers: *mut Registers,
    cache: *mut PicocacheRegisters,
    ahb_clock: pm::Clock,
    hramc1_clock: pm::Clock,
    pb_clock: pm::Clock,
    error_status: Cell<u32>,
    ready: Cell<bool>,
    client: Cell<Option<&'static Client>>,
    current_state: Cell<FlashState>,
    current_command: Cell<Command>,
    page_buffer: MapCell<[u8; PAGE_SIZE as usize]>,
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

/// Trait for a client of the flash controller.
pub trait Client {
    //  Called upon a completed call
    fn command_complete(&self, err: Error);
}

impl FLASHCALW {
    const fn new(base_addr: usize,
                 ahb_clk: pm::HSBClock,
                 hramc1_clk: pm::HSBClock,
                 pb_clk: pm::PBBClock)
                 -> FLASHCALW {
        FLASHCALW {
            registers: base_addr as *mut Registers,
            cache: (base_addr + PICOCACHE_OFFSET) as *mut PicocacheRegisters,
            ahb_clock: pm::Clock::HSB(ahb_clk),
            hramc1_clock: pm::Clock::HSB(hramc1_clk),
            pb_clock: pm::Clock::PBB(pb_clk),
            error_status: Cell::new(0),
            ready: Cell::new(true),
            client: Cell::new(None),
            current_state: Cell::new(FlashState::Unconfigured),
            current_command: Cell::new(Command::None),
            page_buffer: MapCell::new([0; PAGE_SIZE as usize]),
        }
    }


    /// Cache controlling functionality.

    //  Flush the cache. Should be called after every write!
    fn invalidate_cache(&self) {
        let regs: &PicocacheRegisters = unsafe { mem::transmute(self.cache) };
        regs.maintenance_register_0.set(0x1);
    }

    pub fn enable_picocache(&self, enable: bool) {
        let regs: &PicocacheRegisters = unsafe { mem::transmute(self.cache) };
        if enable {
            regs.control.set(0x1);
        } else {
            regs.control.set(0x0);
        }
    }

    pub fn pico_enabled(&self) -> bool {
        let regs: &PicocacheRegisters = unsafe { mem::transmute(self.cache) };
        regs.status.get() & 0x1 != 0
    }

    // Helper to read a flashcalw register (espically if your function is doing so once)
    fn read_register(&self, key: RegKey) -> u32 {
        let registers: &mut Registers = unsafe { mem::transmute(self.registers) };

        match key {
            RegKey::CONTROL => registers.control.get(),
            RegKey::COMMAND => registers.command.get(),
            RegKey::STATUS => registers.status.get(),
            RegKey::PARAMETER => registers.parameter.get(),
            RegKey::VERSION => registers.version.get(),
            RegKey::GPFRHI => registers.general_purpose_fuse_register_hi.get(),
            RegKey::GPFRLO => registers.general_purpose_fuse_register_lo.get(),
        }
    }


    pub fn handle_interrupt(&self) {
        unsafe {
            //  mark the controller as ready and clear pending interrupt
            self.ready.set(true);
            nvic::clear_pending(nvic::NvicIdx::HFLASHC);
        }

        let error_status = self.get_error_status();
        self.error_status.set(error_status);

        //  Since the only interrupt on is FRDY, a command should have
        //  either completed or failed at this point.

        // Check for errors and report to Client if there are any
        if error_status != 0 {
            // reset commands / ready
            self.current_command.set(Command::None);
            self.current_state.set(FlashState::Ready);

            self.client.get().map(|client| {
                // call command complete with error
                match error_status {
                    4 => {
                        client.command_complete(Error::LockE);
                    }
                    8 => {
                        client.command_complete(Error::ProgE);
                    }
                    12 => {
                        client.command_complete(Error::LockProgE);
                    }
                    _ => {}
                }
            });
        }

        //  Part of a command succeeded -- continue onto next steps.

        match self.current_command.get() {
            Command::Write { page } => {
                match self.current_state.get() {
                    FlashState::Unlocking => {
                        self.current_state.set(FlashState::Erasing);
                        self.flashcalw_erase_page(page, true);
                    }
                    FlashState::Erasing => {
                        //  Write page buffer isn't really a command, and
                        //  clear page buffer dosn't trigger an interrupt thus
                        //  I'm combining these with an actual command, write_page,
                        //  which generates and interrupt and saves the page.
                        self.clear_page_buffer();
                        self.write_to_page_buffer(page as usize * PAGE_SIZE as usize);

                        self.current_state.set(FlashState::Writing);
                        self.flashcalw_write_page(page);
                    }
                    FlashState::Writing => {
                        // Flush the cache
                        self.invalidate_cache();
                        self.current_state.set(FlashState::Locking);
                        self.lock_page_region(page, true);
                    }
                    FlashState::Locking => {
                        self.current_state.set(FlashState::Ready);
                        self.current_command.set(Command::None);
                    }
                    _ => {
                        assert!(false) /* should never reach here */
                    }

                }
            }
            Command::Erase { page } => {
                match self.current_state.get() {
                    FlashState::Unlocking => {
                        self.current_state.set(FlashState::Erasing);
                        self.flashcalw_erase_page(page, true);
                    }
                    FlashState::Erasing => {
                        self.current_state.set(FlashState::Locking);
                        self.lock_page_region(page, true);
                    }
                    FlashState::Locking => {
                        self.current_state.set(FlashState::Ready);
                        self.current_command.set(Command::None);
                    }
                    _ => {
                        assert!(false); /* should never happen. */
                    }
                }
            }
            Command::None => {
                self.current_state.set(FlashState::Ready);
            }

        }

        //  If the command is finished call the complete CB.
        if self.current_command.get() == Command::None &&
           self.current_state.get() == FlashState::Ready {
            self.client.get().map(|value| { value.command_complete(Error::CommandComplete); });
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
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        if wait_state == 1 {
            regs.control.set(regs.control.get() | bit!(6));
        } else {
            regs.control.set(regs.control.get() & !bit!(6));
        }
    }

    fn enable_ws1_read_opt(&mut self, enable: bool) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        if enable {
            regs.control.set(regs.control.get() | bit!(7));
        } else {
            regs.control.set(regs.control.get() | !bit!(7));
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


    #[allow(dead_code)]
    fn is_ready_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(0)) != 0
    }

    fn enable_ready_int(&self, enable: bool) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        if enable {
            regs.control.set(regs.control.get() | bit!(0));
        } else {
            regs.control.set(regs.control.get() & !bit!(0));
        }
    }

    #[allow(dead_code)]
    fn is_lock_error_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(2)) != 0
    }

    fn enable_lock_error_int(&self, enable: bool) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        if enable {
            regs.control.set(regs.control.get() | bit!(2));
        } else {
            regs.control.set(regs.control.get() & !bit!(2));
        }
    }

    #[allow(dead_code)]
    fn is_prog_error_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(3)) != 0
    }

    fn enable_prog_error_int(&self, enable: bool) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        if enable {
            regs.control.set(regs.control.get() | bit!(3));
        } else {
            regs.control.set(regs.control.get() & !bit!(3));
        }
    }

    #[allow(dead_code)]
    fn is_ecc_int_enabled(&self) -> bool {
        (self.read_register(RegKey::CONTROL) & bit!(4)) != 0
    }

    fn enable_ecc_int(&self, enable: bool) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        if enable {
            regs.control.set(regs.control.get() | bit!(4));
        } else {
            regs.control.set(regs.control.get() & !bit!(4));
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
            //  enable ready int and mark the controller as being unavaliable.
            self.ready.set(false);
            self.enable_ready_int(true);
        }

        let cmd_regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        let mut reg_val: u32 = cmd_regs.command.get();

        let clear_cmd_mask: u32 = !(bit!(6) - 1);
        reg_val &= clear_cmd_mask;

        // craft the command
        if page_number >= 0 {
            reg_val = FLASHCALW_CMD_KEY | (page_number as u32) << 8 | command as u32;
        } else {
            reg_val |= FLASHCALW_CMD_KEY | command as u32;
        }

        cmd_regs.command.set(reg_val); // write the cmd

        if command == FlashCMD::QPRUP || command == FlashCMD::QPR || command == FlashCMD::CPB ||
           command == FlashCMD::HSEN {
            self.error_status.set(self.get_error_status());
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
        let registers: &mut Registers = unsafe { mem::transmute(self.registers) };
        let status = registers.status.get();

        (status & bit!(5)) != 0
    }

    fn quick_page_read(&self, page_number: i32) -> bool {
        self.issue_command(FlashCMD::QPR, page_number);
        self.is_page_erased()
    }

    fn flashcalw_erase_page(&self, page_number: i32, check: bool) -> bool {
        let mut page_erased = true;

        self.issue_command(FlashCMD::EP, page_number);
        if check {
            let mut error_status: u32 = self.error_status.get();
            page_erased = self.quick_page_read(-1);

            //  issue command should have changed the error status.
            error_status |= self.error_status.get();
            self.error_status.set(error_status);
        }

        page_erased
    }

    fn flashcalw_write_page(&self, page_number: i32) {
        self.issue_command(FlashCMD::WP, page_number);
    }

    /// There's a user_page that isn't contigous with the rest of the flash. Currently
    /// it's not being used.
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

    //  Instead of having several memset/memcpy functions as Atmel's ASF implementation
    //  will only have one to write to the page buffer
    fn write_to_page_buffer(&self, pg_buff_addr: usize) {
        let mut page_buffer: *mut u8 = pg_buff_addr as *mut u8;

        //   Errata 45.1.7 - Need to write a 64-bit all one word for every write to
        //   the page buffer.
        let cleared_double_word: [u8; 8] = [255; 8];
        let clr_ptr: *const u8 = &cleared_double_word[0] as *const u8;

        //  borrow the page buffer from the take cell
        let buffer = self.page_buffer.take().unwrap();

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
        //  replace the page buffer in the take cell
        self.page_buffer.put(buffer);
    }

    // returns the error_status (useful for debugging).
    pub fn debug_error_status(&self) -> u32 {
        self.error_status.get()
    }
}

// Implementation of high level calls using the low-lv functions.
impl FLASHCALW {
    pub fn set_client(&self, client: &'static Client) {
        self.client.set(Some(client));
    }

    pub fn configure(&mut self) {
        // enable all clocks (if they aren't on already...)
        unsafe {
            pm::enable_clock(self.ahb_clock);
            pm::enable_clock(self.hramc1_clock);
            pm::enable_clock(self.pb_clock);

        }

        // enable interrupts from nvic
        unsafe {
            nvic::enable(nvic::NvicIdx::HFLASHC);
        }

        // configure all other interrupts explicitly.
        self.enable_ready_int(false); // note the issue_command function turns this
        // on when need be.
        self.enable_lock_error_int(false);
        self.enable_prog_error_int(false);
        self.enable_ecc_int(false);

        //  enable wait state 1 optimization
        self.enable_ws1_read_opt(true);
        // change speed mode
        self.set_flash_waitstate_and_readmode(48_000_000, 0, false);

        //  By default the picocache ( a cache only for the flash) is turned off.
        //  However the bootloader turns it on. I will explicitly turn it on here.
        //  So if the bootloader changes, nothing breaks.
        self.enable_picocache(true);

        self.current_state.set(FlashState::Ready);
    }

    pub fn get_page_size(&self) -> u32 {
        PAGE_SIZE
    }

    pub fn get_number_pages(&self) -> u32 {
        // check clock and enable just incase
        unsafe {
            pm::enable_clock(self.pb_clock);
        }
        self.get_page_count()
    }

    // Address is some raw address in flash that you want to read.
    pub fn read(&self, address: usize, size: usize, buffer: &mut [u8]) -> i32 {
        // enable clock incase it's off
        unsafe {
            pm::enable_clock(self.ahb_clock);
        }

        //  check that address makes sense and buffer has room
        if address > (self.get_flash_size() as usize) ||
           address + size > (self.get_flash_size() as usize) ||
           address + size < size || buffer.len() < size {
            // invalid flash address
            return -1;
        }

        let mut byte: *const u8 = address as *const u8;
        unsafe {
            for i in 0..size {
                buffer[i] = *byte;
                byte = byte.offset(1);
            }
        }
        0
    }

    pub fn write_page(&self, page_num: i32, data: &[u8]) -> i32 {
        // enable clock incase it's off
        unsafe {
            pm::enable_clock(self.ahb_clock);
        }

        // if we're not ready don't take the command.
        if self.current_state.get() != FlashState::Ready {
            return -1;
        }

        //  check data length is of size 'page_size'
        if data.len() != self.get_page_size() as usize {
            return -1;
        }

        self.page_buffer.map(|value| { value.clone_from_slice(&data); });

        self.current_state.set(FlashState::Unlocking);
        self.current_command.set(Command::Write { page: page_num });
        self.lock_page_region(page_num, false);
        0
    }

    pub fn erase_page(&self, page_num: i32) -> i32 {
        // Enable AHB clock (incase it was off).
        unsafe {
            pm::enable_clock(self.ahb_clock);
        }
        if self.current_state.get() != FlashState::Ready {
            return -1;
        }

        self.current_state.set(FlashState::Unlocking);
        self.current_command.set(Command::Erase { page: page_num });
        self.lock_page_region(page_num, false);
        0
    }
}

///  Assumes the only Peripheral Interrupt enabled for the FLASHCALW is the
///  FRDY (Flash Ready) interrupt.
pub unsafe extern "C" fn flash_handler() {
    use kernel::common::Queue;
    use chip;

    //  disable the nvic interrupt line for flash, turn of the perherial interrupt,
    //  and queue a handle interrupt.
    FLASH_CONTROLLER.enable_ready_int(false);
    nvic::disable(nvic::NvicIdx::HFLASHC);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(nvic::NvicIdx::HFLASHC);
}
