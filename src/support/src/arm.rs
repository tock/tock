use core::ops::FnOnce;

#[cfg(not(test))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[cfg(test)]
/// NOP instruction (mock)
pub fn nop() {}

#[cfg(not(test))]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    asm!("wfi" :::: "volatile");
}

#[cfg(test)]
/// WFI instruction (mock)
pub unsafe fn wfi() {}

pub unsafe fn atomic<F, R>(f: F) -> R
    where F: FnOnce() -> R
{
    // Set PRIMASK
    asm!("cpsid i" :::: "volatile");

    let res = f();

    // Unset PRIMASK
    asm!("cpsie i" :::: "volatile");
    return res;
}

#[cfg(not(test))]
#[lang="eh_personality"]
pub extern "C" fn eh_personality() {}
