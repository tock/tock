// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! I2C master driver.

use core::cell::Cell;

use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

/// The number of bytes the hardware TX/RX FIFOs can each hold.
const I2C_FIFO_LEN: usize = 32;

/// Maximum number of bytes `write()`/`write_read()` can send in one call:
/// the FIFO also has to hold the address byte. See the module
/// documentation for why this driver doesn't stream larger transfers.
const MAX_WRITE_LEN: usize = I2C_FIFO_LEN - 1;

/// Maximum number of bytes `read()`/`write_read()` can receive in one
/// call. See the module documentation.
const MAX_READ_LEN: usize = I2C_FIFO_LEN;

/// GPIO matrix signal indices for the I2C_EXT0 controller's SCL/SDA lines.
///
/// (`I2CEXT0_SCL_IN/OUT_IDX` and `I2CEXT0_SDA_IN/OUT_IDX` in Espressif's
/// `soc/gpio_sig_map.h`; the in/out index is the same signal number for
/// both directions.)
const I2CEXT0_SCL_SIGNAL: u32 = 53;
const I2CEXT0_SDA_SIGNAL: u32 = 54;

/// I2C_EXT0 source clock: the peripheral's clock mux can select either the
/// crystal or the (less accurate) internal RC oscillator; this driver
/// always selects the crystal.
const I2C_SCLK_XTAL_HZ: u32 = 40_000_000;

register_structs! {
    pub I2cRegisters {
        (0x00 => scl_low_period: ReadWrite<u32, TIME9::Register>),
        (0x04 => ctr: ReadWrite<u32, CTR::Register>),
        (0x08 => _reserved0),
        (0x0C => timeout: ReadWrite<u32, TIMEOUT::Register>),
        (0x10 => _reserved1),
        (0x18 => fifo_conf: ReadWrite<u32, FIFO_CONF::Register>),
        (0x1C => fifo_data: ReadWrite<u32, FIFO_DATA::Register>),
        (0x20 => _reserved2),
        (0x24 => int_clr: WriteOnly<u32>),
        (0x28 => int_ena: ReadWrite<u32, INT::Register>),
        (0x2C => int_status: ReadOnly<u32, INT::Register>),
        (0x30 => sda_hold: ReadWrite<u32, TIME9::Register>),
        (0x34 => sda_sample: ReadWrite<u32, TIME9::Register>),
        (0x38 => scl_high_period: ReadWrite<u32, SCL_HIGH_PERIOD::Register>),
        (0x3C => _reserved3),
        (0x40 => scl_start_hold: ReadWrite<u32, TIME9::Register>),
        (0x44 => scl_rstart_setup: ReadWrite<u32, TIME9::Register>),
        (0x48 => scl_stop_hold: ReadWrite<u32, TIME9::Register>),
        (0x4C => scl_stop_setup: ReadWrite<u32, TIME9::Register>),
        (0x50 => _reserved4),
        (0x54 => clk_conf: ReadWrite<u32, CLK_CONF::Register>),
        (0x58 => command: [ReadWrite<u32, COMMAND::Register>; 8]),
        (0x78 => @END),
    }
}

register_bitfields![u32,
    /// Shared layout for `scl_low_period`, `sda_hold`, `sda_sample`,
    /// `scl_start_hold`, `scl_rstart_setup`, `scl_stop_hold`, and
    /// `scl_stop_setup`: each is just a 9-bit core-clock-cycle count.
    TIME9 [
        TIME OFFSET(0) NUMBITS(9) [],
    ],
    CTR [
        SDA_FORCE_OUT OFFSET(0) NUMBITS(1) [],
        SCL_FORCE_OUT OFFSET(1) NUMBITS(1) [],
        MS_MODE OFFSET(4) NUMBITS(1) [],
        TRANS_START OFFSET(5) NUMBITS(1) [],
        CLK_EN OFFSET(8) NUMBITS(1) [],
        CONF_UPGATE OFFSET(11) NUMBITS(1) [],
    ],
    TIMEOUT [
        TIME_OUT_VALUE OFFSET(0) NUMBITS(5) [],
        TIME_OUT_EN OFFSET(5) NUMBITS(1) [],
    ],
    FIFO_CONF [
        RX_FIFO_WM_THRHD OFFSET(0) NUMBITS(5) [],
        TX_FIFO_WM_THRHD OFFSET(5) NUMBITS(5) [],
        NONFIFO_EN OFFSET(10) NUMBITS(1) [],
        RX_FIFO_RST OFFSET(12) NUMBITS(1) [],
        TX_FIFO_RST OFFSET(13) NUMBITS(1) [],
    ],
    FIFO_DATA [
        DATA OFFSET(0) NUMBITS(8) [],
    ],
    /// Shared layout for `int_clr` (write-only, not typed), `int_ena`, and
    /// `int_status`.
    INT [
        RX_FIFO_WM OFFSET(0) NUMBITS(1) [],
        TX_FIFO_WM OFFSET(1) NUMBITS(1) [],
        RX_FIFO_OVF OFFSET(2) NUMBITS(1) [],
        END_DETECT OFFSET(3) NUMBITS(1) [],
        BYTE_TRANS_DONE OFFSET(4) NUMBITS(1) [],
        ARBITRATION_LOST OFFSET(5) NUMBITS(1) [],
        MST_TX_FIFO_UDF OFFSET(6) NUMBITS(1) [],
        TRANS_COMPLETE OFFSET(7) NUMBITS(1) [],
        TIME_OUT OFFSET(8) NUMBITS(1) [],
        TRANS_START OFFSET(9) NUMBITS(1) [],
        NACK OFFSET(10) NUMBITS(1) [],
    ],
    SCL_HIGH_PERIOD [
        PERIOD OFFSET(0) NUMBITS(9) [],
        WAIT_HIGH OFFSET(9) NUMBITS(7) [],
    ],
    CLK_CONF [
        SCLK_DIV_NUM OFFSET(0) NUMBITS(8) [],
        SCLK_SEL OFFSET(20) NUMBITS(1) [],
        SCLK_ACTIVE OFFSET(21) NUMBITS(1) [],
    ],
    COMMAND [
        BYTE_NUM OFFSET(0) NUMBITS(8) [],
        ACK_EN OFFSET(8) NUMBITS(1) [],
        ACK_EXP OFFSET(9) NUMBITS(1) [],
        ACK_VAL OFFSET(10) NUMBITS(1) [],
        OP_CODE OFFSET(11) NUMBITS(3) [],
    ],
];

/// Hardware command opcodes for the `command[]` registers (`COMMAND::OP_CODE`).
const CMD_WRITE: u32 = 1;
const CMD_STOP: u32 = 2;
const CMD_READ: u32 = 3;
const CMD_RESTART: u32 = 6;

/// Pre-computed I2C bus clock timing, calculated the same way as
/// Espressif's `i2c_ll_master_cal_bus_clk()`.
struct ClkConfig {
    clkm_div: u32,
    scl_low: u32,
    scl_wait_high: u32,
    scl_high: u32,
    sda_hold: u32,
    sda_sample: u32,
    setup: u32,
    hold: u32,
    tout: u32,
}

fn calculate_clk_config(source_clk_hz: u32, bus_freq_hz: u32) -> ClkConfig {
    let clkm_div = source_clk_hz / (bus_freq_hz * 1024) + 1;
    let sclk_freq = source_clk_hz / clkm_div;
    let half_cycle = sclk_freq / bus_freq_hz / 2;

    let scl_wait_high = if bus_freq_hz >= 80_000 {
        half_cycle / 2 - 2
    } else {
        half_cycle / 4
    };
    let tout = (32 - (5 * half_cycle).leading_zeros()) + 2;

    ClkConfig {
        clkm_div,
        scl_low: half_cycle,
        scl_wait_high,
        scl_high: half_cycle - scl_wait_high,
        sda_hold: half_cycle / 4,
        sda_sample: half_cycle / 2,
        setup: half_cycle,
        hold: half_cycle,
        tout,
    }
}

/// Default SCL bus frequency used until a board calls [`I2c::set_pins`].
const DEFAULT_BUS_FREQ_HZ: u32 = 100_000;

pub struct I2c<'a> {
    registers: StaticRef<I2cRegisters>,
    configured: Cell<bool>,
    bus_freq_hz: Cell<u32>,

    client: OptionalCell<&'a dyn hil::i2c::I2CHwMasterClient>,
    buffer: TakeCell<'static, [u8]>,
    read_len: Cell<usize>,
    busy: Cell<bool>,
}

impl<'a> I2c<'a> {
    pub fn new(registers: StaticRef<I2cRegisters>) -> I2c<'a> {
        I2c {
            registers,
            bus_freq_hz: Cell::new(DEFAULT_BUS_FREQ_HZ),
            configured: Cell::new(false),
            client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            read_len: Cell::new(0),
            busy: Cell::new(false),
        }
    }

    /// Choose which GPIO pins are wired to SCL/SDA.
    pub fn set_pins(&self, sda: &'a esp32::gpio::GpioPin<'a>, scl: &'a esp32::gpio::GpioPin<'a>) {
        scl.make_open_drain_peripheral_pin(I2CEXT0_SCL_SIGNAL);
        sda.make_open_drain_peripheral_pin(I2CEXT0_SDA_SIGNAL);
    }

    pub fn set_frequency(&self, bus_freq_hz: u32) {
        self.bus_freq_hz.set(bus_freq_hz);
    }

    fn configure(&self) {
        if self.configured.get() {
            return;
        }

        self.registers.ctr.write(
            CTR::MS_MODE::SET
                + CTR::CLK_EN::SET
                + CTR::SDA_FORCE_OUT::SET
                + CTR::SCL_FORCE_OUT::SET,
        );

        self.configure_clock();

        self.configured.set(true);
    }

    fn configure_clock(&self) {
        let cfg = calculate_clk_config(I2C_SCLK_XTAL_HZ, self.bus_freq_hz.get());
        let regs = self.registers;

        // SCLK_SEL: 0 = XTAL, 1 = RC_FAST; this driver always uses XTAL.
        regs.clk_conf.modify(
            CLK_CONF::SCLK_DIV_NUM.val(cfg.clkm_div - 1)
                + CLK_CONF::SCLK_SEL.val(0)
                + CLK_CONF::SCLK_ACTIVE::SET,
        );

        regs.scl_low_period.write(TIME9::TIME.val(cfg.scl_low - 1));
        regs.scl_high_period.write(
            SCL_HIGH_PERIOD::PERIOD.val(cfg.scl_high)
                + SCL_HIGH_PERIOD::WAIT_HIGH.val(cfg.scl_wait_high),
        );
        regs.sda_hold.write(TIME9::TIME.val(cfg.sda_hold - 1));
        regs.sda_sample.write(TIME9::TIME.val(cfg.sda_sample - 1));
        regs.scl_rstart_setup.write(TIME9::TIME.val(cfg.setup - 1));
        regs.scl_stop_setup.write(TIME9::TIME.val(cfg.setup - 1));
        regs.scl_start_hold.write(TIME9::TIME.val(cfg.hold - 1));
        regs.scl_stop_hold.write(TIME9::TIME.val(cfg.hold - 1));
        regs.timeout
            .write(TIMEOUT::TIME_OUT_VALUE.val(cfg.tout) + TIMEOUT::TIME_OUT_EN::SET);

        // Several of the registers above are staged/double-buffered and
        // only take effect once this bit is set.
        regs.ctr.modify(CTR::CONF_UPGATE::SET);
    }

    fn write_command(&self, idx: usize, op_code: u32, byte_num: u8, ack_en: bool, ack_val: bool) {
        self.registers.command[idx].write(
            COMMAND::OP_CODE.val(op_code)
                + COMMAND::BYTE_NUM.val(byte_num as u32)
                + COMMAND::ACK_EN.val(ack_en as u32)
                + COMMAND::ACK_VAL.val(ack_val as u32),
        );
    }

    /// Build the hardware command list for one transaction, pre-load the
    /// TX FIFO with the address byte(s) and any write data, and start it.
    ///
    /// `write_data` is `None` for a plain read; `read_len` is `0` for a
    /// plain write.
    fn start_transaction(&self, addr: u8, write_data: Option<&[u8]>, read_len: usize) {
        let regs = self.registers;

        // Reset both FIFOs so no stale bytes from a previous transaction
        // (e.g. one that ended in an error) linger.
        regs.fifo_conf.modify(FIFO_CONF::TX_FIFO_RST::SET);
        regs.fifo_conf.modify(FIFO_CONF::TX_FIFO_RST::CLEAR);
        regs.fifo_conf.modify(FIFO_CONF::RX_FIFO_RST::SET);
        regs.fifo_conf.modify(FIFO_CONF::RX_FIFO_RST::CLEAR);

        let mut cmd = 0;
        self.write_command(cmd, CMD_RESTART, 0, false, false);
        cmd += 1;

        if let Some(data) = write_data {
            regs.fifo_data
                .write(FIFO_DATA::DATA.val((addr as u32) << 1));
            for &b in data {
                regs.fifo_data.write(FIFO_DATA::DATA.val(b as u32));
            }
            self.write_command(cmd, CMD_WRITE, (data.len() + 1) as u8, true, false);
            cmd += 1;
        }

        if read_len > 0 {
            if write_data.is_some() {
                self.write_command(cmd, CMD_RESTART, 0, false, false);
                cmd += 1;
            }

            regs.fifo_data
                .write(FIFO_DATA::DATA.val(((addr as u32) << 1) | 1));
            self.write_command(cmd, CMD_WRITE, 1, true, false);
            cmd += 1;

            if read_len > 1 {
                // ACK every byte but the last, which is NACK'd below to
                // signal the slave this is the end of the read.
                self.write_command(cmd, CMD_READ, (read_len - 1) as u8, false, false);
                cmd += 1;
            }
            self.write_command(cmd, CMD_READ, 1, false, true);
            cmd += 1;
        }

        self.write_command(cmd, CMD_STOP, 0, false, false);

        regs.int_clr.set(0xFFFF_FFFF);
        regs.int_ena.write(
            INT::NACK::SET
                + INT::TIME_OUT::SET
                + INT::TRANS_COMPLETE::SET
                + INT::ARBITRATION_LOST::SET,
        );

        regs.ctr.modify(CTR::TRANS_START::SET);
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let status = regs.int_status.extract();

        // Matches the priority order of Espressif's own
        // `i2c_ll_master_get_event()`.
        let result = if status.is_set(INT::ARBITRATION_LOST) {
            Some(Err(hil::i2c::Error::ArbitrationLost))
        } else if status.is_set(INT::NACK) {
            // The hardware has one undifferentiated NACK bit for both the
            // address and any data byte; report it as an address NACK,
            // the far more common case (nothing at that address).
            Some(Err(hil::i2c::Error::AddressNak))
        } else if status.is_set(INT::TIME_OUT) {
            // The HIL has no dedicated "bus timeout" error; `Overrun` is
            // the closest available fit.
            Some(Err(hil::i2c::Error::Overrun))
        } else if status.is_set(INT::TRANS_COMPLETE) {
            Some(Ok(()))
        } else {
            None
        };

        if let Some(result) = result {
            regs.int_ena.set(0);
            regs.int_clr.set(0xFFFF_FFFF);
            self.busy.set(false);

            if result.is_ok() {
                let read_len = self.read_len.get();
                if read_len > 0 {
                    self.buffer.map(|buf| {
                        for byte in buf.iter_mut().take(read_len) {
                            *byte = regs.fifo_data.read(FIFO_DATA::DATA) as u8;
                        }
                    });
                }
            }

            self.client.map(|client| {
                self.buffer.take().map(|buf| {
                    client.command_complete(buf, result);
                });
            });
        }
    }
}

impl<'a> hil::i2c::I2CMaster<'a> for I2c<'a> {
    fn set_master_client(&self, master_client: &'a dyn hil::i2c::I2CHwMasterClient) {
        self.client.set(master_client);
    }

    fn enable(&self) {
        self.configure();
    }

    fn disable(&self) {
        self.registers.ctr.set(0);
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: usize,
        read_len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        if self.busy.get() {
            return Err((hil::i2c::Error::Busy, data));
        }
        if write_len == 0
            || read_len == 0
            || write_len > MAX_WRITE_LEN
            || read_len > MAX_READ_LEN
            || write_len > data.len()
            || read_len > data.len()
        {
            return Err((hil::i2c::Error::Overrun, data));
        }

        self.configure();

        self.read_len.set(read_len);
        self.buffer.replace(data);
        self.busy.set(true);
        self.buffer.map(|buf| {
            self.start_transaction(addr, Some(&buf[..write_len]), read_len);
        });

        Ok(())
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        if self.busy.get() {
            return Err((hil::i2c::Error::Busy, data));
        }
        if len == 0 || len > MAX_WRITE_LEN || len > data.len() {
            return Err((hil::i2c::Error::Overrun, data));
        }

        self.configure();

        self.read_len.set(0);
        self.buffer.replace(data);
        self.busy.set(true);
        self.buffer.map(|buf| {
            self.start_transaction(addr, Some(&buf[..len]), 0);
        });

        Ok(())
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: usize,
    ) -> Result<(), (hil::i2c::Error, &'static mut [u8])> {
        if self.busy.get() {
            return Err((hil::i2c::Error::Busy, buffer));
        }
        if len == 0 || len > MAX_READ_LEN || len > buffer.len() {
            return Err((hil::i2c::Error::Overrun, buffer));
        }

        self.configure();

        self.read_len.set(len);
        self.buffer.replace(buffer);
        self.busy.set(true);
        self.start_transaction(addr, None, len);

        Ok(())
    }
}
