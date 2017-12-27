//! Bluetooth Low Energy HIL

use kernel;
use kernel::ReturnCode;


pub trait BleAdvertisementDriver {
    fn set_data(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8];
    fn set_txpower(&self, power: usize) -> ReturnCode;
    fn send_advertisement(&self, appid: kernel::AppId, freq: RadioChannel);
    fn receive_advertisement(&self, appid: kernel::AppId, freq: RadioChannel);
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


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    Freq37 = 37,
    Freq38 = 38,
    Freq39 = 39,
}
