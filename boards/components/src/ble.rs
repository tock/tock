//! Component for creating a ble_advertising_driver.
//!
//! Usage
//! -----
//! ```rust
//! let ble_radio = BLEComponent::new(board_kernel, &nrf52::ble_radio::RADIO, mux_alarm).finalize();
//! ```

use core::mem::MaybeUninit;
use core_capsules;
use core_capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::ble_advertising::BleConfig;
use kernel::hil::time::Alarm;

#[macro_export]
macro_rules! ble_component_static {
    ($A:ty, $B:ty $(,)?) => {{
        let alarm = kernel::static_buf!(core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>);
        let ble = kernel::static_buf!(
            extra_capsules::ble_advertising_driver::BLE<
                'static,
                $B,
                core_capsules::virtual_alarm::VirtualMuxAlarm<'static, $A>,
            >
        );
        let buffer =
            kernel::static_buf!([u8; extra_capsules::ble_advertising_driver::PACKET_LENGTH]);
        (alarm, ble, buffer)
    }};
}

pub struct BLEComponent<
    A: kernel::hil::time::Alarm<'static> + 'static,
    B: kernel::hil::ble_advertising::BleAdvertisementDriver<'static> + BleConfig + 'static,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    radio: &'static B,
    mux_alarm: &'static core_capsules::virtual_alarm::MuxAlarm<'static, A>,
}

impl<
        A: kernel::hil::time::Alarm<'static> + 'static,
        B: kernel::hil::ble_advertising::BleAdvertisementDriver<'static> + BleConfig + 'static,
    > BLEComponent<A, B>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        radio: &'static B,
        mux_alarm: &'static core_capsules::virtual_alarm::MuxAlarm<'static, A>,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            radio,
            mux_alarm,
        }
    }
}

impl<
        A: kernel::hil::time::Alarm<'static> + 'static,
        B: kernel::hil::ble_advertising::BleAdvertisementDriver<'static> + BleConfig + 'static,
    > Component for BLEComponent<A, B>
{
    type StaticInput = (
        &'static mut MaybeUninit<core_capsules::virtual_alarm::VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            extra_capsules::ble_advertising_driver::BLE<'static, B, VirtualMuxAlarm<'static, A>>,
        >,
        &'static mut MaybeUninit<[u8; extra_capsules::ble_advertising_driver::PACKET_LENGTH]>,
    );
    type Output = &'static extra_capsules::ble_advertising_driver::BLE<
        'static,
        B,
        VirtualMuxAlarm<'static, A>,
    >;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ble_radio_virtual_alarm = s.0.write(
            core_capsules::virtual_alarm::VirtualMuxAlarm::new(self.mux_alarm),
        );
        ble_radio_virtual_alarm.setup();
        let buffer =
            s.2.write([0; extra_capsules::ble_advertising_driver::PACKET_LENGTH]);

        let ble_radio = s.1.write(extra_capsules::ble_advertising_driver::BLE::new(
            self.radio,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
            buffer,
            ble_radio_virtual_alarm,
        ));
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
            self.radio, ble_radio,
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
            self.radio, ble_radio,
        );
        ble_radio_virtual_alarm.set_alarm_client(ble_radio);

        ble_radio
    }
}
