#![allow(dead_code, unused_variables, unused_imports)]
#![no_std]

extern crate core as std;
#[macro_use]
pub extern crate cauterize;
use self::cauterize::{Primitive, Error, Encoder, Decoder, Cauterize, Range, Vector};
use std::mem;

pub static SPEC_NAME: &'static str = "msg";
pub const SPEC_FINGERPRINT: [u8;20] = [0x21,0xd1,0xf3,0xb4,0x9c,0xdb,0x0c,0xc4,0x1d,0x8c,0xf0,0xb5,0x87,0x06,0xbf,0x3b,0x5c,0x5c,0xbe,0x96];
pub const SPEC_MIN_SIZE: usize = 1;
pub const SPEC_MAX_SIZE: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub struct Pong {
    pub id: u32, // 1
    pub seq: u32, // 2
}

impl Cauterize for Pong {
    const FINGERPRINT: [u8;20] = [0xa2,0x9e,0x38,0xb7,0x95,0x10,0x79,0x2a,0xbd,0x27,0xa4,0x8f,0x28,0x45,0x13,0xcd,0x64,0x4f,0xea,0xf9];
    const SIZE_MIN: usize = 8;
    const SIZE_MAX: usize = 8;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        self.id.encode(ctx)?;
        self.seq.encode(ctx)?;
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let rec = Pong {
            id: u32::decode(ctx)?,
            seq: u32::decode(ctx)?,
        };
        Ok(rec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pingpong {
    Ping(u32), // 1
    Pong(Pong), // 2
}

impl Cauterize for Pingpong {
    const FINGERPRINT: [u8;20] = [0x77,0x96,0x6d,0x13,0x90,0x06,0x0d,0x7f,0xb5,0xba,0xc8,0x50,0xa9,0x4d,0xba,0x32,0x29,0xc4,0x4f,0xc1];
    const SIZE_MIN: usize = 5;
    const SIZE_MAX: usize = 9;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        match self {
            &Pingpong::Ping(ref val) => {
                let tag: u8 = 1;
                tag.encode(ctx)?;
                val.encode(ctx)?;
            }
            &Pingpong::Pong(ref val) => {
                let tag: u8 = 2;
                tag.encode(ctx)?;
                val.encode(ctx)?;
            }
        };
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let tag = u8::decode(ctx)?;
        match tag {
            1  => Ok(Pingpong::Ping(u32::decode(ctx)?)),
            2  => Ok(Pingpong::Pong(Pong::decode(ctx)?)),
            _  => Err(Error::InvalidTag),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Pingpong(Pingpong), // 1
}

impl Cauterize for Frame {
    const FINGERPRINT: [u8;20] = [0xdb,0x89,0xe6,0x52,0xaa,0x95,0x65,0x6d,0x4a,0x79,0x52,0xa5,0xe1,0x17,0xed,0x40,0xe2,0x59,0x15,0x55];
    const SIZE_MIN: usize = 6;
    const SIZE_MAX: usize = 10;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        match self {
            &Frame::Pingpong(ref val) => {
                let tag: u8 = 1;
                tag.encode(ctx)?;
                val.encode(ctx)?;
            }
        };
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let tag = u8::decode(ctx)?;
        match tag {
            1  => Ok(Frame::Pingpong(Pingpong::decode(ctx)?)),
            _  => Err(Error::InvalidTag),
        }
    }
}

