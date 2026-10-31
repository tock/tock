// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Bit-banged driver for WS2812-compatible RGB LEDs (WS2812, SK68XX).

use esp32::gpio::GpioPin;

/// Send one 24-bit GRB frame on `pin`, after holding it low for 300 us to latch
/// the previous frame. Assumes a 160 MHz CPU clock.
///
/// Every frame starts with a reset, so this can only drive the first LED of a
/// chain of LEDs.
// This can be easily extended to support a chain of LEDs by having both this
// function and `write_frame_asm` take a slice of grb values. Don't try to
// support chains by splitting the reset logic out of the asm and calling
// multiple of these `_asm` routines (one per LED), as that again introduces
// variable timing dependent on Rust's compiler and XIP flash. We don't do this
// right now as it makes the asm more complicated than it needs to be.
pub fn write_frame(pin: &GpioPin, grb: u32) {
    write_frame_asm(pin.mask(), grb);
}

/// Send one 24-bit GRB frame on all of the GPIO pins in `mask`, after holding
/// them low for 300 us.
///
/// Layout of the frame:
/// - 8-bit green value in bits 23 downto 16
/// - 8-bit red value in bits 15 downto 8
/// - 8-bit blue value in bits 7 downto 0
///
/// Every frame starts with a reset, so this can only drive the first LED of a
/// chain of LEDs.
///
/// The LED decodes each bit from the width of a high pulse, and tolerates only
/// about 150 ns of error. A cache miss on the XIP flash or a trap stretches a
/// pulse by much more. Therefore, this function executes from IRAM and disables
/// interrupts (clears `mstatus.MIE`) for the duration of the frame (about 30
/// us). It times the pulses with the `mpccr` cycle counter, and restores
/// `mstatus.MIE` to the previous value before it returns.
///
/// ## `naked`
///
/// The assembly of this function must execute from IRAM, as a cache miss would
/// stretch the pulses. A regular function containing an `asm!` block does not
/// guarantee this: the compiler may inline or outline it into code that
/// executes from flash, which would defeat its `link_section`. A naked function
/// is emitted as exactly its assembly, under its own symbol, and is never
/// inlined. It follows the standard C calling convention and can be invoked
/// directly from Rust.
///
/// The attribute is unsafe because the compiler cannot check the assembly. It
/// only clobbers caller-saved registers, does not use the stack, writes no
/// memory other than the `GPIO_OUT_W1TS` and `GPIO_OUT_W1TC` registers, and
/// leaves `mstatus.MIE` as it found it. Thus, calling it is safe.
///
/// ## `link_section`
///
/// This function must be located entirely in SRAM, not reference the external
/// SPI flash, and be executed from the instruction bus alias of SRAM memory
/// (IRAM). We use the `link_section` attribute to place this function into
/// `.iram.esp32_c3_ws2812_write_frame`. The linker script is responsible for
/// ensuring that this function is located in IRAM.
///
/// The attribute is unsafe because executing this function is only sound once
/// IRAM is bootstrapped from flash. Callers need not ensure this: the chip's
/// entry stub ([`crate::esp32_c3_direct_boot_entry`]) copies the `.iram`
/// section into the `iram` region before any Rust code runs, and the board's
/// linker script places the `iram` region above the kernel stack and outside
/// of the `ram` region, so no code writes it afterwards.
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[unsafe(link_section = ".iram.esp32_c3_ws2812_write_frame")]
#[unsafe(naked)]
extern "C" fn write_frame_asm(mask: u32, grb: u32) {
    core::arch::naked_asm!(
        "
    // Configure the performance counter `mpccr` to count CPU cycles, and to
    // wrap around rather than saturate. We time all pulses against it.
    li   t0, 1                    // t0 = 1
    csrw {mpcer}, t0              // mpcer = 1 (count CPU cycles)
    csrw {mpcmr}, t0              // mpcmr = 1 (enable, don't saturate)

    // Drive the line low.
    li   t1, {w1ts}               // t1 = &GPIO_OUT_W1TS
    li   t2, {w1tc}               // t2 = &GPIO_OUT_W1TC
    sw   a0, 0(t2)                // Drive the line low.

    // Hold it low for the reset time, which latches the previous frame. This
    // logic tolerates cycle count wraparounds.
    csrr a2, {mpccr}              // a2 = mpccr (current cycle count)
    li   t0, {reset_cpu_cycles}   // t0 = reset time
    add  a2, a2, t0               // a2 = end of the pause
2:  csrr t0, {mpccr}              // t0 = mpccr
    sub  t0, t0, a2               // t0 = mpccr - end of the pause
    bltz t0, 2b                   // Loop until the pause has passed.

    // Prepare the bit loop. This routine uses a trick: it reads the `grb` value
    // bit-by-bit in a loop, and needs to wait for different amounts of time
    // depending on whether or not that bit is 0 or 1. An efficient way to do
    // this on RISC-V is to make sure the bit being tested is always the
    // top-most bit, and then we can branch on whether the value is greater than
    // or equal to 0 (`bgez`). So, here we move the frame's first bit (bit 23)
    // into bit 31.
    slli a1, a1, 8                // a1 = grb << 8 (first bit in bit 31)

    // Then, we disable interrupts for the frame:
    csrrci t3, mstatus, 0x8       // t3 = mstatus, then clear mstatus.MIE

    // Set up the loop counter:
    li   a3, 24                   // a3 = number of remaining bits

    // Now, record the start time of the first bit:
    csrr a2, {mpccr}              // a2 = start of the current bit

    // Send the 24 bits of the three `grb` bytes, one per iteration, MSB first.
    // Each bit starts exactly `bit_cpu_cycles` after the previous one (`a2`),
    // *not* when the previous iteration finishes. Thus, the cycles spent on the
    // loop's own instructions do not add up over the frame.
3:  sw   a0, 0(t1)                // Drive the line high.
    li   t0, {t0h_cpu_cycles}     // t0 = high time of a `0` bit
    bgez a1, 4f                   // Keep it if the current bit is `0`.
    li   t0, {t1h_cpu_cycles}     // t0 = high time of a `1` bit

    // High pulse inner loop:
4:  add  a4, a2, t0               // a4 = end of the high pulse
5:  csrr t0, {mpccr}              // t0 = mpccr
    sub  t0, t0, a4               // t0 = mpccr - end of the high pulse
    bltz t0, 5b                   // Loop until the high pulse has passed.

    // High pulse has concluded, drive line low now and load low-pulse loop
    // parameters:
    sw   a0, 0(t2)                // Drive the line low.
    addi a2, a2, {bit_cpu_cycles} // a2 = start of the next bit
    slli a1, a1, 1                // a1 = next bit in bit 31
    addi a3, a3, -1               // a3 -= 1

    // Low pulse inner loop:
6:  csrr t0, {mpccr}              // t0 = mpccr
    sub  t0, t0, a2               // t0 = mpccr - start of the next bit
    bltz t0, 6b                   // Loop until the bit period has passed.

    // Bit written out over the wire, outer loop:
    bnez a3, 3b                   // Send the next bit, if any.

    // Done writing all 24 bits. Restore mstatus.MIE, and return.
    andi t3, t3, 0x8              // t3 = previous mstatus.MIE
    csrs mstatus, t3              // set mstatus.MIE if it was set
    ret
        ",
        // CPU cycles at 160 MHz. A `0` bit is high for 300 ns, a `1` bit for
        // 700 ns, and each bit takes 1.25 us. The SK68XX-MINI-HS latches a
        // frame after 80 us low, WS2812B-V5 and newer parts need 280 us.
        reset_cpu_cycles = const 48_000, // 300 us * 160 MHz = 48000 cycles
        t0h_cpu_cycles = const 48,       // 300 ns * 160 MHz = 48 cycles
        t1h_cpu_cycles = const 112,      // 700 ns * 160 MHz = 112 cycles
        bit_cpu_cycles = const 200,      // 1.25 us * 160 MHz = 200 cycles
        // Espressif's machine performance counter CSRs, described in the
        // ESP32-C3 Technical Reference Manual (CPU chapter). ESP-IDF names them
        // `CSR_PCER_MACHINE`, `CSR_PCMR_MACHINE` and `CSR_PCCR_MACHINE`.
        //
        // `mpcer`: event selection. Each set bit selects one event to count;
        // bit 0 selects CPU cycles, other bits select e.g. retired
        // instructions, loads, stores or taken branches.
        mpcer = const 0x7e0,
        // `mpcmr`: mode. Bit 0 enables the counter. Bit 1 makes it saturate at
        // `u32::MAX` instead of wrapping around to 0.
        mpcmr = const 0x7e1,
        // `mpccr`: the 32-bit count of the selected events. It is writable,
        // but we only read it and compare the difference of two readings.
        mpccr = const 0x7e2,
        // `GPIO_OUT_W1TS` / `GPIO_OUT_W1TC`, shared by all pins; `mask` selects
        // the pin.
        w1ts = const 0x6000_4008,
        w1tc = const 0x6000_400C,
    );
}

/// Mock implementation for static checks and CI.
#[cfg(not(all(target_arch = "riscv32", target_os = "none")))]
extern "C" fn write_frame_asm(_mask: u32, _grb: u32) {
    unimplemented!()
}
