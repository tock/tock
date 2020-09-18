//! Provides userspace access to the Qdec on a board.
//!
//! Usage
//! -----
//!
//! ````
//! let qdec = static_init!(
//!     capsules::qdec::Qdec<'static>,
//!     capsules::qdec::QdecInterface::new(&nrf52::qdec::QDEC,
//!                                         kernel::Grant::create())
//! );
//! kernel::hil::QdecDriver.set_client(qdec);
//! ````
//! #Interrupt Spurred Readings versus Regular Readings
//! An application can either enable interrupts to get the
//! accumulation value or manually read it whenever it wants

use crate::driver;
use kernel::hil;
use kernel::{AppId, Callback, Driver, Grant, ReturnCode};

pub const DRIVER_NUM: usize = driver::NUM::Qdec as usize;

/// This struct contains the resources necessary for the QdecInterface
pub struct QdecInterface<'a> {
    driver: &'a dyn hil::qdec::QdecDriver,
    apps: Grant<App>,
}

#[derive(Default)]
/// This struct contains the necessary fields for an app
pub struct App {
    callback: Option<Callback>,
    position: i32,
}

impl<'a> QdecInterface<'a> {
    /// Create a new instance of the QdecInterface
    pub fn new(driver: &'a dyn hil::qdec::QdecDriver, grant: Grant<App>) -> Self {
        Self {
            driver: driver,
            apps: grant,
        }
    }

    fn configure_callback(&self, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        self.apps
            .enter(app_id, |app, _| {
                app.callback = callback;
                ReturnCode::SUCCESS
            })
            .unwrap_or_else(|err| err.into())
    }
}

impl<'a> hil::qdec::QdecClient for QdecInterface<'a> {
    /// Goes through all the apps and if the app is
    /// subscribed then it sends back the acc value
    fn sample_ready(&self) {
        for cntr in self.apps.iter() {
            cntr.enter(|app, _| {
                app.position = app.position + self.driver.get_acc();
                app.callback
                    .map(|mut cb| cb.schedule(app.position as usize, 0, 0));
            });
        }
    }

    /// Goes through all the apps and if the app recently
    /// had an overflow, it records the occurance
    fn overflow(&self) {
        /*for now, we do not handle overflows*/
    }
}

impl<'a> Driver for QdecInterface<'a> {
    /// Subscribes a client to (newly enabled) interrupts
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        app_id: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            0 => self.configure_callback(callback, app_id),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /// Command switch statement for various essential processes
    /// 0 is a sanity check for the switch statement
    /// 1 enables the qdec
    /// 2 checks that the qdec is enabled
    /// 3 enables inerrupts
    /// 4 gets the current displacement stored in the QDEC
    fn command(&self, command_num: usize, _: usize, _: usize, _app_id: AppId) -> ReturnCode {
        match command_num {
            0 => ReturnCode::SUCCESS,
            1 => self.driver.enable_qdec(),
            2 => self.driver.disable_qdec(),
            3 => self.driver.enable_interrupts(),
            4 => ReturnCode::SuccessWithValue {
                value: self.driver.get_acc() as usize,
            },
            5 => self.driver.disable_qdec(),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
