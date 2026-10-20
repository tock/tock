// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Cortex-R52 Generic Interrupt Controller (GIC)
//!
//! Quick wrapper around the arm-gic crate's GICv3 driver
//! (`arm_gic::gicv3::GicV3`) adapted to Tock interfaces
//!
//! FIQs are used because a few HW benefits over IRQs.
//! GIC is common to ARMv8-A and ARMv8-R chips so this could
//! be reused by removing specific Cortex-R52 details.

use core::cell::RefCell;
use core::ptr::NonNull;

#[allow(unused_imports)]
use arm_gic::Trigger::{Edge, Level};
use arm_gic::gicv3::registers::{Gicd, GicrSgi};
use arm_gic::gicv3::{GicCpuInterface, GicV3, Group, SecureIntGroup};
use arm_gic::{IntId, InterruptGroup, Trigger, UniqueMmioPointer};

// Only one CPU used for the moment
const CPU_IDX: Option<usize> = Some(0);
const CPU_CNT: usize = 1;

/// Representation of a GIC interrupt (IRQ) with its ID and trigger type
/// to allow easy definition.
pub struct GicIrq {
    pub id: IntId,
    pub trig: Trigger,
}

// Default PPI for Cortex-R52, need to substract 16 from the
// Cortex R52 Tech RM Table 10-3.

//   22 Debug Communications Channel interrupt
//   23 Performance Monitor Counter Overflow interrupt
//   24 Cross Trigger Interface interrupt
//   25 Virtual CPU Interface Maintenance interrupt
//   26 Hypervisor timer interrupt
//   27 Virtual timer interrupt
//   30 Non-secure physical timer interrupt

pub const IRQ_GENTIMER_VIRT: GicIrq = GicIrq {
    id: IntId::ppi(11),
    trig: Edge,
};
pub const IRQ_GENTIMER_PHY: GicIrq = GicIrq {
    id: IntId::ppi(14),
    trig: Edge,
};

/// The Cortex-R52 GICv3 driver.
///
/// Just an instance of arm_gic::gicv3::GicV3
pub struct Gic {
    gic: RefCell<GicV3<'static>>,
}

impl Gic {
    /// Creates a new `Gic` driver for the GICv3 distributor and redistributor
    /// at the given base addresses, for a system with `cpu_count` cores.
    pub unsafe fn new(gicd_base: u32, gicr_base: u32) -> Self {
        let gicd = unsafe { UniqueMmioPointer::new(NonNull::new(gicd_base as *mut Gicd).unwrap()) };

        let mut gic = unsafe {
            GicV3::new(
                gicd,
                NonNull::new(gicr_base as *mut GicrSgi).unwrap(),
                CPU_CNT,
            )
        }
        .expect("failed to initialise GICv3 redistributor");

        // all priority below this value are enabled
        GicCpuInterface::set_priority_mask(0x80);

        // Config via the arm_gic api is strange but whatever
        // enable FIQ interrupts (secure group 0)
        GicCpuInterface::enable_group0(true);
        gic.setup(CPU_IDX.unwrap());
        gic.init_cpu(CPU_IDX.unwrap());
        gic.gicd_set_control(arm_gic::gicv3::registers::GicdCtlr::EnableGrp0);

        Self {
            gic: RefCell::new(gic),
        }
    }

    /// Enables and configure the given interrupt for the given core.
    ///
    /// Higher priorities are represented by lower numerical values.
    pub fn irq_enable(&self, cfg: GicIrq, priority: u8) {
        let mut gic = self.gic.borrow_mut();

        gic.set_group(cfg.id, CPU_IDX, Group::Secure(SecureIntGroup::Group0))
            .unwrap();
        gic.set_trigger(cfg.id, CPU_IDX, cfg.trig).unwrap();
        gic.set_interrupt_priority(cfg.id, CPU_IDX, priority)
            .unwrap();
        gic.enable_interrupt(cfg.id, CPU_IDX, true).unwrap();
    }

    /// Acknowledges the highest priority pending Group 1 interrupt, if any,
    /// returning its interrupt ID.
    pub fn next_pending(&self) -> Option<IntId> {
        GicCpuInterface::get_and_acknowledge_interrupt(InterruptGroup::Group0)
    }

    /// Informs the GIC that the CPU has finished handling the given
    /// interrupt, dropping its priority and deactivating it.
    pub fn complete(&self, intid: IntId) {
        GicCpuInterface::end_interrupt(intid, InterruptGroup::Group0);
    }

    /// Returns whether a Group 0 interrupt is currently pending.
    pub fn has_pending(&self) -> bool {
        GicCpuInterface::get_pending_interrupt(InterruptGroup::Group0).is_some()
    }
}
