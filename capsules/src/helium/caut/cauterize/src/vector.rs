/// A Cauterize `Vector` is a variable length array with a maximum length.
///
/// `Vector`s do not require any heap allocations since they are backed by a stack-allocated array.
///
/// # Examples
///
/// ```
/// # #![feature(associated_consts)]
/// # #[macro_use] extern crate cauterize;
/// # use cauterize::Vector;
/// # fn main() {
/// impl_vector!(AVec, u64, 20);
/// let mut v = AVec::new();
///
/// for i in 0..v.capacity() {
///     // Push returns `None` until the vector has reached capacitly
///     assert_eq!(None, v.push(i as u64));
/// }
/// // Since v is at capacity, push will now return `Some`
/// assert_eq!(Some(20u64), v.push(20u64));
/// for i in v.iter() {
///     println!("{}", i);
/// }
///
/// for i in v.iter_mut() {
///     println!("adding 1 to {}", i);
///     *i = 1 + *i;
/// }
///
/// for i in &mut v {
///     println!("adding 2 to {}", i);
///     *i = 2 + *i;
/// }
///
/// for i in &v {
///     println!("{}", i);
/// }
/// # }
/// ```
pub trait Vector: Sized {
    /// Element type.
    type T: Sized + ::core::fmt::Debug + Clone + PartialEq;
    /// Maximum length.
    const CAPACITY: usize;

    /// Creates a new empty vector.
    fn new() -> Self;

    /// Appends an element to end of the vector.
    fn push(&mut self, elem: Self::T) -> Option<Self::T>;

    /// Returns the number of elements in the vector.
    fn len(&self) -> usize;

    /// Returns the capacity, or maximum length, of the vector.
    fn capacity(&self) -> usize {
        Self::CAPACITY
    }

    /// Returns an immutable iterator over elements in the vector.
    fn iter<'a>(&'a self) -> ::core::slice::Iter<'a, Self::T>;

    /// Returns a mutable iterator over elements in the vector.
    fn iter_mut<'a>(&'a mut self) -> ::core::slice::IterMut<'a, Self::T>;
}

#[macro_export]
macro_rules! impl_vector {
    ($name:ident, $eltype:ident, $capacity:expr) => (
        pub struct $name {
            len: usize,
            elems: [$eltype;$capacity],
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = &'a $eltype;
            type IntoIter = ::std::slice::Iter<'a, $eltype>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }

        impl<'a> IntoIterator for &'a mut $name {
            type Item = &'a mut $eltype;
            type IntoIter = ::std::slice::IterMut<'a, $eltype>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter_mut()
            }
        }

        impl Vector for $name {
            type T = $eltype;
            const CAPACITY: usize = $capacity;
            fn new() -> $name {
                use std::mem;
                $name {
                    len: 0,
                    elems: unsafe { mem::uninitialized() },
                }
            }

            fn push(&mut self, elem: $eltype) -> Option<$eltype> {
                assert!(self.len <= $name::CAPACITY);
                if self.len == $name::CAPACITY {
                    return Some(elem);
                }
                unsafe {
                    let end = self.elems.as_mut_ptr().offset(self.len as isize);
                    ::std::ptr::write(end, elem);
                    self.len += 1;
                }
                None
            }

            fn len(&self) -> usize {
                self.len
            }

            fn iter<'a>(&'a self) -> ::std::slice::Iter<'a, $eltype> {
                self.elems[..self.len].iter()
            }

            fn iter_mut<'a>(&'a mut self) -> ::std::slice::IterMut<'a, $eltype> {
                self.elems[..self.len].iter_mut()
            }
        }

        impl AsRef<[$eltype]> for $name {
            fn as_ref(&self) -> &[$eltype] {
                use std;
                unsafe {
                    std::slice::from_raw_parts(&self.elems as *const $eltype, self.len)
                }
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Debug::fmt(self.as_ref(), f)
            }
        }

        impl ::std::cmp::PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                self.as_ref() == other.as_ref()
            }
        }

        impl Clone for $name {
            fn clone(&self) -> $name {
                let mut cloned = $name::new();
                for elem in self {
                    cloned.push(elem.clone());
                }
                cloned
            }
        }
    )
}


#[cfg(test)]
mod tests {
    use super::Vector;

    #[test]
    fn test_vector() {
        impl_vector!(Vector32u8, u8, 32);

        let mut test_vector = Vector32u8::new();
        for i in 0..test_vector.capacity() {
            test_vector.push(i as u8);
        }
        println!("{:?}", test_vector);

        assert_eq!(test_vector, test_vector.clone());
    }
}
