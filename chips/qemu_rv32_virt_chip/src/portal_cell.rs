use core::cell::Cell;

use kernel::platform::KernelResources;
use kernel::platform::chip::{Chip, ChipAtomic};
use kernel::threadlocal::{ThreadId, DynThreadId};
use kernel::smp::portal_cell::{PortalCell, Portalable};
use kernel::smp::shared_channel::SharedChannel;

use crate::channel::{QemuRv32VirtChannel, QemuRv32VirtMessage, QemuRv32VirtMessageBody};
use crate::uart::Uart16550;

pub struct QemuRv32VirtPortalCell<'a, T: ?Sized>(PortalCell<'a, T>);

impl<'a, T: ?Sized> QemuRv32VirtPortalCell<'a, T> {
    pub fn empty(id: usize) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell(PortalCell::empty(id))
    }

    pub fn new(value: &'a mut T, id: usize) -> QemuRv32VirtPortalCell<'a, T> {
        QemuRv32VirtPortalCell(PortalCell::new(value, id))
    }

    pub fn get_id(&self) -> usize { self.0.get_tag() }
    pub fn is_none(&self) -> bool { self.0.is_none() }
    pub fn take(&self) -> Option<&'a mut T> { self.0.take() }

    pub fn enter<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.0.enter(f)
    }

    pub unsafe fn replace_none(&self, val: &'a mut T) -> Option<()> {
        self.0.replace_none(val)
    }

}


impl<'a, 'b, KR, C, T: ?Sized> Portalable<KR, C> for QemuRv32VirtPortalCell<'a, T>
where
    KR: KernelResources<C, SharedChannel=QemuRv32VirtChannel<'b>>,
    C: Chip + ChipAtomic,
{
    type Entrant = &'a mut T;

    fn conjure(&self, resources: &KR, chip: &C) {
        // Note: this will try to flood the channel
        if self.is_none() {
            let receiver = if chip.id().get_id() == 0 { 1 } else { 0 };
            if resources.shared_channel()
                .write(QemuRv32VirtMessage {
                    src: chip.id().get_id(),
                    dst: receiver,
                    body: QemuRv32VirtMessageBody::PortalRequest(self.get_id()),
                })
            {
                let receiver_id = unsafe { DynThreadId::new(receiver) };
                chip.notify(&receiver_id);
                // chip.notify_all();
            }
        }
    }

    fn teleport(&self, resources: &KR, chip: &C, dst: &dyn ThreadId) {
        if let Some(val) = self.take() {
            if resources.shared_channel()
                .write(QemuRv32VirtMessage {
                    src: chip.id().get_id(),
                    dst: dst.get_id(),
                    body: QemuRv32VirtMessageBody::PortalResponse(
                        self.get_id(),
                        val as *mut _ as *const _,
                    ),
                })
            {
                chip.notify(dst);
            }
        }
    }

    fn link(&self, entrant: Self::Entrant) -> Option<()> {
        unsafe { self.0.replace_none(entrant) }
    }
}


use kernel::hil::uart::{Receive, Transmit, Configure, Parameters, TransmitClient, ReceiveClient};
use kernel::ErrorCode;

static mut EMPTY_STRING: [u8; 0] = [0; 0];

impl<T: Configure> Configure for QemuRv32VirtPortalCell<'_, T> {
    fn configure(&self, params: Parameters) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.configure(params))
            .unwrap_or(Err(ErrorCode::BUSY))
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
            .unwrap()
            // .unwrap_or(Err((ErrorCode::BUSY, unsafe { &mut *core::ptr::addr_of_mut!(EMPTY_STRING) })))
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.transmit_abort())
            .unwrap()
            // .unwrap_or(Err(ErrorCode::BUSY))
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.transmit_word(word))
            .unwrap()
            // .unwrap_or(Err(ErrorCode::BUSY))
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
            .unwrap()
            // .unwrap_or(Err((ErrorCode::BUSY, unsafe { &mut *core::ptr::addr_of_mut!(EMPTY_STRING) })))
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.receive_abort())
            .unwrap()
            // .unwrap_or(Err(ErrorCode::BUSY))
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        self.enter(|inner: &mut T| inner.receive_word())
            .unwrap()
            // .unwrap_or(Err(ErrorCode::BUSY))
    }
}
