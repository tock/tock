#![feature(slice_patterns)]
#![allow(unused_imports)]
use std::io;
use std::io::{Read, Write};
extern crate msg;
#[allow(unused_imports)]
use msg::*;
use msg::cauterize::Cauterize;
extern crate byteorder;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const FP_SIZE: usize = 1;

#[derive(Debug)]
enum TestError {
    Io(io::Error),
    Cauterize(cauterize::Error),
    Fingerprint,
}

impl From<io::Error> for TestError {
     fn from(err: io::Error) -> TestError {
        TestError::Io(err)
    }
}

impl From<cauterize::Error> for TestError {
     fn from(err: cauterize::Error) -> TestError {
        TestError::Cauterize(err)
    }
}

#[derive(Debug,Clone)]
struct Header {
    len: usize,
    fingerprint: [u8; FP_SIZE],
}

impl Header {
    fn read(stream: &mut Read) -> Result<Header, TestError> {
        let len = stream.read_u8()?;
        let mut fingerprint = [0u8; FP_SIZE];
        stream.read_exact(&mut fingerprint)?;
        Ok(Header {
            len: len as usize,
            fingerprint: fingerprint,
        })
    }


    fn write(&self, stream: &mut Write) -> Result<(), TestError> {
        stream.write_u8(self.len as u8)?;
        stream.write_all(&self.fingerprint)?;
        Ok(())
    }
}

#[derive(Debug,Clone)]
struct Message {
    header: Header,
    payload: Vec<u8>,
}

impl Message {
    fn read(stream: &mut Read) -> Result<Message, TestError> {
        let header = Header::read(stream)?;
        let mut payload = Vec::new();
        let mut chunk = stream.take(header.len as u64);
        chunk.read_to_end(&mut payload)?;
        let msg = Message {
            header: header,
            payload: payload,
        };
        Ok(msg)
    }

    fn write(&self, stream: &mut Write) -> Result<(), TestError> {
        self.header.write(stream)?;
        stream.write_all(&self.payload)?;
        Ok(())
    }
}

fn decode_then_encode(message: &Message) -> Result<Message, TestError> {
    let mut dctx = ::std::io::Cursor::new(message.payload.clone());

    let ebuf = Vec::new();
    let mut ectx = ::std::io::Cursor::new(ebuf);

    match message.header.fingerprint {
        [0xa2] => {
    let a = Pong::decode(&mut dctx)?;
    a.encode(&mut ectx)?;
    let ebuf = ectx.into_inner();
    let message = Message {
        header: Header {
            len: ebuf.len(),
            fingerprint: [0xa2],
        },
        payload: ebuf,
    };
    Ok(message)
}
[0x77] => {
    let a = Pingpong::decode(&mut dctx)?;
    a.encode(&mut ectx)?;
    let ebuf = ectx.into_inner();
    let message = Message {
        header: Header {
            len: ebuf.len(),
            fingerprint: [0x77],
        },
        payload: ebuf,
    };
    Ok(message)
}
[0xdb] => {
    let a = Frame::decode(&mut dctx)?;
    a.encode(&mut ectx)?;
    let ebuf = ectx.into_inner();
    let message = Message {
        header: Header {
            len: ebuf.len(),
            fingerprint: [0xdb],
        },
        payload: ebuf,
    };
    Ok(message)
}

        _ => Err(TestError::Fingerprint),
    }
}

fn tester() {
    let decoded_message = Message::read(&mut io::stdin()).unwrap();
    let encoded_message = decode_then_encode(&decoded_message).unwrap();
    encoded_message.write(&mut io::stdout()).unwrap();
}

fn main() {
    let t = ::std::thread::Builder::new()
        .stack_size(1024 * 1024 * 16)
        .spawn(tester)
        .unwrap();
    t.join().unwrap();
}
