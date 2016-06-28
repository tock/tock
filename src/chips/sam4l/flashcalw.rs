/// FLASHCALW Driver for the SAM4L.


use helpers::*;
use core::mem;

use pm;


// Listing of the FLASHCALW register memory map.
// Section 14.10 of the datasheet
#[repr(C, packed)]
#[allow(dead_code)]
struct Registers {
    control:                          usize,
    command:                          usize,
    status:                           usize,
    parameter:                        usize,
    version:                          usize,
    general_purpose_fuse_register_hi: usize,
    general_purpose_fuse_register_lo: usize,
}

const FLASHCALW_BASE_ADDRS : *mut Registers = 0x400A0000 as *mut Registers;

// This is the pico cache registers...
// TODO: does this get it's own driver... yea....
/*
struct Picocache_Registers {
    picocache_control:                      usize,
    picocache_status:                       usize,
    picocache_maintenance_register_0:       usize,
    picocache_maintenance_register_1:       usize,
    picocache_montior_configuration:        usize,
    picocache_monitor_enable:               usize,
    picocache_monitor_control:              usize,
    picocache_monitor_status:               usize,
    version:                                usize
}
*/

// There are 18 recognized commands possible to command the flash
// Table 14-5.
#[derive(Clone, Copy)]
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

//The two Flash speeds
#[derive(Clone, Copy)]
pub enum Speed {
    Standard,
    HighSpeed
}

// The FLASHCALW controller
//TODO: finishing beefing up...
pub struct FLASHCALW {
    registers: *mut Registers,
    // might make these more specific
    ahb_clock: pm::Clock,
    hramc1_clock: pm::Clock,
    pb_clock: pm::Clock,
    //client: TakeCell...

}

//static instance for the board. Only one FLASHCALW on chip.
pub static mut flash_controller : FLASHCALW = 
    FLASHCALW::new(FLASHCALW_BASE_ADDRS, pm::HSBClock::FLASHCALW, 
        pm::HSBClock::FLASHCALW, pm::PBBClock::FLASHCALW);


// Few constants relating to module configuration.
const FLASH_PAGE_SIZE : usize = 512;
const FLASH_NB_OF_REGIONS : usize = 16;
const FLASHCALW_REGIONS : usize = FLASH_NB_OF_REGIONS;
const FLASHCALW_CMD_KEY : usize = 0xA5;

// a variable to hold the error statuses of all Flashcalw commands so far.
static mut flashcalw_error_status : u32 = 0;


//A few constants just for readability
const BIT_ON: u32 = 1;

// TODO: should export this to a chip specific module or so... something that gives me size.
//const FLASHCALW_SIZE : usize = 512; // instead I'll just read it straight from the table
                                      // which will be alloced only for a fxn call.

macro_rules! get_bit {
    ($w:expr) => (0x1u32 << $w);
}

impl FLASHCALW {
    //TODO: remove this as more files are linked ( somehow to ./lib.rs
    //#![feature(const_fn)]
    //const fn new(base_addr: *mut Registers, ahb_clk: pm::HSBClock, TODO: change heading back to const_fn
    pub fn new(base_addr: *mut Registers, ahb_clk: pm::HSBClock, 
    hramc1_clk: pm::HSBClock, pb_clk: pm::PBBClock) -> FLASHCALW {
        FLASHCALW {
            registers: base_addr,
            ahb_clock: pm::Clock::HSB(ahb_clk),
            hramc1_clock: pm::Clock::HSB(hramc1_clk),
            pb_clock: pm::Clock::HSB(pb_clk),
        }   
    }

    /// FLASH properties.
    pub fn get_flash_size(&self) -> u32 {
        let flash_sizes = [4,
                           8,
                           16,
                           32,
                           48,
                           64,
                           96,
                           128,
                           192,
                           256,
                           384,
                           512,
                           768,
                           1024,
                           2048];
        flash_sizes[self.registers.parameter & 0xf] // get the FSZ number and 
                                                    // lookup in the table for the size.
        //FLASHCALW_SIZE
        //could also just read this from the flash...
    }

    pub fn get_page_count(&self) -> u32 {
        self.flashcalw_get_flash_size() / FLASH_PAGE_SIZE    
    }

    pub fn get_page_count_per_region(&self) -> u32 {
        self.flashcalw_get_page_count() / FLASH_NB_OF_REGIONS
    }


    pub fn get_page_region(&self, page_number : i32) -> u32 {
        (if page_number >= 0 
            { page_number } 
        else 
            { self.flashcalw_get_page_number() as i32 } 
        / self.flashcalw_get_page_count_per_region())
    }

    pub fn get_region_first_page_number(&self, region : u32) -> u32 {
        region * self.flashcalw_get_page_count_per_region()    
    }


    /// FLASHC Control
        //TODO:
    
    ///Flashcalw status


    ///Flashcalw command control
    pub fn get_command(&self) -> FlashCMD {
        (self.register.command & (get_bit!(6) - 1)) as FlashCMD
    }

    pub fn get_page_number(&self) -> u32 {
        //create a mask for the page number field
        let mut page_mask : u32 = get_bit!(8) - 1;
        page_mask |= page_mask << 24;
        page_mask = !page_mask;

        self.registers.command & page_mask 
    }
    
    pub fn issue_command(&self, command : FlashCMD, page_number : i32) {
        wait_until_ready(); // would be better to avoid busy waiting...
        let mut reg_val : usize = self.registers.command;
        
        let clear_cmd_mask : usize = !(get_bit!(6) - 1);
        reg_val &= clear_cmd_mask;
        
        // craft the command
        if page_number >= 0 {
            reg_val = ( FLASHCALW_CMD_KEY << 24 | 
                        page_number << 8        |
                        command as usize);
        } else {
            reg_val |= (FLASHCALW_CMD_KEY << 24 | command as usize);     
        }
        let cmd_regs : &mut Registers = unsafe {mem::transmute(self.registers)};
        volatile_store(&mut cmd_regs.command, reg_val); // write the cmd

        flashcalw_error_status = get_error_status();
        wait_until_ready();
    }


    /// Flashcalw global commands
    pub fn flashcalw_no_operation(&self) {
        self.issue_command(NOP, -1);        
    }

    pub fn erase_all(&self) {
        self.issue_command(EA, -1);    
    }

    ///FLASHCALW Protection Mechanisms
    //Todo: write out bit positions?
    pub fn is_security_bit_active(&self) -> bool {
        (self.registers.status & get_bit!(4)) != 0
    }

    pub fn set_security_bit(&self) {
       /* let regs : &mut Registers = unsafe {mem::transmute(self.registers)};
        volatile_store(&mut regs.status, (regs.status | get_bit!(4))); */
        self.issue_command(SSB, -1);
    }

    pub fn is_page_region_locked(&self, page_number : u32) -> bool {
        self.is_region_locked(get_page_region(page_number))
    }

    pub fn is_region_locked(&self, region : u32) -> bool {
        (self.registers.status & get_bit!(region + 16)) != 0    
    }
    
    pub fn lock_page_region(&self, page_number : i32, lock : bool) {
        if lock {
            self.issue_command(LP, page_number);
        } else {
            self.issue_command(UP, page_number);
        }
    }

    pub fn lock_region(&self, region : u32, lock : bool) {
        self.lock_page_region(self.get_region_first_page_number(region), lock);    
    }

    pub fn lock_all_regions(&self, lock : bool) {
        let mut error_status = 0;
        let mut num_regions = FLASHCALW_REGIONS;

        while num_regions >= 0 {
            self.lock_region(num_regions, lock);
            unsafe {
                error_status |= flashcalw_error_status;    
            }
            num_regions -= 1;
        }
        
        unsafe {
            flashcalw_error_status = error_status;
        }
    }
}

