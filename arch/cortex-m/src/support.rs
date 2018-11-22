use core::ops::FnOnce;

#[cfg(target_os = "none")]
#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[cfg(not(target_os = "none"))]
/// NOP instruction (mock)
pub fn nop() {}

#[cfg(target_os = "none")]
#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    asm!("wfi" :::: "volatile");
}

#[cfg(not(target_os = "none"))]
/// WFI instruction (mock)
pub unsafe fn wfi() {}

#[cfg(target_os = "none")]
#[inline(always)]
/// DSB instruction
pub unsafe fn dsb() {
    asm!("dsb 0xf" :::: "volatile");
}

#[cfg(not(target_os = "none"))]
/// DSB instruction (mock)
pub unsafe fn dsb() {}

#[cfg(not(target_os = "none"))]
pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

#[cfg(target_os = "none")]
pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Set PRIMASK
    asm!("cpsid i" :::: "volatile");

    let res = f();

    // Unset PRIMASK
    asm!("cpsie i" :::: "volatile");
    return res;
}

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
