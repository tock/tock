//! Helper macros.

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
    ($head:expr $(,)?) => (1usize);
    ($head:expr, $($tail:expr),* $(,)?) => (1usize + count_expressions!($($tail),*));
}
