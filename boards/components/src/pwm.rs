use core::mem::MaybeUninit;
use core_capsules::virtual_pwm::{MuxPwm, PwmPinUser};
use extra_capsules::pwm::Pwm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::pwm;

#[macro_export]
macro_rules! pwm_mux_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_pwm::MuxPwm<'static, $A>)
    };};
}

#[macro_export]
macro_rules! pwm_pin_user_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_pwm::PwmPinUser<'static, $A>)
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
        let pwm = kernel::static_buf!(extra_capsules::pwm::Pwm<'static, NUM_DRIVERS>);
        (pwm, drivers)
    };};
}

pub struct PwmMuxComponent<P: 'static + pwm::Pwm> {
    pwm: &'static P,
}

impl<P: 'static + pwm::Pwm> PwmMuxComponent<P> {
    pub fn new(pwm: &'static P) -> Self {
        PwmMuxComponent { pwm: pwm }
    }
}

impl<P: 'static + pwm::Pwm> Component for PwmMuxComponent<P> {
    type StaticInput = &'static mut MaybeUninit<MuxPwm<'static, P>>;
    type Output = &'static MuxPwm<'static, P>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let pwm_mux = static_buffer.write(MuxPwm::new(self.pwm));

        pwm_mux
    }
}

pub struct PwmPinComponent<P: 'static + pwm::Pwm> {
    pwm_mux: &'static MuxPwm<'static, P>,
    channel: P::Pin,
}

impl<P: 'static + pwm::Pwm> PwmPinComponent<P> {
    pub fn new(mux: &'static MuxPwm<'static, P>, channel: P::Pin) -> Self {
        PwmPinComponent {
            pwm_mux: mux,
            channel: channel,
        }
    }
}

impl<P: 'static + pwm::Pwm> Component for PwmPinComponent<P> {
    type StaticInput = &'static mut MaybeUninit<PwmPinUser<'static, P>>;
    type Output = &'static PwmPinUser<'static, P>;

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
    type Output = &'static extra_capsules::pwm::Pwm<'static, NUM_PINS>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_adc = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let pwm = static_buffer
            .0
            .write(extra_capsules::pwm::Pwm::new(static_buffer.1, grant_adc));

        pwm
    }
}
