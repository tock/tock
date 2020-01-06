//! Utility macros including `static_init!`.

/// Allocates a global array of static size to initialize data structures.
///
/// The global array is initially set to zero. When this macro is hit, it will
/// initialize the array to the value given and return a `&'static mut`
/// reference to it.
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
        {
            use core::mem::MaybeUninit;
            // Statically allocate a read-write buffer for the value, write our
            // initial value into it (without dropping the initial zeros) and
            // return a reference to it.
            static mut BUF: MaybeUninit<$T> = MaybeUninit::uninit();
            $crate::static_init_half!(&mut BUF, $T, $e)
        };
    }
}

/// Same as `static_init!()` but without actually creating the static buffer.
/// The static buffer must be passed in.
#[macro_export]
macro_rules! static_init_half {
    ($B:expr, $T:ty, $e:expr) => {
        {
            use core::mem::MaybeUninit;
            let buf: &'static mut MaybeUninit<$T> = $B;
            buf.as_mut_ptr().write($e);
            // TODO: use MaybeUninit::get_mut() once that is stabilized (see
            // https://github.com/rust-lang/rust/issues/63568).
            &mut *buf.as_mut_ptr() as &'static mut $T
        }
    };
}

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
    ($N:ident, $kB:expr) => {
        #[link_section = ".storage"]
        #[used]
        #[no_mangle]
        pub static $N: [u8; $kB * 1024] = [0x00; $kB * 1024];
    };
}

/// Create an object with the given capability.
///
/// ```ignore
/// use kernel::capabilities::ProcessManagementCapability;
/// use kernel;
///
/// let process_mgmt_cap = create_capability!(ProcessManagementCapability);
/// ```
///
/// This helper macro cannot be called from `#![forbid(unsafe_code)]` crates,
/// and is used by trusted code to generate a capability that it can either use
/// or pass to another module.
#[macro_export]
macro_rules! create_capability {
    ($T:ty) => {{
        struct Cap;
        #[allow(unsafe_code)]
        unsafe impl $T for Cap {}
        Cap
    };};
}
