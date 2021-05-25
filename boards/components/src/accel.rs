//! Components for collections of Hardware Accelerators.
//!
//! Usage
//! -----
//! ```rust
//! ```

use capsules;
use capsules::virtual_accel::MuxAccel;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::accel;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! accel_mux_component_helper {
    ($A:ty, $T:expr $(,)?) => {{
        use capsules::virtual_accel::MuxAccel;
        use capsules::virtual_accel::VirtualMuxAccel;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxAccel<'static, $A, $T>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct AccelMuxComponent<A: 'static + accel::Accel<'static, T>, const T: usize> {
    accel: &'static A,
}

impl<A: 'static + accel::Accel<'static, T>, const T: usize> AccelMuxComponent<A, T> {
    pub fn new(accel: &'static A) -> AccelMuxComponent<A, T> {
        AccelMuxComponent { accel }
    }
}

impl<A: 'static + accel::Accel<'static, T>, const T: usize> Component for AccelMuxComponent<A, T> {
    type StaticInput = &'static mut MaybeUninit<MuxAccel<'static, A, T>>;
    type Output = &'static MuxAccel<'static, A, T>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_accel = static_init_half!(s, MuxAccel<'static, A, T>, MuxAccel::new(self.accel));

        mux_accel
    }
}
