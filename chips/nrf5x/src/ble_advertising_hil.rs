//! Bluetooth Low Energy HIL

use ble_advertising_driver::RadioChannel;
use kernel;
use kernel::ReturnCode;


pub trait BleAdvertisementDriver {
    fn set_advertisement_data(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8];
    fn set_advertisement_txpower(&self, power: usize) -> ReturnCode;
    fn start_advertisement_tx(&self, appid: kernel::AppId, freq: RadioChannel);
    fn start_advertisement_rx(&self, appid: kernel::AppId, freq: RadioChannel);
    fn set_rx_client(&self, client: &'static RxClient);
    fn set_tx_client(&self, client: &'static TxClient);
}


pub trait RxClient {
    fn receive_event(&self,
                     buf: &'static mut [u8],
                     len: u8,
                     result: ReturnCode,
                     appid: kernel::AppId);
}

pub trait TxClient {
    fn send_event(&self, result: ReturnCode, appid: kernel::AppId);
}
