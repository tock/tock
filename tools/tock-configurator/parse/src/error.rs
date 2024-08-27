// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

/// Errors that can occur in either the configuration or the generation process.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Pin {} already in use.", .0)]
    PinInUse(String),
    #[error("Peripheral {} already in use.", .0)]
    PeripheralInUse(String),
    #[error("Peripheral not supported.")]
    NoSupport,
    #[error("Component code not fully provided.")]
    CodeNotProvided,
}
