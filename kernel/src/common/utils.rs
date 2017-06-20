//! Utility macros including `static_init!`.

/// Allocates a global array of static size to initialize data structures.
///
/// The global array is initially set to zero. When this macro is hit, it will
/// initialize the array to the value given and return a `&'static mut`
/// reference to it.
///
/// Note that you will have to specify the array-size as an argument, but a
/// wrong size will result in a compile-time error. This argument will be
/// removed if `std::mem::size_of<T>` ever becomes a `const` function.
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
    ($T:ty, $e:expr, $size:expr) => (static_init!($T, $e));
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
            let mut tmp : &'static mut $T = mem::transmute(&mut BUF);
            ptr::write(tmp as *mut $T, $e);
            tmp
        };
    }
}
