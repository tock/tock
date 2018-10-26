//! Tock generic initial runtime (`rt0`) helpers

#![no_std]

/// Initializes the `static data`, by copying it into memory (RAM) from
/// non-volatile memory (Flash)
// Relocate data segment.
// Assumes data starts right after text segment as specified by the linker
pub unsafe fn init_data(mut edata: *mut u32, mut sdata: *mut u32, sdata_end: *mut u32) {
    while sdata < sdata_end {
        sdata.write(edata.read());
        sdata = sdata.offset(1);
        edata = edata.offset(1);
    }
}

/// Clears non-initialized data
pub unsafe fn zero_bss(mut bss: *mut u32, bss_end: *mut u32) {
    while bss < bss_end {
        // `volatile` to make sure it doesn't get `optimized out`
        bss.write_volatile(0);
        bss = bss.offset(1);
    }
}
