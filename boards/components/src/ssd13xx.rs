// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for the SSD1306/SSD1315 OLED screens.
//!
//! Usage
//! -----
//! ```rust
//! // I2C
//! let ssd1306_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
//!     .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));
//!
//! let ssd1306 = components::ssd13xx::Ssd13xxI2CComponent::new(ssd1306_i2c, true)
//!     .finalize(components::ssd13xx_i2c_component_static!(nrf52840::i2c::TWI));
//!
//! // 4-wire SPI.
//! let ssd1306 = components::ssd13xx::Ssd13xxSpiComponent::new(
//!     &peripherals.spi,
//!     chip_select,
//!     &peripherals.gpio[3], // D/C̅
//!     8_000_000,
//!     true,
//! )
//! .finalize(components::ssd13xx_spi_component_static!(
//!     esp32_c3::spi::Spi,
//!     esp32::gpio::GpioPin,
//! ));
//! ```

use capsules_core::virtualizers::virtual_i2c::I2CDevice;
use capsules_extra::ssd13xx::{I2CTransport, SpiTransport, Ssd13xx, Transport};
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::i2c::I2CDevice as _;

// Setup static space for the objects.
#[macro_export]
macro_rules! ssd13xx_i2c_component_static {
    ($I: ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::ssd13xx::BUFFER_SIZE]);
        let transport = kernel::static_buf!(
            capsules_extra::ssd13xx::I2CTransport<
                'static,
                capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
            >
        );
        let ssd13xx = kernel::static_buf!(
            capsules_extra::ssd13xx::Ssd13xx<
                'static,
                capsules_extra::ssd13xx::I2CTransport<
                    'static,
                    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
                >,
            >
        );

        (buffer, transport, ssd13xx)
    }};
}

#[macro_export]
macro_rules! ssd13xx_spi_component_static {
    ($S: ty, $P: ty $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::ssd13xx::BUFFER_SIZE]);
        let transport = kernel::static_buf!(capsules_extra::ssd13xx::SpiTransport<'static, $S, $P>);
        let ssd13xx = kernel::static_buf!(
            capsules_extra::ssd13xx::Ssd13xx<
                'static,
                capsules_extra::ssd13xx::SpiTransport<'static, $S, $P>,
            >
        );

        (buffer, transport, ssd13xx)
    }};
}

pub type Ssd13xxI2CComponentType<I> =
    Ssd13xx<'static, I2CTransport<'static, I2CDevice<'static, I>>>;

pub type Ssd13xxSpiComponentType<S, P> = Ssd13xx<'static, SpiTransport<'static, S, P>>;

pub struct Ssd13xxI2CComponent<I: hil::i2c::I2CMaster<'static> + 'static> {
    i2c_device: &'static I2CDevice<'static, I>,
    use_charge_pump: bool,
}

impl<I: hil::i2c::I2CMaster<'static> + 'static> Ssd13xxI2CComponent<I> {
    pub fn new(i2c_device: &'static I2CDevice<'static, I>, use_charge_pump: bool) -> Self {
        Ssd13xxI2CComponent {
            i2c_device,
            use_charge_pump,
        }
    }
}

impl<I: hil::i2c::I2CMaster<'static> + 'static> Component for Ssd13xxI2CComponent<I> {
    type StaticInput = (
        &'static mut MaybeUninit<[u8; capsules_extra::ssd13xx::BUFFER_SIZE]>,
        &'static mut MaybeUninit<I2CTransport<'static, I2CDevice<'static, I>>>,
        &'static mut MaybeUninit<Ssd13xxI2CComponentType<I>>,
    );
    type Output = &'static Ssd13xxI2CComponentType<I>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .0
            .write([0; capsules_extra::ssd13xx::BUFFER_SIZE]);

        let transport = static_buffer.1.write(I2CTransport::new(self.i2c_device));
        self.i2c_device.set_client(transport);

        let ssd13xx = static_buffer
            .2
            .write(Ssd13xx::new(transport, buffer, self.use_charge_pump));
        transport.set_client(ssd13xx);

        ssd13xx
    }
}

pub struct Ssd13xxSpiComponent<
    S: hil::spi::SpiMaster<'static> + 'static,
    P: hil::gpio::Pin + 'static,
> {
    spi: &'static S,
    chip_select: S::ChipSelect,
    dc: &'static P,
    rate: u32,
    use_charge_pump: bool,
}

impl<S: hil::spi::SpiMaster<'static> + 'static, P: hil::gpio::Pin + 'static>
    Ssd13xxSpiComponent<S, P>
{
    /// `dc` is the data/command select pin, `rate` the SPI clock in Hz.
    pub fn new(
        spi: &'static S,
        chip_select: S::ChipSelect,
        dc: &'static P,
        rate: u32,
        use_charge_pump: bool,
    ) -> Self {
        Ssd13xxSpiComponent {
            spi,
            chip_select,
            dc,
            rate,
            use_charge_pump,
        }
    }
}

impl<S: hil::spi::SpiMaster<'static> + 'static, P: hil::gpio::Pin + 'static> Component
    for Ssd13xxSpiComponent<S, P>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; capsules_extra::ssd13xx::BUFFER_SIZE]>,
        &'static mut MaybeUninit<SpiTransport<'static, S, P>>,
        &'static mut MaybeUninit<Ssd13xxSpiComponentType<S, P>>,
    );
    type Output = &'static Ssd13xxSpiComponentType<S, P>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .0
            .write([0; capsules_extra::ssd13xx::BUFFER_SIZE]);

        if let Err(error) = self.spi.init() {
            panic!("SPI init failed ({:?})", error);
        }
        // SPI mode 0: the controller samples on the rising clock edge.
        let configured = self
            .spi
            .specify_chip_select(self.chip_select)
            .and(self.spi.set_rate(self.rate).map(|_| ()))
            .and(self.spi.set_polarity(hil::spi::ClockPolarity::IdleLow))
            .and(self.spi.set_phase(hil::spi::ClockPhase::SampleLeading));
        if let Err(error) = configured {
            panic!("SPI configuration failed ({:?})", error);
        }

        self.dc.make_output();

        let transport = static_buffer.1.write(SpiTransport::new(self.spi, self.dc));
        self.spi.set_client(transport);

        let ssd13xx = static_buffer
            .2
            .write(Ssd13xx::new(transport, buffer, self.use_charge_pump));
        transport.set_client(ssd13xx);

        ssd13xx
    }
}
