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
pub unsafe fn atomic_write(location: &mut u64, value: u64) {
    // Set PRIMASK
    asm!("cpsid i" :::: "volatile");
    *location = value;
    // Unset PRIMASK
    asm!("cpsie i" :::: "volatile");
}

#[cfg(target_os = "none")]
pub unsafe fn atomic_read(location: &u64) -> u64 {
    let ret;
    // Set PRIMASK
    asm!("cpsid i" :::: "volatile");
    ret = *location;
    // Unset PRIMASK
    asm!("cpsie i" :::: "volatile");
    ret
}

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
