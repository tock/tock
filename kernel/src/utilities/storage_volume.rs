//! Utility for creating non-volatile storage regions.

/// Allocates space in the kernel image for on-chip non-volatile storage.
///
/// Storage volumes are placed after the kernel code and before relocated
/// variables (those copied into RAM on boot). They are placed in
/// a section called `.storage`.
///
/// Non-volatile storage abstractions can then refer to the block of
/// allocate flash in terms of the name of the volume. For example,
///
/// `storage_volume!(LOG, 32);`
///
/// will allocate 32kB of space in the flash and define a symbol LOG
/// at the start address of that flash region. The intention is that
/// storage abstractions can then be passed this address and size to
/// initialize their state. The linker script kernel_layout.ld makes
/// sure that the .storage section is aligned on a 512-byte boundary
/// and the next section is aligned as well.
#[macro_export]
macro_rules! storage_volume {
    ($N:ident, $kB:expr $(,)?) => {
        #[link_section = ".storage"]
        #[used]
        #[no_mangle]
        pub static $N: [u8; $kB * 1024] = [0x00; $kB * 1024];
    };
}
