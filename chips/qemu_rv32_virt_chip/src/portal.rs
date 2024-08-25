use crate::QemuRv32VirtThreadLocal;
use crate::portal_cell::QemuRv32VirtPortalCell;

pub type QemuRv32VirtPortalable = dyn kernel::smp::portal::Portalable<Entrant=Option<core::ptr::NonNull<()>>>;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum QemuRv32VirtPortalKey {
    Uart16550 = 0,
    Counter,
}

impl TryFrom<usize> for QemuRv32VirtPortalKey {
    type Error = ();

    fn try_from(value: usize) -> Result<QemuRv32VirtPortalKey, Self::Error> {
        use QemuRv32VirtPortalKey as P;
        match value {
            const { P::Uart16550 as usize } => Ok(P::Uart16550),
            const { P::Counter as usize } => Ok(P::Counter),
            _ => Err(())
        }
    }
}

impl QemuRv32VirtPortalKey {
    pub fn id(&self) -> usize {
        (unsafe { *<*const _>::from(self).cast::<u8>() }) as usize
    }
}

pub const NUM_PORTALS: usize = core::mem::variant_count::<QemuRv32VirtPortalKey>();

pub static mut PORTALS: QemuRv32VirtThreadLocal<[Option<&'static QemuRv32VirtPortalable>; NUM_PORTALS]> =
     QemuRv32VirtThreadLocal::init([None; NUM_PORTALS]);
