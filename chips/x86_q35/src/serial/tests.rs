// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

extern crate std;

use super::{DeferredCallInterface, IER, IIR, LCR, LSR, SerialPort, ports};
use core::cell::Cell;
use core::mem::forget;
use core::ptr::{self, NonNull};
use kernel::ErrorCode;
use kernel::deferred_call::DeferredCallClient;
use kernel::hil::uart::{Transmit, TransmitClient};
use std::boxed::Box;
use tock_registers_v0::{FakeRegister, LocalRegisterCopy, NoAccess, Safe};

// Fake version of a subset of the 8250's functionality. This only implements
// enough features to run the test cases in this module.
struct Fake8250 {
    tx_buffer: Cell<Option<u8>>,
    lcr: Cell<LocalRegisterCopy<u8, LCR::Register>>,
    ier: Cell<u8>,
}

impl Fake8250 {
    pub fn new() -> Self {
        Self {
            tx_buffer: Cell::new(None),
            lcr: LocalRegisterCopy::new(0).into(),
            ier: 0.into(),
        }
    }

    /// Simulates sending a byte (empties the transmit buffer). Returns None if
    /// the transmit buffer was empty.
    pub fn simulate_tx(&self) -> Option<u8> {
        self.tx_buffer.take()
    }
}

impl Default for Fake8250 {
    fn default() -> Self {
        Self {
            tx_buffer: Cell::new(None),
            lcr: LocalRegisterCopy::new(0).into(),
            ier: 0.into(),
        }
    }
}

impl ports::Interface for Fake8250 {
    type divisor_lsb<'s> = FakeRegister<(), u8, Safe, Safe>;
    fn divisor_lsb(&self) -> FakeRegister<(), u8, Safe, Safe> {
        FakeRegister::new(())
            .on_read(|()| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }

    type rx_buffer<'s> = FakeRegister<(), u8, Safe, NoAccess>;
    fn rx_buffer(&self) -> FakeRegister<(), u8, Safe, NoAccess> {
        FakeRegister::new(()).on_read(|()| unimplemented!())
    }

    type tx_buffer<'s> = FakeRegister<&'s Self, u8, NoAccess, Safe>;
    fn tx_buffer(&self) -> FakeRegister<&Self, u8, NoAccess, Safe> {
        FakeRegister::new(self).on_write(|s, byte| {
            assert!(
                !s.lcr.get().is_set(LCR::DLAB),
                "wrote tx_buffer w/ DLAB set"
            );
            assert!(
                s.tx_buffer.replace(Some(byte)).is_none(),
                "TX buffer overflow"
            );
        })
    }

    type divisor_msb<'s> = FakeRegister<(), u8, Safe, Safe>;
    fn divisor_msb(&self) -> FakeRegister<(), u8, Safe, Safe> {
        FakeRegister::new(())
            .on_read(|()| unimplemented!())
            .on_write(|_, _| unimplemented!())
    }

    type interrupt_enable<'s> = FakeRegister<&'s Self, IER::Register, Safe, Safe>;
    fn interrupt_enable(&self) -> FakeRegister<&Self, IER::Register, Safe, Safe> {
        FakeRegister::new(self)
            .on_read(|s| {
                assert!(!s.lcr.get().is_set(LCR::DLAB), "read IER w/ DLAB set");
                s.ier.get()
            })
            .on_write(|s, v| {
                assert!(!s.lcr.get().is_set(LCR::DLAB), "write IER w/ DLAB set");
                s.ier.set(v)
            })
    }

    type interrupt_id<'s> = FakeRegister<(), IIR::Register, Safe, NoAccess>;
    fn interrupt_id(&self) -> FakeRegister<(), IIR::Register, Safe, NoAccess> {
        FakeRegister::new(()).on_read(|()| unimplemented!())
    }

    type fifo_control<'s> = FakeRegister<(), u8, NoAccess, Safe>;
    fn fifo_control(&self) -> FakeRegister<(), u8, NoAccess, Safe> {
        FakeRegister::new(()).on_write(|_, _| unimplemented!())
    }

    type line_control_register<'s> = FakeRegister<(), LCR::Register, Safe, Safe>;
    fn line_control_register(&self) -> FakeRegister<(), LCR::Register, Safe, Safe> {
        FakeRegister::new(())
            .on_read(|()| unimplemented!())
            .on_write(|(), _| unimplemented!())
    }

    type modem_control_register<'s> = FakeRegister<(), u8, Safe, Safe>;
    fn modem_control_register(&self) -> FakeRegister<(), u8, Safe, Safe> {
        FakeRegister::new(())
            .on_read(|()| unimplemented!())
            .on_write(|(), _| unimplemented!())
    }

    type line_status_register<'s> = FakeRegister<(), LSR::Register, Safe, NoAccess>;
    fn line_status_register(&self) -> FakeRegister<(), LSR::Register, Safe, NoAccess> {
        FakeRegister::new(()).on_read(|()| unimplemented!())
    }

    type modem_status_register<'s> = FakeRegister<(), u8, Safe, NoAccess>;
    fn modem_status_register(&self) -> FakeRegister<(), u8, Safe, NoAccess> {
        FakeRegister::new(()).on_read(|()| unimplemented!())
    }

    type scratch<'s> = FakeRegister<(), u8, Safe, Safe>;
    fn scratch(&self) -> FakeRegister<(), u8, Safe, Safe> {
        FakeRegister::new(())
            .on_read(|()| unimplemented!())
            .on_write(|(), _| unimplemented!())
    }
}

/// A client that simply stores the value returned in the callback.
struct Client {
    tx_return: Cell<Option<(&'static mut [u8], usize, Result<(), ErrorCode>)>>,
}
impl TransmitClient for Client {
    fn transmitted_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
        rval: Result<(), ErrorCode>,
    ) {
        self.tx_return.set(Some((tx_buffer, tx_len, rval)));
    }
}

#[test]
fn serial_port() {
    let fake = Fake8250::new();
    let driver = SerialPort::new(&fake, FakeDeferredCall::default());
    let client = Client {
        tx_return: Cell::new(None),
    };
    driver.set_transmit_client(&client);
    let (openbox, buffer) = OpenBox::new(Box::new(*b"hi") as _);
    driver.transmit_buffer(buffer, 2).unwrap();
    fake.simulate_tx().unwrap();
    driver.handle_tx_interrupt();
    fake.simulate_tx().unwrap();
    driver.handle_tx_interrupt();
    assert!(fake.simulate_tx().is_none());
    openbox.close(client.tx_return.take().unwrap().0);
}

// -----------------------------------------------------------------------------
// Fake DeferredCall implementation.
// -----------------------------------------------------------------------------

#[derive(Default)]
struct FakeDeferredCall {}

impl DeferredCallInterface for FakeDeferredCall {
    fn register<DC: DeferredCallClient>(&self, _client: &'static DC) {}
    fn set(&self) {}
}

// -----------------------------------------------------------------------------
// OpenBox type.
// -----------------------------------------------------------------------------

// TODO: OpenBox should be moved into its own crate, maybe under `libraries/`?

/// A `Box` that has had its contents taken out of it temporarily.
///
/// When you create an `OpenBox`, you effectively split a `Box` into its
/// contents (`&'static mut T`) and its shell (`OpenBox`). You can use the
/// reference to access the Box's contents. To deallocate the contents, you
/// recombine them with the shell using [`close`] to form a complete `Box`.
// Safety invariant: The contained pointer is the return value of
// Box::into_non_null for a valid Box.
struct OpenBox<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> OpenBox<T> {
    /// Takes the contents out of the given Box.
    pub fn new(b: Box<T>) -> (Self, &'static mut T) {
        let mut allocation = Box::into_non_null(b);
        (
            OpenBox(allocation),
            // Safety:
            // 1. The pointer is aligned, non-null, dereferenceable, and points
            //    to a valid value of type T because it was returned by
            //    converting a valid Box into a NonNull using
            //    Box::into_non_null.
            // 2. The only other reference to this allocation is the returned
            //    OpenBox. OpenBox never uses its reference, so all accesses to
            //    the contents will be through the returned `static mut
            //    reference and values derived from it.
            unsafe { allocation.as_mut() },
        )
    }

    /// Combines this OpenBox with its contents to recreate the original Box.
    pub fn close(self, contents: &'static mut T) -> Box<T> {
        let contents = NonNull::from_mut(contents);
        let matches = ptr::eq(self.0.as_ptr(), contents.as_ptr());
        forget(self);
        assert!(matches, "contents points to wrong or partial allocation");
        // Safety: We've verified that `contents` exactly matches the Box that
        // this OpenBox was originally created with. No other code could have
        // deallocated the pointed-to value. Therefore `contents` points to a
        // `Box`-allocated allocation whose layout matches `T` which we can
        // claim unique ownership over.
        unsafe { Box::from_non_null(contents) }
    }
}

impl<T: ?Sized> Drop for OpenBox<T> {
    fn drop(&mut self) {
        panic!("OpenBox dropped without calling close");
    }
}

#[cfg(test)]
mod openbox_tests {
    use super::*;
    use core::ptr;
    use std::sync::Mutex;

    /// We put SendPtrs into `static` mutexes in some test cases to prevent Miri
    /// from reporting leaked buffers (for tests where OpenBox::close is not
    /// supposed to deallocate the buffer).
    #[expect(dead_code)]
    struct SendPtr(*const u8);
    // Safety: We don't use the contained pointer.
    unsafe impl Send for SendPtr {}

    #[test]
    fn normal_use() {
        let slice: Box<[u8]> = Box::new([1, 2, 3, 4]);
        let (openbox, slice) = OpenBox::new(slice);
        openbox.close(&mut slice[..]);
    }

    #[test]
    #[should_panic(expected = "dropped without calling close")]
    fn not_closed() {
        let slice: Box<[u8]> = Box::new([1, 2, 3, 4]);
        static BUFFER: Mutex<SendPtr> = Mutex::new(SendPtr(ptr::null()));
        *BUFFER.lock().unwrap() = SendPtr((&raw const *slice).cast());
        let (_openbox, _slice) = OpenBox::new(slice);
    }

    #[test]
    #[should_panic(expected = "wrong or partial allocation")]
    fn wrong_len() {
        let slice: Box<[u8]> = Box::new([1, 2, 3, 4]);
        let (openbox, slice) = OpenBox::new(slice);
        static BUFFER: Mutex<SendPtr> = Mutex::new(SendPtr(ptr::null()));
        *BUFFER.lock().unwrap() = SendPtr((&raw const *slice).cast());
        openbox.close(&mut slice[..3]);
    }

    #[test]
    #[should_panic(expected = "wrong or partial allocation")]
    fn wrong_addr() {
        let slice: Box<[u8]> = Box::new([1, 2, 3, 4]);
        let (openbox, slice) = OpenBox::new(slice);
        static BUFFER: Mutex<SendPtr> = Mutex::new(SendPtr(ptr::null()));
        *BUFFER.lock().unwrap() = SendPtr((&raw const *slice).cast());
        openbox.close(&mut slice[1..]);
    }
}
