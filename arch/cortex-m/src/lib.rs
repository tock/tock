//! Generic support for all Cortex-M platforms.

#![crate_name = "cortexm"]
#![crate_type = "rlib"]
#![feature(llvm_asm, lang_items)]
#![no_std]

use core::fmt::Write;

pub mod nvic;
pub mod scb;
pub mod support;
pub mod syscall;
pub mod systick;

pub unsafe fn print_cortexm_state(writer: &mut dyn Write) {
    let _ccr = syscall::SCB_REGISTERS[0];
    let cfsr = syscall::SCB_REGISTERS[1];
    let hfsr = syscall::SCB_REGISTERS[2];
    let mmfar = syscall::SCB_REGISTERS[3];
    let bfar = syscall::SCB_REGISTERS[4];

    let iaccviol = (cfsr & 0x01) == 0x01;
    let daccviol = (cfsr & 0x02) == 0x02;
    let munstkerr = (cfsr & 0x08) == 0x08;
    let mstkerr = (cfsr & 0x10) == 0x10;
    let mlsperr = (cfsr & 0x20) == 0x20;
    let mmfarvalid = (cfsr & 0x80) == 0x80;

    let ibuserr = ((cfsr >> 8) & 0x01) == 0x01;
    let preciserr = ((cfsr >> 8) & 0x02) == 0x02;
    let impreciserr = ((cfsr >> 8) & 0x04) == 0x04;
    let unstkerr = ((cfsr >> 8) & 0x08) == 0x08;
    let stkerr = ((cfsr >> 8) & 0x10) == 0x10;
    let lsperr = ((cfsr >> 8) & 0x20) == 0x20;
    let bfarvalid = ((cfsr >> 8) & 0x80) == 0x80;

    let undefinstr = ((cfsr >> 16) & 0x01) == 0x01;
    let invstate = ((cfsr >> 16) & 0x02) == 0x02;
    let invpc = ((cfsr >> 16) & 0x04) == 0x04;
    let nocp = ((cfsr >> 16) & 0x08) == 0x08;
    let unaligned = ((cfsr >> 16) & 0x100) == 0x100;
    let divbyzero = ((cfsr >> 16) & 0x200) == 0x200;

    let vecttbl = (hfsr & 0x02) == 0x02;
    let forced = (hfsr & 0x40000000) == 0x40000000;

    let _ = writer.write_fmt(format_args!("\r\n---| Fault Status |---\r\n"));

    if iaccviol {
        let _ = writer.write_fmt(format_args!(
            "Instruction Access Violation:       {}\r\n",
            iaccviol
        ));
    }
    if daccviol {
        let _ = writer.write_fmt(format_args!(
            "Data Access Violation:              {}\r\n",
            daccviol
        ));
    }
    if munstkerr {
        let _ = writer.write_fmt(format_args!(
            "Memory Management Unstacking Fault: {}\r\n",
            munstkerr
        ));
    }
    if mstkerr {
        let _ = writer.write_fmt(format_args!(
            "Memory Management Stacking Fault:   {}\r\n",
            mstkerr
        ));
    }
    if mlsperr {
        let _ = writer.write_fmt(format_args!(
            "Memory Management Lazy FP Fault:    {}\r\n",
            mlsperr
        ));
    }

    if ibuserr {
        let _ = writer.write_fmt(format_args!(
            "Instruction Bus Error:              {}\r\n",
            ibuserr
        ));
    }
    if preciserr {
        let _ = writer.write_fmt(format_args!(
            "Precise Data Bus Error:             {}\r\n",
            preciserr
        ));
    }
    if impreciserr {
        let _ = writer.write_fmt(format_args!(
            "Imprecise Data Bus Error:           {}\r\n",
            impreciserr
        ));
    }
    if unstkerr {
        let _ = writer.write_fmt(format_args!(
            "Bus Unstacking Fault:               {}\r\n",
            unstkerr
        ));
    }
    if stkerr {
        let _ = writer.write_fmt(format_args!(
            "Bus Stacking Fault:                 {}\r\n",
            stkerr
        ));
    }
    if lsperr {
        let _ = writer.write_fmt(format_args!(
            "Bus Lazy FP Fault:                  {}\r\n",
            lsperr
        ));
    }
    if undefinstr {
        let _ = writer.write_fmt(format_args!(
            "Undefined Instruction Usage Fault:  {}\r\n",
            undefinstr
        ));
    }
    if invstate {
        let _ = writer.write_fmt(format_args!(
            "Invalid State Usage Fault:          {}\r\n",
            invstate
        ));
    }
    if invpc {
        let _ = writer.write_fmt(format_args!(
            "Invalid PC Load Usage Fault:        {}\r\n",
            invpc
        ));
    }
    if nocp {
        let _ = writer.write_fmt(format_args!(
            "No Coprocessor Usage Fault:         {}\r\n",
            nocp
        ));
    }
    if unaligned {
        let _ = writer.write_fmt(format_args!(
            "Unaligned Access Usage Fault:       {}\r\n",
            unaligned
        ));
    }
    if divbyzero {
        let _ = writer.write_fmt(format_args!(
            "Divide By Zero:                     {}\r\n",
            divbyzero
        ));
    }

    if vecttbl {
        let _ = writer.write_fmt(format_args!(
            "Bus Fault on Vector Table Read:     {}\r\n",
            vecttbl
        ));
    }
    if forced {
        let _ = writer.write_fmt(format_args!(
            "Forced Hard Fault:                  {}\r\n",
            forced
        ));
    }

    if mmfarvalid {
        let _ = writer.write_fmt(format_args!(
            "Faulting Memory Address:            {:#010X}\r\n",
            mmfar
        ));
    }
    if bfarvalid {
        let _ = writer.write_fmt(format_args!(
            "Bus Fault Address:                  {:#010X}\r\n",
            bfar
        ));
    }

    if cfsr == 0 && hfsr == 0 {
        let _ = writer.write_fmt(format_args!("No faults detected.\r\n"));
    } else {
        let _ = writer.write_fmt(format_args!(
            "Fault Status Register (CFSR):       {:#010X}\r\n",
            cfsr
        ));
        let _ = writer.write_fmt(format_args!(
            "Hard Fault Status Register (HFSR):  {:#010X}\r\n",
            hfsr
        ));
    }
}
