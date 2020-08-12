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

use rubble::{
    link::{advertising, data, AddressKind, RadioCmd, Transmitter, MIN_PDU_BUF},
    phy::{AdvertisingChannel, DataChannel},
};

use rubble_nrf5x::radio::{BleRadio, PacketBuffer};

use nrf52840_hal as hal;

/// Send buffer
pub static mut TX_BUF: PacketBuffer = [0; MIN_PDU_BUF];

/// Recv buffer
pub static mut RX_BUF: PacketBuffer = [0; MIN_PDU_BUF];

pub struct Nrf52BleRadio;

fn hil_to_rubble_advertising_channel(
    hil_channel: hil::rubble::AdvertisingChannel,
) -> AdvertisingChannel {
    // TODO: Get rubble's AdvertisingChannel to support direct construction.
    // LLVM should be able to figure out this is a no-op, but it's still a
    // hack.
    let mut rubble_channel = AdvertisingChannel::first();
    for _ in 37..hil_channel.channel() {
        rubble_channel = rubble_channel.cycle();
    }
    rubble_channel
}

impl hil::rubble::BleHardware for Nrf52BleRadio {
    type Transmitter = TransmitterWrapper;

    fn get_device_address() -> hil::rubble::DeviceAddress {
        let address = rubble_nrf5x::utils::get_device_address();

        hil::rubble::DeviceAddress {
            bytes: *address.raw(),
            kind: match address.kind() {
                AddressKind::Public => hil::rubble::AddressKind::Public,
                AddressKind::Random => hil::rubble::AddressKind::Random,
            },
        }
    }

    fn radio_accept_cmd(radio: &mut Self::Transmitter, cmd: hil::rubble::RadioCmd) {
        radio.0.configure_receiver(match cmd {
            hil::rubble::RadioCmd::Off => RadioCmd::Off,
            hil::rubble::RadioCmd::ListenAdvertising { channel } => RadioCmd::ListenAdvertising {
                channel: hil_to_rubble_advertising_channel(channel),
            },
            hil::rubble::RadioCmd::ListenData {
                channel,
                access_address,
                crc_init,
                timeout,
            } => RadioCmd::ListenData {
                channel: DataChannel::new(channel.index()),
                access_address,
                crc_init,
                timeout,
            },
        });
    }
}

pub struct TransmitterWrapper(rubble_nrf5x::radio::BleRadio);

// Note: this is significantly more clunky than it needs to be, since
// we're converting every rubble data structure to its hil equivalent in
// tock_rubble, then doing the conversion right back here. This
// interface will make more sense when rubble_nrf5x is reimplemented in
// of Tock's primitives.
impl hil::rubble::Transmitter for TransmitterWrapper {
    fn tx_payload_buf(&mut self) -> &mut [u8] {
        self.0.tx_payload_buf()
    }
    fn transmit_advertising(
        &mut self,
        header: hil::rubble::AdvertisingHeader,
        channel: hil::rubble::AdvertisingChannel,
    ) {
        let header = advertising::Header::parse(&header.to_bytes());
        let channel = hil_to_rubble_advertising_channel(channel);
        self.0.transmit_advertising(header, channel);
    }
    fn transmit_data(
        &mut self,
        access_address: u32,
        crc_iv: u32,
        header: hil::rubble::DataHeader,
        channel: hil::rubble::DataChannel,
    ) {
        let header = data::Header::parse(&header.to_bytes());
        let channel = DataChannel::new(channel.index());
        self.0
            .transmit_data(access_address, crc_iv, header, channel)
    }
}

pub type Nrf52RubbleImplementation<'a, A> = tock_rubble::Implementation<'a, A, Nrf52BleRadio>;

// Save some deep nesting
type BleCapsule = capsules::rubble::BLE<
    'static,
    VirtualMuxAlarm<'static, Rtc<'static>>,
    Nrf52RubbleImplementation<'static, VirtualMuxAlarm<'static, Rtc<'static>>>,
>;

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
        let radio = tock_rubble::BleRadioWrapper::new(TransmitterWrapper(BleRadio::new(
            peripherals.RADIO,
            &peripherals.FICR,
            &mut TX_BUF,
            &mut RX_BUF,
        )));

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
