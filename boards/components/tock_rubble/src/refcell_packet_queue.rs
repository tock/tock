//! Safe, non-interrupt-driven packet queue written using RefCells.
//!
//! Copied from https://github.com/jonas-schievink/rubble/pull/137/files#diff-80283f6441097bbff4aad2c54957a72a.

use core::cell::{Cell, RefCell};
use rubble::{
    bytes::{ByteReader, ByteWriter, ToBytes},
    link::{
        data::{self, Llid},
        queue::{Consume, Consumer, PacketQueue, Producer},
        MIN_DATA_PAYLOAD_BUF, MIN_DATA_PDU_BUF,
    },
    Error,
};
// TODO: make an implementation of Producer/Consumer which uses an Owned rather
// this.
pub struct RefCellQueue {
    buf: RefCell<[u8; MIN_DATA_PDU_BUF]>,
    full: Cell<bool>,
}

// If this were used outside of the Tock ecosystem, this would be unsound. But
// as we are strictly single-threaded within the kernel, this is OK.
// We need Send/Sync impls to store RefCellQueues in static variables.
unsafe impl Send for RefCellQueue {}
unsafe impl Sync for RefCellQueue {}

impl RefCellQueue {
    pub const fn new() -> Self {
        Self {
            buf: RefCell::new([0; MIN_DATA_PDU_BUF]),
            full: Cell::new(false),
        }
    }
}

impl<'a> PacketQueue for &'a RefCellQueue {
    type Producer = RefCellProducer<'a>;
    type Consumer = RefCellConsumer<'a>;

    fn split(self) -> (Self::Producer, Self::Consumer) {
        let p = RefCellProducer { queue: self };
        let c = RefCellConsumer { queue: self };

        (p, c)
    }
}

pub struct RefCellProducer<'a> {
    queue: &'a RefCellQueue,
}

impl<'a> Producer for RefCellProducer<'a> {
    fn free_space(&self) -> u8 {
        if self.queue.full.get() {
            0
        } else {
            MIN_DATA_PDU_BUF as u8
        }
    }

    fn produce_dyn(
        &mut self,
        payload_bytes: u8,
        f: &mut dyn FnMut(&mut ByteWriter<'_>) -> Result<Llid, Error>,
    ) -> Result<(), Error> {
        assert!(usize::from(payload_bytes) <= MIN_DATA_PAYLOAD_BUF);

        if self.queue.full.get() {
            return Err(Error::Eof);
        }

        let mut buf = self.queue.buf.borrow_mut();

        let mut writer = ByteWriter::new(&mut buf[2..]);
        let free = writer.space_left();
        let llid = f(&mut writer)?;
        let used = free - writer.space_left();

        let mut header = data::Header::new(llid);
        header.set_payload_length(used as u8);
        header.to_bytes(&mut ByteWriter::new(&mut buf[..2]))?;

        self.queue.full.set(true);
        Ok(())
    }
}

pub struct RefCellConsumer<'a> {
    queue: &'a RefCellQueue,
}

impl<'a> Consumer for RefCellConsumer<'a> {
    fn has_data(&self) -> bool {
        self.queue.full.get()
    }

    fn consume_raw_with<R>(
        &mut self,
        f: impl FnOnce(data::Header, &[u8]) -> Consume<R>,
    ) -> Result<R, Error> {
        if !self.has_data() {
            return Err(Error::Eof);
        }

        let buf = self.queue.buf.borrow();

        let mut bytes = ByteReader::new(&*buf);
        let raw_header: [u8; 2] = bytes.read_array().unwrap();
        let header = data::Header::parse(&raw_header);
        let pl_len = usize::from(header.payload_length());
        let raw_payload = bytes.read_slice(pl_len)?;

        let res = f(header, raw_payload);
        if res.should_consume() {
            self.queue.full.set(false);
        }
        res.into_result()
    }
}
