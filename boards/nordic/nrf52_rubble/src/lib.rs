//! Component for Rubble radio on nRF52 based platforms.
//!
//! Usage
//! -----
//! ```rust
//! let ble_radio =
//!     nrf52_rubble::RubbleComponent::new(board_kernel, &nrf52840::ble_radio::RADIO, mux_alarm)
//!        .finalize(());
//! ```
#![no_std]

use capsules::virtual_alarm::VirtualMuxAlarm;

use nrf52::rtc::Rtc;

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::ble_advertising::BleAdvertisementDriver;
use kernel::{create_capability, static_init};

/// Send buffer
pub static mut TX_BUF: tock_rubble::PacketBuffer = [0; tock_rubble::MIN_PDU_BUF];

pub type Nrf52RubbleImplementation<'a, A> =
    tock_rubble::Implementation<'a, A, nrf52::ble_radio::Radio<'a>>;

// Save some deep nesting
type BleCapsule = capsules::rubble::BLE<
    'static,
    VirtualMuxAlarm<'static, Rtc<'static>>,
    Nrf52RubbleImplementation<'static, VirtualMuxAlarm<'static, Rtc<'static>>>,
>;

pub struct RubbleComponent {
    board_kernel: &'static kernel::Kernel,
    radio: &'static nrf52::ble_radio::Radio<'static>,
    mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
}

impl RubbleComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static nrf52::ble_radio::Radio,
        mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc>,
    ) -> RubbleComponent {
        RubbleComponent {
            board_kernel,
            radio,
            mux_alarm,
        }
    }
}

impl Component for RubbleComponent {
    type StaticInput = ();
    type Output = &'static BleCapsule;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ble_radio_virtual_alarm = static_init!(
            capsules::virtual_alarm::VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
            capsules::virtual_alarm::VirtualMuxAlarm::new(self.mux_alarm)
        );

        let radio = tock_rubble::BleRadioWrapper::new(self.radio, &mut TX_BUF);

        let ble_radio = static_init!(
            BleCapsule,
            BleCapsule::new(
                self.board_kernel.create_grant(&grant_cap),
                radio,
                ble_radio_virtual_alarm
            )
        );
        hil::time::Alarm::set_client(ble_radio_virtual_alarm, ble_radio);
        self.radio.set_receive_client(ble_radio);
        self.radio.set_transmit_client(ble_radio);

        ble_radio
    }
}
