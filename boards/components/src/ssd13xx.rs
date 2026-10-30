// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Components for the SSD1306/SSD1309/SSD1315 OLED screens.
//!
//! The panel resolution (width, height) is passed as the last two arguments
//! to the `_component_static!` macros.
//!
//! Usage
//! -----
//! ```rust
//! // I2C
//! let ssd1306_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
//!     .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));
//!
//! let ssd1306 = components::ssd13xx::Ssd13xxI2CComponent::new(
//!     ssd1306_i2c,
//!     true,
//!     capsules_extra::ssd13xx::PanelConfig::default(),
//! )
//! .finalize(components::ssd13xx_i2c_component_static!(
//!     nrf52840::i2c::TWI,
//!     capsules_extra::ssd13xx::Ssd1306,
//!     128,
//!     64,
//! ));
//!
//! // 4-wire SPI, on a 128x32 panel.
//! let ssd1309 = components::ssd13xx::Ssd13xxSpiComponent::new(
//!     &peripherals.spi,
//!     chip_select,
//!     &peripherals.gpio[3], // D/C̅
//!     8_000_000,
//!     false,
//!     capsules_extra::ssd13xx::PanelConfig {
//!         com_pins_alternative: false,
//!         ..capsules_extra::ssd13xx::PanelConfig::default()
//!     },
//! )
//! .finalize(components::ssd13xx_spi_component_static!(
//!     esp32_c3::spi::Spi,
//!     esp32::gpio::GpioPin,
//!     capsules_extra::ssd13xx::Ssd1309,
//!     128,
//!     32,
//! ));
//! ```

use capsules_core::virtualizers::virtual_i2c::I2CDevice;
use capsules_extra::ssd13xx::{
    I2CTransport, PanelConfig, SpiTransport, Ssd13xx, Transport, Variant,
};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::i2c::I2CDevice as _;

// Setup static space for the objects.
#[macro_export]
macro_rules! ssd13xx_i2c_component_static {
    ($I: ty, $V: ty, $WIDTH: expr, $HEIGHT: expr $(,)?) => {{
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
                $V,
                capsules_extra::ssd13xx::I2CTransport<
                    'static,
                    capsules_core::virtualizers::virtual_i2c::I2CDevice<'static, $I>,
                >,
                $WIDTH,
                $HEIGHT,
            >
        );

        (buffer, transport, ssd13xx)
    }};
}

#[macro_export]
macro_rules! ssd13xx_spi_component_static {
    ($S: ty, $P: ty, $V: ty, $WIDTH: expr, $HEIGHT: expr $(,)?) => {{
        let buffer = kernel::static_buf!([u8; capsules_extra::ssd13xx::BUFFER_SIZE]);
        let transport = kernel::static_buf!(capsules_extra::ssd13xx::SpiTransport<'static, $S, $P>);
        let ssd13xx = kernel::static_buf!(
            capsules_extra::ssd13xx::Ssd13xx<
                'static,
                $V,
                capsules_extra::ssd13xx::SpiTransport<'static, $S, $P>,
                $WIDTH,
                $HEIGHT,
            >
        );

        (buffer, transport, ssd13xx)
    }};
}

pub type Ssd13xxI2CComponentType<I, V, const WIDTH: usize, const HEIGHT: usize> =
    Ssd13xx<'static, V, I2CTransport<'static, I2CDevice<'static, I>>, WIDTH, HEIGHT>;

pub type Ssd13xxSpiComponentType<S, P, V, const WIDTH: usize, const HEIGHT: usize> =
    Ssd13xx<'static, V, SpiTransport<'static, S, P>, WIDTH, HEIGHT>;

pub struct Ssd13xxI2CComponent<
    I: hil::i2c::I2CMaster<'static> + 'static,
    V: Variant + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
> {
    i2c_device: &'static I2CDevice<'static, I>,
    use_charge_pump: bool,
    panel: PanelConfig,
    _variant: PhantomData<V>,
}

impl<
    I: hil::i2c::I2CMaster<'static> + 'static,
    V: Variant + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
> Ssd13xxI2CComponent<I, V, WIDTH, HEIGHT>
{
    pub fn new(
        i2c_device: &'static I2CDevice<'static, I>,
        use_charge_pump: bool,
        panel: PanelConfig,
    ) -> Self {
        Ssd13xxI2CComponent {
            i2c_device,
            use_charge_pump,
            panel,
            _variant: PhantomData,
        }
    }
}

impl<
    I: hil::i2c::I2CMaster<'static> + 'static,
    V: Variant + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
> Component for Ssd13xxI2CComponent<I, V, WIDTH, HEIGHT>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; capsules_extra::ssd13xx::BUFFER_SIZE]>,
        &'static mut MaybeUninit<I2CTransport<'static, I2CDevice<'static, I>>>,
        &'static mut MaybeUninit<Ssd13xxI2CComponentType<I, V, WIDTH, HEIGHT>>,
    );
    type Output = &'static Ssd13xxI2CComponentType<I, V, WIDTH, HEIGHT>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let buffer = static_buffer
            .0
            .write([0; capsules_extra::ssd13xx::BUFFER_SIZE]);

        let transport = static_buffer.1.write(I2CTransport::new(self.i2c_device));
        self.i2c_device.set_client(transport);

        let ssd13xx = static_buffer.2.write(Ssd13xx::new(
            transport,
            buffer,
            self.use_charge_pump,
            self.panel,
        ));
        transport.set_client(ssd13xx);

        ssd13xx
    }
}

pub struct Ssd13xxSpiComponent<
    S: hil::spi::SpiMaster<'static> + 'static,
    P: hil::gpio::Pin + 'static,
    V: Variant + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
> {
    spi: &'static S,
    chip_select: S::ChipSelect,
    dc: &'static P,
    rate: u32,
    use_charge_pump: bool,
    panel: PanelConfig,
    _variant: PhantomData<V>,
}

impl<
    S: hil::spi::SpiMaster<'static> + 'static,
    P: hil::gpio::Pin + 'static,
    V: Variant + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
> Ssd13xxSpiComponent<S, P, V, WIDTH, HEIGHT>
{
    /// `dc` is the data/command select pin, `rate` the SPI clock in Hz.
    pub fn new(
        spi: &'static S,
        chip_select: S::ChipSelect,
        dc: &'static P,
        rate: u32,
        use_charge_pump: bool,
        panel: PanelConfig,
    ) -> Self {
        Ssd13xxSpiComponent {
            spi,
            chip_select,
            dc,
            rate,
            use_charge_pump,
            panel,
            _variant: PhantomData,
        }
    }
}

impl<
    S: hil::spi::SpiMaster<'static> + 'static,
    P: hil::gpio::Pin + 'static,
    V: Variant + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
> Component for Ssd13xxSpiComponent<S, P, V, WIDTH, HEIGHT>
{
    type StaticInput = (
        &'static mut MaybeUninit<[u8; capsules_extra::ssd13xx::BUFFER_SIZE]>,
        &'static mut MaybeUninit<SpiTransport<'static, S, P>>,
        &'static mut MaybeUninit<Ssd13xxSpiComponentType<S, P, V, WIDTH, HEIGHT>>,
    );
    type Output = &'static Ssd13xxSpiComponentType<S, P, V, WIDTH, HEIGHT>;

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

        let ssd13xx = static_buffer.2.write(Ssd13xx::new(
            transport,
            buffer,
            self.use_charge_pump,
            self.panel,
        ));
        transport.set_client(ssd13xx);

        ssd13xx
    }
}
