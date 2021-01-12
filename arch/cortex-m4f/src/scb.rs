//! ARM System Control Block
//!
//! <http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dui0553a/CIHFDJCA.html>

pub use cortexm4::scb;
pub use cortexm4::scb::{reset, set_vector_table_offset, unset_sleepdeep};
#[allow(unused_imports)]
use cortexm4::scb::{CoprocessorAccessControl, SCB};

/// Disable the FPU
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe fn disable_fpca() {
    SCB.cpacr
        .modify(CoprocessorAccessControl::CP10::CLEAR + CoprocessorAccessControl::CP11::CLEAR);

    asm!("dsb", "isb", options(nomem, nostack, preserves_flags));

    if SCB.cpacr.read(CoprocessorAccessControl::CP10) != 0
        || SCB.cpacr.read(CoprocessorAccessControl::CP11) != 0
    {
        panic!("Unable to disable FPU");
    }
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe fn disable_fpca() {
    unimplemented!()
}

/// Enable the FPU
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub unsafe fn enable_fpca() {
    SCB.cpacr
        .modify(CoprocessorAccessControl::CP10::SET + CoprocessorAccessControl::CP11::SET);

    asm!("dsb", "isb", options(nomem, nostack, preserves_flags));

    if SCB.cpacr.read(CoprocessorAccessControl::CP10) != 3
        || SCB.cpacr.read(CoprocessorAccessControl::CP11) != 3
    {
        panic!("Unable to enable FPU");
    }
}

// Mock implementation for tests on Travis-CI.
#[cfg(not(any(target_arch = "arm", target_os = "none")))]
pub unsafe fn enable_fpca() {
    unimplemented!()
}
