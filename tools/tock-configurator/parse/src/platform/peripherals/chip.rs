// Copyright OxidOS Automotive 2024.

use super::{
    ble::BleAdvertisement, gpio::Gpio, timer::Timer, uart::Uart, Flash, I2c, Rng, Spi, Temperature,
};
use crate::Component;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NoSupport;

impl std::fmt::Display for NoSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Not Supported")
    }
}

impl Component for NoSupport {}

/// The [`DefaultPeripherals`] trait defines a type that contains all of a chip's supported
/// peripherals. For non-supported peripherals, the unit type `()` can serve as the placeholder
/// for the trait item.
pub trait DefaultPeripherals: Component {
    type Uart: Uart + 'static + for<'de> serde::Deserialize<'de> + serde::Serialize;
    type Timer: Timer + 'static + for<'de> serde::Deserialize<'de> + serde::Serialize;
    type Gpio: Gpio + 'static + PartialEq;
    type Spi: Spi + for<'de> serde::Deserialize<'de> + serde::Serialize + 'static;
    type I2c: I2c + for<'de> serde::Deserialize<'de> + serde::Serialize + 'static;
    type BleAdvertisement: BleAdvertisement
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + 'static;

    type Flash: Flash + for<'de> serde::Deserialize<'de> + serde::Serialize + 'static;
    type Temperature: Temperature + for<'de> serde::Deserialize<'de> + serde::Serialize + 'static;
    type Rng: Rng + for<'de> serde::Deserialize<'de> + serde::Serialize + 'static;

    /// Return an array slice of pointers to the `Gpio` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn gpio(&self) -> Result<&[Rc<Self::Gpio>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `Uart` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn uart(&self) -> Result<&[Rc<Self::Uart>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `Timer` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn timer(&self) -> Result<&[Rc<Self::Timer>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `Spi` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn spi(&self) -> Result<&[Rc<Self::Spi>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `I2c` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn i2c(&self) -> Result<&[Rc<Self::I2c>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `BleAdvertisement` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn ble(&self) -> Result<&[Rc<Self::BleAdvertisement>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `Flash` peripherals or a [`crate::Error`]
    /// if the peripheralis is non-existent.
    fn flash(&self) -> Result<&[Rc<Self::Flash>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `Temperature` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn temp(&self) -> Result<&[Rc<Self::Temperature>], crate::Error> {
        Err(crate::Error::NoSupport)
    }

    /// Return an array slice of pointers to the `Rng` peripherals or a [`crate::Error`]
    /// if the peripheral is non-existent.
    fn rng(&self) -> Result<&[Rc<Self::Rng>], crate::Error> {
        Err(crate::Error::NoSupport)
    }
}

/// The [`Chip`] trait defines a type that contains the default peripherals and optionally a systick
/// for the scheduler timer.
pub trait Chip: Component {
    type Peripherals: DefaultPeripherals
        + 'static
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize;
    type Systick: for<'de> serde::Deserialize<'de> + serde::Serialize + 'static + Component;

    /// Return chip prelude code needed before booting the platform.
    /// If this returns Some, it should be called before setting up the platform
    /// and entering main loop.
    fn before_boot(&self) -> Option<proc_macro2::TokenStream> {
        None
    }

    /// Return a pointer to the chip's default peripherals.
    fn peripherals(&self) -> Rc<Self::Peripherals>;

    /// Return a pointer to the chip's systick.
    fn systick(&self) -> Result<Rc<Self::Systick>, crate::Error>;
}
