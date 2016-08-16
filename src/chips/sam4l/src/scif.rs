/* scif.rs -- System control interface for SAM4L
 *
 * This file includes support for the SCIF (Chapter 13 of SAML manual),
 * which configures system clocks. Does not currently support all
 * features/functionality: only main oscillator and generic clocks.
 *
 * Author: Philip Levis
 * Date: Aug 2, 2015
 */

use core::intrinsics;

pub enum Register {
  IER      = 0x00,
  IDR      = 0x04,
  IMR      = 0x08,
  ISR      = 0x0C,
  ICR      = 0x10,
  PCLKSR   = 0x14,
  UNLOCK   = 0x18,
  CSCR     = 0x1C,
  OSCCTRL0 = 0x20
}

#[allow(non_camel_case_types)]
pub enum ClockSource {
  RCSYS     =  0,
  OSC32K    =  1,
  DFFL0     =  2,
  OSC0      =  3,
  RC80M     =  4,
  RCFAST    =  5,
  RC1M      =  6,
  CLK_CPU   =  7,
  CLK_HSB   =  8,
  CLK_PBA   =  9,
  CLK_PBB   = 10,
  CLK_PBC   = 11,
  CLK_PBD   = 12,
  RC32K     = 13,
  RESERVED1 = 14,
  CLK_1K    = 15,
  PLL0      = 16,
  HRP       = 17,
  FP        = 18,
  GCLK_IN0  = 19,
  GCLK_IN1  = 20,
  GCLK11    = 21,
}

pub enum GenericClock {
  GCLK0,
  GCLK1,
  GCLK2,
  GCLK3,
  GCLK4,
  GCLK5,
  GCLK6,
  GCLK7,
  GCLK8,
  GCLK9,
  GCLK10,
  GCLK11
}

#[repr(C, packed)]
#[allow(missing_copy_implementations)]
struct Registers {
    ier: u32,
    idr: u32,
    imr: u32,
    isr: u32,
    icr: u32,
    pclksr: u32,
    unlock: u32,
    cscr: u32,
    //0x20
    oscctrl0: u32,
    pll0: u32,
    dfll0conf: u32,
    dfll0val: u32,
    dfll0mul: u32,
    dfll0step: u32,
    dfll0ssg: u32,
    dfll0ratio: u32,
    //0x40
    dfll0sync: u32,
    rccr: u32,
    rcfastcfg: u32,
    rfcastsr: u32,
    rc80mcr: u32,
    reserved0: [u32; 4],
    // 0x64
    hrpcr: u32,
    fpcr: u32,
    fpmul: u32,
    fpdiv: u32,
    gcctrl0: u32,
    gcctrl1: u32,
    gcctrl2: u32,
    // 0x80
    gcctrl3: u32,
    gcctrl4: u32,
    gcctrl5: u32,
    gcctrl6: u32,
    gcctrl7: u32,
    gcctrl8: u32,
    gcctrl9: u32,
    gcctrl10: u32,
    // 0xa0
    gcctrl11: u32
    //we leave out versions
}

pub const SCIF_BASE: isize = 0x400E0800;
static mut SCIF: *mut Registers = SCIF_BASE as *mut Registers;

#[allow(missing_copy_implementations)]
#[repr(usize)]
pub enum Clock {
    ClockRCSys = 0,
    ClockOsc32 = 1,
    ClockAPB = 2,
    ClockGclk2 = 3,
    Clock1K = 4
}

pub fn unlock(register: Register) {
    let val: u32 = 0xAA000000 | register as u32;
    unsafe {
        intrinsics::volatile_store(&mut (*SCIF).unlock, val);
    }
}

pub fn oscillator_enable(internal: bool) {
    // Casting a bool to a u32 is 0,1
    let val: u32 = (1 << 16) | internal as u32;
    unlock(Register::OSCCTRL0);
    unsafe {
        intrinsics::volatile_store(&mut (*SCIF).oscctrl0, val);
    }
}

pub fn oscillator_disable() {
    unlock(Register::OSCCTRL0);
    unsafe {
        intrinsics::volatile_store(&mut (*SCIF).oscctrl0, 0);
    }
}

pub fn generic_clock_disable(clock: GenericClock) {
    unsafe { match clock {
        GenericClock::GCLK0  => intrinsics::volatile_store(&mut (*SCIF).gcctrl0, 0),
        GenericClock::GCLK1  => intrinsics::volatile_store(&mut (*SCIF).gcctrl1, 0),
        GenericClock::GCLK2  => intrinsics::volatile_store(&mut (*SCIF).gcctrl2, 0),
        GenericClock::GCLK3  => intrinsics::volatile_store(&mut (*SCIF).gcctrl3, 0),
        GenericClock::GCLK4  => intrinsics::volatile_store(&mut (*SCIF).gcctrl4, 0),
        GenericClock::GCLK5  => intrinsics::volatile_store(&mut (*SCIF).gcctrl5, 0),
        GenericClock::GCLK6  => intrinsics::volatile_store(&mut (*SCIF).gcctrl6, 0),
        GenericClock::GCLK7  => intrinsics::volatile_store(&mut (*SCIF).gcctrl7, 0),
        GenericClock::GCLK8  => intrinsics::volatile_store(&mut (*SCIF).gcctrl8, 0),
        GenericClock::GCLK9  => intrinsics::volatile_store(&mut (*SCIF).gcctrl9, 0),
        GenericClock::GCLK10 => intrinsics::volatile_store(&mut (*SCIF).gcctrl10, 0),
        GenericClock::GCLK11 => intrinsics::volatile_store(&mut (*SCIF).gcctrl11, 0)
         };}
    }

pub fn generic_clock_enable(clock: GenericClock, 
                            source: ClockSource) {
    // Oscillator field is bits 12:9, bit 0 is enable
    let val = (source as u32) << 8 | 1;
    unsafe { match clock {
        GenericClock::GCLK0  => intrinsics::volatile_store(&mut (*SCIF).gcctrl0, val),
        GenericClock::GCLK1  => intrinsics::volatile_store(&mut (*SCIF).gcctrl1, val),
        GenericClock::GCLK2  => intrinsics::volatile_store(&mut (*SCIF).gcctrl2, val),
        GenericClock::GCLK3  => intrinsics::volatile_store(&mut (*SCIF).gcctrl3, val),
        GenericClock::GCLK4  => intrinsics::volatile_store(&mut (*SCIF).gcctrl4, val),
        GenericClock::GCLK5  => intrinsics::volatile_store(&mut (*SCIF).gcctrl5, val),
        GenericClock::GCLK6  => intrinsics::volatile_store(&mut (*SCIF).gcctrl6, val),
        GenericClock::GCLK7  => intrinsics::volatile_store(&mut (*SCIF).gcctrl7, val),
        GenericClock::GCLK8  => intrinsics::volatile_store(&mut (*SCIF).gcctrl8, val),
        GenericClock::GCLK9  => intrinsics::volatile_store(&mut (*SCIF).gcctrl9, val),
        GenericClock::GCLK10 => intrinsics::volatile_store(&mut (*SCIF).gcctrl10, val),
        GenericClock::GCLK11 => intrinsics::volatile_store(&mut (*SCIF).gcctrl11, val)
     };}
} 

    // Note that most clocks can only support 8 bits of divider:
    // interface does not currently check this. -pal
pub fn generic_clock_enable_divided(clock: GenericClock, 
                                    source: ClockSource, 
                                    divider: u16) {
    // Bits 31:16 -  divider
    // Bits 12:8  -  oscillator selection
    // Bit  1     -  divide enabled
    // Bit  0     -  clock enabled
    let val = (divider as u32) << 16 | ((source as u32) << 8) | 2 | 1;
    unsafe { match clock {
        GenericClock::GCLK0  => intrinsics::volatile_store(&mut (*SCIF).gcctrl0, val),
        GenericClock::GCLK1  => intrinsics::volatile_store(&mut (*SCIF).gcctrl1, val),
        GenericClock::GCLK2  => intrinsics::volatile_store(&mut (*SCIF).gcctrl2, val),
        GenericClock::GCLK3  => intrinsics::volatile_store(&mut (*SCIF).gcctrl3, val),
        GenericClock::GCLK4  => intrinsics::volatile_store(&mut (*SCIF).gcctrl4, val),
        GenericClock::GCLK5  => intrinsics::volatile_store(&mut (*SCIF).gcctrl5, val),
        GenericClock::GCLK6  => intrinsics::volatile_store(&mut (*SCIF).gcctrl6, val),
        GenericClock::GCLK7  => intrinsics::volatile_store(&mut (*SCIF).gcctrl7, val),
        GenericClock::GCLK8  => intrinsics::volatile_store(&mut (*SCIF).gcctrl8, val),
        GenericClock::GCLK9  => intrinsics::volatile_store(&mut (*SCIF).gcctrl9, val),
        GenericClock::GCLK10 => intrinsics::volatile_store(&mut (*SCIF).gcctrl10, val),
        GenericClock::GCLK11 => intrinsics::volatile_store(&mut (*SCIF).gcctrl11, val)
     };}
}
