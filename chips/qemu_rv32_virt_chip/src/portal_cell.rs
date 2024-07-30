use kernel::platform::KernelResources;
use kernel::platform::chip::{Chip, ChipAtomic};
use kernel::threadlocal::{ThreadId, DynThreadId};
use kernel::smp::portal::{PortalCell, Portalable};
use kernel::smp::shared_channel::SharedChannel;
use kernel::utilities::registers::interfaces::Readable;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;
use kernel::hil;
use kernel::thread_local_static_access;

use rv32i::csr::CSR;

use crate::channel::{self, QemuRv32VirtChannel, QemuRv32VirtMessage, QemuRv32VirtMessageBody};
use crate::uart::{Uart16550, Uart16550State};
use crate::chip::QemuRv32VirtClint;

pub struct QemuRv32VirtPortalCell<'a, T: ?Sized> {
    portal: PortalCell<'a, T>,
}

impl<'a, T: ?Sized> QemuRv32VirtPortalCell<'a, T> {
    pub fn empty(id: usize) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell {
            portal: PortalCell::empty(id),
        }
    }

    pub fn new(
        value: &'a mut T, id: usize,
    ) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell {
            portal: PortalCell::new(value, id),
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

}


impl<'a, T: ?Sized> Portalable for QemuRv32VirtPortalCell<'a, T> {
    type Entrant = &'a mut T;

    fn conjure(&self) {
        // Note: this will try to flood the channel
        if self.portal.is_none() {
            let id = CSR.mhartid.extract().get();
            // TODO: notify all
            let receiver_id = if id == 0 { 1 } else { 0 };

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
                let closure = |c: &mut QemuRv32VirtClint| c.set_soft_interrupt(receiver_id);
                unsafe {
                    thread_local_static_access!(crate::clint::CLIC, DynThreadId::new(id))
                        .expect("This thread does not have access to CLIC")
                        .enter_nonreentrant(closure);
                }
            }
        }
    }

    fn teleport(&self, dst: &dyn ThreadId) {
        if let Some(val) = self.portal.take() {
            let id = CSR.mhartid.extract().get();
            let dst_id = dst.get_id();

            let do_response = move |channel: &mut Option<QemuRv32VirtChannel>| {
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
                let closure = move |c: &mut QemuRv32VirtClint| c.set_soft_interrupt(dst_id);
                unsafe {
                    thread_local_static_access!(crate::clint::CLIC, DynThreadId::new(id))
                        .expect("This thread does not have access to CLIC")
                        .enter_nonreentrant(closure);
                }
            }
        }
    }

    fn link(&self, entrant: Self::Entrant) -> Option<()> {
        if self.portal.replace(entrant) {
            Some(())
        } else {
            None
        }
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

    fn transmit_hint(&self) {
        if self.is_none() {
            self.conjure();
        }
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
