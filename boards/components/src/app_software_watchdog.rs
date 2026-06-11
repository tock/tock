// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Component for App Software Watchdogs.
//!
//! Example instantiation:
//!
//! ```rust
//!  use kernel::static_init;
//!
//!  /// Capability for Restarting Processes needed for the app software watchdog.
//!  struct PRCapability;
//!  unsafe impl ProcessRestartCapability for PRCapability {}
//!
//!  type AppSoftwareWatchdog = capsules_extra::app_software_watchdog::AppSoftwareWatchdog<
//!      'static,
//!      VirtualMuxAlarm<'static, nrf52840::rtc::Rtc<'static>>,
//!      PRCapability,
//!  >;
//!
//!  // Capability for managing processes that is owned/used by ProcessRestarter
//!  // to restart
//!  struct PMCapability;
//!  unsafe impl ProcessManagementCapability for PMCapability {}
//!  let app_software_watchdog =
//!      components::app_software_watchdog::AppSoftwareWatchdogComponent::new(
//!          mux_alarm,
//!          board_kernel,
//!          PRCapability,
//!          PMCapability,
//!      )
//!      .finalize(components::app_softare_watchdog_component_static!(
//!          nrf52840::rtc::Rtc,
//!          PRCapability,
//!          PMCapability,
//!      ));
//! ```

use capsules_core::virtualizers::virtual_alarm::MuxAlarm;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_extra::app_software_watchdog::AppSoftwareWatchdog;
use capsules_system::process_restart::ProcessRestarter;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::capabilities::ProcessManagementCapability;
use kernel::capabilities::ProcessRestartCapability;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::time::Alarm;
use kernel::Kernel;

#[macro_export]
macro_rules! app_softare_watchdog_component_static {
    ($A:ty, $P:ty, $C:ty $(,)?) => {{
        let sw_watchdog = kernel::static_buf!(
            capsules_extra::app_software_watchdog::AppSoftwareWatchdog<
                'static,
                capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                $P,
            >,
        );

        let process_restarter =
            kernel::static_buf!(capsules_system::process_restart::ProcessRestarter<$C>);

        let alarm = kernel::static_buf!(
            capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>
        );

        (sw_watchdog, process_restarter, alarm)
    };};
}

pub struct AppSoftwareWatchdogComponent<
    'a,
    A: Alarm<'a>,
    P: ProcessRestartCapability,
    C: ProcessManagementCapability,
> {
    mux_alarm: &'a MuxAlarm<'a, A>,
    board_kernel: &'a Kernel,
    pr_capability: P,
    pm_capability: C,
}

impl<'a, A: Alarm<'a>, P: ProcessRestartCapability, C: ProcessManagementCapability>
    AppSoftwareWatchdogComponent<'a, A, P, C>
{
    pub fn new(
        mux_alarm: &'a MuxAlarm<'a, A>,
        board_kernel: &'a Kernel,
        pr_capability: P,
        pm_capability: C,
    ) -> Self {
        Self {
            mux_alarm,
            board_kernel,
            pr_capability,
            pm_capability,
        }
    }
}

impl<
        A: Alarm<'static>,
        P: ProcessRestartCapability + 'static,
        C: ProcessManagementCapability + 'static,
    > Component for AppSoftwareWatchdogComponent<'static, A, P, C>
{
    type StaticInput = (
        &'static mut MaybeUninit<AppSoftwareWatchdog<'static, VirtualMuxAlarm<'static, A>, P>>,
        &'static mut MaybeUninit<ProcessRestarter<C>>,
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
    );
    type Output = &'static AppSoftwareWatchdog<'static, VirtualMuxAlarm<'static, A>, P>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let alarm = static_buffer.2.write(VirtualMuxAlarm::new(self.mux_alarm));
        alarm.setup();

        let process_restarter =
            static_buffer
                .1
                .write(capsules_system::process_restart::ProcessRestarter::new(
                    self.board_kernel,
                    self.pm_capability,
                ));

        let sw_watchdog = static_buffer.0.write(AppSoftwareWatchdog::new(
            self.board_kernel.create_grant(
                capsules_extra::app_software_watchdog::DRIVER_NUM,
                &grant_cap,
            ),
            alarm,
            process_restarter,
            self.pr_capability,
        ));

        alarm.set_alarm_client(sw_watchdog);
        sw_watchdog
    }
}
