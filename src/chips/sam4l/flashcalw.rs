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

//static instance for the board. Only one FLASHCALW needed / on board.
pub static mut flash_controller : FLASHCALW = 
    FLASHCALW::new(FLASHCALW_BASE_ADDRS, pm::HSBClock::FLASHCALW, 
        pm::HSBClock::FLASHCALW, pm::PBBClock::FLASHCALW);


// Few constants relating to module configuration.
const FLASH_PAGE_SIZE : usize = 512;
const FLASH_NB_OF_REGIONS : usize = 16;
const FLASHCALW_REGIONS : usize = FLASH_NB_OF_REGIONS;

// TODO: should export this to a board specific module or so... something that gives me size.
const FLASHCALW_SIZE : usize = 512;

impl FLASHCALW {
    const fn new(base_addr: *mut Registers, ahb_clk: pm::HSBClock, 
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
        FLASHCALW_SIZE    
    }

    pub fn get_page_count(&self) -> u32 {
        flashcalw_get_flash_size() / FLASH_PAGE_SIZE    
    }

    pub fn get_page_count_per_region(&self) -> u32 {
        flashcalw_get_page_count() / FLASH_NB_OF_REGIONS
    }

    pub fn get_page_number(&self) -> u32 {
        1
        // DUMMY IMPLEMENTATION 
    }

    // TODO: implement get_page_number().
    pub fn get_page_region(&self, page_number : i32) -> u32 {
        (if page_number >= 0 
            { page_number } 
        else 
            { flashcalw_get_page_number() as i32 } 
        / flashcalw_get_page_count_per_region())
    }

    pub fn get_region_first_page_number(&self, region : u32) -> u32 {
        region * flashcalw_get_page_count_per_region()    
    }


    /// FLASHC Control
        //TODO:
    

    ///FLASHCALW Protection Mechanisms
    pub fn is_security_bit_active(&self) -> bool {
        //think about this more ( get a better design hmmm...)
        (self.registers.status & 0x16) == 1
    }

    pub fn set_security_bit(&self) {
        self.registers.status |=     
    }

}

