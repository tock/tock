//! Utility macros including `static_init!`.

/// Allocates a global array of static size to initialize data structures.
///
/// The global array is initially set to zero. When this macro is hit, it will
/// initialize the array to the value given and return a `&'static mut`
/// reference to it.
///
/// If `std::mem::size_of<T>` ever becomes a `const` function then `static_init`
/// will be optimized to save up to a word of memory for every use.
///
/// # Safety
///
/// As this macro will write directly to a global area without acquiring a lock
/// or similar, calling this macro is inherently unsafe. The caller should take
/// care to never call the code that initializes this buffer twice, as doing so
/// will overwrite the value from first allocation without running its
/// destructor.
#[macro_export]
macro_rules! static_init {
    ($T:ty, $e:expr) => {
        // Ideally we could use mem::size_of<$T>, uninitialized or zerod here
        // instead of having an `Option`, however that is not currently possible
        // in Rust, so in some cases we're wasting up to a word.
        {
            use core::{mem, ptr};
            // Statically allocate a read-write buffer for the value, write our
            // initial value into it (without dropping the initial zeros) and
            // return a reference to it.
            static mut BUF: Option<$T> = None;
            let tmp : &'static mut $T = mem::transmute(&mut BUF);
            ptr::write(tmp as *mut $T, $e);
            tmp
        };
    }
}

/// Allocates space in the kernel image for on-chip non-volatile storage.
/// Storage volumes are placed after the kernel code and before relocated
/// variables (those copied into RAM on boot). They are placed in
/// a section called ".storage".
///
/// Non-volatile storage abstractions can then refer to the block of
/// allocate flash in terms of the name of the volume. For example,
///
/// `storage_volume(LOG, 32);`
///
/// will allocate 32kB of space in the flash and define a symbol LOG
/// at the start address of that flash region. The intention is that
/// storage abstractions can then be passed this address and size to
/// initialize their state. The linker script kernel_layout.ld makes
/// sure that the .storage section is aligned on a 512-byte boundary
/// and the next section is aligned as well.
#[macro_export]
macro_rules! storage_volume {
    ($N:ident, $kB:expr) => {
        #[link_section = ".storage"]
        #[used]
        pub static $N: [u8; $kB * 1024] = [0x00; $kB * 1024];
    };
}
