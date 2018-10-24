use error;

/// A trait that represents a value that can only withing a certain range.
pub trait Range: Sized + Copy {
    /// Primitive type used to store the value.
    type Prim;
    /// Tag type which is the minimum sized
    type Tag;
    /// Lower bound, or the minimum value the range can represent.
    const OFFSET: Self::Prim;
    /// OFFSET + LENGTH is the upper bound, or the maximum value the range can represent.
    const LENGTH: Self::Prim;

    /// Create a new range object from it's primitive type.
    fn new(val: Self::Prim) -> Result<Self, error::Error>;
    /// Change the value of a range object.
    fn set(&mut self, val: Self::Prim) -> Option<Self::Prim>;
    /// Get the value of a range object.
    fn get(&self) -> Self::Prim;
}

#[macro_export]
macro_rules! impl_range {
    ($name:ident, $prim_type:ty, $tag_type:ty, $offset:expr, $length:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name($prim_type);

        impl Range for $name {
            type Prim = $prim_type;
            type Tag = $tag_type;
            const OFFSET: $prim_type = $offset;
            const LENGTH: $prim_type = $length;

            fn new(val: Self::Prim) -> Result<Self, Error> {
                if (Self::OFFSET <= val) && (val <= Self::OFFSET + Self::LENGTH) {
                    return Ok($name(val));
                }
                Err(Error::OutOfRange)
            }

            fn set(&mut self, val: Self::Prim) -> Option<Self::Prim> {
                if (Self::OFFSET < val) && (val < Self::OFFSET + Self::LENGTH) {
                    self.0 = val;
                    return None;
                }
                Some(val)
            }

            fn get(&self) -> Self::Prim {
                self.0
            }
        }
    };
}

#[cfg(test)]
mod test {
    use error::Error;
    use range::Range;
    #[test]
    fn test_range() {
        impl_range!(Rangeu8, u16, u8, 100, 155);
        let mut test_range = Rangeu8::new(100).unwrap();
        assert_eq!(test_range.set(99), Some(99));
        assert_eq!(test_range.set(254), None);
        assert_eq!(test_range.set(256), Some(256));
        assert_eq!(Rangeu8::new(100), Ok(Rangeu8(100)));
        #[cfg(feature = "std")]
        println!("{:?}", test_range);
    }
}
