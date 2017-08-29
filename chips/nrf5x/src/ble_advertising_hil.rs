//! Bluetooth Low Energy HIL

use kernel::ReturnCode;

pub trait BleAdvertisementDriver {
    fn clear_adv_data(&self);
    fn set_advertisement_address(&self, addr: &'static mut [u8]) -> &'static mut [u8];
    fn set_advertisement_data(&self,
                              ad_type: usize,
                              data: &'static mut [u8],
                              len: usize,
                              offset: usize)
                              -> &'static mut [u8];
    fn set_advertisement_txpower(&self, power: usize) -> ReturnCode;
    fn start_advertisement_tx(&self, ch: usize);
    fn start_advertisement_rx(&self, ch: usize);
    fn set_client(&self, client: &'static RxClient);
}


// Temporary trait for BLE
pub trait RxClient {
    fn receive(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}
