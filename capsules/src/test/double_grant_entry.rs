//! Test that tries to enter a grant twice.
//!
//! This must fail or Tock allows multiple mutable references to the same memory
//! which is undefined behavior.
//!
//! To use, setup this capsule and connect the syscall `Driver` implementation
//! to userspace. Then call the commands to test various double grant entries.
//!
//! # Usage
//!
//! Here is my example usage for hail:
//!
//! ```diff
//! diff --git a/boards/hail/src/main.rs b/boards/hail/src/main.rs
//! index 110d45fa7..e8f4728c2 100644
//! --- a/boards/hail/src/main.rs
//! +++ b/boards/hail/src/main.rs
//! @@ -73,6 +73,7 @@ struct Hail {
//!      ipc: kernel::ipc::IPC<NUM_PROCS>,
//!      crc: &'static capsules::crc::Crc<'static, sam4l::crccu::Crccu<'static>>,
//!      dac: &'static capsules::dac::Dac<'static>,
//! +    dge: &'static capsules::test::double_grant_entry::TestGrantDoubleEntry,
//!  }
//!
//!  /// Mapping of integer syscalls to objects that implement syscalls.
//! @@ -102,6 +103,8 @@ impl Platform for Hail {
//!
//!              capsules::dac::DRIVER_NUM => f(Some(self.dac)),
//!
//! +            capsules::test::double_grant_entry::DRIVER_NUM => f(Some(self.dge)),
//! +
//!              kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
//!              _ => f(None),
//!          }
//! @@ -396,6 +399,14 @@ pub unsafe fn reset_handler() {
//!          capsules::dac::Dac::new(&peripherals.dac)
//!      );
//!
//! +    // Test double grant entry
//! +    let dge = static_init!(
//! +        capsules::test::double_grant_entry::TestGrantDoubleEntry,
//! +        capsules::test::double_grant_entry::TestGrantDoubleEntry::new(
//! +            board_kernel.create_grant(&memory_allocation_capability)
//! +        )
//! +    );
//! +
//!      // // DEBUG Restart All Apps
//!      // //
//!      // // Uncomment to enable a button press to restart all apps.
//! @@ -440,6 +451,7 @@ pub unsafe fn reset_handler() {
//!          ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
//!          crc,
//!          dac,
//! +        dge,
//!      };
//!
//!      // Setup the UART bus for nRF51 serialization..
//!     ```

use kernel::grant::Grant;
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::{ErrorCode, ProcessId};

/// Syscall driver number.
pub const DRIVER_NUM: usize = 0xF001;

/// Need a grant for the process.
#[derive(Default)]
pub struct App {
    pending: bool,
}

pub struct TestGrantDoubleEntry {
    grant: Grant<App, 0>,
}

impl TestGrantDoubleEntry {
    pub fn new(grant: Grant<App, 0>) -> TestGrantDoubleEntry {
        TestGrantDoubleEntry { grant }
    }
}

impl SyscallDriver for TestGrantDoubleEntry {
    fn command(&self, command_num: usize, _: usize, _: usize, appid: ProcessId) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success(),

            1 => {
                // Try Grant.enter() then Grant.iter().enter()

                // Check if we saw a grant with pending as true. If so, we
                // entered the same grant twice.
                let mut found_pending = false;

                // Enter the grant for the app.
                let err = self
                    .grant
                    .enter(appid, |appgrant, _| {
                        // We can now change the state of the app's grant
                        // region.
                        appgrant.pending = true;

                        // Now, try to iterate all grant regions.
                        for grant in self.grant.iter() {
                            // And, try to enter each grant! This should fail.
                            grant.enter(|appgrant2, _| {
                                if appgrant2.pending {
                                    found_pending = true;
                                }
                            });
                        }
                        CommandReturn::success()
                    })
                    .unwrap_or_else(|err| err.into());

                // If found pending is true, things are broken.
                if found_pending {
                    kernel::debug!("ERROR! Entered a grant twice simultaneously!!");
                }

                err
            }

            2 => {
                // Try Grant.iter() then Grant.enter() then Grant.iter().enter()

                // Check if we saw a grant with pending as true. If so, we
                // entered the same grant twice.
                let mut found_pending = false;

                // Make sure the grant is allocated.
                let _ = self.grant.enter(appid, |appgrant, _| {
                    appgrant.pending = false;
                });

                for app in self.grant.iter() {
                    let _ = self.grant.enter(appid, |appgrant, _| {
                        // Mark the field.
                        appgrant.pending = true;

                        // Check if we can access this grant twice.
                        app.enter(|appgrant2, _| {
                            if appgrant2.pending {
                                found_pending = true;
                            }
                        });
                    });
                }

                // If found pending is true, things are broken.
                if found_pending {
                    kernel::debug!("ERROR! Entered a grant twice simultaneously!!");
                }

                // Since we expect a panic these return values don't matter.
                CommandReturn::success()
            }

            3 => {
                // Try Grant.enter() then Grant.enter()

                // Check if we saw a grant with pending as true. If so, we
                // entered the same grant twice.
                let mut found_pending = false;

                let _ = self.grant.enter(appid, |appgrant, _| {
                    appgrant.pending = true;

                    let _ = self.grant.enter(appid, |appgrant2, _| {
                        if appgrant2.pending {
                            found_pending = true;
                        }
                    });
                });

                // If found pending is true, things are broken.
                if found_pending {
                    kernel::debug!("ERROR! Entered a grant twice simultaneously!!");
                }

                // Since we expect a panic these return values don't matter.
                CommandReturn::success()
            }

            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::process::Error> {
        self.grant.enter(processid, |_, _| {})
    }
}
