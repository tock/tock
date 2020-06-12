use super::radio::Radio;
use crate::driver;
use kernel::hil::spi::SpiMasterDevice;
use kernel::{AppId, Driver, ReturnCode};

pub const DRIVER_NUM: usize = driver::NUM::Lora as usize;

pub struct App {}

pub struct RadioDriver<'a, S: SpiMasterDevice> {
    /// Underlying physical device; FIX make private
    pub device: &'a Radio<'a, S>,
}

impl Default for App {
    fn default() -> Self {
        App {}
    }
}

impl<'a, S: SpiMasterDevice> RadioDriver<'a, S> {
    pub fn new(device: &'a Radio<'a, S>) -> RadioDriver<'a, S> {
        RadioDriver { device: device }
    }
}

impl<S: SpiMasterDevice> Driver for RadioDriver<'_, S> {
    /// Command interface.
    ///
    /// ### `command_num`
    ///
    /// - `0`: Return SUCCESS if this driver is included on the platform.
    /// - `1`: Start the radio.
    fn command(&self, command_num: usize, arg1: usize, _: usize, _appid: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,

            1 => self.device.begin(865000000),

            2 => self.device.end(),

            3 => self.device.begin_packet(arg1 != 0),

            4 => self.device.end_packet(arg1 != 0),

            5 => self.device.handle_lora_irq(),

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
