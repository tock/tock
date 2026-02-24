// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

use kernel::utilities::registers::{register_bitfields, register_structs, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// General-purpose I/O 0
    GpioRegisters {
        /// Data Register
        (0x000 => data: ReadWrite<u32, DATA::Register>),
        /// Data Output Register
        (0x004 => dataout: ReadWrite<u32, DATAOUT::Register>),
        (0x008 => _reserved0),
        /// Output enable set Register
        (0x010 => outenset: ReadWrite<u32, OUTENSET::Register>),
        /// Output enable clear Register
        (0x014 => outenclr: ReadWrite<u32, OUTENCLR::Register>),
        /// Alternate function set Register
        (0x018 => altfuncset: ReadWrite<u32, ALTFUNCSET::Register>),
        /// Alternate function clear Register
        (0x01C => altfuncclr: ReadWrite<u32, ALTFUNCCLR::Register>),
        /// Interrupt enable set Register
        (0x020 => intenset: ReadWrite<u32, INTENSET::Register>),
        /// Interrupt enable clear Register
        (0x024 => intenclr: ReadWrite<u32, INTENCLR::Register>),
        /// Interrupt type set Register
        (0x028 => inttypeset: ReadWrite<u32, INTTYPESET::Register>),
        /// Interrupt type clear Register
        (0x02C => inttypeclr: ReadWrite<u32, INTTYPECLR::Register>),
        /// Polarity-level, edge interrupt configuration set Register
        (0x030 => intpolset: ReadWrite<u32, INTPOLSET::Register>),
        /// Polarity-level, edge interrupt configuration clear Register
        (0x034 => intpolclr: ReadWrite<u32, INTPOLCLR::Register>),
        /// Interrupt Status Register
        (0x038 => intstatus: ReadOnly<u32, INTSTATUS::Register>),
        (0x03C => @END),
    }
}
register_bitfields![u32,
DATA [
    VALUE OFFSET (0) NUMBITS (32) []
],
DATAOUT [
    VALUE OFFSET (0) NUMBITS (32) []
],
OUTENSET [
    VALUE OFFSET (0) NUMBITS (32) []
],
OUTENCLR [
    VALUE OFFSET (0) NUMBITS (32) []
],
ALTFUNCSET [
    VALUE OFFSET (0) NUMBITS (32) []
],
ALTFUNCCLR [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTENSET [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTENCLR [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTTYPESET [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTTYPECLR [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTPOLSET [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTPOLCLR [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTSTATUS [
    VALUE OFFSET (0) NUMBITS (32) []
],
INTCLEAR [
    VALUE OFFSET (0) NUMBITS (32) []
]
];

const GPIO_BASE_SEC: StaticRef<GpioRegisters> =
    unsafe { StaticRef::new(0x51000000 as *const GpioRegisters) };
// "GPIO can only be accessed by Secure Privileged access. Non-secure privileged access is not possible."
