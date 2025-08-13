// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Serial Peripheral Interface (SPI) Host Driver
use core::cell::Cell;
use core::cmp;
use kernel::hil;
use kernel::hil::spi::SpiMaster;
use kernel::hil::spi::{ClockPhase, ClockPolarity};
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpiHostStatus {
    SpiTransferCmplt,
    SpiTransferInprog,
}

register_structs! {
    pub SpiHostRegisters {
        //SPI: Interrupt State Register, type rw1c
        (0x000 => intr_state: ReadWrite<u32, intr::Register>),
        //SPI: Interrupt Enable Register
        (0x004 => intr_enable: ReadWrite<u32, intr::Register>),
        //SPI: Interrupt Test Register
        (0x008 => intr_test: WriteOnly<u32, intr::Register>),
        //SPI: Alert Test Register
        (0x00c => alert_test: WriteOnly<u32, alert_test::Register>),
        //SPI: Control register
        (0x010 => ctrl: ReadWrite<u32, ctrl::Register>),
        //SPI: Status register
        (0x014 => status: ReadOnly<u32, status::Register>),
        //SPI: Configuration options register.
        (0x018 => config_opts: ReadWrite<u32, conf_opts::Register>),
        //SPI: Chip-Select ID
        (0x01c => csid: ReadWrite<u32, csid_ctrl::Register>),
        //SPI: Command Register
        (0x020 => command: WriteOnly<u32, command::Register>),
        //SPI: Received Data
        (0x024 => rx_data: ReadWrite<u32, rx_data::Register>),
        //SPI: Transmit Data
        (0x028 => tx_data: WriteOnly<u32, tx_data::Register>),
        //SPI: Controls which classes of errors raise an interrupt.
        (0x02c => err_en: ReadWrite<u32, err_en::Register>),
        //SPI: Indicates that any errors that have occurred, type rw1c
        (0x030 => err_status: ReadWrite<u32, err_status::Register>),
        //SPI: Controls which classes of SPI events raise an interrupt
        (0x034 => event_en: ReadWrite<u32, event_en::Register>),
        (0x38 => @END),
    }
}

register_bitfields![u32,
    intr [
        ERROR OFFSET(0) NUMBITS(1) [],
        SPI_EVENT OFFSET(1) NUMBITS(1) [],
    ],
    alert_test [
        FETAL_FAULT OFFSET(0) NUMBITS(1) [],
    ],
    ctrl [
        RX_WATERMARK OFFSET(0) NUMBITS(8) [],
        TX_WATERMARK OFFSET(8) NUMBITS(8) [],
        //28:16 RESERVED
        OUTPUT_EN OFFSET(29) NUMBITS(1) [],
        SW_RST OFFSET(30) NUMBITS(1) [],
        SPIEN OFFSET(31) NUMBITS(1) []
    ],
    status [
        TXQD OFFSET(0) NUMBITS(8) [],
        RXQD OFFSET(15) NUMBITS(8) [],
        CMDQD OFFSET(16) NUMBITS(1) [],
        RXWM OFFSET(20) NUMBITS(1) [],
        BYTEORDER OFFSET(22) NUMBITS(1) [],
        RXSTALL OFFSET(23) NUMBITS(1) [],
        RXEMPTY OFFSET(24) NUMBITS(1) [],
        RXFULL OFFSET(25) NUMBITS(1) [],
        TXWM OFFSET(26) NUMBITS(1) [],
        TXSTALL OFFSET(27) NUMBITS(1) [],
        TXEMPTY OFFSET(28) NUMBITS(1) [],
        TXFULL OFFSET(29) NUMBITS(1) [],
        ACTIVE OFFSET(30) NUMBITS(1) [],
        READY OFFSET(31) NUMBITS(1) [],
    ],
    conf_opts [
        CLKDIV_0 OFFSET(0) NUMBITS(16) [],
        CSNIDLE_0 OFFSET(16) NUMBITS(3) [],
        CSNTRAIL_0 OFFSET(20) NUMBITS(3) [],
        CSNLEAD_0 OFFSET(24) NUMBITS(3) [],
        //28 Reserved
        FULLCYC_0 OFFSET(29) NUMBITS(1) [],
        CPHA_0 OFFSET(30) NUMBITS(1) [],
        CPOL_0 OFFSET(31) NUMBITS(1) [],
    ],
    csid_ctrl [
        CSID OFFSET(0) NUMBITS(32) [],
    ],
    command [
        LEN OFFSET(0) NUMBITS(8) [],
        CSAAT OFFSET(9) NUMBITS(1) [],
        SPEED OFFSET(10) NUMBITS(2) [],
        DIRECTION OFFSET(12) NUMBITS(2) [],
    ],
    rx_data [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    tx_data [
        DATA OFFSET(0) NUMBITS(32) [],
    ],
    err_en [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
    ],
    err_status [
        CMDBUSY OFFSET(0) NUMBITS(1) [],
        OVERFLOW OFFSET(1) NUMBITS(1) [],
        UNDERFLOW OFFSET(2) NUMBITS(1) [],
        CMDINVAL OFFSET(3) NUMBITS(1) [],
        CSIDINVAL OFFSET(4) NUMBITS(1) [],
        ACCESSINVAL OFFSET(5) NUMBITS(1) [],
    ],
    event_en [
        RXFULL OFFSET(0) NUMBITS(1) [],
        TXEMPTY OFFSET(1) NUMBITS(1) [],
        RXWM OFFSET(2) NUMBITS(1) [],
        TXWM OFFSET(3) NUMBITS(1) [],
        READY OFFSET(4) NUMBITS(1) [],
        IDLE OFFSET(5) NUMBITS(1) [],
    ],
];

pub struct SpiHost<'a> {
    registers: StaticRef<SpiHostRegisters>,
    client: OptionalCell<&'a dyn hil::spi::SpiMasterClient>,
    busy: Cell<bool>,
    cpu_clk: u32,
    tsclk: Cell<u32>,
    tx_buf: MapCell<SubSliceMut<'static, u8>>,
    rx_buf: MapCell<SubSliceMut<'static, u8>>,
    tx_len: Cell<usize>,
    rx_len: Cell<usize>,
    tx_offset: Cell<usize>,
    rx_offset: Cell<usize>,
}
// SPI Host Command Direction: Bidirectional
const SPI_HOST_CMD_BIDIRECTIONAL: u32 = 3;
// SPI Host Command Speed: Standard SPI
const SPI_HOST_CMD_STANDARD_SPI: u32 = 0;

impl SpiHost<'_> {
    pub fn new(base: StaticRef<SpiHostRegisters>, cpu_clk: u32) -> Self {
        SpiHost {
            registers: base,
            client: OptionalCell::empty(),
            busy: Cell::new(false),
            cpu_clk,
            tsclk: Cell::new(0),
            tx_buf: MapCell::empty(),
            rx_buf: MapCell::empty(),
            tx_len: Cell::new(0),
            rx_len: Cell::new(0),
            tx_offset: Cell::new(0),
            rx_offset: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irq = regs.intr_state.extract();
        self.disable_interrupts();

        if irq.is_set(intr::ERROR) {
            //Clear all pending errors.
            self.clear_err_interrupt();
            //Something went wrong, reset IP and clear buffers
            self.reset_spi_ip();
            self.reset_internal_state();
            //r/w_done() may call r/w_bytes() to re-attempt transfer
            self.client.map(|client| match self.tx_buf.take() {
                None => (),
                Some(tx_buf) => {
                    client.read_write_done(tx_buf, self.rx_buf.take(), Err(ErrorCode::FAIL))
                }
            });
            return;
        }

        if irq.is_set(intr::SPI_EVENT) {
            let status = regs.status.extract();
            self.clear_event_interrupt();

            //This could be set at init, so only follow through
            //once a transfer has started (is_busy())
            if status.is_set(status::TXEMPTY) && self.is_busy() {
                match self.continue_transfer() {
                    Ok(SpiHostStatus::SpiTransferCmplt) => {
                        // Transfer success
                        self.client.map(|client| match self.tx_buf.take() {
                            None => (),
                            Some(tx_buf) => client.read_write_done(
                                tx_buf,
                                self.rx_buf.take(),
                                Ok(self.tx_len.get()),
                            ),
                        });

                        self.disable_tx_interrupt();
                        self.reset_internal_state();
                    }
                    Ok(SpiHostStatus::SpiTransferInprog) => {}
                    Err(err) => {
                        //Transfer failed, lets clean up
                        //Clear all pending interrupts.
                        self.clear_err_interrupt();
                        //Something went wrong, reset IP and clear buffers
                        self.reset_spi_ip();
                        self.reset_internal_state();
                        self.client.map(|client| match self.tx_buf.take() {
                            None => (),
                            Some(tx_buf) => {
                                client.read_write_done(tx_buf, self.rx_buf.take(), Err(err))
                            }
                        });
                    }
                }
            } else {
                self.enable_interrupts();
            }
        }
    }

    //Determine if transfer complete or if we need to keep
    //writing from an offset.
    fn continue_transfer(&self) -> Result<SpiHostStatus, ErrorCode> {
        let rc = self
            .rx_buf
            .take()
            .map(|mut rx_buf| -> Result<SpiHostStatus, ErrorCode> {
                let regs = self.registers;
                let mut val32: u32;
                let mut val8: u8;
                let mut shift_mask;
                let rx_len = self.tx_offset.get() - self.rx_offset.get();
                let read_cycles = self.div_up(rx_len, 4);

                //Receive rx_data (Only 4byte reads are supported)
                for _n in 0..read_cycles {
                    val32 = regs.rx_data.read(rx_data::DATA);
                    shift_mask = 0xFF;
                    for i in 0..4 {
                        if self.rx_offset.get() >= self.rx_len.get() {
                            break;
                        }
                        val8 = ((val32 & shift_mask) >> (i * 8)) as u8;
                        if let Some(ptr) = rx_buf.as_mut_slice().get_mut(self.rx_offset.get()) {
                            *ptr = val8;
                        } else {
                            // We have run out of rx buffer size
                            break;
                        }
                        self.rx_offset.set(self.rx_offset.get() + 1);
                        shift_mask <<= 8;
                    }
                }
                //Save buffer!
                self.rx_buf.replace(rx_buf);
                //Transfer was complete */
                if self.tx_offset.get() == self.tx_len.get() {
                    Ok(SpiHostStatus::SpiTransferCmplt)
                } else {
                    //Theres more to transfer, continue writing from the offset
                    self.spi_transfer_progress()
                }
            })
            .map_or_else(|| Err(ErrorCode::FAIL), |rc| rc);

        rc
    }

    /// Continue SPI transfer from offset point
    fn spi_transfer_progress(&self) -> Result<SpiHostStatus, ErrorCode> {
        let mut transfer_complete = false;
        if self
            .tx_buf
            .take()
            .map(|tx_buf| -> Result<(), ErrorCode> {
                let regs = self.registers;
                let mut t_byte: u32;
                let mut tx_slice: [u8; 4];

                if regs.status.read(status::TXQD) != 0 || regs.status.read(status::ACTIVE) != 0 {
                    self.tx_buf.replace(tx_buf);
                    return Err(ErrorCode::BUSY);
                }

                while !regs.status.is_set(status::TXFULL) && regs.status.read(status::TXQD) < 64 {
                    tx_slice = [0, 0, 0, 0];
                    for elem in tx_slice.iter_mut() {
                        if self.tx_offset.get() >= self.tx_len.get() {
                            break;
                        }
                        if let Some(val) = tx_buf.as_slice().get(self.tx_offset.get()) {
                            *elem = *val;
                            self.tx_offset.set(self.tx_offset.get() + 1);
                        } else {
                            //Unexpectedly ran out of tx buffer
                            break;
                        }
                    }
                    t_byte = u32::from_le_bytes(tx_slice);
                    regs.tx_data.write(tx_data::DATA.val(t_byte));

                    //Transfer Complete in one-shot
                    if self.tx_offset.get() >= self.tx_len.get() {
                        transfer_complete = true;
                        break;
                    }
                }

                //Hold tx_buf for offset transfer continue
                self.tx_buf.replace(tx_buf);

                //Set command register to init transfer
                self.start_transceive();
                Ok(())
            })
            .transpose()
            .is_err()
        {
            return Err(ErrorCode::BUSY);
        }

        if transfer_complete {
            Ok(SpiHostStatus::SpiTransferCmplt)
        } else {
            Ok(SpiHostStatus::SpiTransferInprog)
        }
    }

    /// Issue a command to start SPI transaction
    /// Currently only Bi-Directional transactions are supported
    fn start_transceive(&self) {
        let regs = self.registers;
        //TXQD holds number of 32bit words
        let txfifo_num_bytes = regs.status.read(status::TXQD) * 4;

        //8-bits that describe command transfer len (cannot exceed 255)
        let num_transfer_bytes: u32 = if txfifo_num_bytes > u8::MAX as u32 {
            u8::MAX as u32
        } else {
            txfifo_num_bytes
        };

        self.enable_interrupts();
        self.enable_tx_interrupt();

        //Flush all data in TXFIFO and assert CSAAT for all
        // but the last transfer segment.
        if self.tx_offset.get() >= self.tx_len.get() {
            regs.command.write(
                command::LEN.val(num_transfer_bytes)
                    + command::DIRECTION.val(SPI_HOST_CMD_BIDIRECTIONAL)
                    + command::CSAAT::CLEAR
                    + command::SPEED.val(SPI_HOST_CMD_STANDARD_SPI),
            );
        } else {
            regs.command.write(
                command::LEN.val(num_transfer_bytes)
                    + command::DIRECTION.val(SPI_HOST_CMD_BIDIRECTIONAL)
                    + command::CSAAT::SET
                    + command::SPEED.val(SPI_HOST_CMD_STANDARD_SPI),
            );
        }
    }

    /// Reset the soft internal state, should be called once
    /// a spi transaction has been completed.
    fn reset_internal_state(&self) {
        self.clear_spi_busy();
        self.tx_len.set(0);
        self.rx_len.set(0);
        self.tx_offset.set(0);
        self.rx_offset.set(0);

        debug_assert!(self.tx_buf.is_none());
        debug_assert!(self.rx_buf.is_none());
    }

    /// Enable SPI_HOST IP
    /// `dead_code` to silence warnings when not building for mainline qemu
    #[allow(dead_code)]
    fn enable_spi_host(&self) {
        let regs = self.registers;
        //Enables the SPI host
        regs.ctrl.modify(ctrl::SPIEN::SET + ctrl::OUTPUT_EN::SET);
    }

    /// Reset SPI Host
    fn reset_spi_ip(&self) {
        let regs = self.registers;
        //IP to reset state
        regs.ctrl.modify(ctrl::SW_RST::SET);

        //Wait for status ready to be set before continuing
        while regs.status.is_set(status::ACTIVE) {}
        //Wait for both FIFOs to completely drain
        while regs.status.read(status::TXQD) != 0 && regs.status.read(status::RXQD) != 0 {}
        //Clear Reset
        regs.ctrl.modify(ctrl::SW_RST::CLEAR);
    }

    /// Enable both event/err IRQ
    fn enable_interrupts(&self) {
        self.registers
            .intr_state
            .write(intr::ERROR::SET + intr::SPI_EVENT::SET);
        self.registers
            .intr_enable
            .modify(intr::ERROR::SET + intr::SPI_EVENT::SET);
    }

    /// Disable both event/err IRQ
    fn disable_interrupts(&self) {
        let regs = self.registers;
        regs.intr_enable
            .write(intr::ERROR::CLEAR + intr::SPI_EVENT::CLEAR);
    }

    /// Clear the error IRQ
    fn clear_err_interrupt(&self) {
        let regs = self.registers;
        //Clear Error Masks (rw1c)
        regs.err_status.modify(err_status::CMDBUSY::SET);
        regs.err_status.modify(err_status::OVERFLOW::SET);
        regs.err_status.modify(err_status::UNDERFLOW::SET);
        regs.err_status.modify(err_status::CMDINVAL::SET);
        regs.err_status.modify(err_status::CSIDINVAL::SET);
        regs.err_status.modify(err_status::ACCESSINVAL::SET);
        //Clear Error IRQ
        regs.intr_state.modify(intr::ERROR::SET);
    }

    /// Clear the event IRQ
    fn clear_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_state.modify(intr::SPI_EVENT::SET);
    }
    /// Will generate a `test` interrupt on the error irq
    /// Note: Left to allow debug accessibility
    #[allow(dead_code)]
    fn test_error_interrupt(&self) {
        let regs = self.registers;
        regs.intr_test.write(intr::ERROR::SET);
    }
    /// Clear test interrupts
    /// Note: Left to allow debug accessibility
    #[allow(dead_code)]
    fn clear_tests(&self) {
        let regs = self.registers;
        regs.intr_test
            .write(intr::ERROR::CLEAR + intr::SPI_EVENT::CLEAR);
    }

    /// Will generate a `test` interrupt on the event irq
    /// Note: Left to allow debug accessibility
    #[allow(dead_code)]
    fn test_event_interrupt(&self) {
        let regs = self.registers;
        regs.intr_test.write(intr::SPI_EVENT::SET);
    }

    /// Enable required `event interrupts`
    /// `dead_code` to silence warnings when not building for mainline qemu
    #[allow(dead_code)]
    fn event_enable(&self) {
        let regs = self.registers;
        regs.event_en.write(event_en::TXEMPTY::SET);
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.event_en.modify(event_en::TXEMPTY::CLEAR);
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.event_en.modify(event_en::TXEMPTY::SET);
    }

    /// Enable required error interrupts
    /// `dead_code` to silence warnings when not building for mainline qemu
    #[allow(dead_code)]
    fn err_enable(&self) {
        let regs = self.registers;
        regs.err_en.modify(
            err_en::CMDBUSY::SET
                + err_en::CMDINVAL::SET
                + err_en::CSIDINVAL::SET
                + err_en::OVERFLOW::SET
                + err_en::UNDERFLOW::SET,
        );
    }

    fn set_spi_busy(&self) {
        self.busy.set(true);
    }

    fn clear_spi_busy(&self) {
        self.busy.set(false);
    }

    /// Divide a/b and return a value always rounded
    /// up to the nearest integer
    fn div_up(&self, a: usize, b: usize) -> usize {
        a.div_ceil(b)
    }

    /// Calculate the scaler based on a specified tsclk rate
    /// This scaler will pre-scale the cpu_clk and must be <= cpu_clk/2
    fn calculate_tsck_scaler(&self, rate: u32) -> Result<u16, ErrorCode> {
        if rate > self.cpu_clk / 2 {
            return Err(ErrorCode::NOSUPPORT);
        }
        //Divide and truncate
        let mut scaler: u32 = (self.cpu_clk / (2 * rate)) - 1;

        //Increase scaler if the division was not exact, ensuring that it does not overflow
        //or exceed divider specification where tsck is at most <= Tclk/2
        if self.cpu_clk % (2 * rate) != 0 && scaler != 0xFF {
            scaler += 1;
        }
        Ok(scaler as u16)
    }
}

#[derive(Copy, Clone)]
pub struct CS(pub u32);

impl hil::spi::cs::IntoChipSelect<CS, hil::spi::cs::ActiveLow> for CS {
    fn into_cs(self) -> CS {
        self
    }
}

impl<'a> hil::spi::SpiMaster<'a> for SpiHost<'a> {
    type ChipSelect = CS;

    fn init(&self) -> Result<(), ErrorCode> {
        let regs = self.registers;
        self.event_enable();
        self.err_enable();

        self.enable_interrupts();

        self.enable_spi_host();

        //TODO: I think this is bug in OT, where the `first` word written
        // (while TXEMPTY) to TX_DATA is dropped/ignored and not added to TX_FIFO (TXQD = 0).
        // The following write (0x00), works around this `bug`.
        // Could be Verilator specific
        regs.tx_data.write(tx_data::DATA.val(0x00));
        assert_eq!(regs.status.read(status::TXQD), 0);
        Ok(())
    }

    fn set_client(&self, client: &'a dyn hil::spi::SpiMasterClient) {
        self.client.set(client);
    }

    fn is_busy(&self) -> bool {
        self.busy.get()
    }

    fn read_write_bytes(
        &self,
        tx_buf: SubSliceMut<'static, u8>,
        rx_buf: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    > {
        debug_assert!(!self.busy.get());
        debug_assert!(self.tx_buf.is_none());
        debug_assert!(self.rx_buf.is_none());
        let regs = self.registers;

        if self.is_busy() || regs.status.is_set(status::TXFULL) {
            return Err((ErrorCode::BUSY, tx_buf, rx_buf));
        }

        if rx_buf.is_none() {
            return Err((ErrorCode::NOMEM, tx_buf, rx_buf));
        }

        self.tx_len.set(tx_buf.len());

        let mut t_byte: u32;
        let mut tx_slice: [u8; 4];
        //We are committing to the transfer now
        self.set_spi_busy();

        while !regs.status.is_set(status::TXFULL) && regs.status.read(status::TXQD) < 64 {
            tx_slice = [0, 0, 0, 0];
            for elem in tx_slice.iter_mut() {
                if self.tx_offset.get() >= self.tx_len.get() {
                    break;
                }
                *elem = tx_buf[self.tx_offset.get()];
                self.tx_offset.set(self.tx_offset.get() + 1);
            }
            t_byte = u32::from_le_bytes(tx_slice);
            regs.tx_data.write(tx_data::DATA.val(t_byte));

            //Transfer Complete in one-shot
            if self.tx_offset.get() >= self.tx_len.get() {
                break;
            }
        }

        //Hold tx_buf for offset transfer continue
        self.tx_buf.replace(tx_buf);

        //Hold rx_buf for later

        rx_buf.map(|rx_buf_t| {
            self.rx_len.set(cmp::min(self.tx_len.get(), rx_buf_t.len()));
            self.rx_buf.replace(rx_buf_t);
        });

        //Set command register to init transfer
        self.start_transceive();

        Ok(())
    }

    fn write_byte(&self, _val: u8) -> Result<(), ErrorCode> {
        //Use `read_write_bytes()` instead.
        Err(ErrorCode::FAIL)
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        //Use `read_write_bytes()` instead.
        Err(ErrorCode::FAIL)
    }

    fn read_write_byte(&self, _val: u8) -> Result<u8, ErrorCode> {
        //Use `read_write_bytes()` instead.
        Err(ErrorCode::FAIL)
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        let regs = self.registers;

        //CSID will index the CONFIGOPTS multi-register
        regs.csid.write(csid_ctrl::CSID.val(cs.0));

        Ok(())
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        let regs = self.registers;

        match self.calculate_tsck_scaler(rate) {
            Ok(scaler) => {
                regs.config_opts
                    .modify(conf_opts::CLKDIV_0.val(scaler as u32));
                self.tsclk.set(rate);
                Ok(rate)
            }
            Err(e) => Err(e),
        }
    }

    fn get_rate(&self) -> u32 {
        self.tsclk.get()
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        let regs = self.registers;
        match polarity {
            ClockPolarity::IdleLow => regs.config_opts.modify(conf_opts::CPOL_0::CLEAR),
            ClockPolarity::IdleHigh => regs.config_opts.modify(conf_opts::CPOL_0::SET),
        }
        Ok(())
    }

    fn get_polarity(&self) -> ClockPolarity {
        let regs = self.registers;

        match regs.config_opts.read(conf_opts::CPOL_0) {
            0 => ClockPolarity::IdleLow,
            1 => ClockPolarity::IdleHigh,
            _ => unreachable!(),
        }
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        let regs = self.registers;
        match phase {
            ClockPhase::SampleLeading => regs.config_opts.modify(conf_opts::CPHA_0::CLEAR),
            ClockPhase::SampleTrailing => regs.config_opts.modify(conf_opts::CPHA_0::SET),
        }
        Ok(())
    }

    fn get_phase(&self) -> ClockPhase {
        let regs = self.registers;

        match regs.config_opts.read(conf_opts::CPHA_0) {
            1 => ClockPhase::SampleTrailing,
            0 => ClockPhase::SampleLeading,
            _ => unreachable!(),
        }
    }

    /// hold_low is controlled by IP based on command segments issued
    /// force holds are not supported
    fn hold_low(&self) {
        unimplemented!("spi_host: does not support hold low");
    }

    /// release_low is controlled by IP based on command segments issued
    /// force releases are not supported
    fn release_low(&self) {
        unimplemented!("spi_host: does not support release low");
    }
}
