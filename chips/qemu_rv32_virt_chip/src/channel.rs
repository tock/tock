//! Inter-processor communication channel.

use core::cell::Cell;

use kernel::deferred_call::{DeferredCallThread, DeferredCallClient};
use kernel::threadlocal::{ThreadLocal, ThreadLocalAccess, DynThreadId};
use kernel::utilities::registers::interfaces::Readable;

use rv32i::csr::CSR;

use crate::MAX_THREADS;

type Buffer = [u8; BUFFER_SIZE];

pub const BUFFER_SIZE: usize = 100;
pub static mut SHARED_CHANNEL_BUFFER: Buffer = [0; BUFFER_SIZE];
pub static mut CHANNEL_BUFFER: ThreadLocal<MAX_THREADS, Buffer> = ThreadLocal::init([0; BUFFER_SIZE]);

enum Message<'a> {
    Request(&'a [u8]),
    Response(&'a [u8]),
}


#[derive(Copy, Clone)]
enum QemuRv32VirtChannelState {
    Init,
    Process,
    End,
}

pub struct QemuRv32VirtChannel {
    state: Cell<QemuRv32VirtChannelState>,
}

impl QemuRv32VirtChannel {
    pub const fn new() -> Self {
        QemuRv32VirtChannel {
            state: Cell::new(QemuRv32VirtChannelState::Init),
        }
    }

    pub fn service(&self) {
        use QemuRv32VirtChannelState as S;

        match self.state.get() {
            S::Init => {
                let hart_id = CSR.mhartid.extract().get();
                let closure = |buf: &mut Buffer| -> usize {
                    buf.iter().fold(0, |acc, x| acc + *x as usize)
                };
                let res = unsafe {
                    CHANNEL_BUFFER.get_mut(DynThreadId::new(hart_id))
                        .expect("This hart does not have access to the QemuRv32VirtChannel")
                        .enter_nonreentrant(closure)
                };
                self.flush_local_buffer();
                unsafe {
                    crate::chip::MACHINE_SOFT_FIRED_COUNT.fetch_add(res, core::sync::atomic::Ordering::Relaxed);
                }
                self.state.replace(S::End);
            }
            S::Process => todo!(),
            S::End => {
                // TODO: Safety
                DeferredCallThread::unset();
                self.state.replace(S::Init);
            }
        }
    }

    fn flush_local_buffer(&self) {
        let hart_id = CSR.mhartid.extract().get();
        unsafe {
            CHANNEL_BUFFER.get_mut(DynThreadId::new(hart_id))
                .expect("This hart does not have access to the QemuRv32VertChannel")
                .enter_nonreentrant(|buf: &mut Buffer| {
                    *buf = core::mem::zeroed()
                })
        };
    }
}


impl DeferredCallClient for QemuRv32VirtChannel {
    fn handle_deferred_call(&self) {
        self.service()
    }

    fn register(&'static self) {
        DeferredCallThread::register(self)
    }
}

