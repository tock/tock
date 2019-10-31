use core::mem::size_of;
use core::num::Wrapping;

use core::{i16, i32, i64, i8, isize};
use core::{u16, u32, u64, u8, usize};

/// A generic trait for converting a value to a number.
pub trait ToPrimitive {
    /// Converts the value of `self` to an `isize`.
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        self.to_i64().as_ref().and_then(ToPrimitive::to_isize)
    }

    /// Converts the value of `self` to an `i8`.
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        self.to_i64().as_ref().and_then(ToPrimitive::to_i8)
    }

    /// Converts the value of `self` to an `i16`.
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        self.to_i64().as_ref().and_then(ToPrimitive::to_i16)
    }

    /// Converts the value of `self` to an `i32`.
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        self.to_i64().as_ref().and_then(ToPrimitive::to_i32)
    }

    /// Converts the value of `self` to an `i64`.
    fn to_i64(&self) -> Option<i64>;

    /// Converts the value of `self` to a `usize`.
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        self.to_u64().as_ref().and_then(ToPrimitive::to_usize)
    }

    /// Converts the value of `self` to an `u8`.
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        self.to_u64().as_ref().and_then(ToPrimitive::to_u8)
    }

    /// Converts the value of `self` to an `u16`.
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        self.to_u64().as_ref().and_then(ToPrimitive::to_u16)
    }

    /// Converts the value of `self` to an `u32`.
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        self.to_u64().as_ref().and_then(ToPrimitive::to_u32)
    }

    /// Converts the value of `self` to an `u64`.
    fn to_u64(&self) -> Option<u64>;
}

macro_rules! impl_to_primitive_int_to_int {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let min = $DstT::MIN as $SrcT;
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() <= size_of::<$DstT>() || (min <= *self && *self <= max) {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_int_to_uint {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if 0 <= *self && (size_of::<$SrcT>() <= size_of::<$DstT>() || *self <= max) {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_int {
    ($T:ident) => {
        impl ToPrimitive for $T {
            impl_to_primitive_int_to_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
                #[cfg(has_i128)]
                fn to_i128 -> i128;
            }

            impl_to_primitive_int_to_uint! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
                #[cfg(has_i128)]
                fn to_u128 -> u128;
            }
        }
    };
}

impl_to_primitive_int!(isize);
impl_to_primitive_int!(i8);
impl_to_primitive_int!(i16);
impl_to_primitive_int!(i32);
impl_to_primitive_int!(i64);

macro_rules! impl_to_primitive_uint_to_int {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() < size_of::<$DstT>() || *self <= max {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_uint_to_uint {
    ($SrcT:ident : $( $(#[$cfg:meta])* fn $method:ident -> $DstT:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$DstT> {
            let max = $DstT::MAX as $SrcT;
            if size_of::<$SrcT>() <= size_of::<$DstT>() || *self <= max {
                Some(*self as $DstT)
            } else {
                None
            }
        }
    )*}
}

macro_rules! impl_to_primitive_uint {
    ($T:ident) => {
        impl ToPrimitive for $T {
            impl_to_primitive_uint_to_int! { $T:
                fn to_isize -> isize;
                fn to_i8 -> i8;
                fn to_i16 -> i16;
                fn to_i32 -> i32;
                fn to_i64 -> i64;
            }

            impl_to_primitive_uint_to_uint! { $T:
                fn to_usize -> usize;
                fn to_u8 -> u8;
                fn to_u16 -> u16;
                fn to_u32 -> u32;
                fn to_u64 -> u64;
            }
        }
    };
}

impl_to_primitive_uint!(usize);
impl_to_primitive_uint!(u8);
impl_to_primitive_uint!(u16);
impl_to_primitive_uint!(u32);
impl_to_primitive_uint!(u64);

/// A generic trait for converting a number to a value.
pub trait FromPrimitive: Sized {
    /// Convert an `isize` to return an optional value of this type. If the
    /// value cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        n.to_i64().and_then(FromPrimitive::from_i64)
    }

    /// Convert an `i8` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        FromPrimitive::from_i64(From::from(n))
    }

    /// Convert an `i16` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        FromPrimitive::from_i64(From::from(n))
    }

    /// Convert an `i32` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        FromPrimitive::from_i64(From::from(n))
    }

    /// Convert an `i64` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    fn from_i64(n: i64) -> Option<Self>;

    /// Convert a `usize` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        n.to_u64().and_then(FromPrimitive::from_u64)
    }

    /// Convert an `u8` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        FromPrimitive::from_u64(From::from(n))
    }

    /// Convert an `u16` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        FromPrimitive::from_u64(From::from(n))
    }

    /// Convert an `u32` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        FromPrimitive::from_u64(From::from(n))
    }

    /// Convert an `u64` to return an optional value of this type. If the
    /// type cannot be represented by this value, then `None` is returned.
    fn from_u64(n: u64) -> Option<Self>;
}

macro_rules! impl_from_primitive {
    ($T:ty, $to_ty:ident) => {
        #[allow(deprecated)]
        impl FromPrimitive for $T {
            #[inline]
            fn from_isize(n: isize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i8(n: i8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i16(n: i16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i32(n: i32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i64(n: i64) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_usize(n: usize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u8(n: u8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u16(n: u16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u32(n: u32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u64(n: u64) -> Option<$T> {
                n.$to_ty()
            }
        }
    };
}

impl_from_primitive!(isize, to_isize);
impl_from_primitive!(i8, to_i8);
impl_from_primitive!(i16, to_i16);
impl_from_primitive!(i32, to_i32);
impl_from_primitive!(i64, to_i64);
impl_from_primitive!(usize, to_usize);
impl_from_primitive!(u8, to_u8);
impl_from_primitive!(u16, to_u16);
impl_from_primitive!(u32, to_u32);
impl_from_primitive!(u64, to_u64);

macro_rules! impl_to_primitive_wrapping {
    ($( $(#[$cfg:meta])* fn $method:ident -> $i:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$i> {
            (self.0).$method()
        }
    )*}
}

impl<T: ToPrimitive> ToPrimitive for Wrapping<T> {
    impl_to_primitive_wrapping! {
        fn to_isize -> isize;
        fn to_i8 -> i8;
        fn to_i16 -> i16;
        fn to_i32 -> i32;
        fn to_i64 -> i64;

        fn to_usize -> usize;
        fn to_u8 -> u8;
        fn to_u16 -> u16;
        fn to_u32 -> u32;
        fn to_u64 -> u64;
    }
}

macro_rules! impl_from_primitive_wrapping {
    ($( $(#[$cfg:meta])* fn $method:ident ( $i:ident ); )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(n: $i) -> Option<Self> {
            T::$method(n).map(Wrapping)
        }
    )*}
}

impl<T: FromPrimitive> FromPrimitive for Wrapping<T> {
    impl_from_primitive_wrapping! {
        fn from_isize(isize);
        fn from_i8(i8);
        fn from_i16(i16);
        fn from_i32(i32);
        fn from_i64(i64);
        fn from_usize(usize);
        fn from_u8(u8);
        fn from_u16(u16);
        fn from_u32(u32);
        fn from_u64(u64);
    }
}

/// Cast from one machine scalar to another.
///
/// # Examples
///
/// ```
/// # use num_traits as num;
/// let twenty: f32 = num::cast(0x14).unwrap();
/// assert_eq!(twenty, 20f32);
/// ```
///
#[inline]
pub fn cast<T: NumCast, U: NumCast>(n: T) -> Option<U> {
    NumCast::from(n)
}

/// An interface for casting between machine scalars.
pub trait NumCast: Sized + ToPrimitive {
    /// Creates a number from another value that can be converted into
    /// a primitive via the `ToPrimitive` trait.
    fn from<T: ToPrimitive>(n: T) -> Option<Self>;
}

macro_rules! impl_num_cast {
    ($T:ty, $conv:ident) => {
        impl NumCast for $T {
            #[inline]
            #[allow(deprecated)]
            fn from<N: ToPrimitive>(n: N) -> Option<$T> {
                // `$conv` could be generated using `concat_idents!`, but that
                // macro seems to be broken at the moment
                n.$conv()
            }
        }
    };
}

impl_num_cast!(u8, to_u8);
impl_num_cast!(u16, to_u16);
impl_num_cast!(u32, to_u32);
impl_num_cast!(u64, to_u64);
impl_num_cast!(usize, to_usize);
impl_num_cast!(i8, to_i8);
impl_num_cast!(i16, to_i16);
impl_num_cast!(i32, to_i32);
impl_num_cast!(i64, to_i64);
impl_num_cast!(isize, to_isize);

impl<T: NumCast> NumCast for Wrapping<T> {
    fn from<U: ToPrimitive>(n: U) -> Option<Self> {
        T::from(n).map(Wrapping)
    }
}

/// A generic interface for casting between machine scalars with the
/// `as` operator, which admits narrowing and precision loss.
/// Implementers of this trait AsPrimitive should behave like a primitive
/// numeric type (e.g. a newtype around another primitive), and the
/// intended conversion must never fail.
///
/// # Examples
///
/// ```
/// # use num_traits::AsPrimitive;
/// let three: i32 = (3.14159265f32).as_();
/// assert_eq!(three, 3);
/// ```
///
/// # Safety
///
/// Currently, some uses of the `as` operator are not entirely safe.
/// In particular, it is undefined behavior if:
///
/// - A truncated floating point value cannot fit in the target integer
///   type ([#10184](https://github.com/rust-lang/rust/issues/10184));
///
/// ```ignore
/// # use num_traits::AsPrimitive;
/// let x: u8 = (1.04E+17).as_(); // UB
/// ```
///
/// - Or a floating point value does not fit in another floating
///   point type ([#15536](https://github.com/rust-lang/rust/issues/15536)).
///
/// ```ignore
/// # use num_traits::AsPrimitive;
/// let x: f32 = (1e300f64).as_(); // UB
/// ```
///
pub trait AsPrimitive<T>: 'static + Copy
where
    T: 'static + Copy,
{
    /// Convert a value to another, using the `as` operator.
    fn as_(self) -> T;
}

macro_rules! impl_as_primitive {
    (@ $T: ty => $(#[$cfg:meta])* impl $U: ty ) => {
        $(#[$cfg])*
        impl AsPrimitive<$U> for $T {
            #[inline] fn as_(self) -> $U { self as $U }
        }
    };
    (@ $T: ty => { $( $U: ty ),* } ) => {$(
        impl_as_primitive!(@ $T => impl $U);
    )*};
    ($T: ty => { $( $U: ty ),* } ) => {
        impl_as_primitive!(@ $T => { $( $U ),* });
        impl_as_primitive!(@ $T => { u8, u16, u32, u64, usize });
        impl_as_primitive!(@ $T => { i8, i16, i32, i64, isize });
    };
}

impl_as_primitive!(char => { char });
impl_as_primitive!(bool => {});
