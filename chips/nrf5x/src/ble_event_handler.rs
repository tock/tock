use kernel;
use kernel::returncode::ReturnCode;
use ble_advertising_hil::RadioChannel;
use ble_advertising_driver::BusyState;

pub trait BLESender {
    fn transmit_buffer(&self, appid: kernel::AppId);
    fn replace_buffer(&self, edit_buffer: &Fn(&mut [u8]) -> ());

    fn transmit_buffer_edit(
        &self,
        len: usize,
        appid: kernel::AppId,
        edit_buffer: &Fn(&mut [u8]) -> (),
    );

    fn receive_buffer(&self, appid: kernel::AppId);

    fn set_tx_power(&self, power: u8) -> ReturnCode;

    fn set_busy(&self, state: BusyState);

    fn alarm_now(&self) -> u32;

    fn set_access_address(&self, address: u32);
}
