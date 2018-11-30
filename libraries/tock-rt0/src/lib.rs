//! Tock generic initial runtime (`rt0`) helper functions

#![no_std]

/// Initializes the static data, by copying it into memory (RAM) from
/// non-volatile memory (Flash).
///
/// - `start_data_flash`: The address of the start of the data section stored
///                       in non-volatile flash memory.
/// - `start_data_ram`:   The address in RAM where the data section starts and
///                       should be copied to.
/// - `end_data_ram`:     The first address after the end of the data section
///                       in RAM.
pub unsafe fn init_data(
    mut start_data_flash: *mut u32,
    mut start_data_ram: *mut u32,
    end_data_ram: *mut u32,
) {
    while start_data_ram < end_data_ram {
        start_data_ram.write(start_data_flash.read());
        start_data_ram = start_data_ram.offset(1);
        start_data_flash = start_data_flash.offset(1);
    }
}

/// Sets non-initialized data in RAM to 0.
/// This is used to clear the BSS section on initial bootup.
pub unsafe fn zero_bss(mut bss: *mut u32, bss_end: *mut u32) {
    while bss < bss_end {
        // `volatile` to make sure it doesn't get optimized out
        bss.write_volatile(0);
        bss = bss.offset(1);
    }
}
