//! Support for statically initializing objects in memory.

/// Allocates a statically-sized global array of memory and initializes the
/// memory for a particular data structure.
///
/// This macro creates the static buffer, ensures it is initialized to the
/// proper type, and then returns a `&'static mut` reference to it.
///
/// Note: Because this instantiates a static object, you generally cannot pass
/// a type with generic paramters. github.com/tock/tock/issues/2995 for detail.
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
    ($T:ty, $e:expr $(,)?) => {{
        let mut buf = $crate::static_buf!($T);
        buf.write($e)
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
    ($T:ty $(,)?) => {{
        // Statically allocate a read-write buffer for the value without
        // actually writing anything.
        static mut BUF: core::mem::MaybeUninit<$T> = core::mem::MaybeUninit::uninit();
        &mut BUF
    }};
}

/// This macro is deprecated. You should migrate to using `static_buf!`
/// followed by a call to `StaticUninitializedBuffer::initialize()`.
///
/// Same as `static_init!()` but without actually creating the static buffer.
/// The static buffer must be passed in.
#[macro_export]
macro_rules! static_init_half {
    ($B:expr, $T:ty, $e:expr $(,)?) => {
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
