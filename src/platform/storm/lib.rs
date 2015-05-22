#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(asm,core,concat_idents,no_std)]

extern crate core;
extern crate hil;

macro_rules! volatile {
    ($item:expr) => ({
        use core::intrinsics::volatile_load;
        unsafe { volatile_load(&$item) }
    });

    ($item:ident = $value:expr) => ({
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, $value); }
    });
}

macro_rules! volatile_bitset {
    ($item:expr, $value:expr) => ({
        use core::intrinsics::volatile_load;
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, volatile_load(&$item) | $value); }
    });
}

macro_rules! volatile_bitclear {
    ($item:expr, $value:expr) => ({
        // Bitwise negation is Rust is ! instead of ~, because they just had
        // to be different. [fwiw, I agree with the underlying premise that
        // with Rust's type system there's no reason for a logical not operator
        // to exist---comparisons to bool should cast to bool explicitly to be
        // unambiguous---however, they should have used ~ as the operator to
        // preserve consistency with existing languages]
        use core::intrinsics::volatile_load;
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, volatile_load(&$item) & !$value); }
    });
}


/// Macro handles reference counting for clock enable/disable. Inside of a
/// peripheral driver this decrements the number of things using the peripheral
/// (and therefore the clock) and if that count hits 0, disables the clock.
///
/// Right now the macro requires that self.clock exists.
// bradjc: I would like to pass self.clock into the macro, but I don't know
//         how to make that work.
macro_rules! enable_reference_increment {
    ($count:ident, $self_:ident) => ({
        let res = unsafe {
            let num_enabled = &mut $count as *mut isize;
            intrinsics::atomic_xadd(num_enabled, 1)
        };
        if res == 1 {
            sam4l::pm::enable_clock($self_.clock);
        }
    });
}

macro_rules! enable_reference_decrement {
    ($count:ident, $self_:ident) => ({
        let res = unsafe {
            let num_enabled = &mut $count as *mut isize;
            intrinsics::atomic_xsub(num_enabled, 1)
        };
        if res == 0 {
            sam4l::pm::disable_clock($self_.clock);
        }
    });
}

pub mod ast;
pub mod nvic;
pub mod pm;
pub mod gpio;
