use crate::mem::AppSlice;
use crate::mem::Shared;
use crate::AppId;
use crate::Callback;
use crate::Driver;
use crate::Grant;
use crate::ReturnCode;
use core::cell::Cell;
use core::marker::PhantomData;

pub trait DriverInfo<T: Into<usize>>: Driver {
    fn driver(&self) -> &dyn Driver;
    fn driver_type(&self) -> T;
    fn driver_name(&self) -> &'static str;
    fn instance_identifier(&'a self) -> &'a str;
}

#[derive(Default)]
pub struct App {
    identifier_buffer: Option<AppSlice<Shared, u8>>,
}

pub struct DriverRegistry<'a, T: Into<usize>> {
    drivers: Cell<&'a [&'a dyn DriverInfo<T>]>,
    syscall_offset: usize,
    apps: Grant<App>,
    _driver_types: PhantomData<T>,
}

impl<'a, T: Into<usize>> DriverRegistry<'a, T> {
    pub fn new(
        syscall_offset: usize,
        drivers: &'a [&'a dyn DriverInfo<T>],
        grant: Grant<App>,
    ) -> DriverRegistry<'a, T> {
        DriverRegistry {
            drivers: Cell::new(drivers),
            syscall_offset,
            apps: grant,
            _driver_types: PhantomData,
        }
    }

    pub fn update_drivers(&self, drivers: &'a [&'a dyn DriverInfo<T>]) {
        self.drivers.set(drivers)
    }

    pub fn find_instance(
        &'b self,
        driver_type: usize,
        instance_id: &'b [u8],
    ) -> Option<(usize, &'a dyn Driver)> {
        let dtype: usize = driver_type.into();

        self.drivers
            .get()
            .iter()
            .enumerate()
            .find(|(_, driver)| {
                driver.driver_type().into() == dtype
                    && driver.instance_identifier().as_bytes() == instance_id
            })
            .and_then(|(index, driver)| {
                index
                    .checked_add(self.syscall_offset + 1)
                    .map(move |syscall_id| (syscall_id, driver.driver()))
            })
    }

    pub fn from_syscall_id(&self, syscall_id: usize) -> Option<&dyn Driver> {
        if syscall_id == self.syscall_offset {
            Some(self)
        } else {
            syscall_id
                .checked_sub(self.syscall_offset + 1)
                .and_then(|index| self.drivers.get().get(index).map(|driver| driver.driver()))
        }
    }
}

impl<'a, T: Into<usize>> Driver for DriverRegistry<'a, T> {
    fn subscribe(
        &self,
        _minor_num: usize,
        _callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn command(&self, command_num: usize, r2: usize, _r3: usize, app_id: AppId) -> ReturnCode {
        match command_num {
            0 /* Check if the driver is available */ => {
                ReturnCode::SUCCESS
	    },
            1 /* Find syscall number for driver identifier */ => {
                self.apps
                    .enter(app_id, |app, _| {
                        if let Some(id_buf) = &app.identifier_buffer {
                            self.find_instance(r2, id_buf.as_ref())
                                .map(|(syscall_id, _)| ReturnCode::SuccessWithValue {
                                    value: syscall_id,
                                })
                                .unwrap_or(ReturnCode::ENODEVICE)
                        } else {
                            ReturnCode::ERESERVE
                        }
                    })
                    .unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        app_id: AppId,
        minor_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match minor_num {
            0 /* the instance identifier to search for */ => {
                self.apps
                    .enter(app_id, |app, _| {
                        app.identifier_buffer = slice;
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| err.into())
            },
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
