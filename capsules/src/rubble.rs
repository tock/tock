//! Bluetooth Low Energy Driver

use core::cell::RefCell;
use core::cmp;
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::time::Frequency;
use kernel::hil::time::Time;
use kernel::ReturnCode;

use rubble::beacon::Beacon;
use rubble::link::{ad_structure::AdStructure, MIN_PDU_BUF};
use rubble_nrf5x::radio::{BleRadio, PacketBuffer};
use rubble_nrf5x::utils::get_device_address;

use nrf52840_hal as hal;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::RubbleBle as usize;

/// Send buffer
pub static mut TX_BUF: PacketBuffer = [0; MIN_PDU_BUF];

/// Recv buffer
pub static mut RX_BUF: PacketBuffer = [0; MIN_PDU_BUF];

/// Process specific memory
pub struct App;

impl Default for App {
    fn default() -> App {
        App
    }
}

pub struct BLE<'a, T>
where
    T: kernel::hil::time::Alarm<'a>,
{
    radio: RefCell<BleRadio>,
    beacon: Beacon,
    app: kernel::Grant<App>,
    alarm: &'a T,
}

impl<'a, T> BLE<'a, T>
where
    T: kernel::hil::time::Alarm<'a>,
{
    pub fn new(
        container: kernel::Grant<App>,
        tx_buf: &'static mut [u8; MIN_PDU_BUF],
        rx_buf: &'static mut [u8; MIN_PDU_BUF],
        timer: &'a T,
    ) -> Self {
        // TODO: this should be moved into a hardware-specific initialize/new method

        // Determine device address
        let device_address = get_device_address();

        let peripherals = hal::target::Peripherals::take().unwrap();
        // Rubble currently requires an RX buffer even though the radio is only used as a TX-only
        // beacon.
        let radio = BleRadio::new(peripherals.RADIO, &peripherals.FICR, tx_buf, rx_buf);

        let beacon = Beacon::new(
            device_address,
            &[AdStructure::CompleteLocalName("Rusty Tock Beacon (nRF52)")],
        )
        .unwrap();

        BLE {
            radio: RefCell::new(radio),
            beacon,
            app: container,
            alarm: timer,
        }
    }

    // Determines which app timer will expire next and sets the underlying alarm
    // to it.
    //
    // This method iterates through all grants so it should be used somewhat
    // sparingly. Moreover, it should _not_ be called from within a grant,
    // since any open grant will not be iterated over and the wrong timer will
    // likely be chosen.
    pub fn set_alarm(&self) {
        self.alarm
            .set_alarm(self.alarm.now() + 333 * <T as Time>::Frequency::frequency() / 1000);
    }
}

// Timer alarm
impl<'a, T> kernel::hil::time::AlarmClient for BLE<'a, T>
where
    T: kernel::hil::time::Alarm<'a>,
{
    fn fired(&self) {
        let mut radio = self.radio.borrow_mut();
        self.beacon.broadcast(&mut *radio);
        self.set_alarm();
    }
}

// System Call implementation
impl<'a, T> kernel::Driver for BLE<'a, T>
where
    T: kernel::hil::time::Alarm<'a>,
{
    fn command(
        &self,
        command_num: usize,
        data: usize,
        interval: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {
            // Start periodic advertisements
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
