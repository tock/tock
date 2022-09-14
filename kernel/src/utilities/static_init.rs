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

/// An `#[inline(never)]` function that panics internally for use within
/// the `static_buf!()` macro, which removes the size bloat of track_caller
/// saving the location of every single call to `static_init!()`.
/// If you hit this panic, you are either calling `static_buf!()` in
/// a loop or calling a function multiple times which internally
/// contains a call to `static_buf!()`. Typically, calls to
/// `static_buf!()` are hidden within calls to `static_init!()` or
/// component helper macros, so start your search there.
#[inline(never)]
pub fn static_buf_panic() -> ! {
    panic!("Error! Single static_buf!() called twice.");
}

/// Allocates a statically-sized global array of memory for data structures but
/// does not initialize the memory. Checks that the buffer is not aliased and is
/// only used once.
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
        static mut BUF: (core::mem::MaybeUninit<$T>, bool) =
            (core::mem::MaybeUninit::uninit(), false);

        // Check if this `BUF` has already been declared and initialized. If it
        // has, then this is a repeated `static_buf!()` call which is an error
        // as it will alias the same `BUF`.
        let used = BUF.1;
        if used {
            // panic, this buf has already been declared and initialized.
            $crate::utilities::static_init::static_buf_panic();
        } else {
            // Otherwise, mark our uninitialized buffer as used.
            BUF.1 = true;
        }

        // If we get to this point we can wrap our buffer to be eventually
        // initialized.
        &mut BUF.0
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
