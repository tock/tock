use capsules::adc::AdcVirtualized;
use capsules::virtual_adc::{AdcDevice, MuxAdc};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::adc;
use kernel::static_init_half;

#[macro_export]
macro_rules! adc_mux_component_helper {
    ($A:ty $(,)?) => {{
        use capsules::virtual_adc::MuxAdc;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<MuxAdc<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

#[macro_export]
macro_rules! adc_component_helper {
    ($A:ty $(,)?) => {{
        use capsules::virtual_adc::AdcDevice;
        use core::mem::MaybeUninit;
        static mut BUF: MaybeUninit<AdcDevice<'static, $A>> = MaybeUninit::uninit();
        &mut BUF
    };};
}

#[macro_export]
macro_rules! adc_syscall_component_helper {
    ($($P:expr),+ $(,)?) => {{
        use capsules::adc::AdcVirtualized;
        use core::mem::MaybeUninit;
        use kernel::hil;
        use kernel::count_expressions;
        use kernel::static_init;
        const NUM_DRIVERS: usize = count_expressions!($($P),+);

        let drivers = static_init!(
            [&'static dyn kernel::hil::adc::AdcChannel; NUM_DRIVERS],
            [
                $($P,)*
            ]
        );
        static mut BUF: MaybeUninit<AdcVirtualized<'static>> =
            MaybeUninit::uninit();
        (&mut BUF, drivers)
    };};
}

pub struct AdcMuxComponent<A: 'static + adc::Adc> {
    adc: &'static A,
}

pub struct AdcComponent<A: 'static + adc::Adc> {
    adc_mux: &'static MuxAdc<'static, A>,
    channel: A::Channel,
}

impl<A: 'static + adc::Adc> AdcMuxComponent<A> {
    pub fn new(adc: &'static A) -> Self {
        AdcMuxComponent { adc: adc }
    }
}

pub struct AdcVirtualComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
}

impl<A: 'static + adc::Adc> Component for AdcMuxComponent<A> {
    type StaticInput = &'static mut MaybeUninit<MuxAdc<'static, A>>;
    type Output = &'static MuxAdc<'static, A>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_mux = static_init_half!(static_buffer, MuxAdc<'static, A>, MuxAdc::new(self.adc));

        self.adc.set_client(adc_mux);

        adc_mux
    }
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

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let adc_device = static_init_half!(
            static_buffer,
            AdcDevice<'static, A>,
            AdcDevice::new(self.adc_mux, self.channel)
        );

        adc_device.add_to_mux();

        adc_device
    }
}

impl AdcVirtualComponent {
    pub fn new(board_kernel: &'static kernel::Kernel, driver_num: u32) -> AdcVirtualComponent {
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
    type Output = &'static capsules::adc::AdcVirtualized<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);
        let grant_adc = self.board_kernel.create_grant(self.driver_num, &grant_cap);

        let adc = static_init_half!(
            static_buffer.0,
            capsules::adc::AdcVirtualized<'static>,
            capsules::adc::AdcVirtualized::new(static_buffer.1, grant_adc)
        );

        for driver in static_buffer.1 {
            kernel::hil::adc::AdcChannel::set_client(*driver, adc);
        }

        adc
    }
}
