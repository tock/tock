//! Components for using ADC capsules.

use core::mem::MaybeUninit;
use core_capsules::adc::AdcDedicated;
use core_capsules::adc::AdcVirtualized;
use core_capsules::virtual_adc::{AdcDevice, MuxAdc};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::adc;

#[macro_export]
macro_rules! adc_mux_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_adc::MuxAdc<'static, $A>)
    };};
}

#[macro_export]
macro_rules! adc_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_adc::AdcDevice<'static, $A>)
    };};
}

#[macro_export]
macro_rules! adc_syscall_component_helper {
    ($($P:expr),+ $(,)?) => {{
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_DRIVERS: usize = count_expressions!($($P),+);

        let drivers = static_init!(
            [&'static dyn kernel::hil::adc::AdcChannel; NUM_DRIVERS],
            [
                $($P,)*
            ]
        );
        let adc_virtualized = kernel::static_buf!(core_capsules::adc::AdcVirtualized<'static>);
        (adc_virtualized, drivers)
    };};
}

#[macro_export]
macro_rules! adc_dedicated_component_static {
    ($A:ty $(,)?) => {{
        let adc = kernel::static_buf!(core_capsules::adc::AdcDedicated<'static, $A>);
        let buffer1 = kernel::static_buf!([u16; core_capsules::adc::BUF_LEN]);
        let buffer2 = kernel::static_buf!([u16; core_capsules::adc::BUF_LEN]);
        let buffer3 = kernel::static_buf!([u16; core_capsules::adc::BUF_LEN]);

        (adc, buffer1, buffer2, buffer3)
    };};
}

pub struct AdcMuxComponent<A: 'static + adc::Adc> {
    adc: &'static A,
}

impl<A: 'static + adc::Adc> AdcMuxComponent<A> {
    pub fn new(adc: &'static A) -> Self {
        AdcMuxComponent { adc: adc }
    }
}

impl<A: 'static + adc::Adc> Component for AdcMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<MuxAdc<'static, A>>;
    type Output = &'static MuxAdc<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_mux = static_buffer.write(MuxAdc::new(self.adc));

        self.adc.set_client(adc_mux);

        adc_mux
    }
}

pub struct AdcComponent<A: 'static + adc::Adc> {
    adc_mux: &'static MuxAdc<'static, A>,
    channel: A::Channel,
}

impl<A: 'static + adc::Adc> AdcComponent<A> {
    pub fn new(mux: &'static MuxAdc<'static, A>, channel: A::Channel) -> Self {
        AdcComponent {
            adc_mux: mux,
            channel: channel,
        }
    }
}

impl<A: 'static + adc::Adc> Component for AdcComponent<A> {
    type StaticInput = &'static mut MaybeUninit<AdcDevice<'static, A>>;
    type Output = &'static AdcDevice<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_device = static_buffer.write(AdcDevice::new(self.adc_mux, self.channel));

        adc_device.add_to_mux();

        adc_device
    }
}

pub struct AdcVirtualComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl AdcVirtualComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: usize) -> AdcVirtualComponent {
        AdcVirtualComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
        }
    }
}

impl Component for AdcVirtualComponent {
    type StaticInput = (
        &'static mut MaybeUninit<AdcVirtualized<'static>>,
        &'static [&'static dyn kernel::hil::adc::AdcChannel],
    );
    type Output = &'static core_capsules::adc::AdcVirtualized<'static>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_adc = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let adc = static_buffer
            .0
            .write(core_capsules::adc::AdcVirtualized::new(
                static_buffer.1,
                grant_adc,
            ));

        for driver in static_buffer.1 {
            kernel::hil::adc::AdcChannel::set_client(*driver, adc);
        }

        adc
    }
}

pub struct AdcDedicatedComponent<
    A: kernel::hil::adc::Adc + kernel::hil::adc::AdcHighSpeed + 'static,
> {
    adc: &'static A,
    channels: &'static [A::Channel],
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
}

impl<A: kernel::hil::adc::Adc + kernel::hil::adc::AdcHighSpeed + 'static> AdcDedicatedComponent<A> {
    pub fn new(
        adc: &'static A,
        channels: &'static [A::Channel],
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
    ) -> AdcDedicatedComponent<A> {
        AdcDedicatedComponent {
            adc,
            channels,
            board_kernel,
            driver_num,
        }
    }
}

impl<A: kernel::hil::adc::Adc + kernel::hil::adc::AdcHighSpeed + 'static> Component
    for AdcDedicatedComponent<A>
{
    type StaticInput = (
        &'static mut MaybeUninit<AdcDedicated<'static, A>>,
        &'static mut MaybeUninit<[u16; core_capsules::adc::BUF_LEN]>,
        &'static mut MaybeUninit<[u16; core_capsules::adc::BUF_LEN]>,
        &'static mut MaybeUninit<[u16; core_capsules::adc::BUF_LEN]>,
    );
    type Output = &'static AdcDedicated<'static, A>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let buffer1 = s.1.write([0; core_capsules::adc::BUF_LEN]);
        let buffer2 = s.2.write([0; core_capsules::adc::BUF_LEN]);
        let buffer3 = s.3.write([0; core_capsules::adc::BUF_LEN]);

        let adc = s.0.write(AdcDedicated::new(
            &self.adc,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            self.channels,
            buffer1,
            buffer2,
            buffer3,
        ));
        self.adc.set_client(adc);
        self.adc.set_highspeed_client(adc);

        adc
    }
}
