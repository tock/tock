use crate::returncode::ReturnCode;

pub trait RadioConfig {
    fn initialize(
        &self,
        spi_buf: &'static mut [u8],
        reg_write: &'static mut [u8],
        reg_read: &'static mut [u8],
    ) -> ReturnCode;
    fn reset(&self) -> ReturnCode;
    fn start(&self) -> ReturnCode;
    fn stop(&self) -> ReturnCode;
    fn ready(&self) -> ReturnCode;
    fn sleep(&self) -> ReturnCode;
    fn set_header_mode(&self, implicit: bool);
    fn is_on(&self) -> bool;
    fn handle_packet_irq(&self) -> ReturnCode;
}

pub trait RadioData {
    fn transmit(&self, implicit: bool) -> ReturnCode;
    fn transmit_done(&self, asyn: bool) -> ReturnCode;
    fn receive(&self, size: u8);
    fn receive_done(&self, size: usize) -> u8;
    fn read(&self) -> u8;
    fn write(&self, buf: &[u8], size: u8);
}

pub trait PacketConfig {
    fn packet_rssi(&self) -> u8;
    fn packet_snr(&self) -> f32;
    fn packet_frequency_error(&self) -> i64;
    fn set_power(&self, level: i8, output_pin: u8);
    fn set_frequency(&self, frequency: u64);
    fn get_spreading_factor(&self) -> u8;
    fn set_spreading_factor(&self, sf: u8);
    fn get_signal_bandwidth(&self) -> f64;
    fn set_signal_bandwidth(&self, sbw: f64);
    fn set_ldo_flag(&self);
    fn set_coding_rate4(&self, denominator: u8);
    fn set_preamble_length(&self, length: i64);
    fn set_sync_word(&self, sw: u8);
    fn enable_crc(&self);
    fn disable_crc(&self);
    fn enable_invert_iq(&self);
    fn disable_invert_iq(&self);
    fn set_ocp(&self, ma: u8);
}
