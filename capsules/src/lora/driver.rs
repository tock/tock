
use super::radio::Radio;
use kernel::hil::spi::{SpiMasterDevice};
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};
use crate::driver;

pub const DRIVER_NUM: usize = driver::NUM::Lora as usize;

pub struct App {
  callback: Option<Callback>,
}

pub struct RadioDriver<'a, S: SpiMasterDevice> {
  /// Underlying physical device
  device: &'a Radio<'a, S>,

  /// Grant of apps that use this radio driver.
  apps: Grant<App>,
}

impl Default for App {
    fn default() -> Self {
        App {
            callback: None,
        }
    }
}

impl<S: SpiMasterDevice> RadioDriver<'a, S> {
    pub fn new(
      device: &'a Radio<'a, S>,
      grant: Grant<App>,
    ) -> RadioDriver<'a, S> {
      RadioDriver {
        device: device,
        apps: grant,
    }
  }
}


impl<S: SpiMasterDevice> Driver for RadioDriver<'a, S> {
  /// Command interface.
  ///
  /// ### `command_num`
  ///
  /// - `0`: Return SUCCESS if this driver is included on the platform.
  /// - `1`: Start the radio.
  fn command(&self, command_num: usize, arg1: usize, _: usize, appid: AppId) -> ReturnCode {
    match command_num {
      0 => ReturnCode::SUCCESS,

      1 => self.device.begin(865000000),

      _ => ReturnCode::ENOSUPPORT,
    }
  }
}

