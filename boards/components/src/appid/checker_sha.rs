// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for SHA-based credential checkers.

use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::digest;

#[macro_export]
macro_rules! app_checker_sha256_component_static {
    () => {{
        let buffer = kernel::static_buf!([u8; 32]);
        let checker =
            kernel::static_buf!(capsules_system::process_checker::basic::AppCheckerSha256);

        (checker, buffer)
    };};
}

pub type AppCheckerSha256ComponentType = capsules_system::process_checker::basic::AppCheckerSha256;

pub struct AppCheckerSha256Component<S: 'static + digest::Digest<'static, 32>> {
    sha: &'static S,
}

impl<S: 'static + digest::Digest<'static, 32>> AppCheckerSha256Component<S> {
    pub fn new(sha: &'static S) -> Self {
        Self { sha }
    }
}

impl<
        S: kernel::hil::digest::Sha256
            + 'static
            + digest::Digest<'static, 32>
            + kernel::hil::digest::DigestDataVerify<'static, 32>,
    > Component for AppCheckerSha256Component<S>
{
    type StaticInput = (
        &'static mut MaybeUninit<capsules_system::process_checker::basic::AppCheckerSha256>,
        &'static mut MaybeUninit<[u8; 32]>,
    );

    type Output = &'static capsules_system::process_checker::basic::AppCheckerSha256;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let buffer = s.1.write([0; 32]);

        let checker = s.0.write(
            capsules_system::process_checker::basic::AppCheckerSha256::new(self.sha, buffer),
        );

        digest::Digest::set_client(self.sha, checker);

        checker
    }
}
