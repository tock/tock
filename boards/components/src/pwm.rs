use capsules::pwm::Pwm;
use capsules::virtual_pwm::{MuxPwm, PwmPinUser};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::pwm;

#[macro_export]
macro_rules! pwm_mux_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(capsules::virtual_pwm::MuxPwm<'static, $A>)
    };};
}

#[macro_export]
macro_rules! pwm_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(capsules::virtual_pwm::PwmPinUser<'static, $A>)
    };};
}

#[macro_export]
macro_rules! pwm_syscall_component_helper {
    ($($P:expr),+ $(,)?) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_DRIVERS: usize = count_expressions!($($P),+);

        let drivers = static_init!(
            [&'static dyn kernel::hil::pwm::PwmPin; NUM_DRIVERS],
            [
                $($P,)*
            ]
        );
        let pwm = kernel::static_buf!(capsules::pwm::Pwm<'static, NUM_DRIVERS>);
        (pwm, drivers)
    };};
}

pub struct PwmMuxComponent<A: 'static + pwm::Pwm> {
    pwm: &'static A,
}

impl<A: 'static + pwm::Pwm> PwmMuxComponent<A> {
    pub fn new(pwm: &'static A) -> Self {
        PwmMuxComponent { pwm: pwm }
    }
}

impl<A: 'static + pwm::Pwm> Component for PwmMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<MuxPwm<'static, A>>;
    type Output = &'static MuxPwm<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let pwm_mux = static_buffer.write(MuxPwm::new(self.pwm));

        pwm_mux
    }
}

pub struct PwmPinComponent<A: 'static + pwm::Pwm> {
    pwm_mux: &'static MuxPwm<'static, A>,
    channel: A::Pin,
}

impl<A: 'static + pwm::Pwm> PwmPinComponent<A> {
    pub fn new(mux: &'static MuxPwm<'static, A>, channel: A::Pin) -> Self {
        PwmPinComponent {
            pwm_mux: mux,
            channel: channel,
        }
    }
}

impl<A: 'static + pwm::Pwm> Component for PwmPinComponent<A> {
    type StaticInput = &'static mut MaybeUninit<PwmPinUser<'static, A>>;
    type Output = &'static PwmPinUser<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let pwm_pin = static_buffer.write(PwmPinUser::new(self.pwm_mux, self.channel));

        pwm_pin.add_to_mux();

        pwm_pin
    }
}

pub struct PwmVirtualComponent<const NUM_PINS: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<const NUM_PINS: usize> PwmVirtualComponent<NUM_PINS> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> PwmVirtualComponent<NUM_PINS> {
        PwmVirtualComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
        }
    }
}

impl<const NUM_PINS: usize> Component for PwmVirtualComponent<NUM_PINS> {
    type StaticInput = (
        &'static mut MaybeUninit<Pwm<'static, NUM_PINS>>,
        &'static [&'static dyn kernel::hil::pwm::PwmPin; NUM_PINS],
    );
    type Output = &'static capsules::pwm::Pwm<'static, NUM_PINS>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_adc = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let pwm = static_buffer
            .0
            .write(capsules::pwm::Pwm::new(static_buffer.1, grant_adc));

        pwm
    }
}
