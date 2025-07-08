// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Credential checker wrapper that does not require valid credentials.

use kernel::process_checker::{AppCredentialsPolicy, AppCredentialsPolicyClient};
use kernel::ErrorCode;
use tock_tbf::types::TbfFooterV2Credentials;

/// Wrapper struct that changes the return value from
/// `require_credentials()` but otherwise just passes through calls and
/// callbacks.
pub struct AppCheckerCredentialsNotRequired<
    'a,
    C: kernel::process_checker::AppCredentialsPolicy<'static>,
> {
    checker: &'a C,
}

impl<'a, C: kernel::process_checker::AppCredentialsPolicy<'static>>
    AppCheckerCredentialsNotRequired<'a, C>
{
    pub fn new(checker: &'a C) -> AppCheckerCredentialsNotRequired<'a, C> {
        Self { checker }
    }
}

impl<C: kernel::process_checker::AppCredentialsPolicy<'static>> AppCredentialsPolicy<'static>
    for AppCheckerCredentialsNotRequired<'_, C>
{
    fn require_credentials(&self) -> bool {
        false
    }

    fn check_credentials(
        &self,
        credentials: TbfFooterV2Credentials,
        binary: &'static [u8],
    ) -> Result<(), (ErrorCode, TbfFooterV2Credentials, &'static [u8])> {
        self.checker.check_credentials(credentials, binary)
    }

    fn set_client(&self, client: &'static dyn AppCredentialsPolicyClient<'static>) {
        self.checker.set_client(client);
    }
}
