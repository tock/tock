//! Component for Rubble radio on nRF52 based platforms.
//!
//! Usage
//! -----
//! ```rust
//! let ble_radio = RubbleComponent::new(board_kernel, mux_alarm).finalize();
//! ```

use capsules;
use capsules::virtual_alarm::VirtualMuxAlarm;

use nrf52::rtc::Rtc;

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::{create_capability, static_init};

use rubble::link::{DeviceAddress, RadioCmd, MIN_PDU_BUF};

use rubble_nrf5x::radio::{BleRadio, PacketBuffer};

use nrf52840_hal as hal;

/// Send buffer
pub static mut TX_BUF: PacketBuffer = [0; MIN_PDU_BUF];

/// Recv buffer
pub static mut RX_BUF: PacketBuffer = [0; MIN_PDU_BUF];

pub struct Nrf52BleRadio;

impl hil::rubble::BleRadio for Nrf52BleRadio {
    type Transmitter = rubble_nrf5x::radio::BleRadio;

    fn get_device_address() -> DeviceAddress {
        rubble_nrf5x::utils::get_device_address()
    }

    fn radio_accept_cmd(radio: &mut Self::Transmitter, cmd: RadioCmd) {
        radio.configure_receiver(cmd);
    }
}

type BleCapsule =
    capsules::rubble::BLE<'static, Nrf52BleRadio, VirtualMuxAlarm<'static, Rtc<'static>>>;

// Save some deep nesting

pub struct RubbleComponent {
    board_kernel: &'static kernel::Kernel,
    mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
}

impl RubbleComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        mux_alarm: &'static capsules::virtual_alarm::MuxAlarm<'static, nrf52::rtc::Rtc>,
    ) -> RubbleComponent {
        RubbleComponent {
            board_kernel: board_kernel,
            mux_alarm: mux_alarm,
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

        // TODO: replace this
        let peripherals = hal::target::Peripherals::take().unwrap();

        // Rubble currently requires an RX buffer even though the radio is only used as a TX-only
        // beacon.
        let radio = BleRadio::new(
            peripherals.RADIO,
            &peripherals.FICR,
            &mut TX_BUF,
            &mut RX_BUF,
        );

        let ble_radio = static_init!(
            BleCapsule,
            BleCapsule::new(
                self.board_kernel.create_grant(&grant_cap),
                radio,
                ble_radio_virtual_alarm
            )
        );
        hil::time::Alarm::set_client(ble_radio_virtual_alarm, ble_radio);

        ble_radio
    }
}
