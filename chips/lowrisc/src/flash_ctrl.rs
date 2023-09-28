// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Flash Controller

use core::cell::Cell;
use core::ops::{Index, IndexMut};
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::cells::TakeCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

register_structs! {
    pub FlashCtrlRegisters {
        (0x000 => intr_state: ReadWrite<u32, INTR::Register>),
        (0x004 => intr_enable: ReadWrite<u32, INTR::Register>),
        (0x008 => intr_test: WriteOnly<u32, INTR::Register>),
        (0x00C => alert_test: WriteOnly<u32>),
        (0x010 => disable: ReadWrite<u32>),
        (0x014 => exec: ReadWrite<u32>),
        (0x018 => init: ReadWrite<u32, INIT::Register>),
        (0x01C => ctrl_regwen: ReadOnly<u32, CTRL_REGWEN::Register>),
        (0x020 => control: ReadWrite<u32, CONTROL::Register>),
        (0x024 => addr: ReadWrite<u32, ADDR::Register>),
        (0x028 => prog_type_en: ReadWrite<u32, PROG_TYPE_EN::Register>),
        (0x02c => erase_suspend: ReadWrite<u32, ERASE_SUSPEND::Register>),
        (0x030 => region_cfg_regwen: [ReadWrite<u32, REGION_CFG_REGWEN::Register>; 8]),
        (0x050 => mp_region_cfg: [ReadWrite<u32, MP_REGION_CFG::Register>; 8]),
        (0x070 => mp_region: [ReadWrite<u32, MP_REGION::Register>; 8]),
        (0x090 => default_region: ReadWrite<u32, DEFAULT_REGION::Register>),

        (0x094 => bank0_info0_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 10]),
        (0x0BC => bank0_info0_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 10]),
        (0x0E4 => bank0_info1_regwen: ReadWrite<u32, BANK_INFO_REGWEN::Register>),
        (0x0E8 => bank0_info1_page_cfg: ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>),
        (0x0EC => bank0_info2_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 2]),
        (0x0F4 => bank0_info2_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 2]),

        (0x0FC => bank1_info0_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 10]),
        (0x124 => bank1_info0_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 10]),
        (0x14C => bank1_info1_regwen: ReadWrite<u32, BANK_INFO_REGWEN::Register>),
        (0x150 => bank1_info1_page_cfg: ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>),
        (0x154 => bank1_info2_regwen: [ReadWrite<u32, BANK_INFO_REGWEN::Register>; 2]),
        (0x15C => bank1_info2_page_cfg: [ReadWrite<u32, BANK_INFO_PAGE_CFG::Register>; 2]),

        (0x164 => hw_info_cfg_override: ReadWrite<u32>),
        (0x168 => bank_cfg_regwen: ReadWrite<u32, BANK_CFG_REGWEN::Register>),
        (0x16C => mp_bank_cfg_shadowed: ReadWrite<u32, MP_BANK_CFG::Register>),
        (0x170 => op_status: ReadWrite<u32, OP_STATUS::Register>),
        (0x174 => status: ReadOnly<u32, STATUS::Register>),
        (0x178 => debug_state: ReadOnly<u32>),
        (0x17C => err_code: ReadWrite<u32, ERR_CODE::Register>),
        (0x180 => std_fault_status: ReadOnly<u32>),
        (0x184 => fault_status: ReadOnly<u32>),
        (0x188 => err_addr: ReadOnly<u32>),
        (0x18C => ecc_single_err_cnt: ReadOnly<u32>),
        (0x190 => ecc_single_addr: [ReadOnly<u32>; 2]),
        (0x198 => phy_alert_cfg: ReadOnly<u32>),
        (0x19C => phy_status: ReadOnly<u32, PHY_STATUS::Register>),
        (0x1A0 => scratch: ReadWrite<u32, SCRATCH::Register>),
        (0x1A4 => fifo_lvl: ReadWrite<u32, FIFO_LVL::Register>),
        (0x1A8 => fifo_rst: ReadWrite<u32, FIFO_RST::Register>),
        (0x1AC => curr_fifo_lvl: WriteOnly<u32>),
        (0x1B0 => prog_fifo: WriteOnly<u32>),
        (0x1B4 => rd_fifo: ReadOnly<u32>),
        (0x1B8=> @END),
    }
}

register_bitfields![u32,
    INTR [
        PROG_EMPTY OFFSET(0) NUMBITS(1) [],
        PROG_LVL OFFSET(1) NUMBITS(1) [],
        RD_FULL OFFSET(2) NUMBITS(1) [],
        RD_LVL OFFSET(3) NUMBITS(1) [],
        OP_DONE OFFSET(4) NUMBITS(1) [],
        OP_ERROR OFFSET(5) NUMBITS(1) []
    ],
    INIT [
        VAL OFFSET(0) NUMBITS(1) []
    ],
    CTRL_REGWEN [
        EN OFFSET(0) NUMBITS(1) []
    ],
    CONTROL [
        START OFFSET(0) NUMBITS(1) [],
        OP OFFSET(4) NUMBITS(2) [
            READ = 0,
            PROG = 1,
            ERASE = 2
        ],
        PROG_SEL OFFSET(6) NUMBITS(1) [
            NORMAL = 0,
            REPAIR = 1,
        ],
        ERASE_SEL OFFSET(7) NUMBITS(1) [
            PAGE = 0,
            BANK = 1
        ],
        PARTITION_SEL OFFSET(8) NUMBITS(1) [
            // data partition - this is the portion of flash that is
            //     accessible both by the host and by the controller.
            DATA = 0,
            // info partition - this is the portion of flash that is
            //     only accessible by the controller.
            INFO = 1
        ],
        INFO_SEL OFFSET(9) NUMBITS(2) [],
        NUM OFFSET(16) NUMBITS(12) []
    ],
    ERR_CODE [
        OP_ERR OFFSET(0) NUMBITS(1) [],
        MP_ERR OFFSET(1) NUMBITS(1) [],
        RD_ERR OFFSET(2) NUMBITS(1) [],
        PROG_ERR OFFSET(3) NUMBITS(1) [],
        PROG_WIN_ERR OFFSET(4) NUMBITS(1) [],
        PROG_TYPE_ERR OFFSET(5) NUMBITS(1) [],
        UPDATE_ERR OFFSET(6) NUMBITS(1) [],
    ],
    PROG_TYPE_EN [
        NORMAL OFFSET(0) NUMBITS(1) [],
        REPAIR OFFSET(1) NUMBITS(1) [],
    ],
    ERASE_SUSPEND [
        REQ OFFSET(0) NUMBITS(1) [],
    ],
    ADDR [
        START OFFSET(0) NUMBITS(32) []
    ],
    REGION_CFG_REGWEN [
        REGION OFFSET(0) NUMBITS(1) [
            // Once locked, region cannot be modified till next reset
            Locked = 0,
            // Region can be configured.
            Enabled = 1,
        ]
    ],
    MP_REGION_CFG [
        // These config register fields require a special value of
        // 0x6 (0110) to set, or 0x9 (1001) to reset
        EN OFFSET(0) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        RD_EN OFFSET(4) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        PROG_EN OFFSET(8) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ERASE_EN OFFSET(12) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        SCRAMBLE_EN OFFSET(16) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ECC_EN OFFSET(20) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        HE_EN OFFSET(24) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
    ],
    MP_REGION [
        BASE OFFSET(0) NUMBITS(9) [],
        SIZE OFFSET(10) NUMBITS(10) []
    ],
    BANK_INFO_REGWEN [
        REGION OFFSET(0) NUMBITS(1) [
            Locked = 0,
            Enabled =1,
        ]
    ],
    BANK_INFO_PAGE_CFG [
        EN OFFSET(0) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        RD_EN OFFSET(4) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        PROG_EN OFFSET(8) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ERASE_EN OFFSET(12) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        SCRAMBLE_EN OFFSET(16) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ECC_EN OFFSET(20) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        HE_EN OFFSET(24) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
    ],
    BANK_CFG_REGWEN [
        BANK OFFSET(0) NUMBITS(1) []
    ],
    DEFAULT_REGION [
        RD_EN OFFSET(0) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        PROG_EN OFFSET(4) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ERASE_EN OFFSET(8) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        SCRAMBLE_EN OFFSET(12) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        ECC_EN OFFSET(16) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
        HE_EN OFFSET(20) NUMBITS(4) [
            Set = 0x6,
            Clear = 0x9,
        ],
    ],
    MP_BANK_CFG [
        ERASE_EN_0 OFFSET(0) NUMBITS(1) [],
        ERASE_EN_1 OFFSET(1) NUMBITS(1) []
    ],
    OP_STATUS [
        DONE OFFSET(0) NUMBITS(1) [],
        ERR OFFSET(1) NUMBITS(1) []
    ],
    STATUS [
        RD_FULL OFFSET(0) NUMBITS(1) [],
        RD_EMPTY OFFSET(1) NUMBITS(1) [],
        PROG_FULL OFFSET(2) NUMBITS(1) [],
        PROG_EMPTY OFFSET(3) NUMBITS(1) [],
        INIT_WIP OFFSET(4) NUMBITS(1) [],
    ],
    PHY_STATUS [
        INIT_WIP OFFSET(0) NUMBITS(1) [],
        PROG_NORMAL_AVAIL OFFSET(1) NUMBITS(1) [],
        PROG_REPAIR_AVAIL OFFSET(2) NUMBITS(1) []
    ],
    SCRATCH [
        DATA OFFSET(0) NUMBITS(32) []
    ],
    FIFO_LVL [
        PROG OFFSET(0) NUMBITS(5) [],
        RD OFFSET(8) NUMBITS(5) []
    ],
    FIFO_RST [
        EN OFFSET(0) NUMBITS(1) []
    ]
];

pub const PAGE_SIZE: usize = 2048;
pub const FLASH_ADDR_OFFSET: usize = 0x20000000;
pub const FLASH_WORD_SIZE: usize = 8;
pub const FLASH_PAGES_PER_BANK: usize = 256;
pub const FLASH_NUM_BANKS: usize = 2;
pub const FLASH_MAX_PAGES: usize = FLASH_NUM_BANKS * FLASH_PAGES_PER_BANK;
pub const FLASH_NUM_BUSWORDS_PER_BANK: usize = PAGE_SIZE / 4;
pub const FLASH_MP_MAX_CFGS: usize = 8;
// The programming windows size in words (32bit)
pub const FLASH_PROG_WINDOW_SIZE: usize = 16;
pub const FLASH_PROG_WINDOW_MASK: u32 = 0xFFFFFFF0;

pub struct LowRiscPage(pub [u8; PAGE_SIZE]);

/// Defines region permissions for flash memory protection.
/// To be used when requesting the flash controller to set
/// specific permissions for a regions, or when reading
/// the existing permission associated with a region.
#[derive(PartialEq, Debug)]
pub struct FlashMPConfig {
    /// Region can be read.
    pub read_en: bool,
    /// Region can be programmed.
    pub write_en: bool,
    /// Region can be erased
    pub erase_en: bool,
    /// Region is scramble enabled
    pub scramble_en: bool,
    /// Region has ECC enabled
    pub ecc_en: bool,
    /// Region is high endurance enabled
    pub he_en: bool,
}

impl Default for LowRiscPage {
    fn default() -> Self {
        Self([0; PAGE_SIZE])
    }
}

impl Index<usize> for LowRiscPage {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for LowRiscPage {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl AsMut<[u8]> for LowRiscPage {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[derive(PartialEq)]
enum FlashBank {
    BANK0 = 0,
    BANK1 = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum FlashRegion {
    REGION0 = 0,
    REGION1 = 1,
    REGION2 = 2,
    REGION3 = 3,
    REGION4 = 4,
    REGION5 = 5,
    REGION6 = 6,
    REGION7 = 7,
}

pub struct FlashCtrl<'a> {
    registers: StaticRef<FlashCtrlRegisters>,
    flash_client: OptionalCell<&'a dyn hil::flash::Client<FlashCtrl<'a>>>,
    data_configured: Cell<bool>,
    info_configured: Cell<bool>,
    read_buf: TakeCell<'static, LowRiscPage>,
    read_index: Cell<usize>,
    write_buf: TakeCell<'static, LowRiscPage>,
    write_index: Cell<usize>,
    write_word_addr: Cell<usize>,
    region_num: FlashRegion,
}

impl<'a> FlashCtrl<'a> {
    pub fn new(base: StaticRef<FlashCtrlRegisters>, region_num: FlashRegion) -> Self {
        FlashCtrl {
            registers: base,
            flash_client: OptionalCell::empty(),
            data_configured: Cell::new(false),
            info_configured: Cell::new(false),
            read_buf: TakeCell::empty(),
            read_index: Cell::new(0),
            write_buf: TakeCell::empty(),
            write_index: Cell::new(0),
            write_word_addr: Cell::new(0),
            region_num,
        }
    }

    fn enable_interrupts(&self) {
        // Enable relevent interrupts
        self.registers.intr_enable.write(
            INTR::PROG_EMPTY::SET
                + INTR::PROG_LVL::CLEAR
                + INTR::RD_FULL::CLEAR
                + INTR::RD_LVL::SET
                + INTR::OP_DONE::SET
                + INTR::OP_ERROR::SET,
        );
    }

    fn disable_interrupts(&self) {
        // Disable and clear all interrupts
        self.registers.intr_enable.set(0x00);
        self.registers.intr_state.set(0xFFFF_FFFF);
    }

    /// Calculates and returns the max num words that can be programmed without
    /// crossing the programming resolution window boundaries, which
    /// occur at every FLASH_PROG_WINDOW_SIZE words. Note, when setting
    /// the CONTROL::NUM, write (ret_val - 1).
    fn calculate_max_prog_len(&self, word_addr: u32, rem_bytes: u32) -> u32 {
        // Calculate and return the max window limit possible for this transaction in words
        let window_limit =
            ((word_addr + FLASH_PROG_WINDOW_SIZE as u32) & FLASH_PROG_WINDOW_MASK) - word_addr;
        let words_to_write = rem_bytes / 4;

        if words_to_write < window_limit {
            words_to_write
        } else {
            window_limit
        }
    }

    fn configure_data_partition(&self, num: FlashRegion) -> Result<(), ErrorCode> {
        self.registers.default_region.write(
            DEFAULT_REGION::RD_EN::Set
                + DEFAULT_REGION::PROG_EN::Set
                + DEFAULT_REGION::ERASE_EN::Set,
        );

        if let Some(mp_region_cfg) = self.registers.mp_region_cfg.get(num as usize) {
            mp_region_cfg.write(
                MP_REGION_CFG::RD_EN::Set
                    + MP_REGION_CFG::PROG_EN::Set
                    + MP_REGION_CFG::ERASE_EN::Set
                    + MP_REGION_CFG::SCRAMBLE_EN::Clear
                    + MP_REGION_CFG::ECC_EN::Clear
                    + MP_REGION_CFG::EN::Clear,
            );

            if let Some(mp_region) = self.registers.mp_region.get(num as usize) {
                // Size and base are stored in different registers
                mp_region.write(
                    MP_REGION::BASE.val(FLASH_PAGES_PER_BANK as u32) + MP_REGION::SIZE.val(0x1),
                );
            } else {
                return Err(ErrorCode::INVAL);
            }

            // Enable MP Region
            mp_region_cfg.modify(MP_REGION_CFG::EN::Set);
        } else {
            return Err(ErrorCode::INVAL);
        }

        self.data_configured.set(true);
        Ok(())
    }

    fn configure_info_partition(&self, bank: FlashBank, num: FlashRegion) -> Result<(), ErrorCode> {
        if bank == FlashBank::BANK0 {
            if let Some(bank0_info0_page_cfg) =
                self.registers.bank0_info0_page_cfg.get(num as usize)
            {
                bank0_info0_page_cfg.write(
                    BANK_INFO_PAGE_CFG::RD_EN::Set
                        + BANK_INFO_PAGE_CFG::PROG_EN::Set
                        + BANK_INFO_PAGE_CFG::ERASE_EN::Set
                        + BANK_INFO_PAGE_CFG::SCRAMBLE_EN::Set
                        + BANK_INFO_PAGE_CFG::ECC_EN::Set
                        + BANK_INFO_PAGE_CFG::EN::Clear,
                );
                bank0_info0_page_cfg.modify(BANK_INFO_PAGE_CFG::EN::Set);
            } else {
                return Err(ErrorCode::INVAL);
            }
        } else if bank == FlashBank::BANK1 {
            if let Some(bank1_info0_page_cfg) =
                self.registers.bank1_info0_page_cfg.get(num as usize)
            {
                bank1_info0_page_cfg.write(
                    BANK_INFO_PAGE_CFG::RD_EN::Set
                        + BANK_INFO_PAGE_CFG::PROG_EN::Set
                        + BANK_INFO_PAGE_CFG::ERASE_EN::Set
                        + BANK_INFO_PAGE_CFG::SCRAMBLE_EN::Set
                        + BANK_INFO_PAGE_CFG::ECC_EN::Set
                        + BANK_INFO_PAGE_CFG::EN::Clear,
                );
                bank1_info0_page_cfg.modify(BANK_INFO_PAGE_CFG::EN::Set);
            } else {
                return Err(ErrorCode::INVAL);
            }
        } else {
            return Err(ErrorCode::INVAL);
        }
        self.info_configured.set(true);
        Ok(())
    }

    /// Reset the internal FIFOs, used for when recovering from
    /// errors.
    pub fn reset_fifos(&self) {
        // This field is active high, and will hold the FIFO
        // in reset for as long as it is held.
        self.registers.fifo_rst.write(FIFO_RST::EN::SET);
        self.registers.fifo_rst.write(FIFO_RST::EN::CLEAR);
    }

    pub fn handle_interrupt(&self) {
        let irqs = self.registers.intr_state.extract();
        // MP faults don't seem to trigger any errors in intr_state,
        // so lets check for them here.
        let mp_fault = self.registers.err_code.is_set(ERR_CODE::MP_ERR);

        self.disable_interrupts();

        if irqs.is_set(INTR::OP_ERROR) || mp_fault {
            self.registers.op_status.set(0);
            // RW1C Clear any pending errors
            self.registers.err_code.set(0xFFFF_FFFF);
            self.reset_fifos();

            let read_buf = self.read_buf.take();
            let error = if mp_fault {
                hil::flash::Error::FlashMemoryProtectionError
            } else {
                hil::flash::Error::FlashError
            };
            if let Some(buf) = read_buf {
                // We were doing a read
                self.flash_client.map(move |client| {
                    client.read_complete(buf, error);
                });
            }

            let write_buf = self.write_buf.take();
            if let Some(buf) = write_buf {
                // We were doing a write
                self.flash_client.map(move |client| {
                    client.write_complete(buf, error);
                });
            }

            if self.registers.control.matches_all(CONTROL::OP::ERASE) {
                // We were doing an erase
                self.flash_client.map(move |client| {
                    client.erase_complete(error);
                });
            }
        }

        if irqs.is_set(INTR::RD_LVL) {
            self.read_buf.map(|buf| {
                while !self.registers.status.is_set(STATUS::RD_EMPTY)
                    && self.read_index.get() < PAGE_SIZE
                {
                    let data = self.registers.rd_fifo.get().to_ne_bytes();
                    let buf_offset = self.read_index.get();

                    buf[buf_offset] = data[0];
                    buf[buf_offset + 1] = data[1];
                    buf[buf_offset + 2] = data[2];
                    buf[buf_offset + 3] = data[3];

                    self.read_index.set(buf_offset + 4);
                }
                self.enable_interrupts();
            });
        }

        if irqs.is_set(INTR::PROG_EMPTY) {
            self.write_buf.map(|buf| {
                let transaction_word_len = self.calculate_max_prog_len(
                    self.write_word_addr.get() as u32,
                    (buf.0.len() - self.write_index.get()) as u32,
                );

                let mut words_written = 0;

                // Issue program command to the controller
                self.registers.control.write(
                    CONTROL::OP::PROG
                        + CONTROL::PARTITION_SEL::DATA
                        + CONTROL::INFO_SEL::CLEAR
                        + CONTROL::NUM.val(transaction_word_len - 1)
                        + CONTROL::START::CLEAR,
                );

                // Set the address
                self.registers
                    .addr
                    .write(ADDR::START.val(self.write_word_addr.get().saturating_mul(4) as u32));

                // Start the transaction
                self.registers.control.modify(CONTROL::START::SET);

                for i in 0..transaction_word_len {
                    if self.registers.status.is_set(STATUS::PROG_FULL) {
                        words_written = i;
                        break;
                    }
                    let buf_offset = self.write_index.get();
                    let data: u32 = buf[buf_offset] as u32
                        | (buf[buf_offset + 1] as u32) << 8
                        | (buf[buf_offset + 2] as u32) << 16
                        | (buf[buf_offset + 3] as u32) << 24;

                    self.registers.prog_fifo.set(data);

                    self.write_index.set(buf_offset + 4);
                    // loop only semi-inclusive
                    words_written = i + 1;
                }

                self.write_word_addr
                    .set(self.write_word_addr.get() + words_written as usize);
                self.enable_interrupts();
            });
        }

        if irqs.is_set(INTR::OP_DONE) {
            if self.registers.control.matches_all(CONTROL::OP::READ) {
                let read_buf = self.read_buf.take();
                if let Some(buf) = read_buf {
                    // We were doing a read
                    if self.read_index.get() >= buf.0.len() {
                        self.registers.op_status.set(0);
                        // We have all of the data, call the client
                        self.flash_client.map(move |client| {
                            client.read_complete(buf, hil::flash::Error::CommandComplete);
                        });
                    } else {
                        // Still waiting on data, keep waiting
                        self.read_buf.replace(buf);
                        self.enable_interrupts();
                    }
                }
            } else if self.registers.control.matches_all(CONTROL::OP::PROG) {
                let write_buf = self.write_buf.take();
                if let Some(buf) = write_buf {
                    // We were doing a write
                    if self.write_index.get() >= buf.0.len() {
                        self.registers.op_status.set(0);
                        // We sent all of the data, call the client
                        self.flash_client.map(move |client| {
                            client.write_complete(buf, hil::flash::Error::CommandComplete);
                        });
                    } else {
                        // Still writing data, keep trying
                        self.write_buf.replace(buf);
                        self.enable_interrupts();
                    }
                }
            } else if self.registers.control.matches_all(CONTROL::OP::ERASE) {
                self.flash_client.map(move |client| {
                    client.erase_complete(hil::flash::Error::CommandComplete);
                });
            }
        }
    }

    // *** Public API for interfacing to flash memory protection ***

    /// Helper function to convert an address space into page numbers for the flash controller.
    ///
    /// Returns `Ok(start_page_num, n_pages_used)` on successful conversion
    /// Returns `Returns [`NOSUPPORT`](ErrorCode::NOSUPPORT)` if address space is not supported
    ///     or is out of supported bounds
    ///
    /// # Arguments
    ///
    /// * `start_addr`  - Starting address to be converted to a page number
    ///                    Note: This is the absolute address, i.e `FLASH_ADDR_OFFSET` and onwards
    /// * `end_addr`    - End address to be converted to a page number
    ///                    Note: This is the absolute address, i.e `FLASH_ADDR_OFFSET` and onwards
    fn mp_addr_to_page_range(
        &self,
        mut start_addr: usize,
        mut end_addr: usize,
    ) -> Result<(usize, usize), ErrorCode> {
        if start_addr >= end_addr {
            return Err(ErrorCode::NOSUPPORT);
        }

        // Offset Absolute addresses into flash relative addresses
        // i.e 0x20000000 -> 0x00, where 0x00 is the first word in Bank 0, Page 0.
        if let Some(addr) = start_addr.checked_sub(FLASH_ADDR_OFFSET) {
            start_addr = addr;
        } else {
            return Err(ErrorCode::NOSUPPORT);
        }
        if let Some(addr) = end_addr.checked_sub(FLASH_ADDR_OFFSET) {
            end_addr = addr;
        } else {
            return Err(ErrorCode::NOSUPPORT);
        }

        let start_page_num = start_addr.saturating_div(PAGE_SIZE);
        let end_page_num = end_addr.saturating_div(PAGE_SIZE);

        if start_page_num >= FLASH_MAX_PAGES || end_page_num >= FLASH_MAX_PAGES {
            // The address space does not fall within the flash layout
            return Err(ErrorCode::NOSUPPORT);
        }

        // Find the pages utilized by the addr space, at-least one
        // page must be used, even if both start/end addresses fall on the same page.
        let n_pages_used: usize = end_page_num.saturating_sub(start_page_num).max(1);

        // Pages numbers are 0 indexed,
        // For example, if start_page_num is 0 and n_pages_used is 1,
        // then the mp region is defined by page 0.
        // If start_page_num is 0 and n_pages_used is 2, then the region is defined by pages 0 and 1.
        Ok((start_page_num, n_pages_used))
    }

    /// Setup the specified flash memory protection configuration
    ///
    /// Returns `Ok(())` on successfully applying the requested configuration
    /// Returns `[`NOSUPPORT`](ErrorCode::NOSUPPORT)` if address space is not supported,
    ///     or the `region_num` does not exist
    ///
    /// # Arguments
    ///
    /// * `start_addr`  - Starting address that bounds the start of this region.
    ///                    Note: This is the absolute address, i.e `FLASH_ADDR_OFFSET` and onwards
    /// * `end_addr`    - End address that bounds the end of this region
    ///                    Note: This is the absolute address, i.e `FLASH_ADDR_OFFSET` and onwards
    /// * `region_num`  - The configuration region number associated with this region.
    ///                   This associates the specified permissions to a configuration region,
    ///                   the number of simultaneous configs supported
    ///                   should be requested by `mp_get_num_regions()`
    /// * `mp_perms`    - Specifies the permissions to set
    ///
    /// # Examples
    ///
    /// Usage:
    ///
    /// ```ignore
    /// peripherals
    ///     .flash_ctrl
    ///     .mp_set_region_perms(0x0, text_end_addr as usize, 5, &mp_cfg)
    /// ```
    ///
    /// The snippet reads as:
    /// Allow access controls as specified by mp_cfg, for the address space bounded by
    /// `start_addr=0x0` to `end_addr=text_end_addr` and associate this cfg to `region_num=5`.
    ///
    /// If a user would want to modify this region (assuming it wasn't locked), you can index into it with the
    /// associated `region_num`, in this case `5`.
    pub fn mp_set_region_perms(
        &self,
        start_addr: usize,
        end_addr: usize,
        region_num: usize,
        mp_perms: &FlashMPConfig,
    ) -> Result<(), ErrorCode> {
        let (page_number, num_pages) = self.mp_addr_to_page_range(start_addr, end_addr)?;

        if region_num > FlashRegion::REGION7 as usize || page_number >= FLASH_MAX_PAGES {
            return Err(ErrorCode::NOSUPPORT);
        }

        // Number of pages exceeds the number of remaining pages from `page_number`
        if num_pages > FLASH_MAX_PAGES - page_number {
            return Err(ErrorCode::NOSUPPORT);
        }

        let regs = self.registers;

        if !regs.region_cfg_regwen[region_num].is_set(REGION_CFG_REGWEN::REGION) {
            // Region locked, cannot modify until next reset
            return Err(ErrorCode::NOSUPPORT);
        }

        // Clear any existing permissions (reset state)
        self.registers.mp_region_cfg[region_num].write(
            MP_REGION_CFG::EN::Clear
                + MP_REGION_CFG::RD_EN::Clear
                + MP_REGION_CFG::PROG_EN::Clear
                + MP_REGION_CFG::ERASE_EN::Clear
                + MP_REGION_CFG::SCRAMBLE_EN::Clear
                + MP_REGION_CFG::ECC_EN::Clear
                + MP_REGION_CFG::HE_EN::Clear,
        );

        // Set the specified permissions
        if mp_perms.read_en {
            self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::RD_EN::Set);
        }

        if mp_perms.write_en {
            self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::PROG_EN::Set);
        }

        if mp_perms.erase_en {
            self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::ERASE_EN::Set);
        }

        if mp_perms.scramble_en {
            self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::SCRAMBLE_EN::Set);
        }

        if mp_perms.ecc_en {
            self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::ECC_EN::Set);
        }

        if mp_perms.he_en {
            self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::HE_EN::Set);
        }

        // Set the page-range for the cfg to be set
        // For example, if base is 0 and size is 1, then the region is defined by page 0.
        // If base is 0 and size is 2, then the region is defined by pages 0 and 1.
        regs.mp_region[region_num]
            .write(MP_REGION::BASE.val(page_number as u32) + MP_REGION::SIZE.val(num_pages as u32));

        // Activate protection region with specified permissions
        self.registers.mp_region_cfg[region_num].modify(MP_REGION_CFG::EN::Set);

        Ok(())
    }

    /// Read the flash memory protection configuration bounded by the specified region
    ///
    /// Returns `[`FlashMPConfig`](lowrisc::flash_ctrl::FlashMPConfig)` on success, with the permissions
    ///     specified by this region
    /// Returns `[`NOSUPPORT`](ErrorCode::NOSUPPORT)` if the `region_num` does not exist
    ///
    /// # Arguments
    ///
    /// * `region_num`  - The configuration region number associated with this region.
    ///                   This associates the specified permissions to a configuration region.
    pub fn mp_read_region_perms(&self, region_num: usize) -> Result<FlashMPConfig, ErrorCode> {
        if region_num > FlashRegion::REGION7 as usize {
            return Err(ErrorCode::NOSUPPORT);
        }

        let mp_cfg = self.registers.mp_region_cfg[region_num].extract();

        let mut cfg = FlashMPConfig {
            read_en: false,
            write_en: false,
            erase_en: false,
            scramble_en: false,
            ecc_en: false,
            he_en: false,
        };

        if mp_cfg.matches_all(MP_REGION_CFG::RD_EN::Set) {
            cfg.read_en = true;
        }

        if mp_cfg.matches_all(MP_REGION_CFG::PROG_EN::Set) {
            cfg.write_en = true;
        }

        if mp_cfg.matches_all(MP_REGION_CFG::ERASE_EN::Set) {
            cfg.erase_en = true;
        }

        if mp_cfg.matches_all(MP_REGION_CFG::SCRAMBLE_EN::Set) {
            cfg.scramble_en = true;
        }

        if mp_cfg.matches_all(MP_REGION_CFG::ECC_EN::Set) {
            cfg.ecc_en = true;
        }

        if mp_cfg.matches_all(MP_REGION_CFG::HE_EN::Set) {
            cfg.he_en = true;
        }

        Ok(cfg)
    }

    /// Get the number of configuration regions supported by this hardware
    ///
    /// Returns `Ok(FLASH_MP_MAX_CFGS)` where FLASH_MP_MAX_CFGS is the number of
    ///     cfg regions supported
    ///
    /// Note: Indexing starts with 0, this returns the total
    /// number of configuration registers.
    pub fn mp_get_num_regions(&self) -> Result<u32, ErrorCode> {
        Ok(FLASH_MP_MAX_CFGS as u32)
    }

    /// Check if the specified `region_num` is locked by hardware
    ///
    /// Returns `Ok(bool)` on success, if true the region is locked till next reset,
    ///     if false, it is unlocked.
    /// Returns `[`NOSUPPORT`](ErrorCode::NOSUPPORT)` if the `region_num` does not exist
    ///
    /// # Arguments
    ///
    /// * `region_num`  - The configuration region number associated with this region.
    ///                   This associates the specified permissions to a configuration region.
    pub fn mp_is_region_locked(&self, region_num: usize) -> Result<bool, ErrorCode> {
        if region_num > FlashRegion::REGION7 as usize {
            return Err(ErrorCode::NOSUPPORT);
        }

        if !self.registers.region_cfg_regwen[region_num].is_set(REGION_CFG_REGWEN::REGION) {
            // Region locked until next reset
            return Ok(true);
        }
        // Region enabled and can be modified
        Ok(false)
    }

    /// Lock the configuration
    /// Locks the config bounded by `region_num`
    /// such that no further modifications can be made until the next system reset.
    ///
    /// Returns `[`NOSUPPORT`](ErrorCode::NOSUPPORT)` if the `region_num` does not exist
    /// Returns `[`ALREADY`](ErrorCode::ALREADY)` if the `region_num` region is already locked
    /// Returns `Ok(())` on successfully locking the region
    ///
    /// # Arguments
    ///
    /// * `region_num`  - The configuration region number associated with this region.
    ///                   This associates the specified permissions to a configuration region.
    pub fn mp_lock_region_cfg(&self, region_num: usize) -> Result<(), ErrorCode> {
        if region_num > FlashRegion::REGION7 as usize {
            return Err(ErrorCode::NOSUPPORT);
        }

        if !self.registers.region_cfg_regwen[region_num].is_set(REGION_CFG_REGWEN::REGION) {
            // Region already locked
            return Err(ErrorCode::ALREADY);
        }

        self.registers.region_cfg_regwen[region_num].write(REGION_CFG_REGWEN::REGION::Locked);

        Ok(())
    }
}

impl<C: hil::flash::Client<Self>> hil::flash::HasClient<'static, C> for FlashCtrl<'_> {
    fn set_client(&self, client: &'static C) {
        self.flash_client.set(client);
    }
}

impl hil::flash::Flash for FlashCtrl<'_> {
    type Page = LowRiscPage;

    /// The flash controller will truncate to the closest, lower word aligned address.
    /// For example, if 0x13 is supplied, the controller will perform a read at address 0x10.
    fn read_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if page_number >= FLASH_MAX_PAGES {
            return Err((ErrorCode::INVAL, buf));
        }

        let addr = page_number.saturating_mul(PAGE_SIZE);

        if !self.info_configured.get() {
            // The info partitions have no default access. Specifically set up a region.
            if let Err(e) = self.configure_info_partition(FlashBank::BANK1, self.region_num) {
                return Err((e, buf));
            }
        }

        if !self.data_configured.get() {
            // If we aren't configured yet, configure now
            if let Err(e) = self.configure_data_partition(self.region_num) {
                return Err((e, buf));
            }
        }

        // Enable interrupts and set the FIFO level
        self.enable_interrupts();
        self.registers.fifo_lvl.modify(FIFO_LVL::RD.val(0xF));

        // Check control status before we commit
        if !self.registers.ctrl_regwen.is_set(CTRL_REGWEN::EN) {
            return Err((ErrorCode::BUSY, buf));
        }

        // Save the buffer
        self.read_buf.replace(buf);
        self.read_index.set(0);

        // Start the transaction
        self.registers.control.write(
            CONTROL::OP::READ
                + CONTROL::PARTITION_SEL::DATA
                + CONTROL::INFO_SEL::SET
                + CONTROL::NUM.val((FLASH_NUM_BUSWORDS_PER_BANK - 1) as u32)
                + CONTROL::START::CLEAR,
        );

        // Set the address
        self.registers.addr.write(ADDR::START.val(addr as u32));

        // Start Transaction
        self.registers.control.modify(CONTROL::START::SET);

        Ok(())
    }

    /// The flash controller will truncate to the closest, lower word aligned address.
    /// For example, if 0x13 is supplied, the controller will perform a write at address 0x10.
    /// the controller does not have read modified write support.
    fn write_page(
        &self,
        page_number: usize,
        buf: &'static mut Self::Page,
    ) -> Result<(), (ErrorCode, &'static mut Self::Page)> {
        if page_number >= FLASH_MAX_PAGES {
            return Err((ErrorCode::INVAL, buf));
        }
        let addr = page_number.saturating_mul(PAGE_SIZE);

        if !self.info_configured.get() {
            // If we aren't configured yet, configure now
            // The info partitions have no default access. Specifically set up a region.
            if let Err(e) = self.configure_info_partition(FlashBank::BANK1, self.region_num) {
                return Err((e, buf));
            }
        }

        if !self.data_configured.get() {
            // If we aren't configured yet, configure now
            if let Err(e) = self.configure_data_partition(self.region_num) {
                return Err((e, buf));
            }
        }

        // Check control status before we commit
        if !self.registers.ctrl_regwen.is_set(CTRL_REGWEN::EN) {
            return Err((ErrorCode::BUSY, buf));
        }

        // Writes should not cross programming resolution window boundaries, which
        // occur at every FLASH_PROG_WINDOW_SIZE words.
        let word_address = addr / 4;

        let transaction_word_len =
            self.calculate_max_prog_len(word_address as u32, buf.0.len() as u32);

        self.registers.control.write(
            CONTROL::OP::PROG
                + CONTROL::PARTITION_SEL::DATA
                + CONTROL::INFO_SEL::CLEAR
                + CONTROL::NUM.val(transaction_word_len - 1)
                + CONTROL::START::CLEAR,
        );

        // Set the address
        self.registers.addr.write(ADDR::START.val(addr as u32));

        // Reset the write index
        self.write_index.set(0);

        // Start the transaction
        self.registers.control.modify(CONTROL::START::SET);

        let mut words_written = 0;

        // Write the data until we are full or have written all the data
        for i in 0..transaction_word_len {
            if self.registers.status.is_set(STATUS::PROG_FULL) {
                words_written = i;
                break;
            }
            let buf_offset = self.write_index.get();
            let data: u32 = buf[buf_offset] as u32
                | (buf[buf_offset + 1] as u32) << 8
                | (buf[buf_offset + 2] as u32) << 16
                | (buf[buf_offset + 3] as u32) << 24;

            self.registers.prog_fifo.set(data);

            self.write_index.set(buf_offset + 4);
            // loop only semi-inclusive
            words_written = i + 1;
        }

        self.write_word_addr
            .set((addr / 4) + words_written as usize);

        // Save the buffer
        self.write_buf.replace(buf);

        // Enable interrupts and set the FIFO level (interrupt when fully drained)
        self.enable_interrupts();
        self.registers.fifo_lvl.modify(FIFO_LVL::PROG.val(0x00));

        Ok(())
    }

    /// The controller will truncate to the closest lower page aligned address.
    /// Similarly for bank erases, the controller will truncate to the closest
    /// lower bank aligned address.
    fn erase_page(&self, page_number: usize) -> Result<(), ErrorCode> {
        if page_number >= FLASH_MAX_PAGES {
            return Err(ErrorCode::INVAL);
        }
        let addr = page_number.saturating_mul(PAGE_SIZE);

        if !self.data_configured.get() {
            // If we aren't configured yet, configure now
            self.configure_data_partition(self.region_num)?;
        }

        if !self.info_configured.get() {
            // If we aren't configured yet, configure now
            self.configure_info_partition(FlashBank::BANK1, self.region_num)?;
        }

        // Check control status before we commit
        if !self.registers.ctrl_regwen.is_set(CTRL_REGWEN::EN) {
            return Err(ErrorCode::BUSY);
        }

        // Disable bank erase
        for _ in 0..2 {
            self.registers
                .mp_bank_cfg_shadowed
                .modify(MP_BANK_CFG::ERASE_EN_0::CLEAR + MP_BANK_CFG::ERASE_EN_1::CLEAR);
        }

        // Set the address
        self.registers.addr.write(ADDR::START.val(addr as u32));

        // Enable interrupts
        self.enable_interrupts();

        // Start the transaction
        self.registers.control.write(
            CONTROL::OP::ERASE
                + CONTROL::ERASE_SEL::PAGE
                + CONTROL::PARTITION_SEL::DATA
                + CONTROL::START::SET,
        );
        Ok(())
    }
}
