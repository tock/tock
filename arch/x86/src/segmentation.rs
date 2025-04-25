// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.

//! Facilities for managing memory segmentation
//!
//! ## Segmentation Model
//!
//! The memory model implemented by this module is often referred to as _flat segmentation_, because
//! the entire address space of 4Gb is contained in a single unbroken ("flat") memory segment. This
//! technique effectively disables segmentation altogether, allowing the kernel to manage memory
//! using page tables exclusively.
//!
//! To this end, the GDT created by this module contains the following entries:
//!
//! * The null segment
//! * Kernel code segment - DPL of 0. Starts at address zero and extends the full 4Gb.
//! * Kernel data segment - Same as kernel code segment, but with data segment type.
//! * Global TSS - Single task state segment used for both kernel and user contexts.
//! * User code segment - DPL of 3. Starts at address zero and extends the full 4Gb. Memory
//!   protection is done using paging, not segmentation.
//! * User data segment - Same as user code segment, but with data segment type.
//!
//! Segment registers are also populated as follows:
//!
//! * CS register -> kernel code segment
//! * SS/DS/ES/FS/GS registers -> kernel data segment
//!
//! ## Task State Segment
//!
//! This module initializes a single, global TSS to help with context switching.
//!
//! The sole purpose of this TSS is to enable stack switching when transitioning from user to kernel
//! mode. When an interrupt occurs in user mode, the hardware uses the `ss0` and `esp0` fields of
//! the TSS to locate the kernel's stack.
//!
//! Apart from that, the global TSS is unused. In particular, this crate does _not_ use hardware
//! task management.

use core::mem;
use core::ptr;

use crate::registers::bits32::task::TaskStateSegment;
use crate::registers::dtables::{self, DescriptorTablePointer};
use crate::registers::ring::Ring;
use crate::registers::segmentation::{
    self, BuildDescriptor, CodeSegmentType, DataSegmentType, Descriptor, DescriptorBuilder,
    GateDescriptorBuilder, SegmentDescriptorBuilder, SegmentSelector,
};
use crate::registers::task;

use kernel::static_init;

/// Total number of descriptors in the GDT
const NUM_DESCRIPTORS: usize = 6;

/// Selector for the kernel's code segment
pub const KERNEL_CODE: SegmentSelector = SegmentSelector::new(1, Ring::Ring0);

/// Selector for the kernel's data segment
pub const KERNEL_DATA: SegmentSelector = SegmentSelector::new(2, Ring::Ring0);

/// Selector for the global TSS
pub const GLOBAL_TSS: SegmentSelector = SegmentSelector::new(3, Ring::Ring0);

/// Selector for the user mode code segment
pub const USER_CODE: SegmentSelector = SegmentSelector::new(4, Ring::Ring3);

/// Selector for the user mode data segment
pub const USER_DATA: SegmentSelector = SegmentSelector::new(5, Ring::Ring3);

/// Static storage for a global TSS
static mut TSS_INSTANCE: TaskStateSegment = TaskStateSegment::new();

/// Initializes the global TSS instance and returns a descriptor for it.
///
/// ## Safety
///
/// Must never be called more than once, as this would cause the TSS to be reinitialized
/// unexpectedly.
unsafe fn init_tss() -> Descriptor {
    unsafe {
        TSS_INSTANCE.ss0 = KERNEL_DATA.bits();
    }

    let tss_base = ptr::addr_of!(TSS_INSTANCE) as u64;
    let tss_limit = mem::size_of::<TaskStateSegment>() as u64;

    <DescriptorBuilder as GateDescriptorBuilder<u32>>::tss_descriptor(tss_base, tss_limit, true)
        .dpl(Ring::Ring0)
        .present()
        .avl()
        .finish()
}

/// Sets the stack pointer to use for handling interrupts from user mode.
///
/// ## Safety
///
/// When handling interrupts that occur during user mode, the context switching logic has very
/// specific expectations about the layout of the kernel's stack frame. See _return_from_user.rs_ for
/// complete details.
///
/// When calling this function, the stack frame referenced by `esp` must meet these expectations.
#[no_mangle]
pub unsafe extern "cdecl" fn set_tss_esp0(esp: u32) {
    unsafe {
        TSS_INSTANCE.esp0 = esp;
    }
}

/// Performs global initialization of memory segmentation.
///
/// ## Safety
///
/// Memory must be identity-mapped before this function is called. Otherwise the kernel's code/data
/// will suddenly be re-mapped to different addresses, likely resulting in a spectacular crash.
///
/// The GDT created by this function is allocated from static program memory. This function must
/// never be called more than once, or the static GDT will be overwritten.
pub unsafe fn init() {
    let gdt = static_init!(
        [Descriptor; NUM_DESCRIPTORS],
        [Descriptor::NULL; NUM_DESCRIPTORS]
    );

    let kernel_code_desc =
        DescriptorBuilder::code_descriptor(0, u32::MAX, CodeSegmentType::ExecuteRead)
            .present()
            .limit_granularity_4kb()
            .db()
            .finish();
    gdt[KERNEL_CODE.index() as usize] = kernel_code_desc;

    let kernel_data_desc =
        DescriptorBuilder::data_descriptor(0, u32::MAX, DataSegmentType::ReadWrite)
            .present()
            .limit_granularity_4kb()
            .db()
            .finish();
    gdt[KERNEL_DATA.index() as usize] = kernel_data_desc;

    let tss_desc = unsafe { init_tss() };
    gdt[GLOBAL_TSS.index() as usize] = tss_desc;

    let user_code_desc =
        DescriptorBuilder::code_descriptor(0, u32::MAX, CodeSegmentType::ExecuteRead)
            .present()
            .limit_granularity_4kb()
            .db()
            .dpl(Ring::Ring3)
            .finish();
    gdt[USER_CODE.index() as usize] = user_code_desc;

    let user_data_desc =
        DescriptorBuilder::data_descriptor(0, u32::MAX, DataSegmentType::ReadWrite)
            .present()
            .limit_granularity_4kb()
            .db()
            .dpl(Ring::Ring3)
            .finish();
    gdt[USER_DATA.index() as usize] = user_data_desc;

    unsafe {
        dtables::lgdt(&DescriptorTablePointer::new_from_slice(gdt));
        segmentation::load_cs(KERNEL_CODE);
        segmentation::load_ss(KERNEL_DATA);
        segmentation::load_ds(KERNEL_DATA);
        segmentation::load_es(KERNEL_DATA);
        segmentation::load_fs(KERNEL_DATA);
        segmentation::load_gs(KERNEL_DATA);
        task::load_tr(GLOBAL_TSS);
    }
}
