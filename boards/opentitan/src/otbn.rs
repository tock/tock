//! Components for collections of Hardware Accelerators.
//!
//! Usage
//! -----
//! ```rust
//!     let _mux_otbn = crate::otbn::AccelMuxComponent::new(&peripherals.otbn)
//!         .finalize(otbn_mux_component_helper!(1024));
//!
//!     peripherals.otbn.initialise(
//!         dynamic_deferred_caller
//!             .register(&peripherals.otbn)
//!             .expect("dynamic deferred caller out of slots"),
//!     );
//! ```

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::static_init_half;
use lowrisc::otbn::Otbn;
use lowrisc::virtual_otbn::MuxAccel;

// Setup static space for the objects.
#[macro_export]
macro_rules! otbn_mux_component_helper {
    ($T:expr $(,)?) => {{
        use core::mem::MaybeUninit;
        use lowrisc::virtual_otbn::MuxAccel;
        static mut BUF1: MaybeUninit<MuxAccel<'static, $T>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct AccelMuxComponent<const T: usize> {
    otbn: &'static Otbn<'static>,
}

impl<const T: usize> AccelMuxComponent<T> {
    pub fn new(otbn: &'static Otbn<'static>) -> AccelMuxComponent<T> {
        AccelMuxComponent { otbn }
    }
}

impl<const T: usize> Component for AccelMuxComponent<T> {
    type StaticInput = &'static mut MaybeUninit<MuxAccel<'static, T>>;
    type Output = &'static MuxAccel<'static, T>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_otbn = static_init_half!(s, MuxAccel<'static, T>, MuxAccel::new(self.otbn));

        mux_otbn
    }
}
