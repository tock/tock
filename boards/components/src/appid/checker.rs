// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Component for creating a process checking machine.

use core::mem::MaybeUninit;
use kernel::component::Component;

#[macro_export]
macro_rules! process_checker_machine_component_static {
    () => {{
        kernel::static_buf!(kernel::process::ProcessCheckerMachine)
    };};
}

pub type ProcessCheckerMachineComponentType = kernel::process::ProcessCheckerMachine;

pub struct ProcessCheckerMachineComponent {
    policy: &'static dyn kernel::process_checker::AppCredentialsPolicy<'static>,
}

impl ProcessCheckerMachineComponent {
    pub fn new(policy: &'static dyn kernel::process_checker::AppCredentialsPolicy) -> Self {
        Self { policy }
    }
}

impl Component for ProcessCheckerMachineComponent {
    type StaticInput = &'static mut MaybeUninit<kernel::process::ProcessCheckerMachine>;

    type Output = &'static kernel::process::ProcessCheckerMachine;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let checker = s.write(kernel::process::ProcessCheckerMachine::new(self.policy));

        self.policy.set_client(checker);
        checker
    }
}
