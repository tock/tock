//! Component forRadio syscall interface on imix board.
//!
//! This provides one Component, RadioComponent, which implements a
//! userspace syscall interface to a full 802.15.4 stack with a
//! always-on MAC implementation.
//!
//! Usage
//! -----
//! ```rust
//! let (radio_driver, mux_mac) = RadioComponent::new(rf233, PAN_ID, 0x1008).finalize();
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

extern crate kernel;
extern crate nrf52;
extern crate nrf5x;

use capsules;
use capsules::virtual_alarm::VirtualMuxAlarm;

use nrf5x::rtc::Rtc;

use kernel::capabilities;
use kernel::component::Component;
use kernel::{create_capability, static_init};

// Save some deep nesting

pub struct BLEComponent {
    board_kernel: &'static kernel::Kernel,
    radio: &'static nrf52::ble_radio::Radio,
    mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, nrf5x::rtc::Rtc>,
}

impl BLEComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static nrf52::ble_radio::Radio,
        mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, nrf5x::rtc::Rtc>,
    ) -> BLEComponent {
        BLEComponent {
            board_kernel: board_kernel,
            radio: radio,
            mux_alarm: mux_alarm,
        }
    }
}

//static mut RADIO_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// The buffer RF233 packets are received into.
//static mut RADIO_RX_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

impl Component for BLEComponent {
    type Output = (
        &'static capsules::ble_advertising_driver::BLE<
            'static,
            nrf52::ble_radio::Radio,
            VirtualMuxAlarm<'static, Rtc>,
        >,
        &'static capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
    );

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ble_radio_virtual_alarm = static_init!(
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
            capsules::virtual_alarm::VirtualMuxAlarm::new(self.mux_alarm)
        );

        let ble_radio = static_init!(
            capsules::ble_advertising_driver::BLE<
                'static,
                nrf52::ble_radio::Radio,
                VirtualMuxAlarm<'static, Rtc>,
            >,
            capsules::ble_advertising_driver::BLE::new(
                &mut nrf52::ble_radio::RADIO,
                self.board_kernel.create_grant(&grant_cap),
                &mut capsules::ble_advertising_driver::BUF,
                ble_radio_virtual_alarm
            )
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
            &nrf52::ble_radio::RADIO,
            ble_radio,
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
            &nrf52::ble_radio::RADIO,
            ble_radio,
        );
        ble_radio_virtual_alarm.set_client(ble_radio);

        (ble_radio, ble_radio_virtual_alarm)
    }
}
