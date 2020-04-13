//! Components for NineDof
//!
//! Provides two components:
//! 	1. NineDofComponent used ofr the ninedof driver
//! 	2. NineDofDriverComponent
//!
//! Usage
//! -----
//! NineDof
//!
//! ```rust
//! let ninedof = components::ninedof::NineDofComponent::new(board_kernel).finalize(());
//! ```
//!
//!
//! NineDof Driver
//!
//! components::ninedof::NineDofDriverComponent::new(ninedof, l3gd20)
//!     .finalize(components::ninedof_driver_helper!());
//! ```

use capsules::ninedof::NineDofNode;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::{static_init, static_init_half};

#[macro_export]
macro_rules! ninedof_driver_helper {
    () => {{
        use capsules::ninedof::{NineDof, NineDofNode};
        use core::mem::MaybeUninit;
        use kernel::hil;
        static mut BUF: MaybeUninit<NineDofNode<'static, &'static dyn hil::sensors::NineDof>> =
            MaybeUninit::uninit();
        &mut BUF
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
    type StaticInput = ();
    type Output = &'static capsules::ninedof::NineDof<'static>;

    unsafe fn finalize(self, _static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_ninedof = self.board_kernel.create_grant(&grant_cap);

        let ninedof = static_init!(
            capsules::ninedof::NineDof<'static>,
            capsules::ninedof::NineDof::new(grant_ninedof)
        );

        ninedof
    }
}

pub struct NineDofDriverComponent {
    driver: &'static dyn hil::sensors::NineDof,
    ninedof: &'static capsules::ninedof::NineDof<'static>,
}

impl NineDofDriverComponent {
    pub fn new(
        ninedof: &'static capsules::ninedof::NineDof<'static>,
        driver: &'static dyn hil::sensors::NineDof,
    ) -> NineDofDriverComponent {
        NineDofDriverComponent {
            driver: driver,
            ninedof: ninedof,
        }
    }
}

impl Component for NineDofDriverComponent {
    type StaticInput =
        &'static mut MaybeUninit<NineDofNode<'static, &'static dyn hil::sensors::NineDof>>;
    type Output =
        &'static capsules::ninedof::NineDofNode<'static, &'static dyn hil::sensors::NineDof>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let ninedof_driver = static_init_half!(
            static_buffer,
            NineDofNode<'static, &'static dyn hil::sensors::NineDof>,
            NineDofNode::new(self.driver)
        );

        self.ninedof.add_driver(ninedof_driver);
        hil::sensors::NineDof::set_client(self.driver, self.ninedof);

        ninedof_driver
    }
}
