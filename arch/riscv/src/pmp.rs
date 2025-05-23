// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use core::cell::Cell;
use core::num::NonZeroUsize;
use core::ops::Range;
use core::{cmp, fmt};

use kernel::platform::mpu;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::{register_bitfields, LocalRegisterCopy};

use crate::csr;

register_bitfields![u8,
    /// Generic `pmpcfg` octet.
    ///
    /// A PMP entry is configured through `pmpaddrX` and `pmpcfgX` CSRs, where a
    /// single `pmpcfgX` CSRs holds multiple octets, each affecting the access
    /// permission, addressing mode and "lock" attributes of a single `pmpaddrX`
    /// CSR. This bitfield definition represents a single, `u8`-backed `pmpcfg`
    /// octet affecting a single `pmpaddr` entry.
    pub pmpcfg_octet [
        r OFFSET(0) NUMBITS(1) [],
        w OFFSET(1) NUMBITS(1) [],
        x OFFSET(2) NUMBITS(1) [],
        a OFFSET(3) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l OFFSET(7) NUMBITS(1) []
    ]
];

/// Mask for valid values of the `pmpaddrX` CSRs on RISCV platforms.
///
/// RV64 platforms support only a 56 bit physical address space. For this reason
/// (and because addresses in `pmpaddrX` CSRs are left-shifted by 2 bit) the
/// uppermost 10 bits of a `pmpaddrX` CSR are defined as WARL-0. ANDing with
/// this mask achieves the same effect; thus it can be used to determine whether
/// a given PMP region spec would be legal and applied before writing it to a
/// `pmpaddrX` CSR. For RV32 platforms, th whole 32 bit address range is valid.
///
/// This mask will have the value `0x003F_FFFF_FFFF_FFFF` on RV64 platforms, and
/// `0xFFFFFFFF` on RV32 platforms.
const PMPADDR_MASK: usize = (0x003F_FFFF_FFFF_FFFFu64 & usize::MAX as u64) as usize;

/// A `pmpcfg` octet for a user-mode (non-locked) TOR-addressed PMP region.
///
/// This is a wrapper around a [`pmpcfg_octet`] (`u8`) register type, which
/// guarantees that the wrapped `pmpcfg` octet is always set to be either
/// [`TORUserPMPCFG::OFF`] (set to `0x00`), or in a non-locked, TOR-addressed
/// configuration.
///
/// By accepting this type, PMP implements can rely on the above properties to
/// hold by construction and avoid runtime checks. For example, this type is
/// used in the [`TORUserPMP::configure_pmp`] method.
#[derive(Copy, Clone, Debug)]
pub struct TORUserPMPCFG(LocalRegisterCopy<u8, pmpcfg_octet::Register>);

impl TORUserPMPCFG {
    pub const OFF: TORUserPMPCFG = TORUserPMPCFG(LocalRegisterCopy::new(0));

    /// Extract the `u8` representation of the [`pmpcfg_octet`] register.
    pub fn get(&self) -> u8 {
        self.0.get()
    }

    /// Extract a copy of the contained [`pmpcfg_octet`] register.
    pub fn get_reg(&self) -> LocalRegisterCopy<u8, pmpcfg_octet::Register> {
        self.0
    }
}

impl PartialEq<TORUserPMPCFG> for TORUserPMPCFG {
    fn eq(&self, other: &Self) -> bool {
        self.0.get() == other.0.get()
    }
}

impl Eq for TORUserPMPCFG {}

impl From<mpu::Permissions> for TORUserPMPCFG {
    fn from(p: mpu::Permissions) -> Self {
        let fv = match p {
            mpu::Permissions::ReadWriteExecute => {
                pmpcfg_octet::r::SET + pmpcfg_octet::w::SET + pmpcfg_octet::x::SET
            }
            mpu::Permissions::ReadWriteOnly => {
                pmpcfg_octet::r::SET + pmpcfg_octet::w::SET + pmpcfg_octet::x::CLEAR
            }
            mpu::Permissions::ReadExecuteOnly => {
                pmpcfg_octet::r::SET + pmpcfg_octet::w::CLEAR + pmpcfg_octet::x::SET
            }
            mpu::Permissions::ReadOnly => {
                pmpcfg_octet::r::SET + pmpcfg_octet::w::CLEAR + pmpcfg_octet::x::CLEAR
            }
            mpu::Permissions::ExecuteOnly => {
                pmpcfg_octet::r::CLEAR + pmpcfg_octet::w::CLEAR + pmpcfg_octet::x::SET
            }
        };

        TORUserPMPCFG(LocalRegisterCopy::new(
            (fv + pmpcfg_octet::l::CLEAR + pmpcfg_octet::a::TOR).value,
        ))
    }
}

/// A RISC-V PMP memory region specification, configured in NAPOT mode.
///
/// This type checks that the supplied `start` and `size` values meet the RISC-V
/// NAPOT requirements, namely that
///
/// - the region is a power of two bytes in size
/// - the region's start address is aligned to the region size
/// - the region is at least 8 bytes long
///
/// Finally, RISC-V restricts physical address spaces to 34 bit on RV32, and 56
/// bit on RV64 platforms. A `NAPOTRegionSpec` must not cover addresses
/// exceeding this address space, respectively. In practice, this means that on
/// RV64 platforms `NAPOTRegionSpec`s whose encoded `pmpaddrX` CSR contains any
/// non-zero bits in the 10 most significant bits will be rejected.
///
/// By accepting this type, PMP implementations can rely on these requirements
/// to be verified. Furthermore, they can use the [`NAPOTRegionSpec::pmpaddr`]
/// convenience method to retrieve an `pmpaddrX` CSR value encoding this
/// region's address and length.
#[derive(Copy, Clone, Debug)]
pub struct NAPOTRegionSpec {
    pmpaddr: usize,
}

impl NAPOTRegionSpec {
    /// Construct a new [`NAPOTRegionSpec`] from a pmpaddr CSR value.
    ///
    /// For an RV32 platform, every single integer in `[0; usize::MAX]` is a
    /// valid `pmpaddrX` CSR for a region configured in NAPOT mode, and this
    /// operation is thus effectively infallible.
    ///
    /// For RV64 platforms, this operation checks if the range would include any
    /// address outside of the 56 bit physical address space and, in this case,
    /// rejects the `pmpaddr` (tests whether any of the 10 most significant bits
    /// are non-zero).
    pub fn from_pmpaddr_csr(pmpaddr: usize) -> Option<Self> {
        // On 64-bit platforms, the 10 most significant bits must be 0
        // Prevent the `&-masking with zero` lint error in case of RV32
        // The redundant checks in this case are optimized out by the compiler on any 1-3,z opt-level
        #[allow(clippy::bad_bit_mask)]
        (pmpaddr & !PMPADDR_MASK == 0).then_some(NAPOTRegionSpec { pmpaddr })
    }

    /// Construct a new [`NAPOTRegionSpec`] from a start address and size.
    ///
    /// This method accepts a `start` address and a region length. It returns
    /// `Some(region)` when all constraints specified in the
    /// [`NAPOTRegionSpec`]'s documentation are satisfied, otherwise `None`.
    pub fn from_start_size(start: *const u8, size: usize) -> Option<Self> {
        if !size.is_power_of_two() || start.addr() % size != 0 || size < 8 {
            return None;
        }

        Self::from_pmpaddr_csr(
            (start.addr() + (size - 1).overflowing_shr(1).0)
                .overflowing_shr(2)
                .0,
        )
    }

    /// Construct a new [`NAPOTRegionSpec`] from a start address and end address.
    ///
    /// This method accepts a `start` address (inclusive) and `end` address
    /// (exclusive). It returns `Some(region)` when all constraints specified in
    /// the [`NAPOTRegionSpec`]'s documentation are satisfied, otherwise `None`.
    pub fn from_start_end(start: *const u8, end: *const u8) -> Option<Self> {
        end.addr()
            .checked_sub(start.addr())
            .and_then(|size| Self::from_start_size(start, size))
    }

    /// Retrieve a `pmpaddrX`-CSR compatible representation of this
    /// [`NAPOTRegionSpec`]'s address and length. For this value to be valid in
    /// a `CSR` register, the `pmpcfgX` octet's `A` (address mode) value
    /// belonging to this `pmpaddrX`-CSR must be set to `NAPOT` (0b11).
    pub fn pmpaddr(&self) -> usize {
        self.pmpaddr
    }

    /// Return the range of physical addresses covered by this PMP region.
    ///
    /// This follows the regular Rust range semantics (start inclusive, end
    /// exclusive). It returns the addresses as u64-integers to ensure that all
    /// underlying pmpaddrX CSR values can be represented.
    pub fn address_range(&self) -> core::ops::Range<u64> {
        let trailing_ones: u64 = self.pmpaddr.trailing_ones() as u64;
        let size = 0b1000_u64 << trailing_ones;
        let base_addr: u64 =
            (self.pmpaddr as u64 & !((1_u64 << trailing_ones).saturating_sub(1))) << 2;
        base_addr..(base_addr.saturating_add(size))
    }
}

/// A RISC-V PMP memory region specification, configured in TOR mode.
///
/// This type checks that the supplied `start` and `end` addresses meet the
/// RISC-V TOR requirements, namely that
///
/// - the region's start address is aligned to a 4-byte boundary
/// - the region's end address is aligned to a 4-byte boundary
/// - the region is at least 4 bytes long
///
/// Finally, RISC-V restricts physical address spaces to 34 bit on RV32, and 56
/// bit on RV64 platforms. A `TORRegionSpec` must not cover addresses exceeding
/// this address space, respectively. In practice, this means that on RV64
/// platforms `TORRegionSpec`s whose encoded `pmpaddrX` CSR contains any
/// non-zero bits in the 10 most significant bits will be rejected. In
/// particular, with the `end` pmpaddrX CSR / address being exclusive, the
/// region cannot span the last 4 bytes of the 56-bit address space on RV64, or
/// the last 4 bytes of the 34-bit address space on RV32.
///
/// By accepting this type, PMP implementations can rely on these requirements
/// to be verified.
#[derive(Copy, Clone, Debug)]
pub struct TORRegionSpec {
    pmpaddr_a: usize,
    pmpaddr_b: usize,
}

impl TORRegionSpec {
    /// Construct a new [`TORRegionSpec`] from a pair of pmpaddrX CSR values.
    ///
    /// This method accepts two `pmpaddrX` CSR values that together are
    /// configured to describe a single TOR memory region. The second `pmpaddr_b`
    /// must be strictly greater than `pmpaddr_a`, which translates into a
    /// minimum region size of 4 bytes. Otherwise this function returns `None`.
    ///
    /// For RV64 platforms, this operation also checks if the range would
    /// include any address outside of the 56 bit physical address space and, in
    /// this case, returns `None` (tests whether any of the 10 most significant
    /// bits of either `pmpaddr` are non-zero).
    pub fn from_pmpaddr_csrs(pmpaddr_a: usize, pmpaddr_b: usize) -> Option<TORRegionSpec> {
        // Prevent the `&-masking with zero` lint error in case of RV32
        // The redundant checks in this case are optimized out by the compiler on any 1-3,z opt-level
        #[allow(clippy::bad_bit_mask)]
        ((pmpaddr_a < pmpaddr_b)
            && (pmpaddr_a & !PMPADDR_MASK == 0)
            && (pmpaddr_b & !PMPADDR_MASK == 0))
            .then_some(TORRegionSpec {
                pmpaddr_a,
                pmpaddr_b,
            })
    }

    /// Construct a new [`TORRegionSpec`] from a range of addresses.
    ///
    /// This method accepts a `start` and `end` address. It returns
    /// `Some(region)` when all constraints specified in the [`TORRegionSpec`]'s
    /// documentation are satisfied, otherwise `None`.
    pub fn from_start_end(start: *const u8, end: *const u8) -> Option<Self> {
        if (start as usize) % 4 != 0
            || (end as usize) % 4 != 0
            || (end as usize)
                .checked_sub(start as usize)
                .is_none_or(|size| size < 4)
        {
            return None;
        }

        Self::from_pmpaddr_csrs(start.addr() >> 2, end.addr() >> 2)
    }

    /// Get the first `pmpaddrX` CSR value that this TORRegionSpec encodes.
    pub fn pmpaddr_a(&self) -> usize {
        self.pmpaddr_a
    }

    pub fn pmpaddr_b(&self) -> usize {
        self.pmpaddr_b
    }
}

/// Helper method to check if a [`PMPUserMPUConfig`] region overlaps with a
/// region specified by `other_start` and `other_size`.
///
/// Matching the RISC-V spec this checks `pmpaddr[i-i] <= y < pmpaddr[i]` for TOR
/// ranges.
fn region_overlaps(
    region: &(TORUserPMPCFG, *const u8, *const u8),
    other_start: *const u8,
    other_size: usize,
) -> bool {
    // PMP TOR regions are not inclusive on the high end, that is
    //     pmpaddr[i-i] <= y < pmpaddr[i].
    //
    // This happens to coincide with the definition of the Rust half-open Range
    // type, which provides a convenient `.contains()` method:
    let region_range = Range {
        start: region.1 as usize,
        end: region.2 as usize,
    };

    let other_range = Range {
        start: other_start as usize,
        end: other_start as usize + other_size,
    };

    // For a range A to overlap with a range B, either B's first or B's last
    // element must be contained in A, or A's first or A's last element must be
    // contained in B. As we deal with half-open ranges, ensure that neither
    // range is empty.
    //
    // This implementation is simple and stupid, and can be optimized. We leave
    // that as an exercise to the compiler.
    !region_range.is_empty()
        && !other_range.is_empty()
        && (region_range.contains(&other_range.start)
            || region_range.contains(&(other_range.end - 1))
            || other_range.contains(&region_range.start)
            || other_range.contains(&(region_range.end - 1)))
}

#[cfg(test)]
pub mod misc_pmp_test {
    #[test]
    fn test_napot_region_spec_from_pmpaddr_csr() {
        use super::NAPOTRegionSpec;

        // Unfortunatly, we can't run these unit tests for different platforms,
        // with arbitrary bit-widths (at least when using `usize` in the
        // `TORRegionSpec` internally.
        //
        // For now, we check whatever word-size our host-platform has and
        // generate our test vectors according to those expectations.
        let pmpaddr_max: usize = if core::mem::size_of::<usize>() == 8 {
            // This deliberately does not re-use the `PMPADDR_RV64_MASK`
            // constant which should be equal to this value:
            0x003F_FFFF_FFFF_FFFF_u64.try_into().unwrap()
        } else {
            usize::MAX
        };

        for (valid, pmpaddr, start, end) in [
            // Basic sanity checks:
            (true, 0b0000, 0b0000_0000, 0b0000_1000),
            (true, 0b0001, 0b0000_0000, 0b0001_0000),
            (true, 0b0010, 0b0000_1000, 0b0001_0000),
            (true, 0b0011, 0b0000_0000, 0b0010_0000),
            (true, 0b0101, 0b0001_0000, 0b0010_0000),
            (true, 0b1011, 0b0010_0000, 0b0100_0000),
            // Can span the whole address space (up to 34 bit on RV32, and 5
            // bit on RV64, 2^{XLEN + 3) byte NAPOT range).
            (
                true,
                pmpaddr_max,
                0,
                if core::mem::size_of::<usize>() == 8 {
                    0x0200_0000_0000_0000
                } else {
                    0x0000_0008_0000_0000
                },
            ),
            // Cannot create region larger than `pmpaddr_max`:
            (
                core::mem::size_of::<usize>() != 8,
                pmpaddr_max.saturating_add(1),
                0,
                if core::mem::size_of::<usize>() == 8 {
                    // Doesn't matter, operation should fail:
                    0
                } else {
                    0x0000_0008_0000_0000
                },
            ),
        ] {
            match (valid, NAPOTRegionSpec::from_pmpaddr_csr(pmpaddr)) {
                (true, Some(region)) => {
                    assert_eq!(
                        region.pmpaddr(),
                        pmpaddr,
                        "NAPOTRegionSpec::from_pmpaddr_csr yields wrong CSR value (0x{:x?} vs. 0x{:x?})",
                        pmpaddr,
                        region.pmpaddr()
                    );
                    assert_eq!(
                        region.address_range(),
                        start..end,
                        "NAPOTRegionSpec::from_pmpaddr_csr yields wrong address range value for CSR 0x{:x?} (0x{:x?}..0x{:x?} vs. 0x{:x?}..0x{:x?})",
                        pmpaddr,
                        region.address_range().start,
                        region.address_range().end,
                        start,
                        end
                    );
                }

                (true, None) => {
                    panic!(
                        "Failed to create NAPOT region over pmpaddr CSR ({:x?}), but has to succeed!",
                        pmpaddr,
                    );
                }

                (false, Some(region)) => {
                    panic!(
                        "Creation of TOR region over pmpaddr CSR {:x?} must fail, but succeeded: {:?}",
                        pmpaddr, region,
                    );
                }

                (false, None) => {
                    // Good, nothing to do here.
                }
            }
        }
    }

    #[test]
    fn test_tor_region_spec_from_pmpaddr_csrs() {
        use super::TORRegionSpec;
        // Unfortunatly, we can't run these unit tests for different platforms,
        // with arbitrary bit-widths (at least when using `usize` in the
        // `TORRegionSpec` internally.
        //
        // For now, we check whatever word-size our host-platform has and
        // generate our test vectors according to those expectations.
        let pmpaddr_max: usize = if core::mem::size_of::<usize>() == 8 {
            // This deliberately does not re-use the `PMPADDR_RV64_MASK`
            // constant which should be equal to this value:
            0x003F_FFFF_FFFF_FFFF_u64.try_into().unwrap()
        } else {
            usize::MAX
        };

        for (valid, pmpaddr_a, pmpaddr_b) in [
            // Can span the whole address space (up to 34 bit on RV32, and 56
            // bit on RV64):
            (true, 0, 1),
            (true, 0x8badf00d, 0xdeadbeef),
            (true, pmpaddr_max - 1, pmpaddr_max),
            (true, 0, pmpaddr_max),
            // Cannot create region smaller than 4 bytes:
            (false, 0, 0),
            (false, 0xdeadbeef, 0xdeadbeef),
            (false, pmpaddr_max, pmpaddr_max),
            // On 64-bit systems, cannot create region that exceeds 56 bit:
            (
                core::mem::size_of::<usize>() != 8,
                0,
                pmpaddr_max.saturating_add(1),
            ),
            // Cannot create region with end before start:
            (false, 1, 0),
            (false, 0xdeadbeef, 0x8badf00d),
            (false, pmpaddr_max, 0),
        ] {
            match (
                valid,
                TORRegionSpec::from_pmpaddr_csrs(pmpaddr_a, pmpaddr_b),
            ) {
                (true, Some(region)) => {
                    assert_eq!(region.pmpaddr_a(), pmpaddr_a);
                    assert_eq!(region.pmpaddr_b(), pmpaddr_b);
                }

                (true, None) => {
                    panic!(
                        "Failed to create TOR region over pmpaddr CSRS ({:x?}, {:x?}), but has to succeed!",
                        pmpaddr_a, pmpaddr_b,
                    );
                }

                (false, Some(region)) => {
                    panic!(
                        "Creation of TOR region over pmpaddr CSRs ({:x?}, {:x?}) must fail, but succeeded: {:?}",
                        pmpaddr_a, pmpaddr_b, region
                    );
                }

                (false, None) => {
                    // Good, nothing to do here.
                }
            }
        }
    }

    #[test]
    fn test_tor_region_spec_from_start_end_addrs() {
        use super::TORRegionSpec;

        fn panicing_shr_2(i: usize) -> usize {
            assert_eq!(i & 0b11, 0);
            i >> 2
        }

        // Unfortunatly, we can't run these unit tests for different platforms,
        // with arbitrary bit-widths (at least when using `usize` in the
        // `TORRegionSpec` internally.
        //
        // For now, we check whatever word-size our host-platform has and
        // generate our test vectors according to those expectations.
        let last_addr: usize = if core::mem::size_of::<usize>() == 8 {
            0x03F_FFFF_FFFF_FFFC_u64.try_into().unwrap()
        } else {
            // For 32-bit platforms, this cannot actually cover the whole
            // 32-bit address space. We must exclude the last 4 bytes.
            usize::MAX & (!0b11)
        };

        for (valid, start, end) in [
            // Can span the whole address space (up to 34 bit on RV32, and 56
            // bit on RV64):
            (true, 0, 4),
            (true, 0x13374200, 0xdead10cc),
            (true, last_addr - 4, last_addr),
            (true, 0, last_addr),
            // Cannot create region with start and end address not aligned on
            // 4-byte boundary:
            (false, 4, 5),
            (false, 4, 6),
            (false, 4, 7),
            (false, 5, 8),
            (false, 6, 8),
            (false, 7, 8),
            // Cannot create region smaller than 4 bytes:
            (false, 0, 0),
            (false, 0x13374200, 0x13374200),
            (false, 0x13374200, 0x13374201),
            (false, 0x13374200, 0x13374202),
            (false, 0x13374200, 0x13374203),
            (false, last_addr, last_addr),
            // On 64-bit systems, cannot create region that exceeds 56 or covers
            // the last 4 bytes of this address space. On 32-bit, cannot cover
            // the full address space (excluding the last 4 bytes of the address
            // space):
            (false, 0, last_addr.checked_add(1).unwrap()),
            // Cannot create region with end before start:
            (false, 4, 0),
            (false, 0xdeadbeef, 0x8badf00d),
            (false, last_addr, 0),
        ] {
            match (
                valid,
                TORRegionSpec::from_start_end(start as *const u8, end as *const u8),
            ) {
                (true, Some(region)) => {
                    assert_eq!(region.pmpaddr_a(), panicing_shr_2(start));
                    assert_eq!(region.pmpaddr_b(), panicing_shr_2(end));
                }

                (true, None) => {
                    panic!(
                        "Failed to create TOR region from address range [{:x?}, {:x?}), but has to succeed!",
                        start, end,
                    );
                }

                (false, Some(region)) => {
                    panic!(
                        "Creation of TOR region from address range [{:x?}, {:x?}) must fail, but succeeded: {:?}",
                        start, end, region
                    );
                }

                (false, None) => {
                    // Good, nothing to do here.
                }
            }
        }
    }
}

/// Print a table of the configured PMP regions, read from  the HW CSRs.
///
/// # Safety
///
/// This function is unsafe, as it relies on the PMP CSRs to be accessible, and
/// the hardware to feature `PHYSICAL_ENTRIES` PMP CSR entries. If these
/// conditions are not met, calling this function can result in undefinied
/// behavior (e.g., cause a system trap).
pub unsafe fn format_pmp_entries<const PHYSICAL_ENTRIES: usize>(
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    for i in 0..PHYSICAL_ENTRIES {
        // Extract the entry's pmpcfgX register value. The pmpcfgX CSRs are
        // tightly packed and contain 4 octets beloging to individual
        // entries. Convert this into a u8-wide LocalRegisterCopy<u8,
        // pmpcfg_octet> as a generic register type, independent of the entry's
        // offset.
        let pmpcfg: LocalRegisterCopy<u8, pmpcfg_octet::Register> = LocalRegisterCopy::new(
            csr::CSR
                .pmpconfig_get(i / 4)
                .overflowing_shr(((i % 4) * 8) as u32)
                .0 as u8,
        );

        // The address interpretation is different for every mode. Return both a
        // string indicating the PMP entry's mode, as well as the effective
        // start and end address (inclusive) affected by the region. For regions
        // that are OFF, we still want to expose the pmpaddrX register value --
        // thus return the raw unshifted value as the addr, and 0 as the
        // region's end.
        let (start_label, start, end, mode) = match pmpcfg.read_as_enum(pmpcfg_octet::a) {
            Some(pmpcfg_octet::a::Value::OFF) => {
                let addr = csr::CSR.pmpaddr_get(i);
                ("pmpaddr", addr, 0, "OFF  ")
            }

            Some(pmpcfg_octet::a::Value::TOR) => {
                let start = if i > 0 {
                    csr::CSR.pmpaddr_get(i - 1)
                } else {
                    0
                };

                (
                    "  start",
                    start.overflowing_shl(2).0,
                    csr::CSR.pmpaddr_get(i).overflowing_shl(2).0.wrapping_sub(1),
                    "TOR  ",
                )
            }

            Some(pmpcfg_octet::a::Value::NA4) => {
                let addr = csr::CSR.pmpaddr_get(i).overflowing_shl(2).0;
                ("  start", addr, addr | 0b11, "NA4  ")
            }

            Some(pmpcfg_octet::a::Value::NAPOT) => {
                let pmpaddr = csr::CSR.pmpaddr_get(i);
                let encoded_size = pmpaddr.trailing_ones();
                if (encoded_size as usize) < (core::mem::size_of_val(&pmpaddr) * 8 - 1) {
                    let start = pmpaddr - ((1 << encoded_size) - 1);
                    let end = start + (1 << (encoded_size + 1)) - 1;
                    (
                        "  start",
                        start.overflowing_shl(2).0,
                        end.overflowing_shl(2).0 | 0b11,
                        "NAPOT",
                    )
                } else {
                    ("  start", usize::MIN, usize::MAX, "NAPOT")
                }
            }

            None => {
                // We match on a 2-bit value with 4 variants, so this is
                // unreachable. However, don't insert a panic in case this
                // doesn't get optimized away:
                ("", 0, 0, "")
            }
        };

        // Ternary operator shortcut function, to avoid bulky formatting...
        fn t<T>(cond: bool, a: T, b: T) -> T {
            if cond {
                a
            } else {
                b
            }
        }

        write!(
            f,
            "  [{:02}]: {}={:#010X}, end={:#010X}, cfg={:#04X} ({}) ({}{}{}{})\r\n",
            i,
            start_label,
            start,
            end,
            pmpcfg.get(),
            mode,
            t(pmpcfg.is_set(pmpcfg_octet::l), "l", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::r), "r", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::w), "w", "-"),
            t(pmpcfg.is_set(pmpcfg_octet::x), "x", "-"),
        )?;
    }

    Ok(())
}

/// A RISC-V PMP implementation exposing a number of TOR memory protection
/// regions to the [`PMPUserMPU`].
///
/// The RISC-V PMP is complex and can be used to enforce memory protection in
/// various modes (Machine, Supervisor and User mode). Depending on the exact
/// extension set present (e.g., ePMP) and the machine's security configuration
/// bits, it may expose a vastly different set of constraints and application
/// semantics.
///
/// Because we can't possibly capture all of this in a single readable,
/// maintainable and efficient implementation, we implement a two-layer system:
///
/// - a [`TORUserPMP`] is a simple abstraction over some underlying PMP hardware
///   implementation, which exposes an interface to configure regions that are
///   active (enforced) in user-mode and can be configured for arbitrary
///   addresses on a 4-byte granularity.
///
/// - the [`PMPUserMPU`] takes this abstraction and implements the Tock kernel's
///   [`mpu::MPU`] trait. It worries about re-configuring memory protection when
///   switching processes, allocating memory regions of an appropriate size,
///   etc.
///
/// Implementors of a chip are free to define their own [`TORUserPMP`]
/// implementations, adhering to their specific PMP layout & constraints,
/// provided they implement this trait.
///
/// The `MAX_REGIONS` const generic is used to indicate the maximum number of
/// TOR PMP regions available to the [`PMPUserMPU`]. The PMP implementation may
/// provide less regions than indicated through `MAX_REGIONS`, for instance when
/// entries are enforced (locked) in machine mode. The number of available
/// regions may change at runtime. The current number of regions available to
/// the [`PMPUserMPU`] is indicated by the [`TORUserPMP::available_regions`]
/// method. However, when it is known that a number of regions are not available
/// for userspace protection, `MAX_REGIONS` can be used to reduce the memory
/// footprint allocated by stored PMP configurations, as well as the
/// re-configuration overhead.
pub trait TORUserPMP<const MAX_REGIONS: usize> {
    /// A placeholder to define const-assertions which are evaluated in
    /// [`PMPUserMPU::new`]. This can be used to, for instance, assert that the
    /// number of userspace regions does not exceed the number of hardware
    /// regions.
    const CONST_ASSERT_CHECK: ();

    /// The number of TOR regions currently available for userspace memory
    /// protection. Within `[0; MAX_REGIONS]`.
    ///
    /// The PMP implementation may provide less regions than indicated through
    /// `MAX_REGIONS`, for instance when entries are enforced (locked) in
    /// machine mode. The number of available regions may change at runtime. The
    /// implementation is free to map these regions to arbitrary PMP entries
    /// (and change this mapping at runtime), provided that they are enforced
    /// when the hart is in user-mode, and other memory regions are generally
    /// inaccessible when in user-mode.
    ///
    /// When allocating regions for kernel-mode protection, and thus reducing
    /// the number of regions available to userspace, re-configuring the PMP may
    /// fail. This is allowed behavior. However, the PMP must not remove any
    /// regions from the user-mode current configuration while it is active
    /// ([`TORUserPMP::enable_user_pmp`] has been called, and it has not been
    /// disabled through [`TORUserPMP::disable_user_pmp`]).
    fn available_regions(&self) -> usize;

    /// Configure the user-mode memory protection.
    ///
    /// This method configures the user-mode memory protection, to be enforced
    /// on a call to [`TORUserPMP::enable_user_pmp`].
    ///
    /// PMP implementations where configured regions are only enforced in
    /// user-mode may re-configure the PMP on this function invocation and
    /// implement [`TORUserPMP::enable_user_pmp`] as a no-op. If configured
    /// regions are enforced in machine-mode (for instance when using an ePMP
    /// with the machine-mode whitelist policy), the new configuration rules
    /// must not apply until [`TORUserPMP::enable_user_pmp`].
    ///
    /// The tuples as passed in the `regions` parameter are defined as follows:
    ///
    /// - first value ([`TORUserPMPCFG`]): the memory protection mode as
    ///   enforced on the region. A `TORUserPMPCFG` can be created from the
    ///   [`mpu::Permissions`] type. It is in a format compatible to the pmpcfgX
    ///   register, guaranteed to not have the lock (`L`) bit set, and
    ///   configured either as a TOR region (`A = 0b01`), or disabled (all bits
    ///   set to `0`).
    ///
    /// - second value (`*const u8`): the region's start addres. As a PMP TOR
    ///   region has a 4-byte address granularity, this address is rounded down
    ///   to the next 4-byte boundary.
    ///
    /// - third value (`*const u8`): the region's end addres. As a PMP TOR
    ///   region has a 4-byte address granularity, this address is rounded down
    ///   to the next 4-byte boundary.
    ///
    /// To disable a region, set its configuration to [`TORUserPMPCFG::OFF`]. In
    /// this case, the start and end addresses are ignored and can be set to
    /// arbitrary values.
    fn configure_pmp(
        &self,
        regions: &[(TORUserPMPCFG, *const u8, *const u8); MAX_REGIONS],
    ) -> Result<(), ()>;

    /// Enable the user-mode memory protection.
    ///
    /// Enables the memory protection for user-mode, as configured through
    /// [`TORUserPMP::configure_pmp`]. Enabling the PMP for user-mode may make
    /// the user-mode accessible regions inaccessible to the kernel. For PMP
    /// implementations where configured regions are only enforced in user-mode,
    /// this method may be implemented as a no-op.
    ///
    /// If enabling the current configuration is not possible (e.g., because
    /// regions have been allocated to the kernel), this function must return
    /// `Err(())`. Otherwise, this function returns `Ok(())`.
    fn enable_user_pmp(&self) -> Result<(), ()>;

    /// Disable the user-mode memory protection.
    ///
    /// Disables the memory protection for user-mode. If enabling the user-mode
    /// memory protetion made user-mode accessible regions inaccessible to
    /// machine-mode, this method should make these regions accessible again.
    ///
    /// For PMP implementations where configured regions are only enforced in
    /// user-mode, this method may be implemented as a no-op. This method is not
    /// responsible for making regions inaccessible to user-mode. If previously
    /// configured regions must be made inaccessible,
    /// [`TORUserPMP::configure_pmp`] must be used to re-configure the PMP
    /// accordingly.
    fn disable_user_pmp(&self);
}

/// Struct storing userspace memory protection regions for the [`PMPUserMPU`].
pub struct PMPUserMPUConfig<const MAX_REGIONS: usize> {
    /// PMP config identifier, as generated by the issuing PMP implementation.
    id: NonZeroUsize,
    /// Indicates if the configuration has changed since the last time it was
    /// written to hardware.
    is_dirty: Cell<bool>,
    /// Array of MPU regions. Each region requires two physical PMP entries.
    regions: [(TORUserPMPCFG, *const u8, *const u8); MAX_REGIONS],
    /// Which region index (into the `regions` array above) is used
    /// for app memory (if it has been configured).
    app_memory_region: OptionalCell<usize>,
}

impl<const MAX_REGIONS: usize> fmt::Display for PMPUserMPUConfig<MAX_REGIONS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Ternary operator shortcut function, to avoid bulky formatting...
        fn t<T>(cond: bool, a: T, b: T) -> T {
            if cond {
                a
            } else {
                b
            }
        }

        write!(
            f,
            " PMPUserMPUConfig {{\r\n  id: {},\r\n  is_dirty: {},\r\n  app_memory_region: {:?},\r\n  regions:\r\n",
            self.id,
            self.is_dirty.get(),
            self.app_memory_region.get()
        )?;

        for (i, (tor_user_pmpcfg, start, end)) in self.regions.iter().enumerate() {
            let pmpcfg = tor_user_pmpcfg.get_reg();
            write!(
                f,
                "     #{:02}: start={:#010X}, end={:#010X}, cfg={:#04X} ({}) (-{}{}{})\r\n",
                i,
                *start as usize,
                *end as usize,
                pmpcfg.get(),
                t(pmpcfg.is_set(pmpcfg_octet::a), "TOR", "OFF"),
                t(pmpcfg.is_set(pmpcfg_octet::r), "r", "-"),
                t(pmpcfg.is_set(pmpcfg_octet::w), "w", "-"),
                t(pmpcfg.is_set(pmpcfg_octet::x), "x", "-"),
            )?;
        }

        write!(f, " }}\r\n")?;
        Ok(())
    }
}

/// Adapter from a generic PMP implementation exposing TOR-type regions to the
/// Tock [`mpu::MPU`] trait. See [`TORUserPMP`].
pub struct PMPUserMPU<const MAX_REGIONS: usize, P: TORUserPMP<MAX_REGIONS> + 'static> {
    /// Monotonically increasing counter for allocated configurations, used to
    /// assign unique IDs to `PMPUserMPUConfig` instances.
    config_count: Cell<NonZeroUsize>,
    /// The configuration that the PMP was last configured for. Used (along with
    /// the `is_dirty` flag) to determine if PMP can skip writing the
    /// configuration to hardware.
    last_configured_for: OptionalCell<NonZeroUsize>,
    /// Underlying hardware PMP implementation, exposing a number (up to
    /// `P::MAX_REGIONS`) of memory protection regions with a 4-byte enforcement
    /// granularity.
    pub pmp: P,
}

impl<const MAX_REGIONS: usize, P: TORUserPMP<MAX_REGIONS> + 'static> PMPUserMPU<MAX_REGIONS, P> {
    pub fn new(pmp: P) -> Self {
        // Assigning this constant here ensures evaluation of the const
        // expression at compile time, and can thus be used to enforce
        // compile-time assertions based on the desired PMP configuration.
        #[allow(clippy::let_unit_value)]
        let _: () = P::CONST_ASSERT_CHECK;

        PMPUserMPU {
            config_count: Cell::new(NonZeroUsize::MIN),
            last_configured_for: OptionalCell::empty(),
            pmp,
        }
    }
}

impl<const MAX_REGIONS: usize, P: TORUserPMP<MAX_REGIONS> + 'static> kernel::platform::mpu::MPU
    for PMPUserMPU<MAX_REGIONS, P>
{
    type MpuConfig = PMPUserMPUConfig<MAX_REGIONS>;

    fn enable_app_mpu(&self) {
        // TODO: This operation may fail when the PMP is not exclusively used
        // for userspace. Instead of panicing, we should handle this case more
        // gracefully and return an error in the `MPU` trait. Process
        // infrastructure can then attempt to re-schedule the process later on,
        // try to revoke some optional shared memory regions, or suspend the
        // process.
        self.pmp.enable_user_pmp().unwrap()
    }

    fn disable_app_mpu(&self) {
        self.pmp.disable_user_pmp()
    }

    fn number_total_regions(&self) -> usize {
        self.pmp.available_regions()
    }

    fn new_config(&self) -> Option<Self::MpuConfig> {
        let id = self.config_count.get();
        self.config_count.set(id.checked_add(1)?);

        Some(PMPUserMPUConfig {
            id,
            regions: [(
                TORUserPMPCFG::OFF,
                core::ptr::null::<u8>(),
                core::ptr::null::<u8>(),
            ); MAX_REGIONS],
            is_dirty: Cell::new(true),
            app_memory_region: OptionalCell::empty(),
        })
    }

    fn reset_config(&self, config: &mut Self::MpuConfig) {
        config.regions.iter_mut().for_each(|region| {
            *region = (
                TORUserPMPCFG::OFF,
                core::ptr::null::<u8>(),
                core::ptr::null::<u8>(),
            )
        });
        config.app_memory_region.clear();
        config.is_dirty.set(true);
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        // Find a free region slot. If we don't have one, abort early:
        let region_num = config
            .regions
            .iter()
            .enumerate()
            .find(|(_i, (pmpcfg, _, _))| *pmpcfg == TORUserPMPCFG::OFF)
            .map(|(i, _)| i)?;

        // Now, meet the PMP TOR region constraints. For this, start with the
        // provided start address and size, transform them to meet the
        // constraints, and then check that we're still within the bounds of the
        // provided values:
        let mut start = unallocated_memory_start as usize;
        let mut size = min_region_size;

        // Region start always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if start % 4 != 0 {
            start += 4 - (start % 4);
        }

        // Region size always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if size % 4 != 0 {
            size += 4 - (size % 4);
        }

        // Regions must be at least 4 bytes in size.
        if size < 4 {
            size = 4;
        }

        // Now, check to see whether the adjusted start and size still meet the
        // allocation constraints, namely ensure that
        //
        //     start + size <= unallocated_memory_start + unallocated_memory_size
        if start + size > (unallocated_memory_start as usize) + unallocated_memory_size {
            // We're overflowing the provided memory region, can't make
            // allocation. Normally, we'd abort here.
            //
            // However, a previous implementation of this code was incorrect in
            // that performed this check before adjusting the requested region
            // size to meet PMP region layout constraints (4 byte alignment for
            // start and end address). Existing applications whose end-address
            // is aligned on a less than 4-byte bondary would thus be given
            // access to additional memory which should be inaccessible.
            // Unfortunately, we can't fix this without breaking existing
            // applications. Thus, we perform the same insecure hack here, and
            // give the apps at most an extra 3 bytes of memory, as long as the
            // requested region as no write privileges.
            //
            // TODO: Remove this logic with as part of
            // https://github.com/tock/tock/issues/3544
            let writeable = match permissions {
                mpu::Permissions::ReadWriteExecute => true,
                mpu::Permissions::ReadWriteOnly => true,
                mpu::Permissions::ReadExecuteOnly => false,
                mpu::Permissions::ReadOnly => false,
                mpu::Permissions::ExecuteOnly => false,
            };

            if writeable
                || (start + size
                    > (unallocated_memory_start as usize) + unallocated_memory_size + 3)
            {
                return None;
            }
        }

        // Finally, check that this new region does not overlap with any
        // existing configured userspace region:
        for region in config.regions.iter() {
            if region.0 != TORUserPMPCFG::OFF && region_overlaps(region, start as *const u8, size) {
                return None;
            }
        }

        // All checks passed, store region allocation and mark config as dirty:
        config.regions[region_num] = (
            permissions.into(),
            start as *const u8,
            (start + size) as *const u8,
        );
        config.is_dirty.set(true);

        Some(mpu::Region::new(start as *const u8, size))
    }

    fn remove_memory_region(
        &self,
        region: mpu::Region,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let index = config
            .regions
            .iter()
            .enumerate()
            .find(|(_i, r)| {
                // `start as usize + size` in lieu of a safe pointer offset method
                r.0 != TORUserPMPCFG::OFF
                    && core::ptr::eq(r.1, region.start_address())
                    && core::ptr::eq(
                        r.2,
                        (region.start_address() as usize + region.size()) as *const u8,
                    )
            })
            .map(|(i, _)| i)
            .ok_or(())?;

        config.regions[index].0 = TORUserPMPCFG::OFF;
        config.is_dirty.set(true);

        Ok(())
    }

    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // An app memory region can only be allocated once per `MpuConfig`.
        // If we already have one, abort:
        if config.app_memory_region.is_some() {
            return None;
        }

        // Find a free region slot. If we don't have one, abort early:
        let region_num = config
            .regions
            .iter()
            .enumerate()
            .find(|(_i, (pmpcfg, _, _))| *pmpcfg == TORUserPMPCFG::OFF)
            .map(|(i, _)| i)?;

        // Now, meet the PMP TOR region constraints for the region specified by
        // `initial_app_memory_size` (which is the part of the region actually
        // protected by the PMP). For this, start with the provided start
        // address and size, transform them to meet the constraints, and then
        // check that we're still within the bounds of the provided values:
        let mut start = unallocated_memory_start as usize;
        let mut pmp_region_size = initial_app_memory_size;

        // Region start always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if start % 4 != 0 {
            start += 4 - (start % 4);
        }

        // Region size always has to align to 4 bytes. Round up to a 4 byte
        // boundary if required:
        if pmp_region_size % 4 != 0 {
            pmp_region_size += 4 - (pmp_region_size % 4);
        }

        // Regions must be at least 4 bytes in size.
        if pmp_region_size < 4 {
            pmp_region_size = 4;
        }

        // We need to provide a memory block that fits both the initial app and
        // kernel memory sections, and is `min_memory_size` bytes
        // long. Calculate the length of this block with our new PMP-aliged
        // size:
        let memory_block_size = cmp::max(
            min_memory_size,
            pmp_region_size + initial_kernel_memory_size,
        );

        // Now, check to see whether the adjusted start and size still meet the
        // allocation constraints, namely ensure that
        //
        //     start + memory_block_size
        //         <= unallocated_memory_start + unallocated_memory_size
        //
        // , which ensures the PMP constraints didn't push us over the bounds of
        // the provided memory region, and we can fit the entire allocation as
        // requested by the kernel:
        if start + memory_block_size > (unallocated_memory_start as usize) + unallocated_memory_size
        {
            // Overflowing the provided memory region, can't make allocation:
            return None;
        }

        // Finally, check that this new region does not overlap with any
        // existing configured userspace region:
        for region in config.regions.iter() {
            if region.0 != TORUserPMPCFG::OFF
                && region_overlaps(region, start as *const u8, memory_block_size)
            {
                return None;
            }
        }

        // All checks passed, store region allocation, indicate the
        // app_memory_region, and mark config as dirty:
        config.regions[region_num] = (
            permissions.into(),
            start as *const u8,
            (start + pmp_region_size) as *const u8,
        );
        config.is_dirty.set(true);
        config.app_memory_region.replace(region_num);

        Some((start as *const u8, memory_block_size))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let region_num = config.app_memory_region.get().ok_or(())?;

        let mut app_memory_break = app_memory_break as usize;
        let kernel_memory_break = kernel_memory_break as usize;

        // Ensure that the requested app_memory_break complies with PMP
        // alignment constraints, namely that the region's end address is 4 byte
        // aligned:
        if app_memory_break % 4 != 0 {
            app_memory_break += 4 - (app_memory_break % 4);
        }

        // Check if the app has run out of memory:
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        // If we're not out of memory, update the region configuration
        // accordingly:
        config.regions[region_num].0 = permissions.into();
        config.regions[region_num].2 = app_memory_break as *const u8;
        config.is_dirty.set(true);

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        if !self.last_configured_for.contains(&config.id) || config.is_dirty.get() {
            self.pmp.configure_pmp(&config.regions).unwrap();
            config.is_dirty.set(false);
            self.last_configured_for.set(config.id);
        }
    }
}

#[cfg(test)]
pub mod tor_user_pmp_test {
    use super::{TORUserPMP, TORUserPMPCFG};

    struct MockTORUserPMP;
    impl<const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS> for MockTORUserPMP {
        // Don't require any const-assertions in the MockTORUserPMP.
        const CONST_ASSERT_CHECK: () = ();

        fn available_regions(&self) -> usize {
            // For the MockTORUserPMP, we always assume to have the full number
            // of MPU_REGIONS available. More advanced tests may want to return
            // a different number here (to simulate kernel memory protection)
            // and make the configuration fail at runtime, for instance.
            MPU_REGIONS
        }

        fn configure_pmp(
            &self,
            _regions: &[(TORUserPMPCFG, *const u8, *const u8); MPU_REGIONS],
        ) -> Result<(), ()> {
            Ok(())
        }

        fn enable_user_pmp(&self) -> Result<(), ()> {
            Ok(())
        } // The kernel's MPU trait requires

        fn disable_user_pmp(&self) {}
    }

    // TODO: implement more test cases, such as:
    //
    // - Try to update the app memory break with an invalid pointer below its
    //   allocation's start address.

    #[test]
    fn test_mpu_region_no_overlap() {
        use crate::pmp::PMPUserMPU;
        use kernel::platform::mpu::{Permissions, MPU};

        let mpu: PMPUserMPU<8, MockTORUserPMP> = PMPUserMPU::new(MockTORUserPMP);
        let mut config = mpu
            .new_config()
            .expect("Failed to allocate the first MPU config");

        // Allocate a region which spans from 0x40000000 to 0x80000000 (this
        // meets PMP alignment constraints and will work on 32-bit and 64-bit
        // systems)
        let region_0 = mpu
            .allocate_region(
                0x40000000 as *const u8,
                0x40000000,
                0x40000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .expect(
                "Failed to allocate a well-aligned R/W MPU region with \
                 unallocated_memory_size == min_region_size",
            );
        assert!(region_0.start_address() == 0x40000000 as *const u8);
        assert!(region_0.size() == 0x40000000);

        // Try to allocate a region adjacent to `region_0`. This should work:
        let region_1 = mpu
            .allocate_region(
                0x80000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadExecuteOnly,
                &mut config,
            )
            .expect(
                "Failed to allocate a well-aligned R/W MPU region adjacent to \
                 another region",
            );
        assert!(region_1.start_address() == 0x80000000 as *const u8);
        assert!(region_1.size() == 0x10000000);

        // Remove the previously allocated `region_1`:
        mpu.remove_memory_region(region_1, &mut config)
            .expect("Failed to remove valid MPU region allocation");

        // Allocate another region which spans from 0xc0000000 to 0xd0000000
        // (this meets PMP alignment constraints and will work on 32-bit and
        // 64-bit systems), but this time allocate it using the
        // `allocate_app_memory_region` method. We want a region of `0x20000000`
        // bytes, but only the first `0x10000000` should be accessible to the
        // app.
        let (region_2_start, region_2_size) = mpu
            .allocate_app_memory_region(
                0xc0000000 as *const u8,
                0x20000000,
                0x20000000,
                0x10000000,
                0x08000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .expect(
                "Failed to allocate a well-aligned R/W app memory MPU region \
                 with unallocated_memory_size == min_region_size",
            );
        assert!(region_2_start == 0xc0000000 as *const u8);
        assert!(region_2_size == 0x20000000);

        // --> General overlap tests involving both regions

        // Now, try to allocate another region that spans over both memory
        // regions. This should fail.
        assert!(mpu
            .allocate_region(
                0x40000000 as *const u8,
                0xc0000000,
                0xc0000000,
                Permissions::ReadOnly,
                &mut config,
            )
            .is_none());

        // Try to allocate a region that spans over parts of both memory
        // regions. This should fail.
        assert!(mpu
            .allocate_region(
                0x48000000 as *const u8,
                0x80000000,
                0x80000000,
                Permissions::ReadOnly,
                &mut config,
            )
            .is_none());

        // --> Overlap tests involving a single region (region_0)
        //
        // We define these in an array, such that we can run the tests with the
        // `region_0` defined (to confirm that the allocations are indeed
        // refused), and with `region_0` removed (to make sure they would work
        // in general).
        let overlap_region_0_tests = [
            (
                // Try to allocate a region that is contained within
                // `region_0`. This should fail.
                0x41000000 as *const u8,
                0x01000000,
                0x01000000,
                Permissions::ReadWriteOnly,
            ),
            (
                // Try to allocate a region that overlaps with `region_0` in the
                // front. This should fail.
                0x38000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadWriteExecute,
            ),
            (
                // Try to allocate a region that overlaps with `region_0` in the
                // back. This should fail.
                0x48000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ExecuteOnly,
            ),
            (
                // Try to allocate a region that spans over `region_0`. This
                // should fail.
                0x38000000 as *const u8,
                0x20000000,
                0x20000000,
                Permissions::ReadWriteOnly,
            ),
        ];

        // Make sure that the allocation requests fail with `region_0` defined:
        for (memory_start, memory_size, length, perms) in overlap_region_0_tests.iter() {
            assert!(mpu
                .allocate_region(*memory_start, *memory_size, *length, *perms, &mut config,)
                .is_none());
        }

        // Now, remove `region_0` and re-run the tests. Every test-case should
        // succeed now (in isolation, hence removing the successful allocations):
        mpu.remove_memory_region(region_0, &mut config)
            .expect("Failed to remove valid MPU region allocation");

        for region @ (memory_start, memory_size, length, perms) in overlap_region_0_tests.iter() {
            let allocation_res =
                mpu.allocate_region(*memory_start, *memory_size, *length, *perms, &mut config);

            match allocation_res {
                Some(region) => {
                    mpu.remove_memory_region(region, &mut config)
                        .expect("Failed to remove valid MPU region allocation");
                }
                None => {
                    panic!(
                        "Failed to allocate region that does not overlap and should meet alignment constraints: {:?}",
                        region
                    );
                }
            }
        }

        // Make sure we can technically allocate a memory region that overlaps
        // with the kernel part of the `app_memory_region`.
        //
        // It is unclear whether this should be supported.
        let region_2 = mpu
            .allocate_region(
                0xd0000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .unwrap();
        assert!(region_2.start_address() == 0xd0000000 as *const u8);
        assert!(region_2.size() == 0x10000000);

        // Now, we can grow the app memory break into this region:
        mpu.update_app_memory_region(
            0xd0000004 as *const u8,
            0xd8000000 as *const u8,
            Permissions::ReadWriteOnly,
            &mut config,
        )
        .expect("Failed to grow the app memory region into an existing other MPU region");

        // Now, we have two overlapping MPU regions. Remove `region_2`, and try
        // to reallocate it as `region_3`. This should fail now, demonstrating
        // that we managed to reach an invalid intermediate state:
        mpu.remove_memory_region(region_2, &mut config)
            .expect("Failed to remove valid MPU region allocation");
        assert!(mpu
            .allocate_region(
                0xd0000000 as *const u8,
                0x10000000,
                0x10000000,
                Permissions::ReadWriteOnly,
                &mut config,
            )
            .is_none());
    }
}

pub mod simple {
    use super::{pmpcfg_octet, TORUserPMP, TORUserPMPCFG};
    use crate::csr;
    use core::fmt;
    use kernel::utilities::registers::{FieldValue, LocalRegisterCopy};

    /// A "simple" RISC-V PMP implementation.
    ///
    /// The SimplePMP does not support locked regions, kernel memory protection,
    /// or any ePMP features (using the mseccfg CSR). It is generic over the
    /// number of hardware PMP regions available. `AVAILABLE_ENTRIES` is
    /// expected to be set to the number of available entries.
    ///
    /// [`SimplePMP`] implements [`TORUserPMP`] to expose all of its regions as
    /// "top of range" (TOR) regions (each taking up two physical PMP entires)
    /// for use as a user-mode memory protection mechanism.
    ///
    /// Notably, [`SimplePMP`] implements `TORUserPMP<MPU_REGIONS>` over a
    /// generic `MPU_REGIONS` where `MPU_REGIONS <= (AVAILABLE_ENTRIES / 2)`. As
    /// PMP re-configuration can have a significiant runtime overhead, users are
    /// free to specify a small `MPU_REGIONS` const-generic parameter to reduce
    /// the runtime overhead induced through PMP configuration, at the cost of
    /// having less PMP regions available to use for userspace memory
    /// protection.
    pub struct SimplePMP<const AVAILABLE_ENTRIES: usize>;

    impl<const AVAILABLE_ENTRIES: usize> SimplePMP<AVAILABLE_ENTRIES> {
        pub unsafe fn new() -> Result<Self, ()> {
            // The SimplePMP does not support locked regions, kernel memory
            // protection, or any ePMP features (using the mseccfg CSR). Ensure
            // that we don't find any locked regions. If we don't have locked
            // regions and can still successfully execute code, this means that
            // we're not in the ePMP machine-mode lockdown mode, and can treat
            // our hardware as a regular PMP.
            //
            // Furthermore, we test whether we can use each entry (i.e. whether
            // it actually exists in HW) by flipping the RWX bits. If we can't
            // flip them, then `AVAILABLE_ENTRIES` is incorrect.  However, this
            // is not sufficient to check for locked regions, because of the
            // ePMP's rule-lock-bypass bit. If a rule is locked, it might be the
            // reason why we can execute code or read-write data in machine mode
            // right now. Thus, never try to touch a locked region, as we might
            // well revoke access to a kernel region!
            for i in 0..AVAILABLE_ENTRIES {
                // Read the entry's CSR:
                let pmpcfg_csr = csr::CSR.pmpconfig_get(i / 4);

                // Extract the entry's pmpcfg octet:
                let pmpcfg: LocalRegisterCopy<u8, pmpcfg_octet::Register> = LocalRegisterCopy::new(
                    pmpcfg_csr.overflowing_shr(((i % 4) * 8) as u32).0 as u8,
                );

                // As outlined above, we never touch a locked region. Thus, bail
                // out if it's locked:
                if pmpcfg.is_set(pmpcfg_octet::l) {
                    return Err(());
                }

                // Now that it's not locked, we can be sure that regardless of
                // any ePMP bits, this region is either ignored or entirely
                // denied for machine-mode access. Hence, we can change it in
                // arbitrary ways without breaking our own memory access. Try to
                // flip the R/W/X bits:
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr ^ (7 << ((i % 4) * 8)));

                // Check if the CSR changed:
                if pmpcfg_csr == csr::CSR.pmpconfig_get(i / 4) {
                    // Didn't change! This means that this region is not backed
                    // by HW. Return an error as `AVAILABLE_ENTRIES` is
                    // incorrect:
                    return Err(());
                }

                // Finally, turn the region off:
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr & !(0x18 << ((i % 4) * 8)));
            }

            // Hardware PMP is verified to be in a compatible mode / state, and
            // has at least `AVAILABLE_ENTRIES` entries.
            Ok(SimplePMP)
        }
    }

    impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS>
        for SimplePMP<AVAILABLE_ENTRIES>
    {
        // Ensure that the MPU_REGIONS (starting at entry, and occupying two
        // entries per region) don't overflow the available entires.
        const CONST_ASSERT_CHECK: () = assert!(MPU_REGIONS <= (AVAILABLE_ENTRIES / 2));

        fn available_regions(&self) -> usize {
            // Always assume to have `MPU_REGIONS` usable TOR regions. We don't
            // support locked regions, or kernel protection.
            MPU_REGIONS
        }

        // This implementation is specific for 32-bit systems. We use
        // `u32::from_be_bytes` and then cast to usize, as it manages to compile
        // on 64-bit systems as well. However, this implementation will not work
        // on RV64I systems, due to the changed pmpcfgX CSR layout.
        fn configure_pmp(
            &self,
            regions: &[(TORUserPMPCFG, *const u8, *const u8); MPU_REGIONS],
        ) -> Result<(), ()> {
            // Could use `iter_array_chunks` once that's stable.
            let mut regions_iter = regions.iter();
            let mut i = 0;

            while let Some(even_region) = regions_iter.next() {
                let odd_region_opt = regions_iter.next();

                if let Some(odd_region) = odd_region_opt {
                    // We can configure two regions at once which, given that we
                    // start at index 0 (an even offset), translates to a single
                    // CSR write for the pmpcfgX register:
                    csr::CSR.pmpconfig_set(
                        i / 2,
                        u32::from_be_bytes([
                            odd_region.0.get(),
                            TORUserPMPCFG::OFF.get(),
                            even_region.0.get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    );

                    // Now, set the addresses of the respective regions, if they
                    // are enabled, respectively:
                    if even_region.0 != TORUserPMPCFG::OFF {
                        csr::CSR
                            .pmpaddr_set(i * 2 + 0, (even_region.1 as usize).overflowing_shr(2).0);
                        csr::CSR
                            .pmpaddr_set(i * 2 + 1, (even_region.2 as usize).overflowing_shr(2).0);
                    }

                    if odd_region.0 != TORUserPMPCFG::OFF {
                        csr::CSR
                            .pmpaddr_set(i * 2 + 2, (odd_region.1 as usize).overflowing_shr(2).0);
                        csr::CSR
                            .pmpaddr_set(i * 2 + 3, (odd_region.2 as usize).overflowing_shr(2).0);
                    }

                    i += 2;
                } else {
                    // TODO: check overhead of code
                    // Modify the first two pmpcfgX octets for this region:
                    csr::CSR.pmpconfig_modify(
                        i / 2,
                        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                            0x0000FFFF,
                            0,
                            u32::from_be_bytes([
                                0,
                                0,
                                even_region.0.get(),
                                TORUserPMPCFG::OFF.get(),
                            ]) as usize,
                        ),
                    );

                    // Set the addresses if the region is enabled:
                    if even_region.0 != TORUserPMPCFG::OFF {
                        csr::CSR
                            .pmpaddr_set(i * 2 + 0, (even_region.1 as usize).overflowing_shr(2).0);
                        csr::CSR
                            .pmpaddr_set(i * 2 + 1, (even_region.2 as usize).overflowing_shr(2).0);
                    }

                    i += 1;
                }
            }

            Ok(())
        }

        fn enable_user_pmp(&self) -> Result<(), ()> {
            // No-op. The SimplePMP does not have any kernel-enforced regions.
            Ok(())
        }

        fn disable_user_pmp(&self) {
            // No-op. The SimplePMP does not have any kernel-enforced regions.
        }
    }

    impl<const AVAILABLE_ENTRIES: usize> fmt::Display for SimplePMP<AVAILABLE_ENTRIES> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, " PMP hardware configuration -- entries: \r\n")?;
            unsafe { super::format_pmp_entries::<AVAILABLE_ENTRIES>(f) }
        }
    }
}

pub mod kernel_protection {
    use super::{pmpcfg_octet, NAPOTRegionSpec, TORRegionSpec, TORUserPMP, TORUserPMPCFG};
    use crate::csr;
    use core::fmt;
    use kernel::utilities::registers::{FieldValue, LocalRegisterCopy};

    // ---------- Kernel memory-protection PMP memory region wrapper types -----
    //
    // These types exist primarily to avoid argument confusion in the
    // [`KernelProtectionPMP`] constructor, which accepts the addresses of these
    // memory regions as arguments. They further encode whether a region must
    // adhere to the `NAPOT` or `TOR` addressing mode constraints:

    /// The flash memory region address range.
    ///
    /// Configured in the PMP as a `NAPOT` region.
    #[derive(Copy, Clone, Debug)]
    pub struct FlashRegion(pub NAPOTRegionSpec);

    /// The RAM region address range.
    ///
    /// Configured in the PMP as a `NAPOT` region.
    #[derive(Copy, Clone, Debug)]
    pub struct RAMRegion(pub NAPOTRegionSpec);

    /// The MMIO region address range.
    ///
    /// Configured in the PMP as a `NAPOT` region.
    #[derive(Copy, Clone, Debug)]
    pub struct MMIORegion(pub NAPOTRegionSpec);

    /// The PMP region specification for the kernel `.text` section.
    ///
    /// This is to be made accessible to machine-mode as read-execute.
    /// Configured in the PMP as a `TOR` region.
    #[derive(Copy, Clone, Debug)]
    pub struct KernelTextRegion(pub TORRegionSpec);

    /// A RISC-V PMP implementation which supports machine-mode (kernel) memory
    /// protection, with a fixed number of "kernel regions" (such as `.text`,
    /// flash, RAM and MMIO).
    ///
    /// This implementation will configure the PMP in the following way:
    ///
    ///   ```text
    ///   |-------+-----------------------------------------+-------+---+-------|
    ///   | ENTRY | REGION / ADDR                           | MODE  | L | PERMS |
    ///   |-------+-----------------------------------------+-------+---+-------|
    ///   |     0 | /                                     \ | OFF   |   |       |
    ///   |     1 | \ Userspace TOR region #0             / | TOR   |   | ????? |
    ///   |       |                                         |       |   |       |
    ///   |     2 | /                                     \ | OFF   |   |       |
    ///   |     3 | \ Userspace TOR region #1             / | TOR   |   | ????? |
    ///   |       |                                         |       |   |       |
    ///   | 4 ... | /                                     \ |       |   |       |
    ///   | n - 8 | \ Userspace TOR region #x             / |       |   |       |
    ///   |       |                                         |       |   |       |
    ///   | n - 7 | "Deny-all" user-mode rule (all memory)  | NAPOT |   | ----- |
    ///   |       |                                         |       |   |       |
    ///   | n - 6 | --------------------------------------- | OFF   | X | ----- |
    ///   | n - 5 | Kernel .text section                    | TOR   | X | R/X   |
    ///   |       |                                         |       |   |       |
    ///   | n - 4 | FLASH (spanning kernel & apps)          | NAPOT | X | R     |
    ///   |       |                                         |       |   |       |
    ///   | n - 3 | RAM (spanning kernel & apps)            | NAPOT | X | R/W   |
    ///   |       |                                         |       |   |       |
    ///   | n - 2 | MMIO                                    | NAPOT | X | R/W   |
    ///   |       |                                         |       |   |       |
    ///   | n - 1 | "Deny-all" machine-mode    (all memory) | NAPOT | X | ----- |
    ///   |-------+-----------------------------------------+-------+---+-------|
    ///   ```
    ///
    /// This implementation does not use any `mseccfg` protection bits (ePMP
    /// functionality). To protect machine-mode (kernel) memory regions, regions
    /// must be marked as locked. However, locked regions apply to both user-
    /// and machine-mode. Thus, region `n - 7` serves as a "deny-all" user-mode
    /// rule, which prohibits all accesses not explicitly allowed through rules
    /// `< n - 7`. Kernel memory is made accessible underneath this "deny-all"
    /// region, which does not apply to machine-mode.
    ///
    /// This PMP implementation supports the [`TORUserPMP`] interface with
    /// `MPU_REGIONS <= ((AVAILABLE_ENTRIES - 7) / 2)`, to leave sufficient
    /// space for the "deny-all" and kernel regions. This constraint is enforced
    /// through the [`KernelProtectionPMP::CONST_ASSERT_CHECK`] associated
    /// constant, which MUST be evaluated by the consumer of the [`TORUserPMP`]
    /// trait (usually the [`PMPUserMPU`](super::PMPUserMPU) implementation).
    pub struct KernelProtectionPMP<const AVAILABLE_ENTRIES: usize>;

    impl<const AVAILABLE_ENTRIES: usize> KernelProtectionPMP<AVAILABLE_ENTRIES> {
        pub unsafe fn new(
            flash: FlashRegion,
            ram: RAMRegion,
            mmio: MMIORegion,
            kernel_text: KernelTextRegion,
        ) -> Result<Self, ()> {
            for i in 0..AVAILABLE_ENTRIES {
                // Read the entry's CSR:
                let pmpcfg_csr = csr::CSR.pmpconfig_get(i / 4);

                // Extract the entry's pmpcfg octet:
                let pmpcfg: LocalRegisterCopy<u8, pmpcfg_octet::Register> = LocalRegisterCopy::new(
                    pmpcfg_csr.overflowing_shr(((i % 4) * 8) as u32).0 as u8,
                );

                // As outlined above, we never touch a locked region. Thus, bail
                // out if it's locked:
                if pmpcfg.is_set(pmpcfg_octet::l) {
                    return Err(());
                }

                // Now that it's not locked, we can be sure that regardless of
                // any ePMP bits, this region is either ignored or entirely
                // denied for machine-mode access. Hence, we can change it in
                // arbitrary ways without breaking our own memory access. Try to
                // flip the R/W/X bits:
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr ^ (7 << ((i % 4) * 8)));

                // Check if the CSR changed:
                if pmpcfg_csr == csr::CSR.pmpconfig_get(i / 4) {
                    // Didn't change! This means that this region is not backed
                    // by HW. Return an error as `AVAILABLE_ENTRIES` is
                    // incorrect:
                    return Err(());
                }

                // Finally, turn the region off:
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr & !(0x18 << ((i % 4) * 8)));
            }

            // -----------------------------------------------------------------
            // Hardware PMP is verified to be in a compatible mode & state, and
            // has at least `AVAILABLE_ENTRIES` entries.
            // -----------------------------------------------------------------

            // Now we need to set up the various kernel memory protection
            // regions, and the deny-all userspace region (n - 8), never
            // modified.

            // Helper to modify an arbitrary PMP entry. Because we don't know
            // AVAILABLE_ENTRIES in advance, there's no good way to
            // optimize this further.
            fn write_pmpaddr_pmpcfg(i: usize, pmpcfg: u8, pmpaddr: usize) {
                csr::CSR.pmpaddr_set(i, pmpaddr);
                csr::CSR.pmpconfig_modify(
                    i / 4,
                    FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                        0x000000FF_usize,
                        (i % 4) * 8,
                        u32::from_be_bytes([0, 0, 0, pmpcfg]) as usize,
                    ),
                );
            }

            // Set the kernel `.text`, flash, RAM and MMIO regions, in no
            // particular order, with the exception of `.text` and flash:
            // `.text` must precede flash, as otherwise we'd be revoking execute
            // permissions temporarily. Given that we can currently execute
            // code, this should not have any impact on our accessible memory,
            // assuming that the provided regions are not otherwise aliased.

            // MMIO at n - 2:
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 2,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::SET
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                mmio.0.pmpaddr(),
            );

            // RAM at n - 3:
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 3,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::SET
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                ram.0.pmpaddr(),
            );

            // `.text` at n - 6 and n - 5 (TOR region):
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 6,
                (pmpcfg_octet::a::OFF
                    + pmpcfg_octet::r::CLEAR
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                kernel_text.0.pmpaddr_a(),
            );
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 5,
                (pmpcfg_octet::a::TOR
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::SET
                    + pmpcfg_octet::l::SET)
                    .into(),
                kernel_text.0.pmpaddr_b(),
            );

            // flash at n - 4:
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 4,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                flash.0.pmpaddr(),
            );

            // Now that the kernel has explicit region definitions for any
            // memory that it needs to have access to, we can deny other memory
            // accesses in our very last rule (n - 1):
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 1,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::CLEAR
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                // the entire address space:
                0x7FFFFFFF,
            );

            // Finally, we configure the non-locked user-mode deny all
            // rule. This must never be removed, or otherwise usermode will be
            // able to access all locked regions (which are supposed to be
            // exclusively accessible to kernel-mode):
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 7,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::CLEAR
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::CLEAR)
                    .into(),
                // the entire address space:
                0x7FFFFFFF,
            );

            // Setup complete
            Ok(KernelProtectionPMP)
        }
    }

    impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS>
        for KernelProtectionPMP<AVAILABLE_ENTRIES>
    {
        /// Ensure that the MPU_REGIONS (starting at entry, and occupying two
        /// entries per region) don't overflow the available entires, excluding
        /// the 7 entires used for implementing the kernel memory protection.
        const CONST_ASSERT_CHECK: () = assert!(MPU_REGIONS <= ((AVAILABLE_ENTRIES - 7) / 2));

        fn available_regions(&self) -> usize {
            // Always assume to have `MPU_REGIONS` usable TOR regions. We don't
            // support locking additional regions at runtime.
            MPU_REGIONS
        }

        // This implementation is specific for 32-bit systems. We use
        // `u32::from_be_bytes` and then cast to usize, as it manages to compile
        // on 64-bit systems as well. However, this implementation will not work
        // on RV64I systems, due to the changed pmpcfgX CSR layout.
        fn configure_pmp(
            &self,
            regions: &[(TORUserPMPCFG, *const u8, *const u8); MPU_REGIONS],
        ) -> Result<(), ()> {
            // Could use `iter_array_chunks` once that's stable.
            let mut regions_iter = regions.iter();
            let mut i = 0;

            while let Some(even_region) = regions_iter.next() {
                let odd_region_opt = regions_iter.next();

                if let Some(odd_region) = odd_region_opt {
                    // We can configure two regions at once which, given that we
                    // start at index 0 (an even offset), translates to a single
                    // CSR write for the pmpcfgX register:
                    csr::CSR.pmpconfig_set(
                        i / 2,
                        u32::from_be_bytes([
                            odd_region.0.get(),
                            TORUserPMPCFG::OFF.get(),
                            even_region.0.get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    );

                    // Now, set the addresses of the respective regions, if they
                    // are enabled, respectively:
                    if even_region.0 != TORUserPMPCFG::OFF {
                        csr::CSR
                            .pmpaddr_set(i * 2 + 0, (even_region.1 as usize).overflowing_shr(2).0);
                        csr::CSR
                            .pmpaddr_set(i * 2 + 1, (even_region.2 as usize).overflowing_shr(2).0);
                    }

                    if odd_region.0 != TORUserPMPCFG::OFF {
                        csr::CSR
                            .pmpaddr_set(i * 2 + 2, (odd_region.1 as usize).overflowing_shr(2).0);
                        csr::CSR
                            .pmpaddr_set(i * 2 + 3, (odd_region.2 as usize).overflowing_shr(2).0);
                    }

                    i += 2;
                } else {
                    // Modify the first two pmpcfgX octets for this region:
                    csr::CSR.pmpconfig_modify(
                        i / 2,
                        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                            0x0000FFFF,
                            0,
                            u32::from_be_bytes([
                                0,
                                0,
                                even_region.0.get(),
                                TORUserPMPCFG::OFF.get(),
                            ]) as usize,
                        ),
                    );

                    // Set the addresses if the region is enabled:
                    if even_region.0 != TORUserPMPCFG::OFF {
                        csr::CSR
                            .pmpaddr_set(i * 2 + 0, (even_region.1 as usize).overflowing_shr(2).0);
                        csr::CSR
                            .pmpaddr_set(i * 2 + 1, (even_region.2 as usize).overflowing_shr(2).0);
                    }

                    i += 1;
                }
            }

            Ok(())
        }

        fn enable_user_pmp(&self) -> Result<(), ()> {
            // No-op. User-mode regions are never enforced in machine-mode, and
            // thus can be configured direct and may stay enabled in
            // machine-mode.
            Ok(())
        }

        fn disable_user_pmp(&self) {
            // No-op. User-mode regions are never enforced in machine-mode, and
            // thus can be configured direct and may stay enabled in
            // machine-mode.
        }
    }

    impl<const AVAILABLE_ENTRIES: usize> fmt::Display for KernelProtectionPMP<AVAILABLE_ENTRIES> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, " PMP hardware configuration -- entries: \r\n")?;
            unsafe { super::format_pmp_entries::<AVAILABLE_ENTRIES>(f) }
        }
    }
}

pub mod kernel_protection_mml_epmp {
    use super::{pmpcfg_octet, NAPOTRegionSpec, TORRegionSpec, TORUserPMP, TORUserPMPCFG};
    use crate::csr;
    use core::cell::Cell;
    use core::fmt;
    use kernel::platform::mpu;
    use kernel::utilities::registers::interfaces::{Readable, Writeable};
    use kernel::utilities::registers::{FieldValue, LocalRegisterCopy};

    // ---------- Kernel memory-protection PMP memory region wrapper types -----
    //
    // These types exist primarily to avoid argument confusion in the
    // [`KernelProtectionMMLEPMP`] constructor, which accepts the addresses of
    // these memory regions as arguments. They further encode whether a region
    // must adhere to the `NAPOT` or `TOR` addressing mode constraints:

    /// The flash memory region address range.
    ///
    /// Configured in the PMP as a `NAPOT` region.
    #[derive(Copy, Clone, Debug)]
    pub struct FlashRegion(pub NAPOTRegionSpec);

    /// The RAM region address range.
    ///
    /// Configured in the PMP as a `NAPOT` region.
    #[derive(Copy, Clone, Debug)]
    pub struct RAMRegion(pub NAPOTRegionSpec);

    /// The MMIO region address range.
    ///
    /// Configured in the PMP as a `NAPOT` region.
    #[derive(Copy, Clone, Debug)]
    pub struct MMIORegion(pub NAPOTRegionSpec);

    /// The PMP region specification for the kernel `.text` section.
    ///
    /// This is to be made accessible to machine-mode as read-execute.
    /// Configured in the PMP as a `TOR` region.
    #[derive(Copy, Clone, Debug)]
    pub struct KernelTextRegion(pub TORRegionSpec);

    /// A RISC-V ePMP implementation.
    ///
    /// Supports machine-mode (kernel) memory protection by using the
    /// machine-mode lockdown mode (MML), with a fixed number of
    /// "kernel regions" (such as `.text`, flash, RAM and MMIO).
    ///
    /// This implementation will configure the ePMP in the following way:
    ///
    /// - `mseccfg` CSR:
    ///   ```text
    ///   |-------------+-----------------------------------------------+-------|
    ///   | MSECCFG BIT | LABEL                                         | STATE |
    ///   |-------------+-----------------------------------------------+-------|
    ///   |           0 | Machine-Mode Lockdown (MML)                   |     1 |
    ///   |           1 | Machine-Mode Whitelist Policy (MMWP)          |     1 |
    ///   |           2 | Rule-Lock Bypass (RLB)                        |     0 |
    ///   |-------------+-----------------------------------------------+-------|
    ///   ```
    ///
    /// - `pmpaddrX` / `pmpcfgX` CSRs:
    ///   ```text
    ///   |-------+-----------------------------------------+-------+---+-------|
    ///   | ENTRY | REGION / ADDR                           | MODE  | L | PERMS |
    ///   |-------+-----------------------------------------+-------+---+-------|
    ///   |     0 | --------------------------------------- | OFF   | X | ----- |
    ///   |     1 | Kernel .text section                    | TOR   | X | R/X   |
    ///   |       |                                         |       |   |       |
    ///   |     2 | /                                     \ | OFF   |   |       |
    ///   |     3 | \ Userspace TOR region #0             / | TOR   |   | ????? |
    ///   |       |                                         |       |   |       |
    ///   |     4 | /                                     \ | OFF   |   |       |
    ///   |     5 | \ Userspace TOR region #1             / | TOR   |   | ????? |
    ///   |       |                                         |       |   |       |
    ///   | 6 ... | /                                     \ |       |   |       |
    ///   | n - 4 | \ Userspace TOR region #x             / |       |   |       |
    ///   |       |                                         |       |   |       |
    ///   | n - 3 | FLASH (spanning kernel & apps)          | NAPOT | X | R     |
    ///   |       |                                         |       |   |       |
    ///   | n - 2 | RAM (spanning kernel & apps)            | NAPOT | X | R/W   |
    ///   |       |                                         |       |   |       |
    ///   | n - 1 | MMIO                                    | NAPOT | X | R/W   |
    ///   |-------+-----------------------------------------+-------+---+-------|
    ///   ```
    ///
    /// Crucially, this implementation relies on an unconfigured hardware PMP
    /// implementing the ePMP (`mseccfg` CSR) extension, providing the Machine
    /// Lockdown Mode (MML) security bit. This bit is required to ensure that
    /// any machine-mode (kernel) protection regions (lock bit set) are only
    /// accessible to kernel mode.
    pub struct KernelProtectionMMLEPMP<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> {
        user_pmp_enabled: Cell<bool>,
        shadow_user_pmpcfgs: [Cell<TORUserPMPCFG>; MPU_REGIONS],
    }

    impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize>
        KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>
    {
        // Start user-mode TOR regions after the first kernel .text region:
        const TOR_REGIONS_OFFSET: usize = 1;

        pub unsafe fn new(
            flash: FlashRegion,
            ram: RAMRegion,
            mmio: MMIORegion,
            kernel_text: KernelTextRegion,
        ) -> Result<Self, ()> {
            for i in 0..AVAILABLE_ENTRIES {
                // Read the entry's CSR:
                let pmpcfg_csr = csr::CSR.pmpconfig_get(i / 4);

                // Extract the entry's pmpcfg octet:
                let pmpcfg: LocalRegisterCopy<u8, pmpcfg_octet::Register> = LocalRegisterCopy::new(
                    pmpcfg_csr.overflowing_shr(((i % 4) * 8) as u32).0 as u8,
                );

                // As outlined above, we never touch a locked region. Thus, bail
                // out if it's locked:
                if pmpcfg.is_set(pmpcfg_octet::l) {
                    return Err(());
                }

                // Now that it's not locked, we can be sure that regardless of
                // any ePMP bits, this region is either ignored or entirely
                // denied for machine-mode access. Hence, we can change it in
                // arbitrary ways without breaking our own memory access. Try to
                // flip the R/W/X bits:
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr ^ (7 << ((i % 4) * 8)));

                // Check if the CSR changed:
                if pmpcfg_csr == csr::CSR.pmpconfig_get(i / 4) {
                    // Didn't change! This means that this region is not backed
                    // by HW. Return an error as `AVAILABLE_ENTRIES` is
                    // incorrect:
                    return Err(());
                }

                // Finally, turn the region off:
                csr::CSR.pmpconfig_set(i / 4, pmpcfg_csr & !(0x18 << ((i % 4) * 8)));
            }

            // -----------------------------------------------------------------
            // Hardware PMP is verified to be in a compatible mode & state, and
            // has at least `AVAILABLE_ENTRIES` entries. We have not yet checked
            // whether the PMP is actually an _e_PMP. However, we don't want to
            // produce a gadget to set RLB, and so the only safe way to test
            // this is to set up the PMP regions and then try to enable the
            // mseccfg bits.
            // -----------------------------------------------------------------

            // Helper to modify an arbitrary PMP entry. Because we don't know
            // AVAILABLE_ENTRIES in advance, there's no good way to
            // optimize this further.
            fn write_pmpaddr_pmpcfg(i: usize, pmpcfg: u8, pmpaddr: usize) {
                // Important to set the address first. Locking the pmpcfg
                // register will also lock the adress register!
                csr::CSR.pmpaddr_set(i, pmpaddr);
                csr::CSR.pmpconfig_modify(
                    i / 4,
                    FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                        0x000000FF_usize,
                        (i % 4) * 8,
                        u32::from_be_bytes([0, 0, 0, pmpcfg]) as usize,
                    ),
                );
            }

            // Set the kernel `.text`, flash, RAM and MMIO regions, in no
            // particular order, with the exception of `.text` and flash:
            // `.text` must precede flash, as otherwise we'd be revoking execute
            // permissions temporarily. Given that we can currently execute
            // code, this should not have any impact on our accessible memory,
            // assuming that the provided regions are not otherwise aliased.

            // `.text` at n - 5 and n - 4 (TOR region):
            write_pmpaddr_pmpcfg(
                0,
                (pmpcfg_octet::a::OFF
                    + pmpcfg_octet::r::CLEAR
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                kernel_text.0.pmpaddr_a(),
            );
            write_pmpaddr_pmpcfg(
                1,
                (pmpcfg_octet::a::TOR
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::SET
                    + pmpcfg_octet::l::SET)
                    .into(),
                kernel_text.0.pmpaddr_b(),
            );

            // MMIO at n - 1:
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 1,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::SET
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                mmio.0.pmpaddr(),
            );

            // RAM at n - 2:
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 2,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::SET
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                ram.0.pmpaddr(),
            );

            // flash at n - 3:
            write_pmpaddr_pmpcfg(
                AVAILABLE_ENTRIES - 3,
                (pmpcfg_octet::a::NAPOT
                    + pmpcfg_octet::r::SET
                    + pmpcfg_octet::w::CLEAR
                    + pmpcfg_octet::x::CLEAR
                    + pmpcfg_octet::l::SET)
                    .into(),
                flash.0.pmpaddr(),
            );

            // Finally, attempt to enable the MSECCFG security bits, and verify
            // that they have been set correctly. If they have not been set to
            // the written value, this means that this hardware either does not
            // support ePMP, or it was in some invalid state otherwise. We don't
            // need to read back the above regions, as we previous verified that
            // none of their entries were locked -- so writing to them must work
            // even without RLB set.
            //
            // Set RLB(2) = 0, MMWP(1) = 1, MML(0) = 1
            csr::CSR.mseccfg.set(0x00000003);

            // Read back the MSECCFG CSR to ensure that the machine's security
            // configuration was set properly. If this fails, we have set up the
            // PMP in a way that would give userspace access to kernel
            // space. The caller of this method must appropriately handle this
            // error condition by ensuring that the platform will never execute
            // userspace code!
            if csr::CSR.mseccfg.get() != 0x00000003 {
                return Err(());
            }

            // Setup complete
            const DEFAULT_USER_PMPCFG_OCTET: Cell<TORUserPMPCFG> = Cell::new(TORUserPMPCFG::OFF);
            Ok(KernelProtectionMMLEPMP {
                user_pmp_enabled: Cell::new(false),
                shadow_user_pmpcfgs: [DEFAULT_USER_PMPCFG_OCTET; MPU_REGIONS],
            })
        }
    }

    impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> TORUserPMP<MPU_REGIONS>
        for KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>
    {
        // Ensure that the MPU_REGIONS (starting at entry, and occupying two
        // entries per region) don't overflow the available entires, excluding
        // the 7 entries used for implementing the kernel memory protection:
        const CONST_ASSERT_CHECK: () = assert!(MPU_REGIONS <= ((AVAILABLE_ENTRIES - 5) / 2));

        fn available_regions(&self) -> usize {
            // Always assume to have `MPU_REGIONS` usable TOR regions. We don't
            // support locking additional regions at runtime.
            MPU_REGIONS
        }

        // This implementation is specific for 32-bit systems. We use
        // `u32::from_be_bytes` and then cast to usize, as it manages to compile
        // on 64-bit systems as well. However, this implementation will not work
        // on RV64I systems, due to the changed pmpcfgX CSR layout.
        fn configure_pmp(
            &self,
            regions: &[(TORUserPMPCFG, *const u8, *const u8); MPU_REGIONS],
        ) -> Result<(), ()> {
            // Configure all of the regions' addresses and store their pmpcfg octets
            // in our shadow storage. If the user PMP is already enabled, we further
            // apply this configuration (set the pmpcfgX CSRs) by running
            // `enable_user_pmp`:
            for (i, (region, shadow_user_pmpcfg)) in regions
                .iter()
                .zip(self.shadow_user_pmpcfgs.iter())
                .enumerate()
            {
                // The ePMP in MML mode does not support read-write-execute
                // regions. If such a region is to be configured, abort. As this
                // loop here only modifies the shadow state, we can simply abort and
                // return an error. We don't make any promises about the ePMP state
                // if the configuration files, but it is still being activated with
                // `enable_user_pmp`:
                if region.0.get()
                    == <TORUserPMPCFG as From<mpu::Permissions>>::from(
                        mpu::Permissions::ReadWriteExecute,
                    )
                    .get()
                {
                    return Err(());
                }

                // Set the CSR addresses for this region (if its not OFF, in which
                // case the hardware-configured addresses are irrelevant):
                if region.0 != TORUserPMPCFG::OFF {
                    csr::CSR.pmpaddr_set(
                        (i + Self::TOR_REGIONS_OFFSET) * 2 + 0,
                        (region.1 as usize).overflowing_shr(2).0,
                    );
                    csr::CSR.pmpaddr_set(
                        (i + Self::TOR_REGIONS_OFFSET) * 2 + 1,
                        (region.2 as usize).overflowing_shr(2).0,
                    );
                }

                // Store the region's pmpcfg octet:
                shadow_user_pmpcfg.set(region.0);
            }

            // If the PMP is currently active, apply the changes to the CSRs:
            if self.user_pmp_enabled.get() {
                self.enable_user_pmp()?;
            }

            Ok(())
        }

        fn enable_user_pmp(&self) -> Result<(), ()> {
            // We store the "enabled" PMPCFG octets of user regions in the
            // `shadow_user_pmpcfg` field, such that we can re-enable the PMP
            // without a call to `configure_pmp` (where the `TORUserPMPCFG`s are
            // provided by the caller).

            // Could use `iter_array_chunks` once that's stable.
            let mut shadow_user_pmpcfgs_iter = self.shadow_user_pmpcfgs.iter();
            let mut i = Self::TOR_REGIONS_OFFSET;

            while let Some(first_region_pmpcfg) = shadow_user_pmpcfgs_iter.next() {
                // If we're at a "region" offset divisible by two (where "region" =
                // 2 PMP "entries"), then we can configure an entire `pmpcfgX` CSR
                // in one operation. As CSR writes are expensive, this is an
                // operation worth making:
                let second_region_opt = if i % 2 == 0 {
                    shadow_user_pmpcfgs_iter.next()
                } else {
                    None
                };

                if let Some(second_region_pmpcfg) = second_region_opt {
                    // We're at an even index and have two regions to configure, so
                    // do that with a single CSR write:
                    csr::CSR.pmpconfig_set(
                        i / 2,
                        u32::from_be_bytes([
                            second_region_pmpcfg.get().get(),
                            TORUserPMPCFG::OFF.get(),
                            first_region_pmpcfg.get().get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    );

                    i += 2;
                } else if i % 2 == 0 {
                    // This is a single region at an even index. Thus, modify the
                    // first two pmpcfgX octets for this region.
                    csr::CSR.pmpconfig_modify(
                        i / 2,
                        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                            0x0000FFFF,
                            0, // lower two octets
                            u32::from_be_bytes([
                                0,
                                0,
                                first_region_pmpcfg.get().get(),
                                TORUserPMPCFG::OFF.get(),
                            ]) as usize,
                        ),
                    );

                    i += 1;
                } else {
                    // This is a single region at an odd index. Thus, modify the
                    // latter two pmpcfgX octets for this region.
                    csr::CSR.pmpconfig_modify(
                        i / 2,
                        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                            0x0000FFFF,
                            16, // higher two octets
                            u32::from_be_bytes([
                                0,
                                0,
                                first_region_pmpcfg.get().get(),
                                TORUserPMPCFG::OFF.get(),
                            ]) as usize,
                        ),
                    );

                    i += 1;
                }
            }

            self.user_pmp_enabled.set(true);

            Ok(())
        }

        fn disable_user_pmp(&self) {
            // Simply set all of the user-region pmpcfg octets to OFF:

            let mut user_region_pmpcfg_octet_pairs =
                (Self::TOR_REGIONS_OFFSET)..(Self::TOR_REGIONS_OFFSET + MPU_REGIONS);
            while let Some(first_region_idx) = user_region_pmpcfg_octet_pairs.next() {
                let second_region_opt = if first_region_idx % 2 == 0 {
                    user_region_pmpcfg_octet_pairs.next()
                } else {
                    None
                };

                if let Some(_second_region_idx) = second_region_opt {
                    // We're at an even index and have two regions to configure, so
                    // do that with a single CSR write:
                    csr::CSR.pmpconfig_set(
                        first_region_idx / 2,
                        u32::from_be_bytes([
                            TORUserPMPCFG::OFF.get(),
                            TORUserPMPCFG::OFF.get(),
                            TORUserPMPCFG::OFF.get(),
                            TORUserPMPCFG::OFF.get(),
                        ]) as usize,
                    );
                } else if first_region_idx % 2 == 0 {
                    // This is a single region at an even index. Thus, modify the
                    // first two pmpcfgX octets for this region.
                    csr::CSR.pmpconfig_modify(
                        first_region_idx / 2,
                        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                            0x0000FFFF,
                            0, // lower two octets
                            u32::from_be_bytes([
                                0,
                                0,
                                TORUserPMPCFG::OFF.get(),
                                TORUserPMPCFG::OFF.get(),
                            ]) as usize,
                        ),
                    );
                } else {
                    // This is a single region at an odd index. Thus, modify the
                    // latter two pmpcfgX octets for this region.
                    csr::CSR.pmpconfig_modify(
                        first_region_idx / 2,
                        FieldValue::<usize, csr::pmpconfig::pmpcfg::Register>::new(
                            0x0000FFFF,
                            16, // higher two octets
                            u32::from_be_bytes([
                                0,
                                0,
                                TORUserPMPCFG::OFF.get(),
                                TORUserPMPCFG::OFF.get(),
                            ]) as usize,
                        ),
                    );
                }
            }

            self.user_pmp_enabled.set(false);
        }
    }

    impl<const AVAILABLE_ENTRIES: usize, const MPU_REGIONS: usize> fmt::Display
        for KernelProtectionMMLEPMP<AVAILABLE_ENTRIES, MPU_REGIONS>
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                " ePMP configuration:\r\n  mseccfg: {:#08X}, user-mode PMP active: {:?}, entries:\r\n",
                csr::CSR.mseccfg.get(),
                self.user_pmp_enabled.get()
            )?;
            unsafe { super::format_pmp_entries::<AVAILABLE_ENTRIES>(f) }?;

            write!(f, "  Shadow PMP entries for user-mode:\r\n")?;
            for (i, shadowed_pmpcfg) in self.shadow_user_pmpcfgs.iter().enumerate() {
                let (start_pmpaddr_label, startaddr_pmpaddr, endaddr, mode) =
                    if shadowed_pmpcfg.get() == TORUserPMPCFG::OFF {
                        (
                            "pmpaddr",
                            csr::CSR.pmpaddr_get((i + Self::TOR_REGIONS_OFFSET) * 2),
                            0,
                            "OFF",
                        )
                    } else {
                        (
                            "  start",
                            csr::CSR
                                .pmpaddr_get((i + Self::TOR_REGIONS_OFFSET) * 2)
                                .overflowing_shl(2)
                                .0,
                            csr::CSR
                                .pmpaddr_get((i + Self::TOR_REGIONS_OFFSET) * 2 + 1)
                                .overflowing_shl(2)
                                .0
                                | 0b11,
                            "TOR",
                        )
                    };

                write!(
                    f,
                    "  [{:02}]: {}={:#010X}, end={:#010X}, cfg={:#04X} ({}  ) ({}{}{}{})\r\n",
                    (i + Self::TOR_REGIONS_OFFSET) * 2 + 1,
                    start_pmpaddr_label,
                    startaddr_pmpaddr,
                    endaddr,
                    shadowed_pmpcfg.get().get(),
                    mode,
                    if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::l) {
                        "l"
                    } else {
                        "-"
                    },
                    if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::r) {
                        "r"
                    } else {
                        "-"
                    },
                    if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::w) {
                        "w"
                    } else {
                        "-"
                    },
                    if shadowed_pmpcfg.get().get_reg().is_set(pmpcfg_octet::x) {
                        "x"
                    } else {
                        "-"
                    },
                )?;
            }

            Ok(())
        }
    }
}
