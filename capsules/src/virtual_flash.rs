//! Virtualize writing flash.
//!
//! `MuxFlash` provides shared access to a flash interface from multiple clients
//! in the kernel. For instance, a board may wish to expose the internal MCU
//! flash for multiple uses, like allowing userland apps to write their own
//! flash space, and to provide a "scratch space" as the end of flash for all
//! apps to use. Each of these requires a capsule to support the operation, and
//! must use a `FlashUser` instance to contain the per-user state for the
//! virtualization.
//!
//! Usage
//! -----
//!
//! ```
//! // Create the mux.
//! let mux_flash = static_init!(
//!     capsules::virtual_flash::MuxFlash<'static, sam4l::flashcalw::FLASHCALW>,
//!     capsules::virtual_flash::MuxFlash::new(&sam4l::flashcalw::FLASH_CONTROLLER));
//! hil::flash::HasClient::set_client(&sam4l::flashcalw::FLASH_CONTROLLER, mux_flash);
//!
//! // Everything that then uses the virtualized flash must use one of these.
//! let virtual_flash = static_init!(
//!     capsules::virtual_flash::FlashUser<'static, sam4l::flashcalw::FLASHCALW>,
//!     capsules::virtual_flash::FlashUser::new(mux_flash));
//! ```

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::{List, ListLink, ListNode};
use kernel::hil;
use kernel::ReturnCode;

/// Handle keeping a list of active users of flash hardware and serialize their
/// requests. After each completed request the list is checked to see if there
/// is another flash user with an outstanding read, write, or erase request.
pub struct MuxFlash<'a, F: hil::flash::Flash + 'static> {
    flash: &'a F,
    users: List<'a, FlashUser<'a, F>>,
    inflight: OptionalCell<&'a FlashUser<'a, F>>,
}

impl<F: hil::flash::Flash> hil::flash::Client<F> for MuxFlash<'_, F> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, error: hil::flash::Error) {
        self.inflight.take().map(move |user| {
            user.read_complete(pagebuffer, error);
        });
        self.do_next_op();
    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, error: hil::flash::Error) {
        self.inflight.take().map(move |user| {
            user.write_complete(pagebuffer, error);
        });
        self.do_next_op();
    }

    fn erase_complete(&self, error: hil::flash::Error) {
        self.inflight.take().map(move |user| {
            user.erase_complete(error);
        });
        self.do_next_op();
    }
}

impl<'a, F: hil::flash::Flash> MuxFlash<'a, F> {
    pub const fn new(flash: &'a F) -> MuxFlash<'a, F> {
        MuxFlash {
            flash: flash,
            users: List::new(),
            inflight: OptionalCell::empty(),
        }
    }

    /// Scan the list of users and find the first user that has a pending
    /// request, then issue that request to the flash hardware.
    fn do_next_op(&self) {
        if self.inflight.is_none() {
            let mnode = self
                .users
                .iter()
                .find(|node| node.operation.get() != Op::Idle);
            mnode.map(|node| {
                node.buffer.take().map_or_else(
                    || {
                        // Don't need a buffer for erase.
                        match node.operation.get() {
                            Op::Erase(page_number) => {
                                self.flash.erase_page(page_number);
                            }
                            _ => {}
                        };
                    },
                    |buf| {
                        match node.operation.get() {
                            Op::Write(page_number) => {
                                if let Err((_, buf)) = self.flash.write_page(page_number, buf) {
                                    node.buffer.replace(buf);
                                }
                            }
                            Op::Read(page_number) => {
                                if let Err((_, buf)) = self.flash.read_page(page_number, buf) {
                                    node.buffer.replace(buf);
                                }
                            }
                            Op::Erase(page_number) => {
                                self.flash.erase_page(page_number);
                            }
                            Op::Idle => {} // Can't get here...
                        }
                    },
                );
                node.operation.set(Op::Idle);
                self.inflight.set(node);
            });
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Op {
    Idle,
    Write(usize),
    Read(usize),
    Erase(usize),
}

/// Keep state for each flash user. All uses of the virtualized flash interface
/// need to create one of these to be a user of the flash. The `new()` function
/// handles most of the work, a user only has to pass in a reference to the
/// MuxFlash object.
pub struct FlashUser<'a, F: hil::flash::Flash + 'static> {
    mux: &'a MuxFlash<'a, F>,
    buffer: TakeCell<'static, F::Page>,
    operation: Cell<Op>,
    next: ListLink<'a, FlashUser<'a, F>>,
    client: OptionalCell<&'a dyn hil::flash::Client<FlashUser<'a, F>>>,
}

impl<'a, F: hil::flash::Flash> FlashUser<'a, F> {
    pub const fn new(mux: &'a MuxFlash<'a, F>) -> FlashUser<'a, F> {
        FlashUser {
            mux: mux,
            buffer: TakeCell::empty(),
            operation: Cell::new(Op::Idle),
            next: ListLink::empty(),
            client: OptionalCell::empty(),
        }
    }
}

impl<'a, F: hil::flash::Flash, C: hil::flash::Client<Self>> hil::flash::HasClient<'a, C>
    for FlashUser<'a, F>
{
    fn set_client(&'a self, client: &'a C) {
        self.mux.users.push_head(self);
        self.client.set(client);
    }
}

impl<'a, F: hil::flash::Flash> hil::flash::Client<F> for FlashUser<'a, F> {
    fn read_complete(&self, pagebuffer: &'static mut F::Page, error: hil::flash::Error) {
        self.client.map(move |client| {
            client.read_complete(pagebuffer, error);
        });
    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, error: hil::flash::Error) {
        self.client.map(move |client| {
            client.write_complete(pagebuffer, error);
        });
    }

    fn erase_complete(&self, error: hil::flash::Error) {
        self.client.map(move |client| {
            client.erase_complete(error);
        });
    }
}

impl<'a, F: hil::flash::Flash> ListNode<'a, FlashUser<'a, F>> for FlashUser<'a, F> {
    fn next(&'a self) -> &'a ListLink<'a, FlashUser<'a, F>> {
        &self.next
    }
}

impl<F: hil::flash::Flash> hil::flash::Flash for FlashUser<'_, F> {
    type Page = F::Page;

    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.buffer.replace(buf);
        self.operation.set(Op::Read(page_number));
        self.mux.do_next_op();
        Ok(())
    }

    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ReturnCode, &'static mut Self::Page)> {
        self.buffer.replace(buf);
        self.operation.set(Op::Write(page_number));
        self.mux.do_next_op();
        Ok(())
    }

    fn erase_page(&self, page_number: usize) -> ReturnCode {
        self.operation.set(Op::Erase(page_number));
        self.mux.do_next_op();
        ReturnCode::SUCCESS
    }
}
