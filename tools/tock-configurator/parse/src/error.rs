// Copyright OxidOS Automotive 2024.

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
