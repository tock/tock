//! Utility macros including `static_init!`.

/// Allocates a statically-sized global array of memory and initializes the
/// memory for a particular data structure.
///
/// This macro creates the static buffer, ensures it is initialized to the
/// proper type, and then returns a `&'static mut` reference to it.
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
    ($T:ty, $e:expr) => {{
        let mut buf = $crate::static_buf!($T);
        buf.initialize($e)
    }};
}

/// Allocates a statically-sized global array of memory for data structures but
/// does not initialize the memory.
///
/// This macro creates the static buffer, and returns a
/// `StaticUninitializedBuffer` wrapper containing the buffer. The memory is
/// allocated, but it is guaranteed to be uninitialized inside of the wrapper.
///
/// Before the static buffer can be used it must be initialized. For example:
///
/// ```ignore
/// let mut static_buffer = static_buf!(T);
/// let static_reference: &'static mut T = static_buffer.initialize(T::new());
/// ```
///
/// Separating the creation of the static buffer into its own macro is not
/// strictly necessary, but it allows for more flexibility in Rust when boards
/// are initialized and the static structures are being created. Since creating
/// and initializing static buffers requires knowing the particular types (and
/// their sizes), writing shared initialization code (in components for example)
/// where the types are unknown since they vary across boards is difficult. By
/// splitting buffer creating from initialization, creating shared components is
/// possible.
#[macro_export]
macro_rules! static_buf {
    ($T:ty) => {{
        // Statically allocate a read-write buffer for the value, write our
        // initial value into it (without dropping the initial zeros) and
        // return a reference to it.
        static mut BUF: $crate::common::utils::UninitializedBuffer<$T> =
            $crate::common::utils::UninitializedBuffer::new();
        $crate::StaticUninitializedBuffer::new(&mut BUF)
    }};
}

use core::mem::MaybeUninit;

/// The `UninitializedBuffer` type is designed to be statically allocated
/// as a global buffer to hold data structures in Tock. As a static, global
/// buffer the data structure can then be shared in the Tock kernel.
///
/// This type is implemented as a wrapper around a `MaybeUninit<T>` buffer.
/// To enforce that the global static buffer is initialized exactly once,
/// this wrapper type ensures that the underlying memory is uninitialized
/// so that an `UninitializedBuffer` does not contain an initialized value.
///
/// The only way to initialize this buffer is to create a
/// `StaticUninitializedBuffer`, pass the `UninitializedBuffer` to it, and call
/// `initialize()`. This structure ensures that:
///
/// 1. The static buffer is not used while uninitialized. Since the only way to
///    get the necessary `&'static mut T` is to call `initialize()`, the memory
///    is guaranteed to be initialized.
///
/// 2. A static buffer is not initialized twice. Since the underlying memory is
///    owned by `UninitializedBuffer` nothing else can initialize it. Also, once
///    the memory is initialized via `StaticUninitializedBuffer.initialize()`,
///    the internal buffer is consumed and `initialize()` cannot be called
///    again.
#[repr(transparent)]
pub struct UninitializedBuffer<T>(MaybeUninit<T>);

impl<T> UninitializedBuffer<T> {
    /// The only way to construct an `UninitializedBuffer` is via this function,
    /// which initializes it to `MaybeUninit::uninit()`. This guarantees the
    /// invariant that `UninitializedBuffer` does not contain an initialized
    /// value.
    pub const fn new() -> Self {
        UninitializedBuffer(MaybeUninit::uninit())
    }
}

/// The `StaticUninitializedBuffer` type represents a statically allocated
/// buffer that can be converted to another type once it has been initialized.
/// Upon initialization, a static mutable reference is returned and the
/// `StaticUninitializedBuffer` is consumed.
///
/// This type is implemented as a wrapper containing a static mutable reference to
/// an `UninitializedBuffer`. This guarantees that the memory pointed to by the
/// reference has not already been initialized.
///
/// `StaticUninitializedBuffer` provides one operation: `initialize()` that returns a
/// `&'static mut T` reference. This is the only way to get the reference, and
/// ensures that the underlying uninitialized buffer is properly initialized.
/// The wrapper is also consumed when `initialize()` is called, ensuring that
/// the underlying memory cannot be subsequently re-initialized.
pub struct StaticUninitializedBuffer<T: 'static> {
    buf: &'static mut UninitializedBuffer<T>,
}

impl<T> StaticUninitializedBuffer<T> {
    /// This function is not intended to be called publicly. It's only meant to
    /// be called within `static_buf!` macro, but Rust's visibility rules
    /// require it to be public, so that the macro's body can be instantiated.
    pub fn new(buf: &'static mut UninitializedBuffer<T>) -> Self {
        Self { buf }
    }

    /// This function consumes an uninitialized static buffer, initializes it
    /// to some value, and returns a static mutable reference to it. This
    /// allows for runtime initialization of `static` values that do not have a
    /// `const` constructor.
    pub unsafe fn initialize(self, value: T) -> &'static mut T {
        self.buf.0.as_mut_ptr().write(value);
        // TODO: use MaybeUninit::get_mut() once that is stabilized (see
        // https://github.com/rust-lang/rust/issues/63568).
        &mut *self.buf.0.as_mut_ptr() as &'static mut T
    }
}

/// This macro is deprecated. You should migrate to using `static_buf!`
/// followed by a call to `StaticUninitializedBuffer::initialize()`.
///
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

/// Count the number of passed expressions.
/// Useful for constructing variable sized arrays in other macros.
/// Taken from the Little Book of Rust Macros
///
/// ```ignore
/// use kernel:count_expressions;
///
/// let count: usize = count_expressions!(1+2, 3+4);
/// ```
#[macro_export]
macro_rules! count_expressions {
    () => (0usize);
    ($head:expr) => (1usize);
    ($head:expr, $($tail:expr),*) => (1usize + count_expressions!($($tail),*));
}
