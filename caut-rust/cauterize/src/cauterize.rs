use byteorder::{ByteOrder, LittleEndian};
use error::Error;

pub struct Encoder<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> Encoder<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }

    pub fn consume(self) -> usize {
        self.pos
    }
}

pub struct Decoder<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[self.pos..]
    }

    pub fn consume(self) -> usize {
        self.pos
    }
}

pub trait Cauterize: 'static + Sized {
    const FINGERPRINT: [u8; 20];
    const SIZE_MIN: usize;
    const SIZE_MAX: usize;
    fn encode(&self, &mut Encoder) -> Result<(), Error>;
    fn decode(&mut Decoder) -> Result<Self, Error>;
}

pub trait Primitive: 'static + Sized {
    fn encode(&self, &mut Encoder) -> Result<(), Error>;
    fn decode(&mut Decoder) -> Result<Self, Error>;
}

// ****************
// Primitive impls
// ****************

macro_rules! impl_primitive {
    ($T:ty, $READ_FN:path, $WRITE_FN:path) => {
        impl Primitive for $T {
            fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
                let ty_sz = ::std::mem::size_of::<Self>();
                {
                    let buf = ctx.as_mut_slice();
                    if ty_sz > buf.len() {
                        return Err(Error::Encode);
                    }
                    $WRITE_FN(buf, *self);
                }
                ctx.pos += ty_sz;
                Ok(())
            }
            fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
                let ty_sz = ::std::mem::size_of::<Self>();
                let val = {
                    let buf = ctx.as_slice();
                    if ty_sz > buf.len() {
                        return Err(Error::Encode);
                    }
                    $READ_FN(buf)
                };
                ctx.pos += ty_sz;
                Ok(val)
            }
        }
    };
}

impl_primitive!(u16, LittleEndian::read_u16, LittleEndian::write_u16);
impl_primitive!(i16, LittleEndian::read_i16, LittleEndian::write_i16);
impl_primitive!(u32, LittleEndian::read_u32, LittleEndian::write_u32);
impl_primitive!(i32, LittleEndian::read_i32, LittleEndian::write_i32);
impl_primitive!(u64, LittleEndian::read_u64, LittleEndian::write_u64);
impl_primitive!(i64, LittleEndian::read_i64, LittleEndian::write_i64);
impl_primitive!(f32, LittleEndian::read_f32, LittleEndian::write_f32);
impl_primitive!(f64, LittleEndian::read_f64, LittleEndian::write_f64);

// We can't use `impl_primitive!` for u8/i8 since it read/write for u8/i8 does not take paramters
impl Primitive for u8 {
    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        let ty_sz = ::std::mem::size_of::<Self>();
        {
            let buf = ctx.as_mut_slice();
            if ty_sz > buf.len() {
                return Err(Error::Encode);
            }
            buf[0] = *self;
        }
        ctx.pos += ty_sz;
        Ok(())
    }
    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let ty_sz = ::std::mem::size_of::<Self>();
        let val = {
            let buf = ctx.as_slice();
            if ty_sz > buf.len() {
                return Err(Error::Encode);
            }
            buf[0]
        };
        ctx.pos += ty_sz;
        Ok(val)
    }
}

impl Primitive for i8 {
    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        let ty_sz = ::std::mem::size_of::<Self>();
        {
            let buf = ctx.as_mut_slice();
            if ty_sz > buf.len() {
                return Err(Error::Encode);
            }
            buf[0] = *self as u8;
        }
        ctx.pos += ty_sz;
        Ok(())
    }
    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        let ty_sz = ::std::mem::size_of::<Self>();
        let val = {
            let buf = ctx.as_slice();
            if ty_sz > buf.len() {
                return Err(Error::Encode);
            }
            buf[0] as i8
        };
        ctx.pos += ty_sz;
        Ok(val)
    }
}

// We can't use `impl_primitive!` for bool since it
impl Primitive for bool {
    fn encode(&self, ctx: &mut Encoder) -> Result<(), Error> {
        let val = *self as u8;
        val.encode(ctx)
    }

    fn decode(ctx: &mut Decoder) -> Result<Self, Error> {
        match u8::decode(ctx) {
            Ok(0) => Ok(false),
            Ok(1) => Ok(true),
            Err(e) => Err(e),
            _ => Err(Error::InvalidValue),
        }
    }
}
