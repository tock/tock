extern crate byteorder;
use self::byteorder::{ByteOrder, LittleEndian};
// use std::io::{Write, Read};
use stream::{SResult, encode_u8, decode_u8};

use error::Error;

// type CautEndian = LittleEndian;
pub type Encoder = [u8];
pub type Decoder = [u8];

pub trait Cauterize: 'static + Sized {
    const FINGERPRINT: [u8; 20];
    const SIZE_MIN: usize;
    const SIZE_MAX: usize;
    fn encode(&self, &mut [u8]) -> Result<(), Error>;
    fn decode(&mut [u8]) -> Result<Self, Error>;
}

pub trait Primitive: 'static + Sized {
    fn encode(&self, &mut [u8]) -> Result<(), Error>;
    fn decode(&mut [u8]) -> Result<Self, Error>;
}


// ****************
// Primitive impls
// ****************

macro_rules! impl_primitive {
    ($T:ty, $FR:ident, $FW:ident) => (
        impl Primitive for $T {
            fn encode(&self, enc: &mut [u8]) -> Result<(), Error> {
                let _res = LittleEndian::$FW(enc, *self);
                Result::Ok(())
            }
            fn decode(ctx: &mut [u8]) -> Result<Self, Error> {
                let res = LittleEndian::$FR(ctx);
                Result::Ok(res)
            }
        }
    );
}

impl_primitive!(u16, read_u16, write_u16);
impl_primitive!(i16, read_i16, write_i16);
impl_primitive!(u32, read_u32, write_u32);
impl_primitive!(i32, read_i32, write_i32);
impl_primitive!(u64, read_u64, write_u64);
impl_primitive!(i64, read_i64, write_i64);
impl_primitive!(f32, read_f32, write_f32);
impl_primitive!(f64, read_f64, write_f64);


// We can't use `impl_primitive!` for u8/i8 since it read/write for u8/i8 does not take paramters
impl Primitive for u8 {
    fn encode(&self, buf: &mut [u8]) -> Result<(), Error> {
        let result = encode_u8(buf, *self);
        match result {
            SResult::Done(_off, _out) => Ok(()),
            SResult::Needed(_) => Result::Err(Error::Encode),
            SResult::Error(_) => Result::Err(Error::Encode),
        }
    }
    fn decode(ctx: &mut [u8]) -> Result<Self, Error> {
        let result = decode_u8(ctx);
        match result {
            SResult::Done(_off, out) => Ok(out),
            SResult::Needed(_e) => Result::Err(Error::Decode),
            SResult::Error(_) => Result::Err(Error::Decode),
        }
    }
}
/*
impl Primitive for i8 {
    fn encode(&self, buf: &mut [u8]) -> Result<(), Error> {
        let result = encode_u8(buf, *self as u8);
        match result {
            SResult::Done(off, out) => Ok(()),
            SResult::Needed(e) => Result::Err(e),
            SResult::Error(e) => Result::Err(e),
        }
    }
    fn decode(ctx: &mut [u8]) -> Result<Self, Error> {
        let result = decode_u8(ctx);
        match result {
            SResult::Done(off, out) => Ok(()),
            SResult::Needed(e) => Result::Err(e),
            SResult::Error(e) => Result::Err(e),
        }
    }
}
*/

// We can't use `impl_primitive!` for bool since it
impl Primitive for bool {
    fn encode(&self, ctx: &mut [u8]) -> Result<(), Error> {
        let val = *self as u8;
        val.encode(ctx)
    }

    fn decode(ctx: &mut [u8]) -> Result<Self, Error> {
        match u8::decode(ctx) {
            Ok(0) => Ok(false),
            Ok(1) => Ok(true),
            Err(e) => Err(e),
            _ => Err(Error::InvalidValue),
        }

    }
}
