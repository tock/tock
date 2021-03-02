//! Component for BLE radio on Apollo3 based platforms.
//!
//! Usage
//! -----
//! ```rust
//! let ble_radio = BLEComponent::new(board_kernel, &apollo3::ble::BLE, mux_alarm).finalize();
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::{create_capability, static_init};

/// BLE component for Apollo3 BLE
pub struct BLEComponent {
    board_kernel: &'static kernel::Kernel,
    driver_num: u32,
    radio: &'static apollo3::ble::Ble<'static>,
    mux_alarm:
        &'static capsules::virtual_alarm::MuxAlarm<'static, apollo3::stimer::STimer<'static>>,
}

/// BLE component for Apollo3 BLE
impl BLEComponent {
    /// New instance
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: u32,
        radio: &'static apollo3::ble::Ble,
        mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, apollo3::stimer::STimer>,
    ) -> BLEComponent {
        BLEComponent {
            board_kernel: board_kernel,
            driver_num: driver_num,
            radio: radio,
            mux_alarm: mux_alarm,
        }
    }
}

impl Component for BLEComponent {
    type StaticInput = ();
    type Output = &'static capsules::ble_advertising_driver::BLE<
        'static,
        apollo3::ble::Ble<'static>,
        VirtualMuxAlarm<'static, apollo3::stimer::STimer<'static>>,
    >;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ble_radio_virtual_alarm = static_init!(
            capsules::virtual_alarm::VirtualMuxAlarm<'static, apollo3::stimer::STimer>,
            capsules::virtual_alarm::VirtualMuxAlarm::new(self.mux_alarm)
        );

        let ble_radio = static_init!(
            capsules::ble_advertising_driver::BLE<
                'static,
                apollo3::ble::Ble,
                VirtualMuxAlarm<'static, apollo3::stimer::STimer>,
            >,
            capsules::ble_advertising_driver::BLE::new(
                self.radio,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                &mut capsules::ble_advertising_driver::BUF,
                ble_radio_virtual_alarm
            )
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
            self.radio, ble_radio,
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
            self.radio, ble_radio,
        );
        hil::time::Alarm::set_alarm_client(ble_radio_virtual_alarm, ble_radio);

        ble_radio
    }
}
