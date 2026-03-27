// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::cell::Cell;
use core::ptr::{read_volatile, write_volatile};

use kernel::hil::uart;
use kernel::utilities::cells::OptionalCell;
use kernel::ErrorCode;

use crate::svd_addrs::{GPIOA_BASE, RCC_BASE, USART1_BASE};

pub struct Usart<'a> {
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    configured: Cell<bool>,
}

impl<'a> Usart<'a> {
    const RCC_CR_OFFSET: usize = 0x000;
    const RCC_CFGR1_OFFSET: usize = 0x01C;
    const RCC_AHB2ENR1_OFFSET: usize = 0x08C;
    const RCC_APB2ENR_OFFSET: usize = 0x0A4;
    const RCC_CCIPR1_OFFSET: usize = 0x0E0;

    const GPIO_MODER_OFFSET: usize = 0x00;
    const GPIO_OSPEEDR_OFFSET: usize = 0x08;
    const GPIO_AFRH_OFFSET: usize = 0x24;

    const USART_CR1_OFFSET: usize = 0x00;
    const USART_BRR_OFFSET: usize = 0x0C;
    const USART_ISR_OFFSET: usize = 0x1C;
    const USART_TDR_OFFSET: usize = 0x28;

    const RCC_GPIOAEN_BIT: u32 = 1 << 0;
    const RCC_USART1EN_BIT: u32 = 1 << 14;
    const RCC_CR_HSION_BIT: u32 = 1 << 8;
    const RCC_CR_HSIRDY_BIT: u32 = 1 << 10;

    const USART_CR1_UE_BIT: u32 = 1 << 0;
    const USART_CR1_RE_BIT: u32 = 1 << 2;
    const USART_CR1_TE_BIT: u32 = 1 << 3;

    const USART_ISR_TXFNF_BIT: u32 = 1 << 7;

    const HSI16_HZ: u32 = 16_000_000;

    pub const fn new() -> Self {
        Self {
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            configured: Cell::new(false),
        }
    }

    fn mmio_read32(addr: usize) -> u32 {
        // ### Safety
        // `addr` is always one of the known RCC/GPIO/USART register addresses
        // derived from the STM32U545 SVD, and is 32-bit aligned.
        unsafe { read_volatile(addr as *const u32) }
    }

    fn mmio_write32(addr: usize, val: u32) {
        // ### Safety
        // `addr` is always one of the known RCC/GPIO/USART register addresses
        // derived from the STM32U545 SVD, and is 32-bit aligned.
        unsafe { write_volatile(addr as *mut u32, val) }
    }

    fn setup_usart1(&self, baud_rate: u32) -> Result<(), ErrorCode> {
        if baud_rate == 0 {
            return Err(ErrorCode::INVAL);
        }

        self.setup_hsi16_sysclk();

        let rcc_cfgr1 = RCC_BASE + Self::RCC_CFGR1_OFFSET;
        let rcc_ahb2enr1 = RCC_BASE + Self::RCC_AHB2ENR1_OFFSET;
        let rcc_apb2enr = RCC_BASE + Self::RCC_APB2ENR_OFFSET;
        let rcc_ccipr1 = RCC_BASE + Self::RCC_CCIPR1_OFFSET;
        let gpioa_moder = GPIOA_BASE + Self::GPIO_MODER_OFFSET;
        let gpioa_ospeedr = GPIOA_BASE + Self::GPIO_OSPEEDR_OFFSET;
        let gpioa_afrh = GPIOA_BASE + Self::GPIO_AFRH_OFFSET;
        let usart1_cr1 = USART1_BASE + Self::USART_CR1_OFFSET;
        let usart1_brr = USART1_BASE + Self::USART_BRR_OFFSET;

        // Keep AHB/APB prescalers at /1 while selecting HSI16 as SYSCLK.
        let mut cfgr1 = Self::mmio_read32(rcc_cfgr1);
        cfgr1 &= !(0xF << 8);
        cfgr1 &= !(0x7 << 12);
        cfgr1 &= !(0x7 << 16);
        cfgr1 = (cfgr1 & !0x3) | 0x1;
        Self::mmio_write32(rcc_cfgr1, cfgr1);

        // Wait until SYSCLK source switch is effective (SWS = 0b01 => HSI16).
        while ((Self::mmio_read32(rcc_cfgr1) >> 2) & 0x3) != 0x1 {}

        // Select HSI16 as USART1 kernel clock (USART1SEL = 0b10).
        let mut ccipr1 = Self::mmio_read32(rcc_ccipr1);
        ccipr1 = (ccipr1 & !0x3) | 0x2;
        Self::mmio_write32(rcc_ccipr1, ccipr1);

        let ahb2 = Self::mmio_read32(rcc_ahb2enr1) | Self::RCC_GPIOAEN_BIT;
        Self::mmio_write32(rcc_ahb2enr1, ahb2);

        let apb2 = Self::mmio_read32(rcc_apb2enr) | Self::RCC_USART1EN_BIT;
        Self::mmio_write32(rcc_apb2enr, apb2);

        let mut moder = Self::mmio_read32(gpioa_moder);
        moder &= !(0b11 << (9 * 2));
        moder &= !(0b11 << (10 * 2));
        moder |= 0b10 << (9 * 2);
        moder |= 0b10 << (10 * 2);
        Self::mmio_write32(gpioa_moder, moder);

        // Use very high speed on PA9/PA10 to improve signal quality to ST-LINK VCP.
        let mut ospeedr = Self::mmio_read32(gpioa_ospeedr);
        ospeedr &= !(0b11 << (9 * 2));
        ospeedr &= !(0b11 << (10 * 2));
        ospeedr |= 0b11 << (9 * 2);
        ospeedr |= 0b11 << (10 * 2);
        Self::mmio_write32(gpioa_ospeedr, ospeedr);

        let mut afrh = Self::mmio_read32(gpioa_afrh);
        afrh &= !(0xF << ((9 - 8) * 4));
        afrh &= !(0xF << ((10 - 8) * 4));
        afrh |= 0x7 << ((9 - 8) * 4);
        afrh |= 0x7 << ((10 - 8) * 4);
        Self::mmio_write32(gpioa_afrh, afrh);

        Self::mmio_write32(usart1_cr1, 0);
        let brr = (Self::HSI16_HZ + (baud_rate / 2)) / baud_rate;
        Self::mmio_write32(usart1_brr, brr);
        Self::mmio_write32(
            usart1_cr1,
            Self::USART_CR1_UE_BIT | Self::USART_CR1_TE_BIT | Self::USART_CR1_RE_BIT,
        );

        Ok(())
    }

    fn write_byte_blocking(&self, byte: u8) {
        let isr_addr = USART1_BASE + Self::USART_ISR_OFFSET;
        let tdr_addr = USART1_BASE + Self::USART_TDR_OFFSET;

        while (Self::mmio_read32(isr_addr) & Self::USART_ISR_TXFNF_BIT) == 0 {}

        Self::mmio_write32(tdr_addr, u32::from(byte));
    }

    fn setup_hsi16_sysclk(&self) {
        let rcc_cr = RCC_BASE + Self::RCC_CR_OFFSET;

        let cr = Self::mmio_read32(rcc_cr) | Self::RCC_CR_HSION_BIT;
        Self::mmio_write32(rcc_cr, cr);
        while (Self::mmio_read32(rcc_cr) & Self::RCC_CR_HSIRDY_BIT) == 0 {}
    }

    pub fn early_boot_print(msg: &str) {
        let uart = Self::new();
        let _ = uart.setup_usart1(115_200);
        for byte in msg.as_bytes() {
            uart.write_byte_blocking(*byte);
        }
    }

    pub fn handle_interrupt(&self) {
        // RX/TX interrupt-driven paths are not needed for initial serial boot.
    }
}

impl uart::Configure for Usart<'_> {
    fn configure(&self, params: uart::Parameters) -> Result<(), ErrorCode> {
        self.setup_usart1(params.baud_rate)?;
        self.configured.set(true);
        Ok(())
    }
}

impl<'a> uart::Transmit<'a> for Usart<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if !self.configured.get() {
            return Err((ErrorCode::OFF, tx_buffer));
        }

        if tx_len > tx_buffer.len() {
            return Err((ErrorCode::SIZE, tx_buffer));
        }

        for byte in tx_buffer.iter().take(tx_len) {
            self.write_byte_blocking(*byte);
        }

        self.tx_client.map(|client| {
            client.transmitted_buffer(tx_buffer, tx_len, Ok(()));
        });
        Ok(())
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        if !self.configured.get() {
            return Err(ErrorCode::OFF);
        }

        self.write_byte_blocking((word & 0xFF) as u8);

        self.tx_client.map(|client| client.transmitted_word(Ok(())));
        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}

impl<'a> uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let _ = self.rx_client;
        Err((ErrorCode::NOSUPPORT, rx_buffer))
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Ok(())
    }
}
