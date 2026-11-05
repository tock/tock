// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! nRF52-family board support for the Tock hardware CI harness.
//!
//! Uses `openocd` to flash the boards, and connects to the target using the
//! J-Link's serial port.

use crate::boards::helpers::serial::SerialUart;
use crate::boards::helpers::{command, openocd, tockloader};
use crate::boards::{BOARDS, Board, Dut, DutConfig, Open, Timing, Uart};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// nRF52840DK, with an nRF52840.
#[linkme::distributed_slice(BOARDS)]
pub static NRF52840DK: Board = Board {
    name: "nrf52840dk",
    dir: "boards/nordic/nrf52840dk",
    kernel: "target/thumbv7em-none-eabi/release/nrf52840dk.bin",
    tock_targets: &["cortex-m4"],
    timing: Timing {
        flash: Duration::from_secs(30),
        boot: Duration::from_secs(5),
        expect: Duration::from_secs(3),
    },
    open: Open::Physical(|dut| Box::new(Nrf52Board::new(&NRF52840DK, dut))),
};

/// nRF52DK, with an nRF52832.
#[linkme::distributed_slice(BOARDS)]
pub static NRF52DK: Board = Board {
    name: "nrf52dk",
    dir: "boards/nordic/nrf52dk",
    kernel: "target/thumbv7em-none-eabi/release/nrf52dk.bin",
    tock_targets: &["cortex-m4"],
    timing: Timing {
        flash: Duration::from_secs(30),
        boot: Duration::from_secs(5),
        expect: Duration::from_secs(3),
    },
    open: Open::Physical(|dut| Box::new(Nrf52Board::new(&NRF52DK, dut))),
};

/// Baud rate of the boards' console.
const BAUD: u32 = 115_200;

/// Board with an nRF52-family chip, debugged through a J-Link.
pub struct Nrf52Board {
    board: &'static Board,
    dut: DutConfig,
    console: Option<SerialUart>,
}

impl Nrf52Board {
    pub fn new(board: &'static Board, dut: DutConfig) -> Self {
        Nrf52Board {
            board,
            dut,
            console: None,
        }
    }

    /// Construct an OpenOCD invocation which runs `commands` on the board.
    fn openocd(&self, commands: &[&str]) -> Command {
        // OpenOCD 0.12 and earlier keep the config in `target/nrf52.cfg`:
        let target = openocd::source_first(&["target/nordic/nrf52.cfg", "target/nrf52.cfg"]);
        openocd::command(
            &["-f", "interface/jlink.cfg"],
            self.dut.probe.as_deref(),
            &["-c", "transport select swd", "-c", &target],
            commands,
        )
    }

    /// Construct a tockloader invocation with the subcommand `subcommand`,
    /// which accesses the board through OpenOCD.
    fn tockloader(&self, subcommand: &str) -> Command {
        let mut cmd = tockloader::command(subcommand);
        // Tockloader's `nrf52dk` board covers the nRF52840DK as well:
        cmd.args(["--board", "nrf52dk", "--openocd"]);
        if let Some(serial) = &self.dut.probe {
            cmd.args(["--openocd-serial-number", serial]);
        }
        cmd
    }
}

impl Dut for Nrf52Board {
    fn board(&self) -> &'static Board {
        self.board
    }

    fn flash(&mut self, kernel: &Path, apps: &[&Path]) -> Result<()> {
        // Make sure to clear APPROTECT if it's set:
        command::run(&mut self.openocd(&[
            "init",
            "if {[nrf52.dap apreg 1 0xc] != 1} { nrf52_recover }",
            "if {[nrf52.dap apreg 1 0xc] != 1} { shutdown error }",
        ]))
        .context("failed to unlock APPROTECT")?;

        command::run(
            self.tockloader("flash")
                .args(["--address", "0x00000"])
                .arg(kernel),
        )?;
        command::run(self.tockloader("install").arg("--erase").args(apps))?;

        self.console = Some(SerialUart::open(self.dut.console()?, BAUD)?);

        command::run(&mut self.openocd(&["init", "reset run"]))
    }

    fn reset(&mut self) -> Result<()> {
        self.console = None;
        Ok(())
    }

    fn uart(&mut self) -> Option<&mut dyn Uart> {
        self.console.as_mut().map(|c| c as &mut dyn Uart)
    }
}
