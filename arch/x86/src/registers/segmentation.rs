// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

// This is inspired and adapted for Tock from the [x86](https://github.com/gz/rust-x86) crate.

use super::ring::Ring;
use kernel::utilities::registers::register_bitfields;
use tock_registers::LocalRegisterCopy;

#[cfg(target_arch = "x86")]
use core::arch::asm;

macro_rules! bit {
    ($x:expr) => {
        1 << $x
    };
}

register_bitfields![u16,
    /// Specifies which element to load into a segment from
    /// descriptor tables (i.e., is a index to LDT or GDT table
    /// with some additional flags).
    ///
    /// See Intel 3a, Section 3.4.2 "Segment Selectors"
    pub SEGMENT_SELECTOR[
        RPL OFFSET(0) NUMBITS(2) [],
        TI OFFSET (2) NUMBITS(1) [
            GDT = 0,
            LDT = 1
        ],
        INDEX OFFSET (3) NUMBITS (12) []
    ],
];

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct SegmentSelector(LocalRegisterCopy<u16, SEGMENT_SELECTOR::Register>);

impl SegmentSelector {
    /// Create a new SegmentSelector
    /// # Arguments
    ///  * `index` - index in GDT or LDT array.
    ///  * `rpl` - Requested privilege level of the selector
    pub const fn new(index: u16, rpl: Ring) -> SegmentSelector {
        SegmentSelector(LocalRegisterCopy::new(index << 3 | (rpl as u16)))
    }

    /// Returns segment selector's index in GDT or LDT.
    pub fn index(&self) -> u16 {
        self.0.get() >> 3
    }

    /// Make a new segment selector from a untyped u16 value.
    pub fn from_raw(bits: u16) -> SegmentSelector {
        SegmentSelector(LocalRegisterCopy::new(bits))
    }

    pub fn bits(&self) -> u16 {
        self.0.get()
    }
}

/// Entry for IDT, GDT or LDT. Provides size and location of a segment.
/// See Intel 3a, Section 3.4.5 "Segment Descriptors", and Section 3.5.2
#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
pub struct Descriptor {
    pub lower: u32,
    pub upper: u32,
}

impl Descriptor {
    pub const NULL: Descriptor = Descriptor { lower: 0, upper: 0 };

    pub(crate) fn apply_builder_settings(&mut self, builder: &DescriptorBuilder) {
        if let Some(ring) = builder.dpl {
            self.set_dpl(ring)
        }

        if let Some((base, limit)) = builder.base_limit {
            self.set_base_limit(base as u32, limit as u32)
        }

        if let Some((selector, offset)) = builder.selector_offset {
            self.set_selector_offset(selector, offset as u32)
        }

        if builder.present {
            self.set_p();
        }
        if builder.avl {
            self.set_avl();
        }
        if builder.db {
            self.set_db();
        }
        if builder.limit_granularity_4k {
            self.set_g();
        }
        if builder.l {
            self.set_l();
        }
    }

    /// Specifies the privilege level of the segment. The DPL is used to control access to the segment.

    pub fn set_dpl(&mut self, ring: Ring) {
        assert!(ring as u32 <= 0b11);
        self.upper &= !(0b11 << 13);
        self.upper |= (ring as u32) << 13;
    }

    pub fn set_base_limit(&mut self, base: u32, limit: u32) {
        // Clear the base and limit fields in Descriptor
        self.lower = 0;
        self.upper &= 0x00F0FF00;

        // Set the new base
        self.lower |= base << 16;
        self.upper |= (base >> 16) & 0xff;
        self.upper |= (base >> 24) << 24;

        // Set the new limit
        self.lower |= limit & 0xffff;
        let limit_last_four_bits = (limit >> 16) & 0x0f;
        self.upper |= limit_last_four_bits << 16;
    }

    /// Creates a new descriptor with selector and offset (for IDT Gate descriptors,

    /// e.g. Trap, Interrupts and Task gates)

    pub fn set_selector_offset(&mut self, selector: SegmentSelector, offset: u32) {
        // Clear the selector and offset
        self.lower = 0;
        self.upper &= 0x0000ffff;

        // Set selector
        self.lower |= (selector.bits() as u32) << 16;

        // Set offset
        self.lower |= offset & 0x0000ffff;
        self.upper |= offset & 0xffff0000;
    }

    /// Set the type of the descriptor (bits 8-11).
    /// Indicates the segment or gate type and specifies the kinds of access that can be made to the
    /// segment and the direction of growth. The interpretation of this field depends on whether the descriptor
    /// type flag specifies an application (code or data) descriptor or a system descriptor.
    pub fn set_type(&mut self, typ: u8) {
        self.upper &= !(0x0f << 8); // clear
        self.upper |= (typ as u32 & 0x0f) << 8;
    }

    /// Set Present bit.
    /// Indicates whether the segment is present in memory (set) or not present (clear).
    /// If this flag is clear, the processor generates a segment-not-present exception (#NP) when a segment selector
    /// that points to the segment descriptor is loaded into a segment register.
    pub fn set_p(&mut self) {
        self.upper |= bit!(15);
    }

    /// Set AVL bit. System software can use this bit to store information.
    pub fn set_avl(&mut self) {
        self.upper |= bit!(20);
    }

    /// Set D/B.
    /// Performs different functions depending on whether the segment descriptor is an executable code segment,
    /// an expand-down data segment, or a stack segment.
    pub fn set_db(&mut self) {
        self.upper |= bit!(22);
    }

    /// Set G bit
    /// Determines the scaling of the segment limit field.
    /// When the granularity flag is clear, the segment limit is interpreted in byte units;
    /// when flag is set, the segment limit is interpreted in 4-KByte units.
    pub fn set_g(&mut self) {
        self.upper |= bit!(23);
    }

    /// Set L
    /// In IA-32e mode, bit 21 of the second doubleword of the segment descriptor indicates whether a
    /// code segment contains native 64-bit code. A value of 1 indicates instructions in this code
    /// segment are executed in 64-bit mode. A value of 0 indicates the instructions in this code segment
    /// are executed in compatibility mode. If L-bit is set, then D-bit must be cleared.
    pub fn set_l(&mut self) {
        self.upper |= bit!(21);
    }

    /// Specifies whether the segment descriptor is for a system segment (S flag is clear) or a code or data segment (S flag is set).
    pub fn set_s(&mut self) {
        self.upper |= bit!(12);
    }
}

pub trait BuildDescriptor<Descriptor> {
    fn finish(&self) -> Descriptor;
}

impl BuildDescriptor<Descriptor> for DescriptorBuilder {
    fn finish(&self) -> Descriptor {
        let mut desc = Descriptor::default();
        desc.apply_builder_settings(self);
        let typ = match self.typ {
            Some(DescriptorType::System32(typ)) => typ as u8,
            Some(DescriptorType::Data(typ)) => {
                desc.set_s();
                typ as u8
            }
            Some(DescriptorType::Code(typ)) => {
                desc.set_s();
                typ as u8
            }
            None => unreachable!("Type not set, this is a library bug in x86."),
        };
        desc.set_type(typ);
        desc
    }
}

/// Code Segment types for descriptors.
/// See also Intel 3a, Table 3-1 Code- and Data-Segment Types.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CodeSegmentType {
    /// Code Execute-Only
    Execute = 0b1000,

    /// Code Execute-Only, accessed
    ExecuteAccessed = 0b1001,

    /// Code Execute/Read
    ExecuteRead = 0b1010,

    /// Code Execute/Read, accessed
    ExecuteReadAccessed = 0b1011,

    /// Code Execute-Only, conforming
    ExecuteConforming = 0b1100,

    /// Code Execute-Only, conforming, accessed
    ExecuteConformingAccessed = 0b1101,

    /// Code Execute/Read, conforming
    ExecuteReadConforming = 0b1110,

    /// Code Execute/Read, conforming, accessed
    ExecuteReadConformingAccessed = 0b1111,
}

/// Data Segment types for descriptors.
/// See also Intel 3a, Table 3-1 Code- and Data-Segment Types.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataSegmentType {
    /// Data Read-Only
    ReadOnly = 0b0000,

    /// Data Read-Only, accessed
    ReadOnlyAccessed = 0b0001,

    /// Data Read/Write
    ReadWrite = 0b0010,

    /// Data Read/Write, accessed
    ReadWriteAccessed = 0b0011,

    /// Data Read-Only, expand-down
    ReadExpand = 0b0100,

    /// Data Read-Only, expand-down, accessed
    ReadExpandAccessed = 0b0101,

    /// Data Read/Write, expand-down
    ReadWriteExpand = 0b0110,

    /// Data Read/Write, expand-down, accessed
    ReadWriteExpandAccessed = 0b0111,
}

/// Trait that defines the architecture specific functions for building various system segment descriptors
/// which are available on all 16, 32, and 64 bits.
pub trait GateDescriptorBuilder<Size> {
    fn tss_descriptor(base: u64, limit: u64, available: bool) -> Self;
    fn call_gate_descriptor(selector: SegmentSelector, offset: Size) -> Self;
    fn interrupt_descriptor(selector: SegmentSelector, offset: Size) -> Self;
    fn trap_gate_descriptor(selector: SegmentSelector, offset: Size) -> Self;
}

impl GateDescriptorBuilder<u32> for DescriptorBuilder {
    fn tss_descriptor(base: u64, limit: u64, available: bool) -> DescriptorBuilder {
        let typ = match available {
            true => DescriptorType::System32(SystemDescriptorTypes32::TssAvailable32),
            false => DescriptorType::System32(SystemDescriptorTypes32::TssBusy32),
        };
        DescriptorBuilder::with_base_limit(base, limit).set_type(typ)
    }

    fn call_gate_descriptor(selector: SegmentSelector, offset: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(
            DescriptorType::System32(SystemDescriptorTypes32::CallGate32),
        )
    }

    fn interrupt_descriptor(selector: SegmentSelector, offset: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(
            DescriptorType::System32(SystemDescriptorTypes32::InterruptGate32),
        )
    }

    fn trap_gate_descriptor(selector: SegmentSelector, offset: u32) -> DescriptorBuilder {
        DescriptorBuilder::with_selector_offset(selector, offset.into()).set_type(
            DescriptorType::System32(SystemDescriptorTypes32::TrapGate32),
        )
    }
}

/// Trait to define functions that build architecture specific code and data descriptors.
pub trait SegmentDescriptorBuilder<Size> {
    fn code_descriptor(base: Size, limit: Size, cst: CodeSegmentType) -> Self;
    fn data_descriptor(base: Size, limit: Size, dst: DataSegmentType) -> Self;
}

impl SegmentDescriptorBuilder<u32> for DescriptorBuilder {
    fn code_descriptor(base: u32, limit: u32, cst: CodeSegmentType) -> DescriptorBuilder {
        DescriptorBuilder::with_base_limit(base.into(), limit.into())
            .set_type(DescriptorType::Code(cst))
    }

    fn data_descriptor(base: u32, limit: u32, dst: DataSegmentType) -> DescriptorBuilder {
        DescriptorBuilder::with_base_limit(base.into(), limit.into())
            .set_type(DescriptorType::Data(dst))
    }
}

/// Makes building descriptors easier (hopefully).
pub struct DescriptorBuilder {
    /// The base defines the location of byte 0 of the segment within the 4-GByte linear address space.

    /// The limit is the size of the range covered by the segment. Really a 20bit value.
    pub(crate) base_limit: Option<(u64, u64)>,

    /// Alternative to base_limit we use a selector that points to a segment and an an offset for certain descriptors.
    pub(crate) selector_offset: Option<(SegmentSelector, u64)>,

    /// Descriptor type
    pub(crate) typ: Option<DescriptorType>,

    /// Specifies the privilege level of the segment. The privilege level can range from 0 to 3, with 0 being the most privileged level.
    pub(crate) dpl: Option<Ring>,

    /// Indicates whether the segment is present in memory (set) or not present (clear).
    pub(crate) present: bool,

    /// Available for use by system software
    pub(crate) avl: bool,

    /// Default operation size
    pub(crate) db: bool,

    /// Determines the scaling of the segment limit field. When the granularity flag is clear, the segment limit is interpreted in byte units; when flag is set, the segment limit is interpreted in 4-KByte units.
    pub(crate) limit_granularity_4k: bool,

    /// 64-bit code segment (IA-32e mode only)
    pub(crate) l: bool,
}

impl DescriptorBuilder {
    /// Start building a new descriptor with a base and limit.
    pub(crate) fn with_base_limit(base: u64, limit: u64) -> DescriptorBuilder {
        DescriptorBuilder {
            base_limit: Some((base, limit)),
            selector_offset: None,
            typ: None,
            dpl: None,
            present: false,
            avl: false,
            db: false,
            limit_granularity_4k: false,
            l: false,
        }
    }

    /// Start building a new descriptor with a segment selector and offset.
    pub(crate) fn with_selector_offset(
        selector: SegmentSelector,
        offset: u64,
    ) -> DescriptorBuilder {
        DescriptorBuilder {
            base_limit: None,
            selector_offset: Some((selector, offset)),
            typ: None,
            dpl: None,
            present: false,
            avl: false,
            db: false,
            limit_granularity_4k: false,
            l: false,
        }
    }

    /// The segment limit is interpreted in 4-KByte units if this is set.
    pub fn limit_granularity_4kb(mut self) -> DescriptorBuilder {
        self.limit_granularity_4k = true;
        self
    }

    /// Indicates whether the segment is present in memory (set) or not present (clear).
    pub fn present(mut self) -> DescriptorBuilder {
        self.present = true;
        self
    }

    /// Specifies the privilege level of the segment.
    pub fn dpl(mut self, dpl: Ring) -> DescriptorBuilder {
        self.dpl = Some(dpl);
        self
    }

    /// Toggle the AVL bit.
    pub fn avl(mut self) -> DescriptorBuilder {
        self.avl = true;
        self
    }

    /// Set default operation size (false for 16bit segment, true for 32bit segments).
    pub fn db(mut self) -> DescriptorBuilder {
        self.db = true;
        self
    }

    pub(crate) fn set_type(mut self, typ: DescriptorType) -> DescriptorBuilder {
        self.typ = Some(typ);
        self
    }
}

/// Reload stack segment register.
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_ss(sel: SegmentSelector) {
    unsafe {
        asm!("movw {0:x}, %ss", in(reg) sel.bits(), options(att_syntax));
    }
}

/// Reload data segment register.
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_ds(sel: SegmentSelector) {
    unsafe {
        asm!("movw {0:x}, %ds", in(reg) sel.bits(), options(att_syntax));
    }
}

/// Reload es segment register.
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_es(sel: SegmentSelector) {
    unsafe {
        asm!("movw {0:x}, %es", in(reg) sel.bits(), options(att_syntax));
    }
}

/// Reload fs segment register.
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_fs(sel: SegmentSelector) {
    unsafe {
        asm!("movw {0:x}, %fs", in(reg) sel.bits(), options(att_syntax));
    }
}

/// Reload gs segment register.
/// # Safety
/// Needs CPL 0.
#[cfg(target_arch = "x86")]
pub unsafe fn load_gs(sel: SegmentSelector) {
    unsafe {
        asm!("movw {0:x}, %gs", in(reg) sel.bits(), options(att_syntax));
    }
}

#[cfg(target_arch = "x86")]
pub unsafe fn load_cs(sel: SegmentSelector) {
    unsafe {
        asm!("pushl {0}; \
            pushl $1f; \
            lretl; \
            1:", in(reg) sel.bits() as u32, options(att_syntax));
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DescriptorType {
    System32(SystemDescriptorTypes32),
    Data(DataSegmentType),
    Code(CodeSegmentType),
}

/// System-Segment and Gate-Descriptor Types 32-bit mode.

/// See also Intel 3a, Table 3-2 System Segment and Gate-Descriptor Types.

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SystemDescriptorTypes32 {
    //Reserved0 = 0b0000,
    TSSAvailable16 = 0b0001,
    LDT = 0b0010,
    TSSBusy16 = 0b0011,
    CallGate16 = 0b0100,
    TaskGate = 0b0101,
    InterruptGate16 = 0b0110,
    TrapGate16 = 0b0111,

    //Reserved1 = 0b1000,
    TssAvailable32 = 0b1001,

    //Reserved2 = 0b1010,
    TssBusy32 = 0b1011,
    CallGate32 = 0b1100,

    //Reserved3 = 0b1101,
    InterruptGate32 = 0b1110,
    TrapGate32 = 0b1111,
}

//For CI only

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_ss(_sel: SegmentSelector) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_ds(_sel: SegmentSelector) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_es(_sel: SegmentSelector) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_fs(_sel: SegmentSelector) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_gs(_sel: SegmentSelector) {
    unimplemented!()
}

#[cfg(not(any(doc, target_arch = "x86")))]
pub unsafe fn load_cs(_sel: SegmentSelector) {
    unimplemented!()
}
