// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// CM33_0/1 CA APB interface
    ICache0Registers {
        /// Cache control
        (0x000 => ctl: ReadWrite<u32, CTL::Register>),
        (0x004 => _reserved0),
        /// Cache command
        (0x008 => cmd: ReadWrite<u32, CMD::Register>),
        (0x00C => _reserved1),
        /// Cache status 0
        (0x080 => status0: ReadWrite<u32>),
        /// Cache status 1
        (0x084 => status1: ReadWrite<u32>),
        /// Cache status 2
        (0x088 => status2: ReadWrite<u32>),
        (0x08C => @END),
    }
}
register_bitfields![u32,
CTL [
    /// Enable ECC checking for cache accesses:
    /// 0: Disabled.
    /// 1: Enabled.
    ECC_EN OFFSET(0) NUMBITS(1) [],
    /// Enable error injection for cache.
    /// When '1', the parity (ECC_CTL.PARITY[6:0]) is used when a cache refill is done to the ECC_CTL.WORD_ADDR[23:0] word address.
    ECC_INJ_EN OFFSET(1) NUMBITS(1) [],
    /// Specifies the cache way for which cache information is provided in STATUS0/1/2.
    WAY OFFSET(16) NUMBITS(2) [],
    /// Specifies the cache set for which cache information is provided in STATUS0/1/2.
    SET_ADDR OFFSET(24) NUMBITS(5) [],
    /// Prefetch enable:
    /// 0: Disabled.
    /// 1: Enabled.
    ///
    /// Prefetching requires the cache to be enabled; i.e. ENABLED is '1'.
    PREF_EN OFFSET(30) NUMBITS(1) [],
    /// Cache enable:
    /// 0: Disabled. The cache tag valid bits are reset to '0's and the cache LRU information is set to '1's (making way 0 the LRU way and way 3 the MRU way).
    /// 1: Enabled.
    CA_EN OFFSET(31) NUMBITS(1) []
],
CMD [
    /// Invalidation of cache and buffer. SW writes a '1' to clear the caches. HW sets this field to '0' when the operation is completed. The caches' LRU structures are also reset to their default state.
    INV OFFSET(0) NUMBITS(1) [],
    /// Invalidation of  buffers (does not invalidate the caches). SW writes a '1' to clear the buffers. HW sets this field to '0' when the operation is completed.
    BUFF_INV OFFSET(1) NUMBITS(1) []
],
STATUS0 [
    /// Sixteen valid bits of the cache line specified by CTL.WAY and CTL.SET_ADDR.
    VALID32 OFFSET(0) NUMBITS(32) []
],
STATUS1 [
    /// Cache line address of the cache line specified by CTL.WAY and CTL.SET_ADDR.
    TAG OFFSET(0) NUMBITS(32) []
],
STATUS2 [
    /// Six bit LRU representation of the cache set specified by CTL.SET_ADDR. The encoding of the field is as follows ('X_LRU_Y' indicates that way X is Less Recently Used than way Y):
    /// Bit 5: 0_LRU_1: way 0 less recently used than way 1.
    /// Bit 4: 0_LRU_2.
    /// Bit 3: 0_LRU_3.
    /// Bit 2: 1_LRU_2.
    /// Bit 1: 1_LRU_3.
    /// Bit 0: 2_LRU_3.
    LRU OFFSET(0) NUMBITS(6) []
]
];
const ICACHE0_BASE: StaticRef<ICache0Registers> =
    unsafe { StaticRef::new(0x42103000 as *const ICache0Registers) };

/// Enable the instruction cache.
/// For system initialization.
pub fn sys_init_enable_cache() {
    ICACHE0_BASE.ctl.modify(CTL::CA_EN::CLEAR);
    ICACHE0_BASE.ctl.modify(CTL::CA_EN::SET);
}
