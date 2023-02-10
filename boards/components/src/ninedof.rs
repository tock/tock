//! Component for 9DOF
//!
//! Usage
//! -----
//!
//! ```rust
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel)
//!     .finalize(components::ninedof_component_static!(driver1, driver2, ...));
//! ```

use core::mem::MaybeUninit;
use extra_capsules::ninedof::NineDof;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;

#[macro_export]
macro_rules! ninedof_component_static {
    ($($P:expr),+ $(,)?) => {{
        use kernel::count_expressions;

        const NUM_DRIVERS: usize = count_expressions!($($P),+);

        let drivers = kernel::static_init!(
            [&'static dyn kernel::hil::sensors::NineDof; NUM_DRIVERS],
            [
                $($P,)*
            ]
        );
        let ninedof = kernel::static_buf!(extra_capsules::ninedof::NineDof<'static>);
        (ninedof, drivers)
    };};
}

pub struct NineDofComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl NineDofComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> NineDofComponent {
        NineDofComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
        }
    }
}

impl Component for NineDofComponent {
    type StaticInput = (
        &'static mut MaybeUninit<NineDof<'static>>,
        &'static [&'static dyn kernel::hil::sensors::NineDof<'static>],
    );
    type Output = &'static extra_capsules::ninedof::NineDof<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_ninedof = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let ninedof = static_buffer.0.write(extra_capsules::ninedof::NineDof::new(
            static_buffer.1,
            grant_ninedof,
        ));

        for driver in static_buffer.1 {
            kernel::hil::sensors::NineDof::set_client(*driver, ninedof);
        }

        ninedof
    }
}
