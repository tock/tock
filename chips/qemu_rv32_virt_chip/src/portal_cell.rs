use kernel::platform::KernelResources;
use kernel::platform::chip::{Chip, ChipAtomic};
use kernel::threadlocal::{ThreadId, DynThreadId};
use kernel::smp::portal::{PortalCell, Portalable};
use kernel::smp::shared_channel::SharedChannel;
use kernel::thread_local_static_access;
use kernel::utilities::registers::interfaces::Readable;

use rv32i::csr::CSR;

use crate::channel::{QemuRv32VirtChannel, QemuRv32VirtMessage, QemuRv32VirtMessageBody};
use crate::uart::Uart16550;
use crate::chip::QemuRv32VirtClint;

pub struct QemuRv32VirtPortalCell<'a, T: ?Sized> {
    portal: PortalCell<'a, T>,
    shared_channel: &'a QemuRv32VirtChannel<'a>,
}

impl<'a, T: ?Sized> QemuRv32VirtPortalCell<'a, T> {
    pub fn empty(
        id: usize, shared_channel: &'a QemuRv32VirtChannel<'a>,
    ) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell {
            portal: PortalCell::empty(id),
            shared_channel,
        }
    }

    pub fn new(
        value: &'a mut T, id: usize, shared_channel: &'a QemuRv32VirtChannel<'a>,
    ) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell {
            portal: PortalCell::new(value, id),
            shared_channel,
        }
    }

    pub fn get_id(&self) -> usize {
        self.portal.get_id()
    }

    pub fn is_none(&self) -> bool {
        self.portal.is_none()
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

            if self.shared_channel
                .write(QemuRv32VirtMessage {
                    src: id,
                    dst: receiver_id,
                    body: QemuRv32VirtMessageBody::PortalRequest(self.portal.get_id()),
                })
            {
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
            if self.shared_channel
                .write(QemuRv32VirtMessage {
                    src: id,
                    dst: dst.get_id(),
                    body: QemuRv32VirtMessageBody::PortalResponse(
                        self.portal.get_id(),
                        val as *mut _ as *const _,
                    ),
                })
            {
                let closure = |c: &mut QemuRv32VirtClint| c.set_soft_interrupt(dst.get_id());
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


use kernel::hil::uart::{Receive, Transmit, Configure, Parameters, TransmitClient, ReceiveClient};
use kernel::ErrorCode;

static mut EMPTY_STRING: [u8; 0] = [0; 0];

impl<T: Configure> Configure for QemuRv32VirtPortalCell<'_, T> {
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.configure(params))
            .unwrap_or(Ok(())) // Optimistically return Ok if not owning the type
    }
}

impl<'a, T: Transmit<'a>> Transmit<'a> for QemuRv32VirtPortalCell<'a, T> {
    fn set_transmit_client(&self, client: &'a dyn TransmitClient) {
        let _ = self.enter(|inner: &mut T| inner.set_transmit_client(client));
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.enter(|inner: &mut T| inner.transmit_buffer(tx_data, tx_len))
            .unwrap_or_else(|| {
                self.conjure();
                todo!();
                Err((ErrorCode::BUSY, unsafe { &mut *core::ptr::addr_of_mut!(EMPTY_STRING) }))
            })
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.transmit_abort())
            .unwrap_or_else(|| {
                self.conjure();
                todo!();
                Err(ErrorCode::BUSY)
            })
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.transmit_word(word))
            .unwrap_or_else(|| {
                self.conjure();
                todo!();
                Err(ErrorCode::BUSY)
            })
    }
}

impl<'a, T: Receive<'a>> Receive<'a> for QemuRv32VirtPortalCell<'a, T> {
    fn set_receive_client(&self, client: &'a dyn ReceiveClient) {
        let _ = self.enter(|inner: &mut T| inner.set_receive_client(client));
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.enter(|inner: &mut T| inner.receive_buffer(rx_buffer, rx_len))
            .unwrap_or_else(|| {
                self.conjure();
                todo!();
                Err((ErrorCode::BUSY, unsafe { &mut *core::ptr::addr_of_mut!(EMPTY_STRING) }))
            })
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.receive_abort())
            .unwrap_or_else(|| {
                self.conjure();
                todo!();
                Err(ErrorCode::BUSY)
            })
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.receive_word())
            .unwrap_or_else(|| {
                self.conjure();
                todo!();
                Err(ErrorCode::BUSY)
            })
    }
}
