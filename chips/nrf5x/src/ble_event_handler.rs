use ble_advertising_driver::{App, BLEPduType};
use kernel;
use kernel::returncode::ReturnCode;
use ble_advertising_hil::RadioChannel;
use ble_advertising_driver::BusyState;
use ble_advertising_hil::Pdu_writer;


pub trait BLESender {
    fn transmit_buffer(&self,
                       buf: &'static mut [u8],
                       len: usize, appid: kernel::AppId);

    fn transmit_buffer_edit(&self, len: usize, appid: kernel::AppId, edit_buffer: &Fn(&mut [u8]) -> ());

    fn receive_buffer(&self, channel: RadioChannel, appid: kernel::AppId);

    fn set_tx_power(&self, power: u8) -> ReturnCode;

    fn set_busy(&self, state: BusyState);

    fn alarm_now(&self) -> u32;

    fn set_access_address(&self, address: u32);
}

pub trait BLEEventHandler<S>
{
    fn handle_rx_event<A>(
        state: S,
        app: &mut App,
        ble: &BLESender,
        appid: kernel::AppId,
        pdu: &BLEPduType,
    ) -> S where A: kernel::hil::time::Alarm, Self: Sized;

    fn handle_tx_event<A>(
        state: S,
        app: &mut App,
        ble: &BLESender,
        appid: kernel::AppId,
    ) -> S where A: kernel::hil::time::Alarm, Self: Sized;

    fn handle_timer_event<A>(
        state: S,
        app: &mut App,
        ble: &BLESender,
        appid: kernel::AppId,
    ) -> S where A: kernel::hil::time::Alarm, Self: Sized;
}
