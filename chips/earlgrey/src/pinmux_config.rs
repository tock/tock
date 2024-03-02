// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Chip sepcific pinmux Configurations

use crate::pinmux::{SelectInput, SelectOutput};
use crate::registers::top_earlgrey::{
    MuxedPads, PinmuxInsel, PinmuxOutsel, PinmuxPeripheralIn, NUM_MIO_PADS,
};

/// Number of input selector entry (Last input + 1)
pub const INPUT_NUM: usize = PinmuxPeripheralIn::UsbdevSense as usize + 1;
/// Number of output selctor entry
pub const OUTPUT_NUM: usize = NUM_MIO_PADS;

/// Representations of Earlgrey pinmux configuration on targeted board
pub trait EarlGreyPinmuxConfig {
    /// Array representing configuration of pinmux input selctor
    const INPUT: &'static [PinmuxInsel; INPUT_NUM];

    /// Array representing configurations of pinmux output selecto
    const OUTPUT: &'static [PinmuxOutsel; OUTPUT_NUM];

    /// Setup pinmux configurations for all multiplexed pads
    fn setup() {
        // setup pinmux input
        for index in 0..INPUT_NUM {
            if let Ok(peripheral) = PinmuxPeripheralIn::try_from(index as u32) {
                peripheral.connect_input(Self::INPUT[index]);
            }
        }
        // setup pinmux output
        for index in 0..OUTPUT_NUM {
            if let Ok(pad) = MuxedPads::try_from(index as u32) {
                pad.connect_output(Self::OUTPUT[index]);
            }
        }
    }
}
