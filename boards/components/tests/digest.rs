//! Host-side register-model tests, not a simulation of the HASH algorithm or timing.
//! Keep these in a board crate because fake MMIO construction requires unsafe.

use core::cell::{Cell, RefCell};
use kernel::ErrorCode;
use kernel::deferred_call::{self, DeferredCall, DeferredCallClient};
use kernel::hil::crypto::digest::{
    Digest, DigestAlgorithm, DigestClient, Md5, Sha1, Sha224, Sha256,
};
use kernel::platform::chip::ThreadIdProvider;
use kernel::utilities::StaticRef;
use kernel::utilities::registers::interfaces::{Readable, Writeable};

#[path = "../../../chips/stm32u5xx/src/hash/hash.rs"]
mod hash;
#[path = "../../../chips/stm32u5xx/src/hash/regs.rs"]
mod regs;

enum TestThread {}

// ### Safety
// This executable has one test, does not spawn threads, and accesses deferred
// call state only from that test thread for its entire lifetime.
unsafe impl ThreadIdProvider for TestThread {
    fn running_thread_id() -> usize {
        0
    }
}

struct RegisterModel {
    words: &'static [Cell<u32>; 0x330 / 4],
    registers: StaticRef<regs::HashRegisters>,
}

impl RegisterModel {
    fn new() -> Self {
        let words: &'static [Cell<u32>; 0x330 / 4] =
            Box::leak(Box::new([const { Cell::new(0) }; 0x330 / 4]));
        assert_eq!(core::mem::size_of::<regs::HashRegisters>(), 0x330);
        assert_eq!(core::mem::align_of::<regs::HashRegisters>(), 4);
        // ### Safety
        // The leaked allocation has the register block's size and alignment.
        // All registers are u32 cells and all initialized bits are valid. Both
        // views use interior mutability; only register offsets (not reserved
        // bytes) are written, on this test's single thread. Neither view creates
        // mutable references and the allocation remains valid indefinitely.
        let registers = unsafe { StaticRef::new(words.as_ptr().cast::<regs::HashRegisters>()) };
        Self { words, registers }
    }

    fn capacity(&self, words: u32) {
        self.registers.sr.write(regs::SR::NBWE.val(words));
    }

    fn complete(&self, engine: &hash::Hash) {
        for index in 0..8 {
            self.words[0x310 / 4 + index].set(0x01020304 + index as u32);
        }
        self.registers.sr.write(regs::SR::DCIS::SET);
        engine.handle_interrupt();
    }
}

#[derive(Clone, Copy)]
enum ReadMode {
    Normal,
    Zero,
    Oversized,
    Error(ErrorCode),
}

struct Client {
    input: RefCell<Vec<u8>>,
    offset: Cell<usize>,
    limit: Cell<usize>,
    mode: Cell<ReadMode>,
    output_error: Cell<Option<ErrorCode>>,
    reads: Cell<usize>,
    output: RefCell<Vec<u8>>,
    completions: RefCell<Vec<Result<(), ErrorCode>>>,
    events: RefCell<Vec<&'static str>>,
    engine: &'static hash::Hash,
    restart: Cell<bool>,
}

impl Client {
    fn new(engine: &'static hash::Hash) -> Self {
        Self {
            input: RefCell::new(Vec::new()),
            offset: Cell::new(0),
            limit: Cell::new(64),
            mode: Cell::new(ReadMode::Normal),
            output_error: Cell::new(None),
            reads: Cell::new(0),
            output: RefCell::new(Vec::new()),
            completions: RefCell::new(Vec::new()),
            events: RefCell::new(Vec::new()),
            engine,
            restart: Cell::new(false),
        }
    }

    fn reset(&self, input: &[u8], limit: usize) {
        *self.input.borrow_mut() = input.to_vec();
        self.offset.set(0);
        self.limit.set(limit);
        self.mode.set(ReadMode::Normal);
        self.output_error.set(None);
        self.reads.set(0);
        self.output.borrow_mut().clear();
        self.completions.borrow_mut().clear();
        self.events.borrow_mut().clear();
    }
}

impl<Algorithm: DigestAlgorithm> DigestClient<Algorithm> for Client {
    fn read_input(&self, input: &mut [u8]) -> Result<usize, ErrorCode> {
        assert!(!input.is_empty());
        assert!(input.len() <= 64);
        assert!(input.len() <= self.input.borrow().len() - self.offset.get());
        assert_eq!(
            Digest::<Sha256>::digest(self.engine, 0),
            Err(ErrorCode::BUSY)
        );
        self.reads.set(self.reads.get() + 1);
        self.events.borrow_mut().push("read");
        match self.mode.get() {
            ReadMode::Zero => Ok(0),
            ReadMode::Oversized => Ok(input.len() + 1),
            ReadMode::Error(error) => Err(error),
            ReadMode::Normal => {
                let count = input.len().min(self.limit.get());
                let offset = self.offset.get();
                input[..count].copy_from_slice(&self.input.borrow()[offset..offset + count]);
                self.offset.set(offset + count);
                Ok(count)
            }
        }
    }

    fn write_digest(&self, digest: &Algorithm::Output) -> Result<(), ErrorCode> {
        assert!(self.output.borrow().is_empty());
        assert!(self.completions.borrow().is_empty());
        assert_eq!(self.offset.get(), self.input.borrow().len());
        assert_eq!(
            Digest::<Sha256>::digest(self.engine, 0),
            Err(ErrorCode::BUSY)
        );
        self.events.borrow_mut().push("write");
        *self.output.borrow_mut() = digest.as_ref().to_vec();
        self.output_error.get().map_or(Ok(()), Err)
    }

    fn digest_done(&self, result: Result<(), ErrorCode>) {
        self.events.borrow_mut().push("done");
        self.completions.borrow_mut().push(result);
        if self.restart.replace(false) {
            assert_eq!(Digest::<Sha256>::digest(self.engine, 0), Ok(()));
        }
    }
}

fn service() {
    assert!(DeferredCall::has_tasks());
    DeferredCall::service_next_pending();
}

fn exercise_algorithm<Algorithm: DigestAlgorithm + 'static>(
    engine: &'static hash::Hash,
    model: &RegisterModel,
    client: &'static Client,
    mode: u32,
) where
    hash::Hash: Digest<Algorithm>,
{
    Digest::<Algorithm>::set_client(engine, client);
    for len in [0, 1, 2, 3, 4, 55, 56, 63, 64, 65, 67, 68, 127, 128, 129] {
        let input: Vec<u8> = (0..len).map(|index| index as u8).collect();
        client.reset(&input, 1);
        model.capacity(16);
        assert_eq!(Digest::<Algorithm>::digest(engine, len), Ok(()));
        assert!(client.events.borrow().is_empty());
        assert_eq!(model.registers.cr.read(regs::CR::ALGO), mode);
        assert_eq!(model.registers.cr.read(regs::CR::DATATYPE), 2);
        assert!(!model.registers.cr.is_set(regs::CR::MODE));
        assert!(!model.registers.cr.is_set(regs::CR::DMAE));
        assert_eq!(
            model.registers.str.read(regs::STR::NBLW),
            (len % 4 * 8) as u32
        );
        assert_eq!(Digest::<Md5>::digest(engine, 0), Err(ErrorCode::BUSY));
        assert_eq!(Digest::<Sha1>::digest(engine, 0), Err(ErrorCode::BUSY));
        assert_eq!(Digest::<Sha224>::digest(engine, 0), Err(ErrorCode::BUSY));
        assert_eq!(Digest::<Sha256>::digest(engine, 0), Err(ErrorCode::BUSY));
        let mut written = 0;
        for count in 1..=len.max(1) {
            service();
            if len != 0 && (count % 4 == 0 || count == len) {
                let mut word = [0; 4];
                word[..count - written].copy_from_slice(&input[written..count]);
                assert_eq!(model.words[1].get(), u32::from_le_bytes(word));
                written = count;
            }
        }
        assert!(!DeferredCall::has_tasks());
        assert_eq!(client.reads.get(), len);
        assert_eq!(client.offset.get(), len);
        assert!(model.registers.str.is_set(regs::STR::DCAL));
        assert_eq!(model.registers.imr.get(), 2);
        engine.handle_interrupt();
        assert!(client.completions.borrow().is_empty());
        assert_eq!(model.registers.imr.get(), 2);
        model.complete(engine);
        let expected: Vec<u8> = (0..Algorithm::DIGEST_LEN / 4)
            .flat_map(|index| (0x01020304 + index as u32).to_be_bytes())
            .collect();
        assert_eq!(*client.output.borrow(), expected);
        assert_eq!(*client.completions.borrow(), [Ok(())]);
        assert!(client.events.borrow().ends_with(&["write", "done"]));
        assert_eq!(model.registers.imr.get(), 0);
        assert!(!model.registers.sr.is_set(regs::SR::DCIS));
        engine.handle_interrupt();
        assert_eq!(client.completions.borrow().len(), 1);
    }
}

#[test]
fn stm32u5_digest_state_machine() {
    deferred_call::initialize_deferred_call_state::<TestThread>();
    let model = RegisterModel::new();
    let _hardware_base = regs::HASH_BASE;
    let engine = Box::leak(Box::new(hash::Hash::new(model.registers)));
    engine.register();
    assert_eq!(Digest::<Md5>::digest(engine, 0), Err(ErrorCode::RESERVE));
    assert_eq!(Digest::<Sha1>::digest(engine, 0), Err(ErrorCode::RESERVE));
    assert_eq!(Digest::<Sha224>::digest(engine, 0), Err(ErrorCode::RESERVE));
    assert_eq!(Digest::<Sha256>::digest(engine, 0), Err(ErrorCode::RESERVE));
    assert!(!DeferredCall::has_tasks());
    let client = Box::leak(Box::new(Client::new(engine)));
    exercise_algorithm::<Md5>(engine, &model, client, 1);
    exercise_algorithm::<Sha1>(engine, &model, client, 0);
    exercise_algorithm::<Sha224>(engine, &model, client, 2);
    exercise_algorithm::<Sha256>(engine, &model, client, 3);

    let input: Vec<u8> = (0..129).collect();
    client.reset(&input, 64);
    model.capacity(17);
    Digest::<Sha256>::digest(engine, input.len()).unwrap();
    service();
    assert_eq!(client.offset.get(), 64);
    assert_eq!(model.words[1].get(), 0x3f3e3d3c);
    model.capacity(0);
    service();
    assert_eq!(client.reads.get(), 1);
    assert_eq!(model.registers.imr.get(), 1);
    assert!(!DeferredCall::has_tasks());
    model.capacity(16);
    engine.handle_interrupt();
    assert_eq!(client.offset.get(), 128);
    assert_eq!(model.words[1].get(), 0x7f7e7d7c);
    service();
    assert_eq!(model.words[1].get(), 128);
    assert_eq!(client.reads.get(), 3);
    model.complete(engine);
    assert_eq!(*client.completions.borrow(), [Ok(())]);

    client.reset(&[1, 2, 3, 4, 5], 3);
    model.capacity(0);
    Digest::<Sha256>::digest(engine, 5).unwrap();
    service();
    assert_eq!(client.reads.get(), 0);
    assert_eq!(model.registers.imr.get(), 1);
    assert!(!DeferredCall::has_tasks());
    model.capacity(1);
    engine.handle_interrupt();
    assert_eq!(client.offset.get(), 3);
    service();
    assert_eq!(client.offset.get(), 4);
    assert_eq!(model.words[1].get(), 0x04030201);
    service();
    assert_eq!(model.words[1].get(), 5);
    model.complete(engine);
    assert_eq!(*client.completions.borrow(), [Ok(())]);

    for (mode, error) in [
        (ReadMode::Zero, ErrorCode::SIZE),
        (ReadMode::Oversized, ErrorCode::SIZE),
        (ReadMode::Error(ErrorCode::CANCEL), ErrorCode::CANCEL),
    ] {
        client.reset(&[1, 2, 3, 4, 5], 3);
        model.capacity(16);
        Digest::<Sha256>::digest(engine, 5).unwrap();
        service();
        client.mode.set(mode);
        service();
        assert_eq!(*client.completions.borrow(), [Err(error)]);
        assert!(client.output.borrow().is_empty());
        assert_eq!(client.reads.get(), 2);
        assert_eq!(model.registers.imr.get(), 0);
        engine.handle_interrupt();
        assert!(!DeferredCall::has_tasks());
        assert_eq!(client.completions.borrow().len(), 1);
    }

    client.reset(&[], 64);
    client.output_error.set(Some(ErrorCode::NOMEM));
    Digest::<Sha256>::digest(engine, 0).unwrap();
    service();
    model.complete(engine);
    assert_eq!(*client.completions.borrow(), [Err(ErrorCode::NOMEM)]);
    assert_eq!(*client.events.borrow(), ["write", "done"]);

    let replacement = Box::leak(Box::new(Client::new(engine)));
    client.reset(&[9], 64);
    model.capacity(16);
    Digest::<Sha256>::digest(engine, 1).unwrap();
    Digest::<Sha256>::set_client(engine, replacement);
    service();
    client.restart.set(true);
    model.complete(engine);
    assert!(replacement.events.borrow().is_empty());
    assert_eq!(*client.completions.borrow(), [Ok(())]);
    service();
    model.complete(engine);
    assert_eq!(*replacement.events.borrow(), ["write", "done"]);
    assert_eq!(*replacement.completions.borrow(), [Ok(())]);
    assert!(!DeferredCall::has_tasks());
}
