use crate::QemuRv32VirtThreadLocal;

pub const NUM_PORTALS: usize = core::mem::variant_count::<QemuRv32VirtPortal>() - 1; // ignore None

pub static mut PORTALS: QemuRv32VirtThreadLocal<[QemuRv32VirtPortal; NUM_PORTALS]> =
     QemuRv32VirtThreadLocal::init([QemuRv32VirtPortal::None; NUM_PORTALS]);

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum QemuRv32VirtPortal {
    Uart16550(*const ()) = 0,
    Counter(*const ()),
    None
}

impl QemuRv32VirtPortal {
    // fn validate(&self) -> bool {
    //     let id = unsafe { *<*const _>::from(self).cast::<u8>() } as usize;
    //     match self {
    //         Uart16550(x) => id == x,
    //         None => true,
    //     }
    // }

    pub fn id(&self) -> usize {
        (unsafe { *<*const _>::from(self).cast::<u8>() }) as usize
    }
}
