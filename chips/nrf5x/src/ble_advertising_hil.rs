//! Bluetooth Low Energy HIL

use kernel::ReturnCode;

pub trait BleAdvertisementDriver {
    fn set_tx_power(&self, power: usize) -> ReturnCode;
    fn transmit_advertisement(
        &self,
        buf: &'static mut [u8],
        len: usize,
        freq: RadioChannel,
    ) -> &'static mut [u8];
    fn receive_advertisement(&self, freq: RadioChannel);
    fn set_receive_client(&self, client: &'static RxClient);
    fn set_transmit_client(&self, client: &'static TxClient);
}

pub trait RxClient {
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode);
}

pub trait TxClient {
    fn transmit_event(&self, result: ReturnCode);
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RadioChannel {
    Freq37 = 37,
    Freq38 = 38,
    Freq39 = 39,
}
