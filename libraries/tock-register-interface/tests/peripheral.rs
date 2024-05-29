// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.

#![no_implicit_prelude]

// Verifies that putting #[cfg()] annotations on peripheral definitions works
// correctly.
::tock_registers::peripheral! {
    #[cfg(any())]
    CfgNoCompile {
        0x0 => should_not_compile: DoesNotExist {},
    }
}

// Verifies that putting #[cfg()] annotations on fields works correctly.
::tock_registers::peripheral! {
    CfgNoField {
        #[cfg(any())]
        0x0 => should_not_compile: DoesNotExist {},
    }
}

::tock_registers::peripheral! {
    Foo2 {
    }
}
