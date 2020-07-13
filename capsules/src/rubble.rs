//! Bluetooth Low Energy Driver

use core::cell::RefCell;
use kernel::debug;
use kernel::hil::time::Frequency;
use kernel::hil::time::Time;
use kernel::ReturnCode;

use rubble::beacon::Beacon;
use rubble::link::{ad_structure::AdStructure, Transmitter};
use rubble_nrf5x::utils::get_device_address;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::RubbleBle as usize;

/// Process specific memory
pub struct App;

impl Default for App {
    fn default() -> App {
        App
    }
}

pub struct BLE<'a, T, A>
where
    T: Transmitter,
    A: kernel::hil::time::Alarm<'a>,
{
    radio: RefCell<T>,
    beacon: Beacon,
    app: kernel::Grant<App>,
    alarm: &'a A,
}

impl<'a, T, A> BLE<'a, T, A>
where
    T: Transmitter,
    A: kernel::hil::time::Alarm<'a>,
{
    pub fn new(container: kernel::Grant<App>, radio: T, alarm: &'a A) -> Self {
        // TODO: this should be moved into a hardware-specific initialize/new method

        // Determine device address
        let device_address = get_device_address();

        let beacon = Beacon::new(
            device_address,
            &[AdStructure::CompleteLocalName("Beacon (nRF52840)")],
        )
        .unwrap();

        BLE {
            radio: RefCell::new(radio),
            beacon,
            app: container,
            alarm,
        }
    }

    pub fn set_alarm(&self) {
        self.alarm
            .set_alarm(self.alarm.now() + 333 * <A as Time>::Frequency::frequency() / 1000);
    }
}

// Timer alarm
impl<'a, T, A> kernel::hil::time::AlarmClient for BLE<'a, T, A>
where
    T: Transmitter,
    A: kernel::hil::time::Alarm<'a>,
{
    fn fired(&self) {
        let mut radio = self.radio.borrow_mut();
        self.beacon.broadcast(&mut *radio);
        self.set_alarm();
    }
}

// System Call implementation
impl<'a, T, A> kernel::Driver for BLE<'a, T, A>
where
    T: Transmitter,
    A: kernel::hil::time::Alarm<'a>,
{
    fn command(
        &self,
        command_num: usize,
        data: usize,
        interval: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {
            // Start periodic advertisements (not currently used)
            0 => self
                .app
                .enter(appid, |app, _| {
                    self.set_alarm();
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
