//! Component for 9DOF
//!
//! Usage
//! -----
//! NineDof
//!
//! ```rust
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
//!     .finalize(components::ninedof_component_helper!(driver1, driver2, ...));
//! ```

use capsules::ninedof::NineDof;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::static_init_half;

#[macro_export]
macro_rules! ninedof_component_helper {
    ($($P:expr),+ ) => {{
        use capsules::ninedof::NineDof;
        use core::mem::MaybeUninit;
        use kernel::hil;
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_DRIVERS: usize = count_expressions!($($P),+);

        let drivers = static_init!(
            [&'static dyn kernel::hil::sensors::NineDof; NUM_DRIVERS],
            [
                $($P,)*
            ]
        );
        static mut BUF: MaybeUninit<NineDof<'static>> =
            MaybeUninit::uninit();
        (&mut BUF, drivers)
    };};
}

pub struct NineDofComponent {
    board_kernel: &'static kernel::Kernel,
}

impl NineDofComponent {
    pub fn new(board_kernel: &'static kernel::Kernel) -> NineDofComponent {
        NineDofComponent {
            board_kernel: board_kernel,
        }
    }
}

impl Component for NineDofComponent {
    type StaticInput = (
        &'static mut MaybeUninit<NineDof<'static>>,
        &'static [&'static dyn kernel::hil::sensors::NineDof<'static>],
    );
    type Output = &'static capsules::ninedof::NineDof<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_ninedof = self.board_kernel.create_grant(&grant_cap);

        let ninedof = static_init_half!(
            static_buffer.0,
            capsules::ninedof::NineDof<'static>,
            capsules::ninedof::NineDof::new(static_buffer.1, grant_ninedof)
        );

        for driver in static_buffer.1 {
            kernel::hil::sensors::NineDof::set_client(*driver, ninedof);
        }

        ninedof
    }
}
