// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Cortex-M Memory Protection Unit (MPU)
//!
//! Implementation of the memory protection unit for the Cortex-M0+, Cortex-M3,
//! Cortex-M4, and Cortex-M7.

use kernel::memory_management::pages::Page4KiB;
use kernel::memory_management::permissions::Permissions;
use kernel::memory_management::regions::PhysicalProtectedAllocatedRegion;
use kernel::platform::mmu::{Asid, MpuMmuCommon, MPU as MpuTrait};
use kernel::utilities::math;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_bitfields, FieldValue, ReadOnly, ReadWrite};
use kernel::utilities::StaticRef;

/// MPU Registers for the Cortex-M3, Cortex-M4 and Cortex-M7 families
/// Described in section 4.5 of
/// <http://infocenter.arm.com/help/topic/com.arm.doc.dui0553a/DUI0553A_cortex_m4_dgug.pdf>
#[repr(C)]
pub struct MpuRegisters {
    /// Indicates whether the MPU is present and, if so, how many regions it
    /// supports.
    pub mpu_type: ReadOnly<u32, Type::Register>,

    /// The control register:
    ///   * Enables the MPU (bit 0).
    ///   * Enables MPU in hard-fault, non-maskable interrupt (NMI).
    ///   * Enables the default memory map background region in privileged mode.
    pub ctrl: ReadWrite<u32, Control::Register>,

    /// Selects the region number (zero-indexed) referenced by the region base
    /// address and region attribute and size registers.
    pub rnr: ReadWrite<u32, RegionNumber::Register>,

    /// Defines the base address of the currently selected MPU region.
    pub rbar: ReadWrite<u32, RegionBaseAddress::Register>,

    /// Defines the region size and memory attributes of the selected MPU
    /// region. The bits are defined as in 4.5.5 of the Cortex-M4 user guide.
    pub rasr: ReadWrite<u32, RegionAttributes::Register>,
}

register_bitfields![u32,
    Type [
        /// The number of MPU instructions regions supported. Always reads 0.
        IREGION OFFSET(16) NUMBITS(8) [],
        /// The number of data regions supported. If this field reads-as-zero the
        /// processor does not implement an MPU
        DREGION OFFSET(8) NUMBITS(8) [],
        /// Indicates whether the processor support unified (0) or separate
        /// (1) instruction and data regions. Always reads 0 on the
        /// Cortex-M4.
        SEPARATE OFFSET(0) NUMBITS(1) []
    ],

    Control [
        /// Enables privileged software access to the default
        /// memory map
        PRIVDEFENA OFFSET(2) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ],
        /// Enables the operation of MPU during hard fault, NMI,
        /// and FAULTMASK handlers
        HFNMIENA OFFSET(1) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ],
        /// Enables the MPU
        ENABLE OFFSET(0) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],

    RegionNumber [
        /// Region indicating the MPU region referenced by the MPU_RBAR and
        /// MPU_RASR registers. Range 0-7 corresponding to the MPU regions.
        REGION OFFSET(0) NUMBITS(8) []
    ],

    RegionBaseAddress [
        /// Base address of the currently selected MPU region.
        ADDR OFFSET(5) NUMBITS(27) [],
        /// MPU Region Number valid bit.
        VALID OFFSET(4) NUMBITS(1) [
            /// Use the base address specified in Region Number Register (RNR)
            UseRNR = 0,
            /// Use the value of the REGION field in this register (RBAR)
            UseRBAR = 1
        ],
        /// Specifies which MPU region to set if VALID is set to 1.
        REGION OFFSET(0) NUMBITS(4) []
    ],

    RegionAttributes [
        /// Enables instruction fetches/execute permission
        XN OFFSET(28) NUMBITS(1) [
            Enable = 0,
            Disable = 1
        ],
        /// Defines access permissions
        AP OFFSET(24) NUMBITS(3) [
            //                                 Privileged  Unprivileged
            //                                 Access      Access
            NoAccess = 0b000,               // --          --
            PrivilegedOnly = 0b001,         // RW          --
            UnprivilegedReadOnly = 0b010,   // RW          R-
            ReadWrite = 0b011,              // RW          RW
            Reserved = 0b100,               // undef       undef
            PrivilegedOnlyReadOnly = 0b101, // R-          --
            ReadOnly = 0b110,               // R-          R-
            ReadOnlyAlias = 0b111           // R-          R-
        ],
        /// Subregion disable bits
        SRD OFFSET(8) NUMBITS(8) [],
        /// Specifies the region size, being 2^(SIZE+1) (minimum 3)
        SIZE OFFSET(1) NUMBITS(5) [],
        /// Enables the region
        ENABLE OFFSET(0) NUMBITS(1) []
    ]
];

/// State related to the real physical MPU.
///
/// There should only be one instantiation of this object as it represents
/// real hardware.
pub struct MPU<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> {
    /// MMIO reference to MPU registers.
    registers: StaticRef<MpuRegisters>,
}

impl<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> MPU<NUM_REGIONS, MIN_REGION_SIZE> {
    const PROG_REGION_INDEX: usize = 0;
    const RAM_REGION_INDEX: usize = 1;

    pub unsafe fn new(registers: StaticRef<MpuRegisters>) -> Self {
        let mpu = Self { registers };

        // Mark all regions as empty.
        for region_index in 0..NUM_REGIONS {
            let empty_cortex_m_region = CortexMRegion::empty(region_index);
            mpu.write_region(empty_cortex_m_region);
        }

        mpu
    }

    fn write_region(&self, cortex_m_region: CortexMRegion) {
        self.registers.rbar.write(cortex_m_region.base_address());
        self.registers.rasr.write(cortex_m_region.attributes());
    }

    // Function useful for boards where the bootloader sets up some
    // MPU configuration that conflicts with Tock's configuration:
    pub unsafe fn clear_mpu(&self) {
        self.registers.ctrl.write(Control::ENABLE::CLEAR);
    }

    fn protect_region(
        &self,
        index: usize,
        protected_region: &PhysicalProtectedAllocatedRegion<Page4KiB>,
    ) {
        let starting_pointer = protected_region.get_starting_pointer().infallible_cast();
        let raw_starting_pointer = unsafe { starting_pointer.to_raw() };
        let protected_length_bytes = protected_region.get_protected_length_bytes();
        let permissions = protected_region.get_permissions();

        if let Some(cortex_m_region) = CortexMRegion::new(
            raw_starting_pointer,
            protected_length_bytes.get(),
            index,
            None,
            permissions,
        ) {
            self.write_region(cortex_m_region);
        }
    }
}

/// Struct storing configuration for a Cortex-M MPU region.
#[derive(Copy, Clone)]
pub struct CortexMRegion {
    base_address: FieldValue<u32, RegionBaseAddress::Register>,
    attributes: FieldValue<u32, RegionAttributes::Register>,
}

impl CortexMRegion {
    fn new(
        region_start: *const u8,
        region_size: usize,
        region_num: usize,
        subregions: Option<(usize, usize)>,
        permissions: Permissions,
    ) -> Option<CortexMRegion> {
        let size_value = math::log_base_two(region_size as u32) - 1;

        if size_value == 0 {
            return None;
        } else if !region_start.is_aligned_to(1 << (size_value as usize)) {
            return None;
        }

        // Determine access and execute permissions
        let (access, execute) = match permissions {
            Permissions::ReadWrite => (
                RegionAttributes::AP::ReadWrite,
                RegionAttributes::XN::Disable,
            ),
            Permissions::ReadExecute => (
                RegionAttributes::AP::UnprivilegedReadOnly,
                RegionAttributes::XN::Enable,
            ),
            Permissions::ReadOnly => (
                RegionAttributes::AP::UnprivilegedReadOnly,
                RegionAttributes::XN::Disable,
            ),
        };

        // Base address register
        let base_address = RegionBaseAddress::ADDR.val((region_start as u32) >> 5)
            + RegionBaseAddress::VALID::UseRBAR
            + RegionBaseAddress::REGION.val(region_num as u32);

        // Attributes register
        let mut attributes = RegionAttributes::ENABLE::SET
            + RegionAttributes::SIZE.val(size_value)
            + access
            + execute;

        // If using subregions, add a subregion mask. The mask is a 8-bit
        // bitfield where `0` indicates that the corresponding subregion is enabled.
        // To compute the mask, we start with all subregions disabled and enable
        // the ones in the inclusive range [min_subregion, max_subregion].
        if let Some((min_subregion, max_subregion)) = subregions {
            let mask = (min_subregion..=max_subregion).fold(u8::MAX, |res, i| {
                // Enable subregions bit by bit (1 ^ 1 == 0)
                res ^ (1 << i)
            });
            attributes += RegionAttributes::SRD.val(mask as u32);
        }

        Some(CortexMRegion {
            base_address,
            attributes,
        })
    }

    fn empty(region_num: usize) -> CortexMRegion {
        CortexMRegion {
            base_address: RegionBaseAddress::VALID::UseRBAR
                + RegionBaseAddress::REGION.val(region_num as u32),
            attributes: RegionAttributes::ENABLE::CLEAR,
        }
    }

    fn base_address(&self) -> FieldValue<u32, RegionBaseAddress::Register> {
        self.base_address
    }

    fn attributes(&self) -> FieldValue<u32, RegionAttributes::Register> {
        self.attributes
    }
}

impl<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> MpuMmuCommon
    for MPU<NUM_REGIONS, MIN_REGION_SIZE>
{
    type Granule = Page4KiB;

    fn enable_user_protection(&self, _asid: Asid) {
        // Enable the MPU, disable it during HardFault/NMI handlers, and allow
        // privileged code access to all unprotected memory.
        self.registers
            .ctrl
            .write(Control::ENABLE::SET + Control::HFNMIENA::CLEAR + Control::PRIVDEFENA::SET);
    }

    fn disable_user_protection(&self) {
        // The MPU is not enabled for privileged mode, so we don't have to do
        // anything
        self.registers.ctrl.write(Control::ENABLE::CLEAR);
    }
}

impl<const NUM_REGIONS: usize, const MIN_REGION_SIZE: usize> MpuTrait
    for MPU<NUM_REGIONS, MIN_REGION_SIZE>
{
    fn protect_user_prog_region(
        &self,
        protected_region: &PhysicalProtectedAllocatedRegion<Self::Granule>,
    ) {
        self.protect_region(Self::PROG_REGION_INDEX, protected_region);
    }

    fn protect_user_ram_region(
        &self,
        protected_region: &PhysicalProtectedAllocatedRegion<Self::Granule>,
    ) {
        self.protect_region(Self::RAM_REGION_INDEX, protected_region);
    }
}
