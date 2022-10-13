use capsules::can::CanCapsule;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::can;

#[macro_export]
macro_rules! can_component_helper {
    ($C:ty $(,)?) => {{
        kernel::static_buf!(capsules::can::CanCapsule<'static, $C>)
    };};
}

pub struct CanComponent<A: 'static + can::Can> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    can: &'static A,
}

impl<A: 'static + can::Can> CanComponent<A> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        can: &'static A,
    ) -> CanComponent<A> {
        CanComponent {
            board_kernel,
            driver_num,
            can,
        }
    }
}

pub static mut CAN_TX_BUF: [u8; can::STANDARD_CAN_PACKET_SIZE] = [0; can::STANDARD_CAN_PACKET_SIZE];
pub static mut CAN_RX_BUF: [u8; can::STANDARD_CAN_PACKET_SIZE] = [0; can::STANDARD_CAN_PACKET_SIZE];

impl<A: 'static + can::Can> Component for CanComponent<A> {
    type StaticInput = &'static mut MaybeUninit<CanCapsule<'static, A>>;
    type Output = &'static CanCapsule<'static, A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_can = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let can = static_buffer.write(capsules::can::CanCapsule::new(
            self.can,
            grant_can,
            &mut CAN_TX_BUF,
            &mut CAN_RX_BUF,
        ));
        kernel::hil::can::Controller::set_client(self.can, Some(can));
        kernel::hil::can::Transmit::set_client(self.can, Some(can));
        kernel::hil::can::Receive::set_client(self.can, Some(can));

        can
    }
}
