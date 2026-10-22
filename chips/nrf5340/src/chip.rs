// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use core::fmt::Write;
use cortexm33::{CortexM33, CortexMVariant, nvic};
use kernel::platform::chip::InterruptService;
use kernel::utilities::StaticRef;

pub const SPU_BASE: StaticRef<crate::spu::SpuRegisters> =
    unsafe { StaticRef::new(0x50003000 as *const crate::spu::SpuRegisters) };

/// Peripheral base addresses as seen from one security state.
///
/// Secure code uses the secure aliases. Non-secure code uses the non-secure
/// aliases, 0x1000_0000 lower, of the peripherals the SPU makes non-secure,
/// and GPIOTE1, as GPIOTE0 is secure only.
pub struct PeripheralBases {
    pub clock: StaticRef<crate::clock::ClockRegisters>,
    pub gpio: [StaticRef<crate::gpio::GpioRegisters>; 2],
    pub gpiote: StaticRef<crate::gpio::GpioteRegisters>,
    pub ipc: StaticRef<crate::ipc::IpcRegisters>,
    pub oscillators: StaticRef<crate::lfxo::OscillatorsRegisters>,
    pub pin_cnf: [StaticRef<crate::network::PinCnfRegisters>; 2],
    pub reset: StaticRef<crate::network::ResetRegisters>,
    pub rtc1: StaticRef<crate::rtc::RtcRegisters>,
    pub saadc: StaticRef<crate::adc::AdcRegisters>,
    pub spim1: StaticRef<crate::spi::SpimRegisters>,
    pub uarte0: StaticRef<crate::uarte::UarteRegisters>,
    pub usbd: StaticRef<crate::usbd::UsbdRegisters<'static>>,
    pub usbreg: StaticRef<crate::usbreg::UsbRegRegisters>,
}

impl PeripheralBases {
    /// The bases for `offset` below the secure aliases, with `gpiote`.
    const fn at(offset: usize, gpiote: usize) -> Self {
        // SAFETY: These are the nRF5340 application core peripheral
        // addresses, in the alias that `offset` selects.
        unsafe {
            Self {
                clock: StaticRef::new((0x50005000 - offset) as *const _),
                gpio: [
                    StaticRef::new((0x50842000 - offset) as *const _),
                    StaticRef::new((0x50842300 - offset) as *const _),
                ],
                gpiote: StaticRef::new(gpiote as *const _),
                ipc: StaticRef::new((0x5002A000 - offset) as *const _),
                oscillators: StaticRef::new((0x50004000 - offset) as *const _),
                pin_cnf: [
                    StaticRef::new((0x50842000 - offset) as *const _),
                    StaticRef::new((0x50842300 - offset) as *const _),
                ],
                reset: StaticRef::new((0x50005000 - offset) as *const _),
                rtc1: StaticRef::new((0x50015000 - offset) as *const _),
                saadc: StaticRef::new((0x5000E000 - offset) as *const _),
                spim1: StaticRef::new((0x50009000 - offset) as *const _),
                uarte0: StaticRef::new((0x50008000 - offset) as *const _),
                usbd: StaticRef::new((0x50036000 - offset) as *const _),
                usbreg: StaticRef::new((0x50037000 - offset) as *const _),
            }
        }
    }
}

pub const SECURE_BASES: PeripheralBases = PeripheralBases::at(0, 0x5000D000);
pub const NONSECURE_BASES: PeripheralBases = PeripheralBases::at(0x1000_0000, 0x4002F000);

/// The peripheral base addresses for the security state this code runs in.
pub fn bases() -> &'static PeripheralBases {
    match cortexm33::trustzone::is_secure() {
        true => &SECURE_BASES,
        false => &NONSECURE_BASES,
    }
}

pub struct Nrf5340<'a, I: InterruptService + 'a> {
    mpu: cortexm33::mpu::MPU<8>,
    userspace_kernel_boundary: cortexm33::syscall::SysCall,
    interrupt_service: &'a I,
}

impl<'a, I: InterruptService + 'a> Nrf5340<'a, I> {
    pub fn new(interrupt_service: &'a I) -> Self {
        // SAFETY: The nRF5340 application core is a Cortex-M33 with an MPU.
        let mpu = unsafe { cortexm33::mpu::new::<8>() };

        Self {
            mpu,
            userspace_kernel_boundary: cortexm33::syscall::SysCall::new(),
            interrupt_service,
        }
    }
}

/// This struct, when initialized, instantiates all peripheral drivers for the
/// nRF5340 application core.
pub struct Nrf5340DefaultPeripherals<'a> {
    pub adc: crate::adc::Adc<'a>,
    pub clock: crate::clock::Clock,
    pub gpio_port: crate::gpio::Port<'a, { crate::gpio::NUM_PINS }>,
    pub ipc: crate::ipc::Ipc<'a>,
    pub lfxo: crate::lfxo::Lfxo,
    pub network: crate::network::NetworkCore,
    pub rtc: crate::rtc::Rtc<'a>,
    pub spim1: crate::spi::SPIM<'a>,
    pub spu: crate::spu::Spu,
    pub uarte0: crate::uarte::Uarte<'a>,
    pub usbd: crate::usbd::Usbd<'a>,
    pub usbreg: crate::usbreg::UsbReg<'a>,
}

impl Nrf5340DefaultPeripherals<'_> {
    /// Create default peripherals for the nRF5340 application core, at the
    /// addresses for the security state this code runs in. In non-secure
    /// state, a secure stage must already have made these peripherals, their
    /// interrupts and the GPIO pins non-secure.
    ///
    /// # Safety
    ///
    /// Some of these peripherals use DMA. As such, the default peripherals must
    /// be unique. This requires:
    ///
    /// - This constructor must be called at most once.
    /// - There must not be additional instances of the DMA-enabled peripheral
    ///   drivers.
    /// - There must not be any other code that accesses the DMA buffer and
    ///   length registers of the DMA-enabled peripherals.
    pub unsafe fn new() -> Self {
        let bases = bases();

        // SAFETY: See function-level doc.
        let uarte0_registers = unsafe { crate::uarte::UarteRegistersManager::new(bases.uarte0) };

        Self {
            // SAFETY: See function-level doc.
            adc: crate::adc::Adc::new(bases.saadc, 3300, unsafe { crate::adc::AdcSample::new() }),
            clock: crate::clock::Clock::new(bases.clock),
            gpio_port: crate::gpio::nrf5340_gpio_create(bases.gpiote, bases.gpio),
            ipc: crate::ipc::Ipc::new(bases.ipc),
            lfxo: crate::lfxo::Lfxo::new(bases.oscillators, bases.pin_cnf[0]),
            network: crate::network::NetworkCore::new(bases.reset, bases.pin_cnf),
            rtc: crate::rtc::Rtc::new(bases.rtc1),
            spim1: crate::spi::SPIM::new(bases.spim1),
            spu: crate::spu::Spu::new(SPU_BASE),
            uarte0: crate::uarte::Uarte::new(uarte0_registers),
            usbd: crate::usbd::Usbd::new(bases.usbd, None),
            usbreg: crate::usbreg::UsbReg::new(bases.usbreg),
        }
    }

    // Necessary for setting up circular dependencies
    pub fn init(&'static self) {
        self.usbreg.set_usbd(&self.usbd);
        self.usbd.set_power_ref(&self.usbreg);
    }
}

impl InterruptService for Nrf5340DefaultPeripherals<'_> {
    fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            crate::peripheral_interrupts::SAADC => self.adc.handle_interrupt(),
            crate::peripheral_interrupts::GPIOTE0 | crate::peripheral_interrupts::GPIOTE1 => {
                self.gpio_port.handle_interrupt()
            }
            crate::peripheral_interrupts::IPC => self.ipc.handle_interrupt(),
            crate::peripheral_interrupts::RTC1 => self.rtc.handle_interrupt(),
            crate::peripheral_interrupts::SERIAL0 => self.uarte0.handle_interrupt(),
            crate::peripheral_interrupts::SERIAL1 => self.spim1.handle_interrupt(),
            crate::peripheral_interrupts::USBD => self.usbd.handle_interrupt(),
            crate::peripheral_interrupts::USBREGULATOR => self.usbreg.handle_interrupt(),
            _ => return false,
        }
        true
    }
}

impl<'a, I: InterruptService + 'a> kernel::platform::chip::Chip for Nrf5340<'a, I> {
    type MPU = cortexm33::mpu::MPU<8>;
    type UserspaceKernelBoundary = cortexm33::syscall::SysCall;
    type ThreadIdProvider = cortexm33::thread_id::CortexMThreadIdProvider;

    fn init() {
        // In non-secure state, the secure stage applied the errata.
        if cortexm33::trustzone::is_secure() {
            // # Safety
            //
            // We are setting up the nRF5340 application core in secure state,
            // so these memory locations are valid and this is OK to call.
            unsafe {
                crate::crt1::fix_errata();
            }
        }
        crate::crt1::initialize_vector_table();

        nvic::enable_all();
    }

    fn mpu(&self) -> &Self::MPU {
        &self.mpu
    }

    fn userspace_kernel_boundary(&self) -> &Self::UserspaceKernelBoundary {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        while let Some(interrupt) = nvic::next_pending() {
            if !self.interrupt_service.service_interrupt(interrupt) {
                panic!("unhandled interrupt {}", interrupt);
            }
            let n = nvic::Nvic::new(interrupt);
            n.clear_pending();
            n.enable();
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        nvic::has_pending()
    }

    fn sleep(&self) {
        unsafe {
            cortexm33::support::wfi();
        }
    }

    fn with_interrupts_disabled<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm33::support::with_interrupts_disabled(f)
    }

    unsafe fn print_state(_this: Option<&Self>, write: &mut dyn Write) {
        CortexM33::print_cortexm_state(write);
    }
}
