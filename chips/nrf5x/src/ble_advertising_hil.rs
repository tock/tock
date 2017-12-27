//! Bluetooth Low Energy HIL

use kernel::ReturnCode;


pub trait BleAdvertisementDriver {
    fn set_data(&self, buf: &'static mut [u8], len: usize) -> &'static mut [u8];
    fn set_txpower(&self, power: usize) -> ReturnCode;
    fn send_advertisement(&self, freq: RadioChannel);
    fn receive_advertisement(&self, freq: RadioChannel);
    fn set_rx_client(&self, client: &'static RxClient);
    fn set_tx_client(&self, client: &'static TxClient);
}


pub trait RxClient {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait TxClient {
    fn send_event(&self, result: ReturnCode);
}


#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    Freq37 = 37,
    Freq38 = 38,
    Freq39 = 39,
}
