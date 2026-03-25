// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

use kernel::utilities::registers::interfaces::ReadWriteable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// FLASHC
    FlashcRegisters {
        /// Flash control
        (0x000 => flash_ctl: ReadWrite<u32, FLASH_CTL::Register>),
        (0x004 => _reserved0),
        /// # of sector of a FM
        (0x104 => flash_sector_m: ReadWrite<u32>),
        /// Size of MAIN in 8KB block for each pair of sectors
        (0x108 => flash_main_n: ReadWrite<u32>),
        /// Size of WORK in 8KB block for each pair of sectors
        (0x10C => flash_work_z: ReadWrite<u32>),
        /// Size of SLFASH in 8KB block in Sector 1
        (0x110 => flash_sflash_y: ReadWrite<u32, FLASH_SFLASH_Y::Register>),
        /// Size of refresh rows for each sector
        (0x114 => flash_refresh_row: ReadWrite<u32>),
        (0x118 => _reserved1),
        /// Command
        (0x200 => flash_cmd: ReadWrite<u32>),
        /// Flash Controller Lock Register
        (0x204 => flash_lock: ReadWrite<u32, FLASH_LOCK::Register>),
        /// Flash power control
        (0x208 => flash_pwr_ctl: ReadWrite<u32, FLASH_PWR_CTL::Register>),
        /// Shadow bit of FLASH_CTL.ENFORCE_PC_LOCK
        (0x20C => enforce_pc_lock_shadow: ReadWrite<u32>),
        (0x210 => _reserved2),
        /// ECC injection enable on read
        (0x800 => ecc_inj_en: ReadWrite<u32, ECC_INJ_EN::Register>),
        /// ECC injection control
        (0x804 => ecc_inj_ctl: ReadWrite<u32, ECC_INJ_CTL::Register>),
        /// Interrupt threshold for number of ECC correctable error
        (0x808 => ecc_logir: ReadWrite<u32, ECC_LOGIR::Register>),
        (0x80C => _reserved3),
        /// Config register with error response, RegionID PPC_MPC_MAIN is the security owner PC. The error response configuration is located in CFG.RESPONSE, only one such configuration exists applying to all protection contexts in the system.
        (0x1000 => mpc_cfg: ReadWrite<u32>),
        (0x1004 => _reserved4),
        /// Control register with lock bit and auto-increment only (Separate CTRL for each PC depends on access_pc)
        (0x1100 => mpc_ctrl: ReadWrite<u32, MPC_CTRL::Register>),
        /// Max value of block-based index register
        (0x1104 => mpc_blk_max: ReadWrite<u32>),
        /// Block size & initialization in progress
        (0x1108 => mpc_blk_cfg: ReadWrite<u32, MPC_BLK_CFG::Register>),
        /// Index of 32-block group accessed through BLK_LUT (Separate IDX for each PC depending on access_pc)
        (0x110C => mpc_blk_idx: ReadWrite<u32>),
        /// NS status for 32 blocks at BLK_IDX with PC=<access_pc>
        (0x1110 => mpc_blk_lut: ReadWrite<u32, MPC_BLK_LUT::Register>),
        (0x1114 => _reserved5),
        /// Control register with lock bit and auto-increment only
        (0x1200 => mpc_rot_ctrl: ReadWrite<u32, MPC_ROT_CTRL::Register>),
        (0x1204 => _reserved6),
        /// Max value of block-based index register for ROT
        (0x1208 => mpc_rot_blk_max: ReadWrite<u32>),
        /// Same as BLK_CFG
        (0x120C => mpc_rot_blk_cfg: ReadWrite<u32, MPC_ROT_BLK_CFG::Register>),
        /// Index of 8-block group accessed through ROT_BLK_LUT_*
        (0x1210 => mpc_rot_blk_idx: ReadWrite<u32>),
        /// Protection context of 8-block group accesses through ROT_BLK_LUT
        (0x1214 => mpc_rot_blk_pc: ReadWrite<u32>),
        /// (R,W,NS) bits for 8 blocks at ROT_BLK_IDX for PC=ROT_BKL_PC
        (0x1218 => mpc_rot_blk_lut: ReadWrite<u32, MPC_ROT_BLK_LUT::Register>),
        (0x121C => _reserved7),
        /// Redundancy Control normal sectors 0,1
        (0x2040 => fm_ctl_red_ctl01: ReadWrite<u32, FM_CTL_RED_CTL01::Register>),
        /// Redundancy Control normal sectors 2,3
        (0x2044 => fm_ctl_red_ctl23: ReadWrite<u32, FM_CTL_RED_CTL23::Register>),
        (0x2048 => _reserved8),
        /// Flash macro Page Latches data
        (0x2800 => fm_ctl_fm_pl_datas: [ReadWrite<u32, FM_CTL_FM_PL_DATA::Register>; 132]),
        /// Flash macro control
        (0x2A10 => fm_ctl_flash_macro_ctl: ReadWrite<u32, FM_CTL_FLASH_MACRO_CTL::Register>),
        /// Status
        (0x2A14 => fm_ctl_status: ReadWrite<u32, FM_CTL_STATUS::Register>),
        /// Flash macro address
        (0x2A18 => fm_ctl_fm_addr: ReadWrite<u32, FM_CTL_FM_ADDR::Register>),
        /// Bookmark register - keeps the current FW HV seq
        (0x2A1C => fm_ctl_bookmark: ReadWrite<u32>),
        /// Regular flash geometry
        (0x2A20 => fm_ctl_geometry: ReadWrite<u32, FM_CTL_GEOMETRY::Register>),
        /// Supervisory flash geometry
        (0x2A24 => fm_ctl_geometry_supervisory: ReadWrite<u32, FM_CTL_GEOMETRY_SUPERVISORY::Register>),
        /// Analog control 0
        (0x2A28 => fm_ctl_ana_ctl0: ReadWrite<u32, FM_CTL_ANA_CTL0::Register>),
        /// Analog control 1
        (0x2A2C => fm_ctl_ana_ctl1: ReadWrite<u32, FM_CTL_ANA_CTL1::Register>),
        /// Flash macro write page latches all
        (0x2A30 => fm_ctl_fm_pl_wrdata_all: ReadWrite<u32>),
        /// Address bit to point to scratch area
        (0x2A34 => fm_ctl_fm_refresh_addr: ReadWrite<u32, FM_CTL_FM_REFRESH_ADDR::Register>),
        /// R-grant delay for erase
        (0x2A38 => fm_ctl_rgrant_delay_ers: ReadWrite<u32, FM_CTL_RGRANT_DELAY_ERS::Register>),
        /// R-grant delay scale for erase
        (0x2A3C => fm_ctl_rgrant_scale_ers: ReadWrite<u32, FM_CTL_RGRANT_SCALE_ERS::Register>),
        /// HV Pulse Delay for seq2 post & seq3
        (0x2A40 => fm_ctl_pw_seq23: ReadWrite<u32, FM_CTL_PW_SEQ23::Register>),
        /// HV Pulse Delay for seq 1&2 pre
        (0x2A44 => fm_ctl_pw_seq12: ReadWrite<u32, FM_CTL_PW_SEQ12::Register>),
        /// Wait State control
        (0x2A48 => fm_ctl_wait_ctl: ReadWrite<u32, FM_CTL_WAIT_CTL::Register>),
        /// R-grant delay for program
        (0x2A4C => fm_ctl_rgrant_delay_prg: ReadWrite<u32, FM_CTL_RGRANT_DELAY_PRG::Register>),
        /// Timer prescaler (clk_t to timer clock frequency divider)
        (0x2A50 => fm_ctl_timer_clk_ctl: ReadWrite<u32, FM_CTL_TIMER_CLK_CTL::Register>),
        /// Timer control
        (0x2A54 => fm_ctl_timer_ctl: ReadWrite<u32, FM_CTL_TIMER_CTL::Register>),
        /// MPCON clock
        (0x2A58 => fm_ctl_aclk_ctl: ReadWrite<u32>),
        /// Interrupt
        (0x2A5C => fm_ctl_intr: ReadWrite<u32>),
        /// Interrupt set
        (0x2A60 => fm_ctl_intr_set: ReadWrite<u32>),
        /// Interrupt mask
        (0x2A64 => fm_ctl_intr_mask: ReadWrite<u32>),
        /// Interrupt masked
        (0x2A68 => fm_ctl_intr_masked: ReadWrite<u32>),
        /// Cal control - VCT, VBG, CDAC, IPREF
        (0x2A6C => fm_ctl_cal_ctl0: ReadWrite<u32, FM_CTL_CAL_CTL0::Register>),
        /// Cal control - ICREF, IPREF
        (0x2A70 => fm_ctl_cal_ctl1: ReadWrite<u32, FM_CTL_CAL_CTL1::Register>),
        /// Cal control - IDAC, IBS_CTL, LAT_DIS
        (0x2A74 => fm_ctl_cal_ctl2: ReadWrite<u32, FM_CTL_CAL_CTL2::Register>),
        /// Cal control - OSC trims, FDIV, REG_ACT, TURBO, LP_ULP_SW
        (0x2A78 => fm_ctl_cal_ctl3: ReadWrite<u32, FM_CTL_CAL_CTL3::Register>),
        /// Cal control - VLIM, IDAC, SDAC, ITIM ULP trims
        (0x2A7C => fm_ctl_cal_ctl4: ReadWrite<u32, FM_CTL_CAL_CTL4::Register>),
        /// Cal control - VLIM, IDAC, SDAC, ITIM LP trims
        (0x2A80 => fm_ctl_cal_ctl5: ReadWrite<u32, FM_CTL_CAL_CTL5::Register>),
        /// Cal control - SA CTL LP/ULP trims
        (0x2A84 => fm_ctl_cal_ctl6: ReadWrite<u32, FM_CTL_CAL_CTL6::Register>),
        /// Cal control - ERSX8_CLK_SEL, FM_ACTIVE, TURBO_EXT
        (0x2A88 => fm_ctl_cal_ctl7: ReadWrite<u32, FM_CTL_CAL_CTL7::Register>),
        (0x2A8C => _reserved9),
        /// Flash macro Page Latches ECC
        (0x2B00 => fm_ctl_fm_pl_ecc: [ReadWrite<u32, FM_CTL_FM_PL_ECC::Register>; 33]),
        (0x2B84 => _reserved10),
        /// Flash macro memory sense amplifier data
        (0x2C00 => fm_ctl_fm_mem_datas: [ReadWrite<u32, FM_CTL_FM_MEM_DATA::Register>; 132]),
        (0x2E10 => _reserved11),
        /// Flash macro memory sense amplifier ECC.
        (0x2F00 => fm_ctl_fm_mem_eccs: [ReadWrite<u32, FM_CTL_FM_MEM_ECC::Register>; 33]),
        (0x2F84 => @END),
    }
}
register_bitfields![u32,
FLASH_CTL [
    /// FLASH macro main interface (R-bus) wait states:
    /// '0': 0 wait states.
    /// ...
    /// '15': 15 wait states
    RBUS_WS OFFSET(0) NUMBITS(4) [],
    /// Specifies mapping of FLASH macro main subregion.
    /// 00: MAIN (Mapping A), WORK (Mapping A).
    /// 01: MAIN (Mapping B), WORK (Mapping A).
    /// 10: MAIN (Mapping A), WORK (Mapping B).
    /// 11: MAIN (Mapping B), WORK (Mapping B).
    ///
    /// This field is only used when MAIN_BANK_MODE is '1' (dual bank mode).
    BANK_MAPPING OFFSET(8) NUMBITS(2) [],
    /// Specifies bank mode of FLASH macro main array.
    /// 0: Single bank mode.
    /// 1: Dual bank mode.
    BANK_MODE OFFSET(12) NUMBITS(1) [],
    /// Enable ECC checking for FLASH main (R-bus) interface:
    /// 0: Disabled. ECC checking/reporting on FLASH main interface is disabled. No correctable or non-correctable faults are reported.
    /// 1: Enabled.
    ECC_EN OFFSET(16) NUMBITS(1) [],
    /// Please note that it is SW's responsibility that I$ of M33 must be disabled before setting RBUS_ERR_SILENT HIGH. Otherwise, the erroneous goes to I$ which is NOT desired.
    /// Specifies bus transfer behavior for a non-recoverable error on the FLASH macro main interface (either a non-correctable ECC error, a FLASH macro main interface internal error, a FLASH macro main interface memory hole access):
    /// 0: Bus transfer has a bus error.
    /// 1: Bus transfer does NOT have a bus error; i.e. the error is 'silent'
    /// In either case, the erroneous FLASH macro data is returned to CPU since I$ is disabled.
    ///
    /// This field is ONLY used by CPU bus transfers. Non-CPU bus transfers always have a bus transfer with a bus error and fault/interrupt, in case of a non-recoverable error.
    ///
    /// Note: All CPU bus masters have dedicated status registers (CM33 to register the occurrence of FLASH macro main interface (R-bus) internal errors.
    ///
    /// Note: fault reporting can be used to identify the error that occurred:
    /// - FLASH macro main interface internal error.
    /// - FLASH macro main interface non-recoverable ECC error.
    /// - FLASH macro main interface recoverable ECC error (over its threshold).
    /// - FLASH macro main interface memory hole error.
    RBUS_ERR_SILENT OFFSET(18) NUMBITS(1) [],
    /// This bit can be set once and not cleared thereafter.  When set the PC inheritiance and locking mechanism described with the FLASH_LOCK register is enabled.  When cleared, access to the flash controller and flash macro is possible from any protection context with appropriate PPC permissions.
    ENFORCE_PC_LOCK OFFSET(24) NUMBITS(1) [],
    /// This bit can be set once and not cleared thereafter.  When set it is no longer possible to perform sector erase or sector DFT operations.  The flash controller will block any write operations to the FM_CTL register pertaining to such operations.
    BLOCK_SECTOR_OPERATIONS OFFSET(25) NUMBITS(1) [],
    /// This bit can be set once and not cleared thereafter.  When set it is no longer possible to perform subsector erase or subsector DFT operations.  The flash controller will block any write operations to the FM_CTL register pertaining to such operations.
    BLOCK_SUBSECTOR_OPERATIONS OFFSET(26) NUMBITS(1) [],
    /// This bit can be set once and not cleared thereafter.  When set it is no longer possible to perform bulk erase or bulk DFT operations.  The flash controller will block any write operations to the FM_CTL register pertaining to such operations.
    BLOCK_BULK_OPERATIONS OFFSET(27) NUMBITS(1) []
],
FLASH_SECTOR_M [
    /// # of sectors of a FM, must be an even number, same value as RTL parameter SECTOR_M.
    SECTOR_M OFFSET(0) NUMBITS(32) []
],
FLASH_MAIN_N [
    /// Size of MAIN_NVM in 8KB blocks for each pair of sectors, same value as RTL parameter MAIN_N.
    MAIN_N OFFSET(0) NUMBITS(32) []
],
FLASH_WORK_Z [
    /// Size of WORK_NVM in 8KB blocks for each pair of sectors, same value as RTL parameter WORK_Z, the value WORK_Z can be zero.
    WORK_Z OFFSET(0) NUMBITS(32) []
],
FLASH_SFLASH_Y [
    /// Size of SFLASH_NVM in 8KB blocks in Sector 1, same value as RTL parameter SFLASH_Y.
    SFLASH_Y OFFSET(0) NUMBITS(8) [],
    /// 0: Not allowed
    /// 1: SM only in Sector 1
    SFLASH_SECNUM OFFSET(31) NUMBITS(1) []
],
FLASH_REFRESH_ROW [
    /// Size of refresh rows for each sector, same value as RTL parameter REFRESH_ROW
    /// All sectors of a FM must have the same refresh rows
    /// It is 4 per sector for s40flash.3 FM.
    REFRESH_ROW OFFSET(0) NUMBITS(32) []
],
FLASH_CMD [
    /// Invalidation of ALL buffers. SW writes a '1' to clear the buffers. HW sets this field to '0' when the operation is completed. The operation takes a maximum of three clock cycles.
    BUFF_INV OFFSET(1) NUMBITS(1) []
],
FLASH_LOCK [
    /// When FLASH_LOCK is acquired, even PPC allows, further MMIO access is possible only to this FLASH_LOCK.PC (& same HMASTER_ID) irrespective of corresponding PPC attributes. Other PCs or the same PC but different HMASTER_ID violation results in bus error & operation ignored. There is no further interrupt/fault triggered for this violation.
    PC OFFSET(0) NUMBITS(4) [],
    /// Software writes this register bit to 1 to 'lock' the flash controller to its own protection context (after setting the protection context by writing to a flash memory location).
    /// Once set, any subsequent (AHB) writes result in bus error and operation ignored.
    /// Software (FLASH.PC) reads back this field to check whether the lock succeeded.  Software (FLASH_LOCK.PC) clears this field when it has completed a program/erase operation.
    /// To avoid deadlock and for management purpose, HW entitles PC0 the omnipotent capability to read FLASH_LOCK.PC and to release FLASH_LOCK.LOCKED no matter which PC acquires it.
    /// All PCs allowed by PPC can access FLASH_LOCK when its LOCKED bit is LOW (not locked).
    LOCKED OFFSET(31) NUMBITS(1) []
],
FLASH_PWR_CTL [
    /// Controls 'enable' pin of the Flash memory.
    ENABLE OFFSET(0) NUMBITS(1) [],
    /// Controls 'enable_hv' pin of the Flash memory.
    ENABLE_HV OFFSET(1) NUMBITS(1) []
],
ENFORCE_PC_LOCK_SHADOW [
    /// Shadow register of FLASH_CTL.ENFORCE_PC_LOCK. Read only irrespective of PPC's configuration.
    PC_LOCK_SHADOW OFFSET(0) NUMBITS(1) []
],
ECC_INJ_EN [
    /// Enable ECC error injection for FLASH R-bus interface (while FLASH_CTL.ECC_EN enabled) .
    /// 1'b0: ECC_INJ_EN is disabled.
    /// 1'b1: ECC_INJ_EN is enabled.
    /// Only the PC specified by ECC_INJ_PC can access (read/write)  ECC_INJ_ENABLE and ECC_ERROR when ECC_INJ_ENABLE is high except PC0 which can access it at any time to break the potential deadlock of ECC_INJ_EN.
    ECC_INJ_ENABLE OFFSET(0) NUMBITS(1) [],
    /// 1'b0: If the injected ECC does not trigger any non-recoverable error (ECC errors <= 1).
    /// 1'b1: If the injected ECC triggers non-recoverable error (ECC errors >= 2). The AHB read transaction results in bus error. There is no additional fault/interrupt trigged.
    ECC_ERROR OFFSET(8) NUMBITS(1) [],
    /// The PC is inherited from the master who enabled ECC_INJ_ENABLE (while ECC_INJ_ENABLE is low)
    ECC_INJ_PC OFFSET(28) NUMBITS(4) []
],
ECC_INJ_CTL [
    /// Specifies the word address where an error will be injected.
    /// The word address WORD_ADDR[22:0] is FM column address (module-internal offset), including cxa, bax, axa, sector, row/page definitions. On a FLASH R-bus read and when ECC_INJ_EN bit is '1', and when ECC_INJ_EN.ECC_INJ_PC value matches, the parity (PARITY[8:0]) replaces the FM parity.
    /// When ECC_INJ_ENABLE is 1'b1, only PC specified by ECC_INJ_EN.ECC_INJ_PC can access (read/write) WORD_ADDR  and PARITY.
    WORD_ADDR OFFSET(0) NUMBITS(23) [],
    /// ECC parity to use for ECC error injection at address WORD_ADDR.
    /// The 9-bit ECC PARITY[8:0] is for a 128bit long word.
    PARITY OFFSET(23) NUMBITS(9) []
],
ECC_LOGIR [
    /// Interrupt/fault threshold for number of ECC single-bit failures indicated in
    /// bit[31:16].
    ECCTHRESHOLD OFFSET(0) NUMBITS(16) [],
    /// Number of ECC single-bit failures detected and corrected during the memory read operations
    /// SW writes to register ECC_LOGIR.ECC1CNT are ignored if SW writes any number other than the real RTL counter. But write value of 16'b0 is allowed to clear ECC_LOGIR.ECC1CNT (when the number of ECC single-bit failures detected and corrected reaches threshold value).
    ECC1CNT OFFSET(16) NUMBITS(16) []
],
MPC_CFG [
    /// Response Configuration for Security and PC violations
    /// 0: Read-Zero Write Ignore (RAZ/WI)
    /// 1: Bus Error
    RESPONSE OFFSET(4) NUMBITS(1) []
],
MPC_CTRL [
    /// Auto-increment BLK_IDX by 1 for this protection context as a side effect of each read/write access to BLK_LUT
    AUTO_INC OFFSET(8) NUMBITS(1) [],
    /// Security lockdown for this protection context. Software can set this bit but not clear it once set.  When set, write operations to BLK_LUT are not possible  from this protection context. Setting LOCK also blocks writes to CTRL itself (for that PC copy). All writes are ignored.
    LOCK OFFSET(31) NUMBITS(1) []
],
MPC_BLK_MAX [
    /// Maximum value of block-based index register.  The number and size blocks in an MPC is design time configurable and for embedded memories defaults to covering the entire memory using 4kB blocks; See product datasheet for details on protection of external memories.
    VALUE OFFSET(0) NUMBITS(32) []
],
MPC_BLK_CFG [
    /// Block size of individually protected blocks (0: 32B, 1: 64B, ... up to 15: 1MB)
    /// Block size= (1<<(BLOCK_SIZE+5))
    /// The number and size blocks in an MPC is design time configurable and for embedded memories defaults to covering the entire memory using 4kB blocks; see product datasheet for details on protection of external memories.
    BLOCK_SIZE OFFSET(0) NUMBITS(4) [],
    /// During initialization INIT_IN_PROGRESS is '1' and MMIO register accesses to BLK_LUT is blocked (BLK_IDX increment is also ignored). The block attributes are retained in DeepSleep (and obviously Active) power mode. Initialization is only required from a power mode in which the block attributes are not retained. E.g., initialization is required for a cold boot (after a Power-on-Reset).
    /// HW initializes the block attributes: the NS attributes are set to '0' (secure), the R attributes are set to '1' (read access allowed) and the W attributes are set to '1' (write access allowed). During initialization, the MPC supports memory accesses (memory accesses are NOT blocked) with the initialization block attribute values as mentioned above. This e.g. allows MPC initialization to proceed in parallel with boot program memory accesses (as opposed to serializing the two), improving device boot time.
    INIT_IN_PROGRESS OFFSET(31) NUMBITS(1) []
],
MPC_BLK_IDX [
    /// Index value for accessing block-based lookup table using BLK_LUT. Programming out of LUT range is an user error and it loops back to '0' once overflow occurs.
    VALUE OFFSET(0) NUMBITS(32) []
],
MPC_BLK_LUT [
    /// NS bit for block 0 based on BLK_IDX
    ATTR_NS0 OFFSET(0) NUMBITS(1) [],
    /// NS bit for block 1 based on BLK_IDX
    ATTR_NS1 OFFSET(1) NUMBITS(1) [],
    /// NS bit for block 2 based on BLK_IDX
    ATTR_NS2 OFFSET(2) NUMBITS(1) [],
    /// NS bit for block 3 based on BLK_IDX
    ATTR_NS3 OFFSET(3) NUMBITS(1) [],
    /// NS bit for block 4 based on BLK_IDX
    ATTR_NS4 OFFSET(4) NUMBITS(1) [],
    /// NS bit for block 5 based on BLK_IDX
    ATTR_NS5 OFFSET(5) NUMBITS(1) [],
    /// NS bit for block 6 based on BLK_IDX
    ATTR_NS6 OFFSET(6) NUMBITS(1) [],
    /// NS bit for block 7 based on BLK_IDX
    ATTR_NS7 OFFSET(7) NUMBITS(1) [],
    /// NS bit for block 8 based on BLK_IDX
    ATTR_NS8 OFFSET(8) NUMBITS(1) [],
    /// NS bit for block 9 based on BLK_IDX
    ATTR_NS9 OFFSET(9) NUMBITS(1) [],
    /// NS bit for block 10 based on BLK_IDX
    ATTR_NS10 OFFSET(10) NUMBITS(1) [],
    /// NS bit for block 11 based on BLK_IDX
    ATTR_NS11 OFFSET(11) NUMBITS(1) [],
    /// NS bit for block 12 based on BLK_IDX
    ATTR_NS12 OFFSET(12) NUMBITS(1) [],
    /// NS bit for block 13 based on BLK_IDX
    ATTR_NS13 OFFSET(13) NUMBITS(1) [],
    /// NS bit for block 14 based on BLK_IDX
    ATTR_NS14 OFFSET(14) NUMBITS(1) [],
    /// NS bit for block 15 based on BLK_IDX
    ATTR_NS15 OFFSET(15) NUMBITS(1) [],
    /// NS bit for block 16 based on BLK_IDX
    ATTR_NS16 OFFSET(16) NUMBITS(1) [],
    /// NS bit for block 17 based on BLK_IDX
    ATTR_NS17 OFFSET(17) NUMBITS(1) [],
    /// NS bit for block 18 based on BLK_IDX
    ATTR_NS18 OFFSET(18) NUMBITS(1) [],
    /// NS bit for block 19 based on BLK_IDX
    ATTR_NS19 OFFSET(19) NUMBITS(1) [],
    /// NS bit for block 20 based on BLK_IDX
    ATTR_NS20 OFFSET(20) NUMBITS(1) [],
    /// NS bit for block 21 based on BLK_IDX
    ATTR_NS21 OFFSET(21) NUMBITS(1) [],
    /// NS bit for block 22 based on BLK_IDX
    ATTR_NS22 OFFSET(22) NUMBITS(1) [],
    /// NS bit for block 23 based on BLK_IDX
    ATTR_NS23 OFFSET(23) NUMBITS(1) [],
    /// NS bit for block 24 based on BLK_IDX
    ATTR_NS24 OFFSET(24) NUMBITS(1) [],
    /// NS bit for block 25 based on BLK_IDX
    ATTR_NS25 OFFSET(25) NUMBITS(1) [],
    /// NS bit for block 26 based on BLK_IDX
    ATTR_NS26 OFFSET(26) NUMBITS(1) [],
    /// NS bit for block 27 based on BLK_IDX
    ATTR_NS27 OFFSET(27) NUMBITS(1) [],
    /// NS bit for block 28 based on BLK_IDX
    ATTR_NS28 OFFSET(28) NUMBITS(1) [],
    /// NS bit for block 29 based on BLK_IDX
    ATTR_NS29 OFFSET(29) NUMBITS(1) [],
    /// NS bit for block 30 based on BLK_IDX
    ATTR_NS30 OFFSET(30) NUMBITS(1) [],
    /// NS bit for block 31 based on BLK_IDX
    ATTR_NS31 OFFSET(31) NUMBITS(1) []
],
MPC_ROT_CTRL [
    /// Auto-increment BLK_IDX by 1 for each read/write of ROT_BLK_LUT
    AUTO_INC OFFSET(8) NUMBITS(1) [],
    /// Security lockdown for the root-of-trust configuration registers. Software can set this bit but not clear it once set.  When set, write operations to ROT_BLK_LUT are not possible. Write is ignored.
    LOCK OFFSET(31) NUMBITS(1) []
],
MPC_ROT_BLK_MAX [
    /// Maximum value of block-based index register.  The number and size blocks in an MPC is design time configurable and for embedded memories defaults to covering the entire memory using 4kB blocks; see product datasheet for details on protection of external memories.
    VALUE OFFSET(0) NUMBITS(32) []
],
MPC_ROT_BLK_CFG [
    /// Block size of individually protected blocks (0: 32B, 1: 64B, ...up to 15:1MB)
    /// Block size= (1<<(BLOCK_SIZE+5))
    /// The number and size blocks in an MPC is design time configurable and for embedded memories defaults to covering the entire memory using 4kB blocks; see product datasheet for details on protection of external memories.
    BLOCK_SIZE OFFSET(0) NUMBITS(4) [],
    /// During initialization INIT_IN_PROGRESS is '1' and MMIO register accesses to ROT_BLK_LUT is RAZWI. The block attributes are retained in DeepSleep (and obviously Active) power mode. Initialization is only required from a power mode in which the block attributes are not retained. E.g., initialization is required for a cold boot (after a Power-on-Reset).
    /// HW initializes the block attributes: the NS attributes are set to '0' (secure), the R attributes are set to '1' (read access allowed) and the W attributes are set to '1' (write access allowed). During initialization, the MPC supports memory accesses (memory accesses are NOT blocked) with the initialization block attribute values as mentioned above. This e.g. allows MPC initialization to proceed in parallel with boot program memory accesses (as opposed to serializing the two), improving device boot time.
    INIT_IN_PROGRESS OFFSET(31) NUMBITS(1) []
],
MPC_ROT_BLK_IDX [
    /// Index value for accessing block-based lookup table using ROT_BLK_LUT. Programming out of LUT range is an user error and it loops back to '0' once overflow occurs.
    VALUE OFFSET(0) NUMBITS(32) []
],
MPC_ROT_BLK_PC [
    /// Specify PC values for ROT_BLK_IDX and ROT_BLK_LUT
    PC OFFSET(0) NUMBITS(4) []
],
MPC_ROT_BLK_LUT [
    /// W/R/NS bits for block 0 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR0 OFFSET(0) NUMBITS(3) [],
    /// W/R/NS bits for block 1 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR1 OFFSET(4) NUMBITS(3) [],
    /// W/R/NS bits for block 2 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR2 OFFSET(8) NUMBITS(3) [],
    /// W/R/NS bits for block 3 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR3 OFFSET(12) NUMBITS(3) [],
    /// W/R/NS bits for block 4 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR4 OFFSET(16) NUMBITS(3) [],
    /// W/R/NS bits for block 5 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR5 OFFSET(20) NUMBITS(3) [],
    /// W/R/NS bits for block 6 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR6 OFFSET(24) NUMBITS(3) [],
    /// W/R/NS bits for block 7 indicated by ROT_BLK_IDX for ROT_BLK_PC PC
    ATTR7 OFFSET(28) NUMBITS(3) []
],
FM_CTL_RED_CTL01 [
    /// Bad Row Pair Address for Sector 0
    RED_ADDR_0 OFFSET(0) NUMBITS(8) [],
    /// '1': Redundancy Enable for Sector 0
    RED_EN_0 OFFSET(8) NUMBITS(1) [],
    /// Bad Row Pair Address for Sector 1
    RED_ADDR_1 OFFSET(16) NUMBITS(8) [],
    /// '1': Redundancy Enable for Sector 1
    RED_EN_1 OFFSET(24) NUMBITS(1) [],
    /// '1': Redundancy Enable for SM Rows in Sector 1; Uses RED_ADDR_1 bits for Bad Row Pair Address
    RED_AXA OFFSET(25) NUMBITS(1) []
],
FM_CTL_RED_CTL23 [
    /// Bad Row Pair Address for Sector 2
    RED_ADDR_2 OFFSET(0) NUMBITS(8) [],
    /// 1': Redundancy Enable for Sector 2
    RED_EN_2 OFFSET(8) NUMBITS(1) [],
    /// Bad Row Pair Address for Sector 3
    RED_ADDR_3 OFFSET(16) NUMBITS(8) [],
    /// 1': Redundancy Enable for Sector 3
    RED_EN_3 OFFSET(24) NUMBITS(1) []
],
FM_CTL_FM_PL_DATA [
    /// Normal PL data read: four page latch Bytes
    /// When reading the page latches it requires FM_CTL.IF_SEL to be '1'
    /// Note: the high Voltage page latches are readable for test mode functionality.
    DATA32 OFFSET(0) NUMBITS(32) []
],
FM_CTL_FLASH_MACRO_CTL [
    /// Requires (IF_SEL|WR_EN)=1
    /// Flash macro mode selection
    FM_MODE OFFSET(0) NUMBITS(4) [],
    /// Requires (IF_SEL|WR_EN)=1
    /// Flash macro sequence selection
    FM_SEQ OFFSET(8) NUMBITS(2) [],
    /// Direct memory cell access address.
    DAA_MUX_SEL OFFSET(16) NUMBITS(8) [],
    /// Interface selection. Specifies the interface that is used for flash memory read operations:
    /// 0: R interface is used (default value). In this case, the flash memory address is provided as part of the R signal interface.
    /// 1: C interface is used. In this case, the flash memory address is provided by FM_MEM_ADDR (the page address) and by the C interface access offset in the FM_MEM_DATA structure.
    /// Note: IF_SEL and WR_EN cannot be changed at the same time
    IF_SEL OFFSET(24) NUMBITS(1) [],
    /// 0: normal mode
    /// 1: Fm Write Enable
    /// Note: IF_SEL and WR_EN cannot be changed at the same time
    WR_EN OFFSET(25) NUMBITS(1) []
],
FM_CTL_STATUS [
    /// This is the timer_en bit set by writing a '1' in the TIMER_CTL bit 31. It is reset by HW when the timer expires
    /// 0: timer not running
    /// 1: Timer is enabled and not expired yet
    TIMER_STATUS OFFSET(0) NUMBITS(1) [],
    /// Indicates the isolation status at HV trim and redundancy registers inputs
    /// 0: Not isolated, writing permitted
    /// 1: isolated writing disabled
    HV_REGS_ISOLATED OFFSET(1) NUMBITS(1) [],
    /// Indicates a bulk,sector erase, program has been requested when axa=1
    /// 0: no error
    /// 1: illegal HV operation error
    ILLEGAL_HVOP OFFSET(2) NUMBITS(1) [],
    /// After FM power up indicates the analog blocks currents are boosted to faster reach their functional state..
    /// Used in the testchip boot only as an 'FM READY' flag.
    /// 0: turbo mode
    /// 1: normal mode
    TURBO_N OFFSET(3) NUMBITS(1) [],
    /// FM_CTL.WR_EN bit after being synchronized in clk_r domain
    WR_EN_MON OFFSET(4) NUMBITS(1) [],
    /// FM_CTL.IF_SEL bit after being synchronized in clk_r domain
    IF_SEL_MON OFFSET(5) NUMBITS(1) [],
    /// The actual timer state sync-ed in clk_c domain:
    /// 0: timer is not running:
    /// 1: timer is running;
    TIMER_PE_SYNC OFFSET(6) NUMBITS(1) [],
    /// 0: R_GRANT_DELAY timer is not running
    /// 1: R_GRANT_DELAY timer is running
    R_GRANT_DELAY_STATUS OFFSET(7) NUMBITS(1) [],
    /// 0': FM not busy
    /// 1: FM BUSY : R_GRANT is 0 as result of a busy request from FM ready, or from HV operations.
    FM_BUSY OFFSET(8) NUMBITS(1) [],
    /// 0: FM not ready
    /// 1: FM ready
    FM_READY OFFSET(9) NUMBITS(1) [],
    /// POS pump VLO
    POS_PUMP_VLO OFFSET(10) NUMBITS(1) [],
    /// NEG pump VHI
    NEG_PUMP_VHI OFFSET(11) NUMBITS(1) [],
    /// FM Type  (Read While Write or Not Read While Write):
    /// 0: Non RWW FM Type
    /// 1:  RWW FM Type
    RWW OFFSET(12) NUMBITS(1) [],
    /// Geometry ECC configuration:
    /// 0: FM with No ECC
    /// 1: FM with ECC
    ECC_CFG OFFSET(13) NUMBITS(1) [],
    /// 0:  Sector 1 does not contain special rows. The special rows are located in separate special sectors.
    /// 1:  Sector 1 contains special rows
    SECTOR1_SR OFFSET(14) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  reset_mm
    RESET_MM OFFSET(15) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  row_odd
    ROW_ODD OFFSET(16) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  row_even
    ROW_EVEN OFFSET(17) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  bk_subb
    HVOP_SUB_SECTOR_N OFFSET(18) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  bk_sec
    HVOP_SECTOR OFFSET(19) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  bk_all
    HVOP_BULK_ALL OFFSET(20) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  ra match
    CBUS_RA_MATCH OFFSET(21) NUMBITS(1) [],
    /// Test_only, internal node: mpcon  red_row_en
    CBUS_RED_ROW_EN OFFSET(22) NUMBITS(1) [],
    /// Test_only, internal node:  rq_error  sync-de in clk_c domain
    RQ_ERROR OFFSET(23) NUMBITS(1) [],
    /// Test_only, internal node: regif pdac outputs to pos pump
    PUMP_PDAC OFFSET(24) NUMBITS(4) [],
    /// Test_only, internal node: regif ndac outputs to pos pump
    PUMP_NDAC OFFSET(28) NUMBITS(4) []
],
FM_CTL_FM_ADDR [
    /// Row address.
    RA OFFSET(0) NUMBITS(16) [],
    /// Bank address.
    BA OFFSET(16) NUMBITS(8) [],
    /// Auxiliairy address field:
    /// 0: regular flash memory.
    /// 1: supervisory flash memory.
    AXA OFFSET(24) NUMBITS(1) []
],
FM_CTL_BOOKMARK [
    /// Used by FW. Keeps the Current HV cycle sequence
    BOOKMARK OFFSET(0) NUMBITS(32) []
],
FM_CTL_GEOMETRY [
    /// Number of rows (minus 1):
    /// 0: 1 row
    /// 1: 2 rows
    /// 2: 3 rows
    /// ...
    /// '65535': 65536 rows
    /// For 128kB macro the value of this field  is x7F (128 rows)
    /// For 256kB macro the value of this field  is xFF (256 rows)
    /// For 512kB macro the value of this field  is x1FF (512 rows)
    /// For 1MB macro the value of this field  is x1FF (512 rows)
    ROW_COUNT OFFSET(0) NUMBITS(16) [],
    /// Number of banks (minus 1):
    /// 0: 1 bank
    /// 1: 2 banks
    /// ...
    /// '255': 256 banks
    /// For 128kB, 256kB and 512kB macros the value of this field  is 1 (2 banks)
    /// For 1MB macro the value of this field  is 3 (4 banks)
    BANK_COUNT OFFSET(16) NUMBITS(8) [],
    /// Number of Bytes per word (log 2). A word is defined as the data that is read from the flash macro over the R interface with a single read access:
    /// 0: 1 Byte
    /// 1: 2 Bytes
    /// 2: 4 Bytes
    /// ...
    /// 3: 128 Bytes
    ///
    /// The currently planned flash macros have a word size of either 32-bit, 64-bit or 128-bit, resulting in WORD_SIZE_LOG2 settings of 2, 3 and 4 respectively.
    /// All 4 macros used in PSOC C3 family will see this field as 4
    WORD_SIZE_LOG2 OFFSET(24) NUMBITS(4) [],
    /// Number of Bytes per page (log 2):
    /// 0: 1 Byte
    /// 1: 2 Bytes
    /// 2: 4 Bytes
    /// ...
    /// 15: 32768 Bytes
    ///
    /// The currently planned flash macros have a page size of either 256 Byte or 512 Byte, resulting in PAGE_SIZE_LOG2 settings of 8 and 9 respectively.
    /// All 4 macros used in PSOC C3 family will see this field as 9
    PAGE_SIZE_LOG2 OFFSET(28) NUMBITS(4) []
],
FM_CTL_GEOMETRY_SUPERVISORY [
    /// Number of rows (minus 1). ROW_COUNT is typically less than GEOMETRY.ROW_COUNT
    ROW_COUNT OFFSET(0) NUMBITS(16) [],
    /// Number of banks (minus 1). BANK_COUNT is less or equal to GEOMETRY.BANK_COUNT.
    BANK_COUNT OFFSET(16) NUMBITS(8) [],
    /// Number of Bytes per word (log 2). See GEOMETRY.WORD_SIZE_LOG2. Typically, WORD_SIZE_LOG2 equals GEOMETRY.WORD_SIZE_LOG2.
    WORD_SIZE_LOG2 OFFSET(24) NUMBITS(4) [],
    /// Number of Bytes per page (log 2). See GEOMETRY.PAGE_SIZE_LOG2. Typically, PAGE_SIZE_LOG2 equals GEOMETRY.PAGE_SIZE_LOG2.
    PAGE_SIZE_LOG2 OFFSET(28) NUMBITS(4) []
],
FM_CTL_ANA_CTL0 [
    /// Trimming of the output margin Voltage as a function of Vpos and Vneg.
    MDAC OFFSET(0) NUMBITS(7) [],
    /// Spare bit
    SPARE_ANA_CTL0 OFFSET(7) NUMBITS(1) [],
    /// 0': ECC encoder is enabled for the FM with ECC feature. The PL ECC bits are loaded automatically
    /// '1': ECC encoder disabled. - The Macro needs to be in C-BUS mode - IF_SEL=1 to write and keep this bit at 1
    ECC_ENC_DIS OFFSET(8) NUMBITS(1) [],
    /// Do Not Use this bit as it is for test Mode use only. Write only to 0 in normal mode  (tm_ecc_dis in RTL)
    DNU_2_TM_ECC_DIS OFFSET(9) NUMBITS(1) [],
    /// 1:  Page Latches Soft Reset
    RST_SFT_HVPL OFFSET(10) NUMBITS(1) [],
    /// Flips amuxbusa and amuxbusb
    /// 0: amuxbusa, amuxbusb
    /// 1:  amuxbusb, amuxbusb
    FLIP_AMUXBUS_AB OFFSET(11) NUMBITS(1) [],
    /// NDAC staircase min value
    NDAC_MIN OFFSET(12) NUMBITS(4) [],
    /// PDAC staircase min value
    PDAC_MIN OFFSET(16) NUMBITS(4) [],
    /// PROG&PRE_PROG: Scale for R_GRANT_DELAY on seq0-seq1 transition:
    /// 00: 0.125uS
    /// 01: 1uS
    /// 10: 10uS
    /// 11: 100uS
    SCALE_PRG_SEQ01 OFFSET(20) NUMBITS(2) [],
    /// PROG&PRE_PROG: Scale for R_GRANT_DELAY on seq1-seq2 transition:
    /// 00: 0.125uS
    /// 01: 1uS
    /// 10: 10uS
    /// 11: 100uS
    SCALE_PRG_SEQ12 OFFSET(22) NUMBITS(2) [],
    /// PROG&PRE_PROG: Scale for R_GRANT_DELAY on seq2-seq3 transition:
    /// 00: 0.125uS
    /// 01: 1uS
    /// 10: 10uS
    /// 11: 100uS
    SCALE_PRG_SEQ23 OFFSET(24) NUMBITS(2) [],
    /// PROG&PRE_PROG& ERASE: Scale for R_GRANT_DELAY on seq3-seq0 transition:
    /// 00: 0.125uS
    /// 01: 1uS
    /// 10: 10uS
    /// 11: 100uS
    SCALE_SEQ30 OFFSET(26) NUMBITS(2) [],
    /// PROG&PRE_PROG: Scale for R_GRANT_DELAY on PE On transition:
    /// 00: 0.125uS
    /// 01: 1uS
    /// 10: 10uS
    /// 11: 100uS
    SCALE_PRG_PEON OFFSET(28) NUMBITS(2) [],
    /// PROG&PRE_PROG: Scale for R_GRANT_DELAY on PE OFF transition:
    /// 00: 0.125uS
    /// 01: 1uS
    /// 10: 10uS
    /// 11: 100uS
    SCALE_PRG_PEOFF OFFSET(30) NUMBITS(2) []
],
FM_CTL_ANA_CTL1 [
    /// Ndac Max Value.Trimming of negative pump output Voltage.
    NDAC_MAX OFFSET(0) NUMBITS(4) [],
    /// Ndac step increment
    NDAC_STEP OFFSET(4) NUMBITS(4) [],
    /// Pdac Max Value.Trimming of positive pump output Voltage:
    PDAC_MAX OFFSET(8) NUMBITS(4) [],
    /// Pdac step increment
    PDAC_STEP OFFSET(12) NUMBITS(4) [],
    /// Ndac/Pdac step duration: (1uS .. 255uS) * 8
    /// When = 0 N/PDAC_MAX control the pumps
    NPDAC_STEP_TIME OFFSET(16) NUMBITS(8) [],
    /// Ndac/Pdac LO duration: (1uS .. 255uS) * 8
    /// When 0, N/PDAC don't return to 0
    NPDAC_ZERO_TIME OFFSET(24) NUMBITS(8) []
],
FM_CTL_FM_PL_WRDATA_ALL [
    /// Write all high Voltage page latches with the same 32-bit data in a single write cycle. In order to also write same lower 8bit from 32-bit data to all the ECC bits in the page latches, set ANA_CTL0.ECC_ENC_DIS=1.
    /// Read always returns 0. Used for test mode and sims only
    DATA32 OFFSET(0) NUMBITS(32) []
],
FM_CTL_FM_REFRESH_ADDR [
    /// Address bit to point to scratch area
    /// 0: Point to normal rows in sector
    /// 1: Point to scratch rows in sector. For engineering use only.
    FM_BXA OFFSET(0) NUMBITS(1) [],
    /// Address bit to point to Column 33
    /// 0: Point to normal columns in sector
    /// 1: Point to Column 33 used for BL Disturb Counter. For engineering use only.
    FM_CXA OFFSET(4) NUMBITS(1) []
],
FM_CTL_RGRANT_DELAY_ERS [
    /// ERASE: R-grant blocking delay on seq0-seq1 transition. Scale = ANA_CTL0.SCALE_SEQ01
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_ERS_SEQ01 OFFSET(0) NUMBITS(8) [],
    /// ERASE: R-grant blocking delay on seq1-seq2 transition. Scale = ANA_CTL0.SCALE_SEQ12
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_ERS_SEQ12 OFFSET(8) NUMBITS(8) [],
    /// ERASE: R-grant blocking delay on seq2-seq3 transition. Scale = ANA_CTL0.SCALE_SEQ23
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_ERS_SEQ23 OFFSET(16) NUMBITS(8) []
],
FM_CTL_RGRANT_SCALE_ERS [
    /// ERASE: Scale for R_GRANT_DELAY on seq0-seq1 transition:
    /// '00': 0.125uS
    /// '01': 1uS
    /// '10': 10uS
    /// '11': 100uS
    SCALE_ERS_SEQ01 OFFSET(0) NUMBITS(2) [],
    /// ERASE: Scale for R_GRANT_DELAY on seq1-seq2 transition:
    /// '00': 0.125uS
    /// '01': 1uS
    /// '10': 10uS
    /// '11': 100uS
    SCALE_ERS_SEQ12 OFFSET(2) NUMBITS(2) [],
    /// ERASE: Scale for R_GRANT_DELAY on seq2-seq3 transition:
    /// '00': 0.125uS
    /// '01': 1uS
    /// '10': 10uS
    /// '11': 100uS
    SCALE_ERS_SEQ23 OFFSET(4) NUMBITS(2) [],
    /// ERASE: Scale for R_GRANT_DELAY on PE On transition:
    /// '00': 0.125uS
    /// '01': 1uS
    /// '10': 10uS
    /// '11': 100uS
    SCALE_ERS_PEON OFFSET(6) NUMBITS(2) [],
    /// ERASE: Scale for R_GRANT_DELAY on PE OFF transition:
    /// '00': 0.125uS
    /// '01': 1uS
    /// '10': 10uS
    /// '11': 100uS
    SCALE_ERS_PEOFF OFFSET(8) NUMBITS(2) [],
    /// ERASE: R-grant blocking delay on PE ON. Scale = ANA_CTL0.SCALE_PEON
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_ERS_PEON OFFSET(16) NUMBITS(8) [],
    /// ERASE: R-grant blocking delay on PE OFF. Scale = ANA_CTL0.SCALE_PEOFF
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_ERS_PEOFF OFFSET(24) NUMBITS(8) []
],
FM_CTL_PW_SEQ23 [
    /// Seq2 post delay
    PW_SEQ2_POST OFFSET(0) NUMBITS(16) [],
    /// Seq3 delay
    PW_SEQ3 OFFSET(16) NUMBITS(16) []
],
FM_CTL_PW_SEQ12 [
    /// Seq1 delay
    PW_SEQ1 OFFSET(0) NUMBITS(16) [],
    /// Seq2 pre delay
    PW_SEQ2_PRE OFFSET(16) NUMBITS(16) []
],
FM_CTL_WAIT_CTL [
    /// Number of C interface wait cycles (on 'clk_c') for a read from the memory
    WAIT_FM_MEM_RD OFFSET(0) NUMBITS(4) [],
    /// Number of C interface wait cycles (on 'clk_c') for a read from the Page Latches.
    /// Common for reading HV Page Latches and the DATA_COMP_RESULT bit
    WAIT_FM_HV_RD OFFSET(8) NUMBITS(4) [],
    /// Number of C interface wait cycles (on 'clk_c') for a write to the Page Latches.
    WAIT_FM_HV_WR OFFSET(16) NUMBITS(3) [],
    /// 2'b00: Full CBUS MODE
    /// 2'b01: RWW
    /// 2'b10: RWW. R_GRANT is stalling r_bus for the whole program/erase duration
    FM_RWW_MODE OFFSET(24) NUMBITS(2) [],
    /// Spare register
    LV_SPARE_1 OFFSET(26) NUMBITS(1) [],
    /// Page latch soft set enable, 0 = disabled, 1 = enabled (at end of seq_2), taken care in API
    PL_SOFT_SET_EN OFFSET(29) NUMBITS(1) []
],
FM_CTL_RGRANT_DELAY_PRG [
    /// PROG&PRE_PROG: R-grant blocking delay on seq1-seq2 transition. Scale = ANA_CTL0.SCALE_SEQ12
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_PRG_SEQ12 OFFSET(0) NUMBITS(8) [],
    /// PROG&PRE_PROG: R-grant blocking delay on seq2-seq3 transition. Scale = ANA_CTL0.SCALE_SEQ23
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_PRG_SEQ23 OFFSET(8) NUMBITS(8) [],
    /// PROG&PRE_PROG & ERASE: R-grant blocking delay on seq3-seq0 transition. Scale = ANA_CTL0.SCALE_SEQ30
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_SEQ30 OFFSET(16) NUMBITS(8) [],
    /// Frequency divider from clk_t  to create the 8MHz reference clock for R_grant delay.
    /// The value of 0 is equivalent with 1. If the clock clk_t = 8MHz the value needs to be 1
    /// The value of this field is the integer result of 'clk_t frequency / 8'.
    /// Example: for clk_t=100 this field is INT(100/8) =12.
    /// This field is updated at runtime with the  'SW_RGRANT_DELAY_CLK ' value from the HV parameters table
    RGRANT_DELAY_CLK OFFSET(24) NUMBITS(4) [],
    /// 0': HV Pulse common params not loaded
    /// '1': HV Pulse common params  loaded: r-grant delays, r-grant scale, prescaler, timer values for seq1,seq2_pre, seq2_post, seq3
    HV_PARAMS_LOADED OFFSET(31) NUMBITS(1) []
],
FM_CTL_TIMER_CLK_CTL [
    /// Clk_t frequency divider to provide the 1MHz reference clock for the Regif Timer.
    /// Equal to the frequency in MHz of the timer clock 'clk_t'.
    /// Example: if 'clk_t' has a frequency of 4 MHz then this field value is '4'
    /// Max clk_t frequency = 100MHz.
    /// This field is updated at runtime with the  'SW_TIMER_CLOCK_FREQ ' value from the HV parameters table
    TIMER_CLOCK_FREQ OFFSET(0) NUMBITS(8) [],
    /// PROG&PRE_PROG: R-grant blocking delay on PE ON. Scale = ANA_CTL0.SCALE_PEON
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_PRG_PEON OFFSET(8) NUMBITS(8) [],
    /// PROG&PRE_PROG: R-grant blocking delay on PE OFF. Scale = ANA_CTL0.SCALE_PEOFF
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_PRG_PEOFF OFFSET(16) NUMBITS(8) [],
    /// PROG&PRE_PROG: R-grant blocking delay on seq0-seq1 transition. Scale = ANA_CTL0.SCALE_SEQ01
    /// When = 0  R_GRANT_DELAY control is disabled
    /// when IF_SEL=1  R_GRANT_DELAY control is disabled
    RGRANT_DELAY_PRG_SEQ01 OFFSET(24) NUMBITS(8) []
],
FM_CTL_TIMER_CTL [
    /// Timer period in either microseconds (SCALE is '0') or 100's of microseconds (SCALE is '1') multiples.
    PERIOD OFFSET(0) NUMBITS(15) [],
    /// Timer tick scale:
    /// 0: 1 microsecond.
    /// 1: 100 microseconds.
    SCALE OFFSET(15) NUMBITS(1) [],
    /// 1': Starts1 the HV automatic sequencing
    /// Cleared by HW
    AUTO_SEQUENCE OFFSET(24) NUMBITS(1) [],
    /// 1 during pre-program operation
    PRE_PROG OFFSET(25) NUMBITS(1) [],
    /// 0: CSL lines driven by MDAC
    /// 1: CSL lines driven by VNEG_G
    PRE_PROG_CSL OFFSET(26) NUMBITS(1) [],
    /// Pump enable:
    /// 0: disabled
    /// 1: enabled (also requires FM_CTL.IF_SEL to be'1', this additional restriction is required to prevent non intentional clearing of the FM).
    /// SW sets this field to '1' to generate a single PE pulse.
    /// HW clears this field when timer is expired.
    PUMP_EN OFFSET(29) NUMBITS(1) [],
    /// ACLK enable (generates a single cycle pulse for the FM):
    /// 0: disabled
    /// 1: enabled. SW set this field to '1' to generate a single cycle pulse. HW sets this field to '0' when the pulse is generated.
    ACLK_EN OFFSET(30) NUMBITS(1) [],
    /// Timer enable:
    /// 0: disabled
    /// 1: enabled. SW sets this field to '1' to start the timer. HW sets this field to '0' when the timer is expired.
    TIMER_EN OFFSET(31) NUMBITS(1) []
],
FM_CTL_ACLK_CTL [
    /// Write '1b1'  to generate one clock pulse for HV control registers (mpcon outputs)
    ACLK_GEN OFFSET(0) NUMBITS(1) []
],
FM_CTL_INTR [
    /// Set to '1', when event is detected. Write INTR field with '1', to clear bit. Write INTR_SET field with '1', to set bit.
    TIMER_EXPIRED OFFSET(0) NUMBITS(1) []
],
FM_CTL_INTR_SET [
    /// Write INTR_SET field with '1' to set corresponding INTR field (a write of '0' has no effect).
    TIMER_EXPIRED OFFSET(0) NUMBITS(1) []
],
FM_CTL_INTR_MASK [
    /// Mask for corresponding field in INTR register.
    TIMER_EXPIRED OFFSET(0) NUMBITS(1) []
],
FM_CTL_INTR_MASKED [
    /// Logical and of corresponding request and mask fields.
    TIMER_EXPIRED OFFSET(0) NUMBITS(1) []
],
FM_CTL_CAL_CTL0 [
    /// Bandgap Voltage Temperature Compensation trim control.
    VCT_TRIM_HV OFFSET(0) NUMBITS(5) [],
    /// Temperature compensated trim DAC. To control Vctat slope for VNEG.
    CDAC_HV OFFSET(5) NUMBITS(3) [],
    /// Bandgap Voltage trim control.
    VBG_TRIM_HV OFFSET(8) NUMBITS(6) [],
    /// Bandgap Voltage Temperature Compensation trim control
    VBG_TC_TRIM_HV OFFSET(14) NUMBITS(4) [],
    /// Adds 100-150nA boost on IPREF
    IPREF_TRIMA_HV OFFSET(18) NUMBITS(1) [],
    /// Spare trim bits, DNU
    SPARE_CTL0_HV OFFSET(19) NUMBITS(1) []
],
FM_CTL_CAL_CTL1 [
    /// Bandgap Current  trim control.
    ICREF_TRIM_HV OFFSET(0) NUMBITS(6) [],
    /// Bandgap Current Temperature Compensation trim control
    ICREF_TC_TRIM_HV OFFSET(6) NUMBITS(4) [],
    /// Bandgap IPTAT trim control.
    IPREF_TRIM_HV OFFSET(10) NUMBITS(5) [],
    /// IPREF Slope Control
    IPREF_TC_HV OFFSET(15) NUMBITS(4) [],
    /// Spare trim bit, DNU
    SPARE_CTL1_HV OFFSET(19) NUMBITS(1) []
],
FM_CTL_CAL_CTL2 [
    /// Sets the sense current reference offset value. Refer to trim tables for details.
    IDAC_ULP_HV OFFSET(0) NUMBITS(8) [],
    /// Spare bit to be used in ULP configuration
    SPARE_ULP_CTL2_HV OFFSET(8) NUMBITS(1) [],
    /// 0: Uses VBG as reference for VLIM - ULP Mode
    /// 1: Uses VCTAT as reference for VLIM - ULP mode
    VREF_SEL_ULP_HV OFFSET(9) NUMBITS(1) [],
    /// Sets the sense current reference offset value. Refer to trim tables for details.
    IDAC_LP_HV OFFSET(10) NUMBITS(8) [],
    /// Spare bit to be used in LP configuration
    SPARE_LP_CTL2_HV OFFSET(18) NUMBITS(1) [],
    /// 0: Uses VBG as reference for VLIM - LP Mode
    /// 1: Uses VCTAT as reference for VLIM - LP mode
    VREF_SEL_LP_HV OFFSET(19) NUMBITS(1) []
],
FM_CTL_CAL_CTL3 [
    /// Flash macro pump clock trim control.
    OSC_TRIM_HV OFFSET(0) NUMBITS(4) [],
    /// 0: Oscillator Low Frequency range
    /// 1: Oscillator High Frequency Range
    OSC_RANGE_TRIM_HV OFFSET(4) NUMBITS(1) [],
    /// Forces VPROT in active mode all the time
    VPROT_ACT_HV OFFSET(5) NUMBITS(1) [],
    /// 0: Uses VBG as reference
    /// 1: Uses VCTAT as reference
    OSC_TEMPCO_HV OFFSET(6) NUMBITS(1) [],
    /// 0: Enable saen3 control for data out latches
    /// 1: Disable saen3 control for data out latches
    LAT_DIS3_HV OFFSET(7) NUMBITS(1) [],
    /// 0: Sense Amp bias similar to _ver2
    /// 1: pbias enabled in Sense Amp for Better Margin
    PM_EN_HV OFFSET(8) NUMBITS(1) [],
    /// 0: VBST regulator will operate in active/standby mode based on control signal.
    /// 1: Forces the VBST regulator in active mode all the time
    REG_ACT_HV OFFSET(9) NUMBITS(1) [],
    /// FDIV_TRIM_HV[1:0]: Assuming oscillator frequency of 8MHz in standby.
    /// Following are the clock frequencies seen by doubler
    /// 00: F = 0.5MHz
    /// 01: F = 1MHz
    /// 10: F = 2MHz
    /// 11: F = 4MHz
    FDIV_TRIM_HV OFFSET(10) NUMBITS(2) [],
    /// 0: vdd < 2.3V
    /// 1: vdd >= 2.3V
    /// '0' setting can used for vdd > 2.3V also, but with a current penalty.
    VDDHI_HV OFFSET(12) NUMBITS(1) [],
    /// Turbo pulse width trim (Typical)
    /// 00: 40 us
    /// 01: 20 us
    /// 10: 15 us
    /// 11: 8 us
    TURBO_PULSEW_HV OFFSET(13) NUMBITS(2) [],
    /// Oscillator Bias Current Trim during Standby
    /// 0.33 uA -- 1.65 uA
    IOSC_TRIM_HV OFFSET(15) NUMBITS(2) [],
    /// 0: The internal logic controlls the CL isolation
    /// 1: Forces CL bypass
    CL_ISO_DIS_HV OFFSET(17) NUMBITS(1) [],
    /// 0: r_grant handshake disabled, r_grant always 1.
    /// 1: r_grant handshake  enabled
    R_GRANT_EN_HV OFFSET(18) NUMBITS(1) [],
    /// LP<-->ULP switch for trim signals:
    /// 0: LP
    /// 1: ULP
    LP_ULP_SW_HV OFFSET(19) NUMBITS(1) []
],
FM_CTL_CAL_CTL4 [
    /// VLIM_TRIM[1:0]:
    /// 00: V2 = 650mV
    /// 01: V2 = 700mV
    /// 10: V2 = 750mV - Default
    /// 11: V2 = 800mV
    VLIM_TRIM_ULP_HV OFFSET(0) NUMBITS(2) [],
    /// N/A
    SPARE_CTL4_ULP_HV OFFSET(2) NUMBITS(3) [],
    /// Sets the sense current reference temp slope. Refer to trim tables for details.
    SDAC_ULP_HV OFFSET(5) NUMBITS(2) [],
    /// Trimming of timing current
    ITIM_ULP_HV OFFSET(7) NUMBITS(6) [],
    /// 00: Default : delay 1ns
    /// 01: Delayed by 1.5us
    /// 10: Delayed by 2.0us
    /// 11: Delayed by 2.5us
    FM_READY_DEL_ULP_HV OFFSET(13) NUMBITS(2) [],
    /// saen3 pulse width trim (Current trim)
    SA_CTL_TRIM_T8_ULP_HV OFFSET(15) NUMBITS(1) [],
    /// Toggle: 1-->0, ready goes low, ready will remain low as long as the bit is low. Toggle the bit back to 1 to activate the ready logic. To be used by API only.
    READY_RESTART_N_HV OFFSET(16) NUMBITS(1) [],
    /// 0: VBST_S voltage for each sector to allow VBST level to be dropped to VCC during Erase in the selected sector, reducing coupling to GBL.
    /// 1: VBST_S voltage for each sector stays at VBST level during Erase in the selected sector.
    VBST_S_DIS_HV OFFSET(17) NUMBITS(1) [],
    /// 0: HV Pulse controlled by FW
    /// 1: HV Pulse controlled by Hardware
    AUTO_HVPULSE_HV OFFSET(18) NUMBITS(1) [],
    /// UGB enable in TM control
    UGB_EN_HV OFFSET(19) NUMBITS(1) []
],
FM_CTL_CAL_CTL5 [
    /// VLIM_TRIM[1:0]:
    /// 00: V2 = 650mV
    /// 01: V2 = 700mV
    /// 10: V2 = 750mV - Default
    /// 11: V2 = 800mV
    VLIM_TRIM_LP_HV OFFSET(0) NUMBITS(2) [],
    /// Spare Bit, not used
    SPARE_CTL5_LP_HV OFFSET(2) NUMBITS(3) [],
    /// Sets the sense current reference temp slope. Refer to trim tables for details.
    SDAC_LP_HV OFFSET(5) NUMBITS(2) [],
    /// Trimming of timing current
    ITIM_LP_HV OFFSET(7) NUMBITS(6) [],
    /// 00: Delayed by 1us
    /// 01: Delayed by 1.5us
    /// 10: Delayed by 2.0us
    /// 11: Delayed by 2.5us
    FM_READY_DEL_LP_HV OFFSET(13) NUMBITS(2) [],
    /// saen3 pulse width trim (Current trim)
    SA_CTL_TRIM_T8_LP_HV OFFSET(15) NUMBITS(1) [],
    /// Spare Bit, not used
    SPARE2_CTL5_LP_HV OFFSET(16) NUMBITS(2) [],
    /// Amux Select in AMUX_UGB
    /// 00: Bypass UGB for both amuxbusa and amuxbusb
    /// 01: Bypass UGB for amuxbusb while passing amuxbusa through UGB.
    /// 10: Bypass UGB for amuxbusa while passing amuxbusb through UGB.
    /// 11: UGB Calibrate mode
    AMUX_SEL_HV OFFSET(18) NUMBITS(2) []
],
FM_CTL_CAL_CTL6 [
    /// clk_trk delay
    SA_CTL_TRIM_T1_ULP_HV OFFSET(0) NUMBITS(2) [],
    /// SA_CTL_TRIM_T4_ULP_HV<2>= eqi (eq current trim)
    /// SA_CTL_TRIM_T4_ULP_HV<1:0> = eqc (eq cap trim)
    SA_CTL_TRIM_T4_ULP_HV OFFSET(2) NUMBITS(3) [],
    /// SA_CTL_TRIM_T5_ULP_HV<2>= evi (integration current trim)
    /// SA_CTL_TRIM_T5_ULP_HV<1:0> = evc (integration cap trim)
    SA_CTL_TRIM_T5_ULP_HV OFFSET(5) NUMBITS(3) [],
    /// SA_CTL_TRIM_T6_ULP_HV<1>= eni (enable current trim)
    /// SA_CTL_TRIM_T6_ULP_HV<0> = ecn (enable cap trim)
    SA_CTL_TRIM_T6_ULP_HV OFFSET(8) NUMBITS(2) [],
    /// clk_trk delay
    SA_CTL_TRIM_T1_LP_HV OFFSET(10) NUMBITS(2) [],
    /// SA_CTL_TRIM_T4_LP_HV<2>= eqi (eq current trim)
    /// SA_CTL_TRIM_T4_LP_HV<1:0> = eqc (eq cap trim)
    SA_CTL_TRIM_T4_LP_HV OFFSET(12) NUMBITS(3) [],
    /// SA_CTL_TRIM_T5_LP_HV<2>= evi (integration current trim)
    /// SA_CTL_TRIM_T5_LP_HV<1:0> = evc (integration cap trim)
    SA_CTL_TRIM_T5_LP_HV OFFSET(15) NUMBITS(3) [],
    /// SA_CTL_TRIM_T6_LP_HV<1>= eni (enable current trim)
    /// SA_CTL_TRIM_T6_LP_HV<0> = ecn (enable cap trim)
    SA_CTL_TRIM_T6_LP_HV OFFSET(18) NUMBITS(2) []
],
FM_CTL_CAL_CTL7 [
    /// Clock frequency into the ersx8 shift register block
    /// 00: Oscillator clock
    /// 01: Oscillator clock / 2
    /// 10: Oscillator clock / 4
    /// 11: Oscillator clock / 8
    ERSX8_CLK_SEL_HV OFFSET(0) NUMBITS(2) [],
    /// 0: Normal operation
    /// 1: Forces FM SYS in active mode
    FM_ACTIVE_HV OFFSET(2) NUMBITS(1) [],
    /// 0: Normal operation
    /// 1: Uses external turbo pulse
    TURBO_EXT_HV OFFSET(3) NUMBITS(1) [],
    /// 0': ndac, pdac staircase hardware controlled
    /// 1: ndac, pdac staircase disabled. Enables FW control.
    NPDAC_HWCTL_DIS_HV OFFSET(4) NUMBITS(1) [],
    /// 0': fm ready is enabled
    /// 1: fm ready is disabled (fm_ready is always '1')
    FM_READY_DIS_HV OFFSET(5) NUMBITS(1) [],
    /// 0': Staggered turn on/off of GWL
    /// 1: GWL are turned on/off at the same time (old FM legacy)
    ERSX8_EN_ALL_HV OFFSET(6) NUMBITS(1) [],
    /// 0: Ready Delay trim.
    /// 1: Ready Delay trim.
    READY_DEL_HV OFFSET(7) NUMBITS(1) [],
    /// N/A
    SPARE_CTL7_HV OFFSET(8) NUMBITS(2) [],
    /// SA PBIAS Trim for ULP Operation
    PTRIM_ULP_HV OFFSET(10) NUMBITS(2) [],
    /// N/A
    SPARE2_CTL7_ULP_HV OFFSET(12) NUMBITS(3) [],
    /// SA PBIAS Trim for LP Operation
    PTRIM_LP_HV OFFSET(15) NUMBITS(2) [],
    /// N/A
    SPARE3_CTL7_LP_HV OFFSET(17) NUMBITS(3) []
],
FM_CTL_FM_PL_ECC [
    /// if_sel must be 1
    /// Normal PL ECC  data read: one page latch Byte
    /// When reading the page latches it requires FM_CTL.IF_SEL to be '1'
    /// Note: the high Voltage page latches are readable for test mode functionality.
    /// Only even addresses, address step =8.
    DATA9 OFFSET(0) NUMBITS(9) [],
    /// ECC encoder output. Can be read any time at address 0xa00
    DATA9_1 OFFSET(16) NUMBITS(9) []
],
FM_CTL_FM_MEM_DATA [
    /// Sense amplifier and column multiplexer structure Bytes. The read data is dependent on FM_CTL.IF_SEL
    /// - IF_SEL is 0: data as specified by the R interface address; If accessed when IF_SEL=0, the data read is not valid.
    /// - IF_SEL is 1: data as specified by FM_MEM_ADDR and the offset of the accessed FM_MEM_DATA register.
    ///
    /// Four FM data out Bytes.
    /// Each access is a full FM core read through C-BUS.
    /// The row address is given by the FM_ADDRESS register, the word and column addresses are driven by c_addr bus.
    DATA32 OFFSET(0) NUMBITS(32) []
],
FM_CTL_FM_MEM_ECC [
    /// Sense amplifier ECC Bits.  FM_CTL.IF_SEL must be 1.
    /// FM ECC data.
    /// Each access is a full FM core read through C-BUS.
    /// The row address is given by the FM_ADDRESS register, the word and column addresses are driven by c_addr bus.
    ECC_PARITY OFFSET(0) NUMBITS(9) []
],
];
const FLASHC_BASE: StaticRef<FlashcRegisters> =
    unsafe { StaticRef::new(0x42150000 as *const FlashcRegisters) };

pub struct FlashC {
    registers: StaticRef<FlashcRegisters>,
}

impl FlashC {
    pub const fn new() -> FlashC {
        FlashC {
            registers: FLASHC_BASE,
        }
    }

    /// Set the number of flash wait states.
    ///
    /// This updates `FLASH_CTL.RBUS_WS`. Call this before increasing `HFClk0`.
    ///
    /// # Arguments
    ///
    /// - `ulp_mode`: Target power mode.
    ///   - `true`: ULP mode (core regulator nominal `0.9V`).
    ///   - `false`: LP mode (core regulator nominal `1.1V`).
    /// - `clk_hf_mhz`: `HFClk0` frequency in `MHz`.
    ///   Values above the supported maximum are treated as the maximum.
    pub fn set_waitstates(&self, ulp_mode: bool, clk_hf_mhz: u32) {
        const COEFFICIENT_ULP: u32 = 80;
        const COEFFICIENT: u32 = 60;
        let wait_states = if ulp_mode {
            ((COEFFICIENT_ULP * clk_hf_mhz) / 1000) + 1
        } else {
            ((COEFFICIENT * clk_hf_mhz) / 1000) + 1
        };
        self.registers
            .flash_ctl
            .modify(FLASH_CTL::RBUS_WS.val(wait_states));
    }
}
