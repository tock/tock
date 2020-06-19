#[derive(Debug)]
pub enum SResult<Output = (), Error = ()> {
    // `Done(off, out)`: No errors encountered. We are currently at `off` in the
    // buffer, and the previous encoder/decoder produced output `out`.
    Done(usize, Output),

    // `Needed(bytes)`: Could not proceed because we needed to have `bytes`
    // bytes in the buffer, but there weren't.
    Needed(usize),

    // `Error(err)`: Some other error occurred.
    Error(Error),
}

impl<Output, Error> SResult<Output, Error> {
    pub fn is_done(&self) -> bool {
        match *self {
            SResult::Done(_, _) => true,
            _ => false,
        }
    }

    pub fn is_needed(&self) -> bool {
        match *self {
            SResult::Needed(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match *self {
            SResult::Error(_) => true,
            _ => false,
        }
    }

    pub fn done(self) -> Option<(usize, Output)> {
        match self {
            SResult::Done(offset, out) => Some((offset, out)),
            _ => None,
        }
    }

    pub fn needed(self) -> Option<usize> {
        match self {
            SResult::Needed(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn err(self) -> Option<Error> {
        match self {
            SResult::Error(err) => Some(err),
            _ => None,
        }
    }
}

/// Returns the result of encoding/decoding
#[macro_export]
macro_rules! stream_done {
    ($bytes:expr, $out:expr) => {
        return SResult::Done($bytes, $out);
    };
    ($bytes:expr) => {
        stream_done!($bytes, ());
    };
}

/// Returns a buffer length error if there are not enough bytes
#[macro_export]
macro_rules! stream_len_cond {
    ($buf:expr, $bytes:expr) => {
        if $buf.len() < $bytes {
            return SResult::Needed($bytes);
        }
    };
}

/// Returns an error
#[macro_export]
macro_rules! stream_err {
    ($err:expr) => {
        return SResult::Error($err);
    };
    () => {
        stream_err!(());
    };
}

/// Returns an error if a condition is unmet
#[macro_export]
macro_rules! stream_cond {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return SResult::Error($err);
        }
    };
    ($cond:expr) => {
        stream_cond!($cond, ());
    };
}

/// Gets the result of an Option<T>, throwing a stream error if it is None
#[macro_export]
macro_rules! stream_from_option {
    ($opt:expr, $err:expr) => {
        match $opt {
            Some(opt) => opt,
            None => stream_err!($err),
        }
    };
    ($opt:expr) => {
        stream_from_option!($opt, ());
    };
}

/// Extracts the result of encoding/decoding (the new offset and the output) only
/// if no errors were encountered in encoding. This macro makes it possible to
/// handle offsets easily for the following use cases:
///
/// `enc_try!(result, offset)`: Unwrap an already-provided result that
/// represents starting from `offset` in the buffer.
/// `enc_try!(buf, offset; encoder, args..)`: Use the encoder function, called with the
/// optionally provided arguments, on the buffer starting from `offset`.
/// `enc_try!(buf, offset; object; method, args..)`: Same as the above, but the
/// encoder function is a member method of object.
///
/// Additionally, the offset can always be omitted from any of the above, which
/// would result in it defaulting to 0. Idiomatically, the way to combine
/// encoders is to define another encoder as follows:
///
/// ```rust
/// # use capsules::{enc_try, stream_done};
/// # use capsules::net::stream::SResult;
///
/// // call a simple encoder
/// let (bytes, out1) = enc_try!(buf; encoder1);
/// /* Do something with out1 */
///
/// // call another encoder, but starting at the previous offset
/// let (bytes, out2) = enc_try!(buf, bytes; encoder2);
/// /* Note that bytes is shadowed. Alternatively you could mutably update a
/// variable. */
///
/// // subsequently, encode a struct into the buffer, with some extra arguments
/// let (bytes, out3) = enc_try!(buf, bytes; someheader; encode, 2, 5);
///
/// // report success without returning a meaningful output
/// stream_done!(bytes);
/// ```
///
/// Then, using an encoder can be done simply by:
///
/// ```
/// # use capsules::net::stream::SResult;
///
/// match encoder(&mut buf) {
///     SResult::Done(off, out) => { /* celebrate */ }
///     SResult::Needed(off) => { /* get more memory? */ }
///     SResult::Error(err) => { /* give up */ }
/// }
/// ```
#[macro_export]
macro_rules! enc_try {
    ($result:expr, $offset:expr) => {
        match $result {
            SResult::Done(offset, out) => ($offset + offset, out),
            SResult::Needed(bytes) => { return SResult::Needed($offset + bytes); }
            SResult::Error(error) => { return SResult::Error(error); }
        }
    };
    ($result:expr)
        => { enc_try!($result, 0) };
    ($buf:expr, $offset:expr; $fun:expr)
        => { enc_try!($fun(&mut $buf[$offset..]), $offset) };
    ($buf:expr, $offset:expr; $fun:expr, $($args:expr),+)
        => { enc_try!($fun(&mut $buf[$offset..], $($args),+), $offset) };
    ($buf:expr, $offset:expr; $object:expr; $fun:ident)
        => { enc_try!($object.$fun(&mut $buf[$offset..]), $offset) };
    ($buf:expr, $offset:expr; $object:expr; $fun:ident, $($args:expr),+)
        => { enc_try!($object.$fun(&mut $buf[$offset..], $($args),+), $offset) };
    ($buf:expr; $($tts:tt)+)
        => { enc_try!($buf, 0; $($tts)+) };
}

/// This is the aforementioned version of the unwrapping macro that only returns
/// the offset. With this, it can be simpler to programmatically chain multiple
/// headers together when the outputs do not have to be collated.
#[macro_export]
macro_rules! enc_consume {
    ($($tts:tt)*) => { {
        let (offset, _) = enc_try!($($tts)*);
        offset
    } };
}

/// The decoding equivalent of `enc_try`. The only difference is that only an
/// immutable borrow of the buffer is required each time.
#[macro_export]
macro_rules! dec_try {
    ($result:expr, $offset:expr) => {
        match $result {
            SResult::Done(offset, out) => ($offset + offset, out),
            SResult::Needed(bytes) => { return SResult::Needed($offset + bytes); }
            SResult::Error(error) => { return SResult::Error(error); }
        }
    };
    ($result:expr)
        => { dec_try!($result, 0) };
    ($buf:expr, $offset:expr; $fun:expr)
        => { dec_try!($fun(&$buf[$offset..]), $offset) };
    ($buf:expr, $offset:expr; $fun:expr, $($args:expr),+)
        => { dec_try!($fun(&$buf[$offset..], $($args),+), $offset) };
    ($buf:expr, $offset:expr; $object:expr; $fun:ident)
        => { dec_try!($object.$fun(&$buf[$offset..]), $offset) };
    ($buf:expr, $offset:expr; $object:expr; $fun:ident, $($args:expr),+)
        => { dec_try!($object.$fun(&$buf[$offset..], $($args),+), $offset) };
    ($buf:expr; $($tts:tt)+)
        => { dec_try!($buf, 0; $($tts)+) };
}

/// The decoding equivalent of `enc_consume`
#[macro_export]
macro_rules! dec_consume {
    ($($tts:tt)*) => { {
        let (offset, _) = dec_try!($($tts)*);
        offset
    } };
}

pub fn encode_u8(buf: &mut [u8], b: u8) -> SResult {
    stream_len_cond!(buf, 1);
    buf[0] = b;
    stream_done!(1);
}

pub fn encode_u16(buf: &mut [u8], b: u16) -> SResult {
    stream_len_cond!(buf, 2);
    buf[0] = (b >> 8) as u8;
    buf[1] = b as u8;
    stream_done!(2);
}

pub fn encode_u32(buf: &mut [u8], b: u32) -> SResult {
    stream_len_cond!(buf, 4);
    buf[0] = (b >> 24) as u8;
    buf[1] = (b >> 16) as u8;
    buf[2] = (b >> 8) as u8;
    buf[3] = b as u8;
    stream_done!(4);
}

pub fn encode_bytes(buf: &mut [u8], bs: &[u8]) -> SResult {
    stream_len_cond!(buf, bs.len());
    buf[..bs.len()].copy_from_slice(bs);
    stream_done!(bs.len());
}

// This function assumes that the host is little-endian
pub fn encode_bytes_be(buf: &mut [u8], bs: &[u8]) -> SResult {
    stream_len_cond!(buf, bs.len());
    for (i, b) in bs.iter().rev().enumerate() {
        buf[i] = *b;
    }
    stream_done!(bs.len());
}

pub fn decode_u8(buf: &[u8]) -> SResult<u8> {
    stream_len_cond!(buf, 1);
    stream_done!(1, buf[0]);
}

pub fn decode_u16(buf: &[u8]) -> SResult<u16> {
    stream_len_cond!(buf, 2);
    stream_done!(2, (buf[0] as u16) << 8 | (buf[1] as u16));
}

pub fn decode_u32(buf: &[u8]) -> SResult<u32> {
    stream_len_cond!(buf, 4);
    let b = (buf[0] as u32) << 24 | (buf[1] as u32) << 16 | (buf[2] as u32) << 8 | (buf[3] as u32);
    stream_done!(4, b);
}

pub fn decode_bytes(buf: &[u8], out: &mut [u8]) -> SResult {
    stream_len_cond!(buf, out.len());
    let len = out.len();
    out.copy_from_slice(&buf[..len]);
    stream_done!(out.len());
}

// This function assumes that the host is little-endian
pub fn decode_bytes_be(buf: &[u8], out: &mut [u8]) -> SResult {
    stream_len_cond!(buf, out.len());
    for (i, b) in buf[..out.len()].iter().rev().enumerate() {
        out[i] = *b;
    }
    stream_done!(out.len());
}
