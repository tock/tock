//! Bluetooth Low Energy HIL

use kernel;
use kernel::ReturnCode;


pub trait BleAdvertisementDriver {
    fn set_advertisement_data(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8];
    fn set_advertisement_txpower(&self, power: usize) -> ReturnCode;
    fn start_advertisement_tx(&self, appid: kernel::AppId);
    fn start_advertisement_rx(&self, appid: kernel::AppId);
    fn set_client(&self, client: &'static RxClient);
}


// Temporary trait for BLE
pub trait RxClient {
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode, appid: kernel::AppId);
    fn advertisement_fired(&self, appid: kernel::AppId);
}
