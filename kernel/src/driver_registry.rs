//! Driver registry, dynamically mapping syscall driver numbers to
//! driver instances
//!
//! This enables the kernel to expose multiple distinguishable
//! instances of the same driver to userspace, similar to major and
//! minor numbers on Linux.

use crate::mem::AppSlice;
use crate::mem::Shared;
use crate::AppId;
use crate::Callback;
use crate::Driver;
use crate::Grant;
use crate::ReturnCode;
use core::cell::Cell;
use core::marker::PhantomData;

/// Driver information queriable by apps
///
/// Each driver registered must implement this trait. It is used to
/// find either one instance of a driver type, or a specific instance.
pub trait DriverInfo<T: Into<usize>>: Driver {
    fn driver(&self) -> &dyn Driver;
    fn driver_type(&self) -> T;
    fn driver_name(&self) -> &'static str;
    fn instance_identifier(&'a self) -> &'a str;
}

/// Per application driver registry state
#[derive(Default)]
pub struct App {
    /// Storing the app's memory region for string comparison of the
    /// instance_identifier
    identifier_buffer: Option<AppSlice<Shared, u8>>,
}

/// Implementation of a driver registry, dynamically assigning drivers
/// syscall driver ids to be used by applications
///
/// This implementation tries to be as efficient as possible. The
/// dynamically assigned driver ids are counted starting from a fixed
/// offset `syscall_offset + 1` and directly map to the drivers-array
/// position.
///
/// The  driver  registry  being  a driver  itself  is  accessible  at
/// `syscall_offset`
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

    /// Find a driver instance based on both a driver type
    /// (DRIVER_NUM) and an instance identifier
    pub fn find_instance(
        &self,
        driver_type: usize,
        instance_id: Option<&[u8]>,
    ) -> Option<(usize, &'a dyn Driver)> {
        let dtype: usize = driver_type.into();

        self.drivers
            .get()
            .iter()
            .enumerate()
            .find(|(_, driver)| {
                // If the desired instance id is None (get primary
                // instance), return true. The iterator find method
                // will return the first (primary) instance
                let instance_id_match = instance_id
                    .map(|id| driver.instance_identifier().as_bytes() == id)
                    .unwrap_or(true);

                driver.driver_type().into() == dtype && instance_id_match
            })
            .and_then(|(index, driver)| {
                index
                    .checked_add(self.syscall_offset + 1)
                    .map(move |syscall_id| (syscall_id, driver.driver()))
            })
    }

    /// From a dynamical syscall driver id, resolve back to the driver
    /// instance
    pub fn from_syscall_id(&self, syscall_id: usize) -> Option<&dyn Driver> {
        // The implementation of this must be as efficient as
        // possible, as this will be run for every syscall. The
        // current approach is to check whether the driver registry
        // itself is addressed and then choose the corresponding
        // driver based on a bounds-checked array access alone.

        if syscall_id == self.syscall_offset {
            Some(self)
        } else {
            syscall_id
                .checked_sub(self.syscall_offset + 1)
                .and_then(|index| self.drivers.get().get(index).map(|driver| driver.driver()))
        }
    }
}

/// Allow userspace to interact with the driver registry
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
	    1 /* Find syscall number of first instance of given driver type */ => {
		self.find_instance(r2, None)
		    .map(|(syscall_id, _)| ReturnCode::SuccessWithValue {
			value: syscall_id,
		    }).unwrap_or(ReturnCode::ENODEVICE)
	    },
            2 /* Find syscall number for specific driver identifier */ => {
                self.apps
                    .enter(app_id, |app, _| {
                        if let Some(id_buf) = &app.identifier_buffer {
                            self.find_instance(r2, Some(id_buf.as_ref()))
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
