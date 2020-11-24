use super::radio::Radio;
use crate::driver;
use kernel::hil::lora::{PacketConfig, RadioConfig, RadioData};
use kernel::{AppId, Driver, ReturnCode};

pub const DRIVER_NUM: usize = driver::NUM::Lora as usize;

pub struct App {}

pub struct RadioDriver<'a> {
    /// Underlying physical device; FIX make private
    pub device: &'a Radio<'a>,
}

impl Default for App {
    fn default() -> Self {
        App {}
    }
}

impl<'a> RadioDriver<'a> {
    pub fn new(device: &'a Radio<'a>) -> RadioDriver<'a> {
        RadioDriver { device: device }
    }
}

impl Driver for RadioDriver<'_> {
    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return SUCCESS if this driver is included on the platform.
    /// - `1`: Start the radio.
    fn command(&self, command_num: usize, arg1: usize, _: usize, _appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,

            1 => self.device.start(),

            2 => self.device.stop(),

            3 => self.device.transmit(arg1 != 0),

            4 => {
                self.device.receive(1);
                ReturnCode::SUCCESS
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
