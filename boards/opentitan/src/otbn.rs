//! Components for collections of Hardware Accelerators.
//!
//! Usage
//! -----
//! ```rust
//!     let _mux_otbn = crate::otbn::AccelMuxComponent::new(&peripherals.otbn)
//!         .finalize(otbn_mux_component_helper!());
//!
//!     peripherals.otbn.initialise(
//!         dynamic_deferred_caller
//!             .register(&peripherals.otbn)
//!             .unwrap(), // Unwrap fail = dynamic deferred caller out of slots
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
        static mut BUF1: MaybeUninit<MuxAccel<'static>> = MaybeUninit::uninit();
        &mut BUF1
    }};
}

pub struct AccelMuxComponent {
    otbn: &'static Otbn<'static>,
}

impl AccelMuxComponent {
    pub fn new(otbn: &'static Otbn<'static>) -> AccelMuxComponent {
        AccelMuxComponent { otbn }
    }
}

impl Component for AccelMuxComponent {
    type StaticInput = &'static mut MaybeUninit<MuxAccel<'static>>;
    type Output = &'static MuxAccel<'static>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_otbn = static_init_half!(s, MuxAccel<'static>, MuxAccel::new(self.otbn));

        mux_otbn
    }
}
