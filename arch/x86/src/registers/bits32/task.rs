// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

//! Helpers to program the task state segment.
//! See Intel 3a, Chapter 7

use core::mem::size_of;

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    pub link: u16,
    reserved0: u16,
    pub esp0: u32,
    pub ss0: u16,
    reserved1: u16,
    pub esp1: u32,
    pub ss1: u16,
    reserved2: u16,
    pub esp2: u32,
    pub ss2: u16,
    reserved3: u16,

    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,

    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,

    pub es: u16,
    reserved4: u16,
    pub cs: u16,
    reserved5: u16,
    pub ss: u16,
    reserved6: u16,
    pub ds: u16,
    reserved7: u16,
    pub fs: u16,
    reserved8: u16,
    pub gs: u16,
    reserved9: u16,
    pub ldtr: u16,
    reserved10: u32,
    pub iobp_offset: u16,
}

impl Default for TaskStateSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskStateSegment {
    pub const fn new() -> TaskStateSegment {
        TaskStateSegment {
            link: 0,
            reserved0: 0,
            esp0: 0,
            ss0: 0,
            reserved1: 0,
            esp1: 0,
            ss1: 0,
            reserved2: 0,
            esp2: 0,
            ss2: 0,
            reserved3: 0,
            cr3: 0,
            eip: 0,
            eflags: 0,
            eax: 0,
            ecx: 0,
            edx: 0,
            ebx: 0,
            esp: 0,
            ebp: 0,
            esi: 0,
            edi: 0,
            es: 0,
            reserved4: 0,
            cs: 0,
            reserved5: 0,
            ss: 0,
            reserved6: 0,
            ds: 0,
            reserved7: 0,
            fs: 0,
            reserved8: 0,
            gs: 0,
            reserved9: 0,
            ldtr: 0,
            reserved10: 0,
            iobp_offset: size_of::<TaskStateSegment>() as u16,
        }
    }
}
