// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Secure stage for the `nrf5340dk-ns` kernel.
//!
//! The application core starts here, in secure state. This stage applies the
//! nRF5340 startup errata; connects the 32.768 kHz crystal; hands the network
//! core its console pins and releases it; makes the kernel's flash and RAM,
//! the peripherals the kernel uses and all GPIO pins non-secure; sends their
//! interrupts and the BusFault, HardFault and NMI exceptions to non-secure
//! state; then jumps to the kernel.
//!
//! After that, secure code runs only when the kernel calls the CryptoCell
//! [`gateway`]. The SPU, CryptoCell, and the NVMC with the KMU stay secure, so
//! the kernel cannot write flash. The first 8 KB of RAM holds this stage's
//! stack and stays secure.

#![no_std]
#![no_main]

use cortexm33::trustzone;
use nrf5340::chip::{SECURE_BASES, SPU_BASE};
use nrf5340::gpio::Pin;
use nrf5340::lfxo::{Lfxo, LoadCapacitance};
use nrf5340::network::NetworkCore;
use nrf5340::spu::Spu;

// Only the gateway's assembly, built for the chip alone, reaches its
// operations.
#[cfg_attr(not(all(target_arch = "arm", target_os = "none")), allow(dead_code))]
mod gateway;

/// The kernel's vector table, right after this stage's 16 KB.
const KERNEL: usize = 0x4000;
const FLASH_END: usize = 0x10_0000;

/// The peripherals the kernel uses, by ID.
///
/// CLOCK, POWER and RESET (5), UARTE0 (8), SPIM1 (9), SAADC (14), RTC1 (21),
/// IPC (42), USBD (54), USBREGULATOR (55), and GPIO P0 and P1 (66). A
/// peripheral's ID is also its interrupt number.
const PERIPHERALS: [usize; 9] = [5, 8, 9, 14, 21, 42, 54, 55, 66];
/// GPIOTE1 is always non-secure, but its interrupt is not.
const GPIOTE1_INTERRUPT: usize = 47;

/// The network core's console pins.
const NET_UART: [Pin; 2] = [Pin::P1_01, Pin::P1_00];

#[unsafe(no_mangle)]
unsafe extern "C" fn reset_handler() -> ! {
    // SAFETY: This runs on the nRF5340 application core in secure state.
    unsafe { nrf5340::crt1::fix_errata() };

    // The DK's 32.768 kHz crystal, before the network core may select it.
    Lfxo::new(SECURE_BASES.oscillators, SECURE_BASES.pin_cnf[0])
        .connect(LoadCapacitance::Internal7pF);

    // Before RESET becomes non-secure, which faults its secure alias.
    NetworkCore::new(SECURE_BASES.reset, SECURE_BASES.pin_cnf).start(&NET_UART);

    let spu = Spu::new(SPU_BASE);
    spu.make_flash_nonsecure(KERNEL..FLASH_END);
    spu.make_flash_nonsecure_callable(gateway::WINDOW);
    spu.make_ram_nonsecure(gateway::KERNEL_RAM);
    for id in PERIPHERALS {
        spu.make_peripheral_nonsecure(id);
        trustzone::interrupt_target_nonsecure(id);
    }
    trustzone::interrupt_target_nonsecure(GPIOTE1_INTERRUPT);
    spu.make_gpio_nonsecure(0);
    spu.make_gpio_nonsecure(1);
    trustzone::faults_target_nonsecure();
    trustzone::sau_all_nonsecure();

    // SAFETY: The kernel image is at `KERNEL`, and its memory and peripherals
    // are now non-secure.
    unsafe { trustzone::jump_to_nonsecure(KERNEL) }
}

/// Secure faults stop here, where a debugger can find them.
#[unsafe(no_mangle)]
extern "C" fn fault_handler() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    fault_handler()
}
