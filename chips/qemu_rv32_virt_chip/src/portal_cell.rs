use kernel::threadlocal::{ThreadId, DynThreadId};
use kernel::smp::portal::{PortalCell, Portalable};
use kernel::smp::shared_channel::SharedChannel;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;
use kernel::hil;

use rv32i::csr::CSR;

use core::cell::Cell;
use core::ptr::NonNull;

use crate::channel::{self, QemuRv32VirtChannel, QemuRv32VirtMessage, QemuRv32VirtMessageKind};
use crate::uart::Uart16550;
use crate::chip::QemuRv32VirtClint;
use crate::{plic, interrupts};

pub struct QemuRv32VirtPortalCell<'a, T> {
    portal: PortalCell<'a, T>,
    conjured: Cell<bool>,
}

impl<'a, T> QemuRv32VirtPortalCell<'a, T> {
    pub fn empty(id: usize) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell {
            portal: PortalCell::empty(id),
            conjured: Cell::new(false),
        }
    }

    pub fn new(
        value: &'a mut T, id: usize,
    ) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell {
            portal: PortalCell::new(value, id),
            conjured: Cell::new(false),
        }
    }

    pub fn get_id(&self) -> usize {
        self.portal.get_id()
    }

    pub fn is_none(&self) -> bool {
        self.portal.is_none()
    }

    pub fn is_some(&self) -> bool {
        self.portal.is_some()
    }

    pub fn take(&self) -> Option<&'a mut T> {
        self.portal.take()
    }

    pub fn enter<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.portal.enter(f)
    }

    fn conjure_from_all(&self) -> bool {
        (self.portal.is_none() && !self.conjured.get())
            .then(|| {
                let id = rv32i::support::current_hart_id().get_id();
                let receiver_id = if id == 0 { 1 } else { 0 }; // TODO: notify all kernel threads

                let do_request = move |channel: &mut Option<QemuRv32VirtChannel>| {
                    channel
                        .as_mut()
                        .expect("Uninitialized channel")
                        .write(QemuRv32VirtMessage::prepare(
                            unsafe { DynThreadId::new(receiver_id) },
                            QemuRv32VirtMessageKind::PortalRequest(self.portal.get_id().try_into().unwrap()),
                        ).finish())
                };

                let success = unsafe { channel::with_shared_channel_panic(do_request) };

                success
                    .then(|| {
                        self.conjured.set(true);
                        unsafe { crate::clint::with_clic_panic(|c| {
                            c.set_soft_interrupt(receiver_id)
                        }) };
                    })
                    .is_some()
            })
            .unwrap_or(false)
    }

    fn teleport_with_context<F>(&self, receiver: &dyn ThreadId, save_context: F) -> bool
    where
        F: FnOnce(&mut T),
    {
        self.portal.take().map_or_else(
            // Failure path
            || {
                let do_failure_response = |channel: &mut Option<QemuRv32VirtChannel>| {
                    channel
                        .as_mut()
                        .expect("Uninitialized channel")
                        .write(QemuRv32VirtMessage::prepare(
                            unsafe { DynThreadId::new(receiver.get_id()) },
                            QemuRv32VirtMessageKind::PortalResponseFailure(
                                self.portal.get_id().try_into().unwrap(),
                            )
                        ).finish())
                };

                let success = unsafe {
                    channel::with_shared_channel_panic(do_failure_response)
                };

                success
                    .then(|| {
                        unsafe { crate::clint::with_clic_panic(|c| {
                            c.set_soft_interrupt(receiver.get_id())
                        }) };
                    })
                    .is_some()
            },
            // Success path
            |val| {
                // Save portal context before teleporting
                save_context(val);

                let do_success_response = |channel: &mut Option<QemuRv32VirtChannel>| {
                    channel
                        .as_mut()
                        .expect("Uninitialized channel")
                        .write(QemuRv32VirtMessage::prepare(
                            unsafe { DynThreadId::new(receiver.get_id()) },
                            QemuRv32VirtMessageKind::PortalResponseSuccess(
                                self.portal.get_id().try_into().unwrap(),
                                NonNull::new(val).unwrap().cast::<()>(),
                            )
                        ).finish())
                };

                let success = unsafe { channel::with_shared_channel_panic(do_success_response) };

                if success {
                    unsafe { crate::clint::with_clic_panic(|c| {
                        c.set_soft_interrupt(receiver.get_id())
                    }) };
                } else {
                    self.portal.replace(val);
                }

                success
            }
        )
    }

    fn link_with_context<F>(&self, entrant: NonNull<()>, restore_context: F) -> Option<()>
    where
        F: FnOnce(&mut T),
    {
        let entrant = unsafe { entrant.cast::<T>().as_mut() };

        self.portal.replace(entrant).then(|| {
            self.conjured.set(false);
            self.enter(restore_context);
        })
    }
}

trait ContextFree {}
impl ContextFree for usize {}

impl<'a, T: ContextFree> Portalable for QemuRv32VirtPortalCell<'a, T> {
    type Entrant = Option<NonNull<()>>;

    fn conjure(&self) {
        self.conjure_from_all();
    }

    fn teleport(&self, dst: &dyn ThreadId) -> bool {
        self.teleport_with_context(dst, |_| ())
    }

    fn link(&self, entrant: Self::Entrant) -> Option<()> {
        entrant.map_or_else(
            || Some(self.conjured.set(false)),
            |e| self.link_with_context(e, |_| ())
        )
    }
}

impl<'a> Portalable for QemuRv32VirtPortalCell<'a, Uart16550> {
    type Entrant = Option<NonNull<()>>;

    fn conjure(&self) {
        self.conjure_from_all();
    }

    fn teleport(&self, dst: &dyn ThreadId) -> bool {
        let hart_id = CSR.mhartid.extract().get();
        self.teleport_with_context(dst, move |uart| {
            uart.save_context();
            unsafe {
                plic::with_plic_panic(|plic| {
                    plic.disable(hart_id * 2,
                                 (interrupts::UART0 as u32).try_into().unwrap());
                });
            }
        })
    }

    fn link(&self, entrant: Self::Entrant) -> Option<()> {
        entrant.map_or_else(
            || Some(self.conjured.set(false)),
            |e| self.link_with_context(e, move |uart| {
                // Restore UART context
                uart.restore_context();
                // Re-enable uart interrupts
                unsafe {
                    plic::with_plic_panic(|plic| {
                        plic.enable(rv32i::support::current_hart_id().get_id() * 2,
                                    (interrupts::UART0 as u32).try_into().unwrap());
                    });
                }
                // Continue from the last transmit in case of missing interrupts
                let _ = uart.try_transmit_continue();
            })
        )
    }
}


impl hil::uart::Configure for QemuRv32VirtPortalCell<'_, Uart16550> {
    fn configure(&self, params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        self.enter(|inner| inner.configure(params))
            .unwrap_or(Ok(())) // Optimistically return Ok if not owning the type
    }
}

impl hil::uart::Transmit<'static> for QemuRv32VirtPortalCell<'static, Uart16550> {
    fn set_transmit_client(&self, client: &'static dyn hil::uart::TransmitClient) {
        Uart16550::set_transmit_client(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.is_some() {
            self.enter(|inner| {
                inner.transmit_buffer(tx_data, tx_len)
            }).unwrap_or_else(|| unreachable!())
        } else {
            self.conjure();
            Err((ErrorCode::BUSY, tx_data))
        }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        self.enter(|inner| inner.transmit_abort())
            .unwrap_or_else(|| {
                self.conjure();
                Err(ErrorCode::BUSY)
            })
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        self.enter(|inner| inner.transmit_word(word))
            .unwrap_or_else(|| {
                self.conjure();
                Err(ErrorCode::BUSY)
            })
    }
}

impl hil::uart::Receive<'static> for QemuRv32VirtPortalCell<'static, Uart16550> {
    fn set_receive_client(&self, client: &'static dyn hil::uart::ReceiveClient) {
        Uart16550::set_receive_client(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if self.is_some() {
            self.enter(|inner| inner.receive_buffer(rx_buffer, rx_len))
                .unwrap_or_else(|| unreachable!())
        } else {
            self.conjure();
            Err((ErrorCode::BUSY, rx_buffer))
        }
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        self.enter(|inner| inner.receive_abort())
            .unwrap_or_else(|| {
                self.conjure();
                Err(ErrorCode::BUSY)
            })
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        self.enter(|inner| inner.receive_word())
            .unwrap_or_else(|| {
                self.conjure();
                Err(ErrorCode::BUSY)
            })
    }
}
