#![allow(dead_code, unused_variables, unused_imports)]
#![no_std]

extern crate core as std;
#[macro_use]
pub extern crate cauterize;
use self::cauterize::{Cauterize, Decoder, Encoder, Error, Primitive, Range, Vector};
use std::mem;

pub static SPEC_NAME: &'static str = "msg";
pub const SPEC_FINGERPRINT: [u8; 20] = [
    0x2e, 0x1a, 0x97, 0x28, 0x30, 0x22, 0x6c, 0xbf, 0xd7, 0x5d, 0x3b, 0xc2, 0x53, 0x19, 0x21, 0xa9,
    0xe3, 0x89, 0x29, 0x8d,
];
pub const SPEC_MIN_SIZE: usize = 1;
pub const SPEC_MAX_SIZE: usize = 200;

impl_vector!(Payload, u8, 180);

impl Cauterize for Payload {
    const FINGERPRINT: [u8; 20] = [
        0xa5, 0xb7, 0xbd, 0xaa, 0x64, 0xe1, 0x98, 0x50, 0x21, 0x51, 0x7b, 0x2c, 0x24, 0xe4, 0x23,
        0x2a, 0x62, 0x19, 0x98, 0xfe,
    ];
    const SIZE_MIN: usize = 1;
    const SIZE_MAX: usize = 181;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        if self.len > 180 {
            return Err(Error::ElementCount);
        }
        (self.len as u8).encode(ctx)?;
        for i in 0..self.len {
            self.elems[i].encode(ctx)?
        }
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let len = u8::decode(ctx)? as usize;
        if len > 180 {
            return Err(Error::ElementCount);
        }
        let mut v = Payload::new();
        for _ in 0..len {
            v.push(u8::decode(ctx)?);
        }
        Ok(v)
    }
}

impl_array!(Addr, u8, 10);

impl Cauterize for Addr {
    const FINGERPRINT: [u8; 20] = [
        0xea, 0x0d, 0xda, 0x38, 0x21, 0x29, 0x99, 0xbb, 0x35, 0x50, 0x72, 0x20, 0xd4, 0xbc, 0xc0,
        0xd8, 0xaa, 0x19, 0xef, 0xef,
    ];
    const SIZE_MIN: usize = 10;
    const SIZE_MAX: usize = 10;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        let ref elems = self.0;
        for elem in elems.iter() {
            elem.encode(ctx)?;
        }
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let mut arr: [u8; 10] = unsafe { mem::uninitialized() };
        for i in 0..10 {
            arr[i] = u8::decode(ctx)?;
        }
        Ok(Addr(arr))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ping {
    pub id: u16,       // 1
    pub address: Addr, // 2
    pub seq: u8,       // 3
    pub len: u32,      // 4
    pub data: Payload, // 5
}

impl Cauterize for Ping {
    const FINGERPRINT: [u8; 20] = [
        0xdf, 0x0b, 0x92, 0x3e, 0x39, 0x5d, 0xc0, 0x33, 0xdb, 0xd1, 0x69, 0x93, 0x5f, 0xb6, 0x3a,
        0x5a, 0x9e, 0xf9, 0x2d, 0x65,
    ];
    const SIZE_MIN: usize = 18;
    const SIZE_MAX: usize = 198;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        self.id.encode(ctx)?;
        self.address.encode(ctx)?;
        self.seq.encode(ctx)?;
        self.len.encode(ctx)?;
        self.data.encode(ctx)?;
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let rec = Ping {
            id: u16::decode(ctx)?,
            address: Addr::decode(ctx)?,
            seq: u8::decode(ctx)?,
            len: u32::decode(ctx)?,
            data: Payload::decode(ctx)?,
        };
        Ok(rec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pong {
    pub id: u8,        // 1
    pub address: Addr, // 2
    pub seq: u8,       // 3
    pub len: u32,      // 4
}

impl Cauterize for Pong {
    const FINGERPRINT: [u8; 20] = [
        0x70, 0xfc, 0x74, 0x6f, 0x4d, 0x64, 0xf4, 0xe8, 0xd4, 0x8f, 0xb2, 0xa9, 0xa5, 0xfe, 0x84,
        0xcf, 0x51, 0x6b, 0x92, 0x33,
    ];
    const SIZE_MIN: usize = 16;
    const SIZE_MAX: usize = 16;

    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        self.id.encode(ctx)?;
        self.address.encode(ctx)?;
        self.seq.encode(ctx)?;
        self.len.encode(ctx)?;
        Ok(())
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let rec = Pong {
            id: u8::decode(ctx)?,
            address: Addr::decode(ctx)?,
            seq: u8::decode(ctx)?,
            len: u32::decode(ctx)?,
        };
        Ok(rec)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pingpong {
    Ping(Ping), // 1
    Pong(Pong), // 2
}

impl Cauterize for Pingpong {
    const FINGERPRINT: [u8; 20] = [
        0x53, 0xad, 0x70, 0xe5, 0xd7, 0x56, 0x3c, 0x17, 0x58, 0x19, 0x08, 0x71, 0xcb, 0xc4, 0x22,
        0x48, 0x1a, 0xdf, 0xb3, 0xb6,
    ];
    const SIZE_MIN: usize = 17;
    const SIZE_MAX: usize = 199;

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
            1 => Ok(Pingpong::Ping(Ping::decode(ctx)?)),
            2 => Ok(Pingpong::Pong(Pong::decode(ctx)?)),
            _ => Err(Error::InvalidTag),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Pingpong(Pingpong), // 1
}

impl Cauterize for Frame {
    const FINGERPRINT: [u8; 20] = [
        0x4b, 0x5f, 0x29, 0x1b, 0x6d, 0x67, 0xe5, 0xdf, 0x2a, 0x34, 0xf1, 0x7c, 0x5a, 0x64, 0xae,
        0x46, 0x47, 0x94, 0x7b, 0x86,
    ];
    const SIZE_MIN: usize = 18;
    const SIZE_MAX: usize = 200;

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
            1 => Ok(Frame::Pingpong(Pingpong::decode(ctx)?)),
            _ => Err(Error::InvalidTag),
        }
    }
}
