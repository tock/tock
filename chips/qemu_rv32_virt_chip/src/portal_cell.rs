use kernel::threadlocal::ThreadId;
use kernel::smp::portal::{PortalCell, Portalable};
use kernel::smp::shared_channel::SharedChannel;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;
use kernel::hil;

use rv32i::csr::CSR;

use core::cell::Cell;

use crate::channel::{self, QemuRv32VirtChannel, QemuRv32VirtMessage, QemuRv32VirtMessageBody};
use crate::uart::Uart16550;
use crate::chip::QemuRv32VirtClint;
use crate::{plic, interrupts};

pub struct QemuRv32VirtPortalCell<'a, T: ?Sized> {
    portal: PortalCell<'a, T>,
    conjured: Cell<bool>,
}

impl<'a, T: ?Sized> QemuRv32VirtPortalCell<'a, T> {
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

    fn conjure_from_all(&self) {
        if self.portal.is_none() {
            let id = CSR.mhartid.extract().get();
            let receiver_id = if id == 0 { 1 } else { 0 }; // TODO: notify all kernel threads

            if !self.conjured.get() {
                let do_request = move |channel: &mut Option<QemuRv32VirtChannel>| {
                    channel
                        .as_mut()
                        .expect("Uninitialized channel")
                        .write(QemuRv32VirtMessage {
                            src: id,
                            dst: receiver_id,
                            body: QemuRv32VirtMessageBody::PortalRequest(self.portal.get_id()),
                        })
                };

                let success = unsafe { channel::with_shared_channel_panic(do_request) };

                if success {
                    self.conjured.set(true);
                }
            }

            if self.conjured.get() {
                let closure = |c: &mut QemuRv32VirtClint| c.set_soft_interrupt(receiver_id);
                unsafe {
                    crate::clint::with_clic_panic(closure);
                }
            }
        }
    }

    fn teleport_with_context<F>(&self, dst: &dyn ThreadId, save_context: F)
    where
        F: FnOnce(&mut T),
    {
        if let Some(val) = self.portal.take() {
            let id = CSR.mhartid.extract().get();
            let dst_id = dst.get_id();

            save_context(val);

            let do_response = |channel: &mut Option<QemuRv32VirtChannel>| {
                channel
                    .as_mut()
                    .expect("Uninitialized channel")
                    .write(QemuRv32VirtMessage {
                        src: id,
                        dst: dst_id,
                        body: QemuRv32VirtMessageBody::PortalResponse(
                            self.portal.get_id(),
                            val as *mut _ as *const _,
                        ),
                    })
            };

            let success = unsafe { channel::with_shared_channel_panic(do_response) };

            if success {
                self.conjured.set(false);
                let closure = move |c: &mut QemuRv32VirtClint| c.set_soft_interrupt(dst_id);
                unsafe {
                    crate::clint::with_clic_panic(closure);
                }
            } else {
                assert!(self.portal.replace(val))
            }
        }
    }
}

trait ContextFree {}
impl ContextFree for usize {}

impl<'a, T: ?Sized + ContextFree> Portalable for QemuRv32VirtPortalCell<'a, T> {
    type Entrant = &'a mut T;

    fn conjure(&self) {
        self.conjure_from_all()
    }

    fn teleport(&self, dst: &dyn ThreadId) {
        self.teleport_with_context(dst, |_| ());
    }

    fn link(&self, entrant: Self::Entrant) -> Option<()> {
        self.portal.replace(entrant).then(|| ())
    }
}

impl<'a> Portalable for QemuRv32VirtPortalCell<'a, Uart16550> {
    type Entrant = &'a mut Uart16550;

    fn conjure(&self) {
        self.conjure_from_all()
    }

    fn teleport(&self, dst: &dyn ThreadId) {
        let hart_id = CSR.mhartid.extract().get();
        self.teleport_with_context(dst, move |uart| {
            uart.save_context();
            unsafe {
                plic::with_plic_panic(|plic| {
                    plic.disable(hart_id * 2,
                                 (interrupts::UART0 as u32).try_into().unwrap());
                });
            }
        });
    }

    fn link(&self, entrant: Self::Entrant) -> Option<()> {
        self.portal.replace(entrant).then(|| ())
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
