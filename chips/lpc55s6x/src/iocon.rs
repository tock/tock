// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

use crate::gpio::LPCPin;
use kernel::utilities::registers::interfaces::Writeable;
use kernel::utilities::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::utilities::StaticRef;

register_structs! {
    /// I/O pin configuration (IOCON)
    pub IoconRegisters {
        /// Digital I/O control for port 0 pins PIO0_0
        (0x000 => pio0_0: ReadWrite<u32, PIO0_0::Register>),
        /// Digital I/O control for port 0 pins PIO0_1
        (0x004 => pio0_1: ReadWrite<u32, PIO0_1::Register>),
        /// Digital I/O control for port 0 pins PIO0_2
        (0x008 => pio0_2: ReadWrite<u32, PIO0_2::Register>),
        /// Digital I/O control for port 0 pins PIO0_3
        (0x00C => pio0_3: ReadWrite<u32, PIO0_3::Register>),
        /// Digital I/O control for port 0 pins PIO0_4
        (0x010 => pio0_4: ReadWrite<u32, PIO0_4::Register>),
        /// Digital I/O control for port 0 pins PIO0_5
        (0x014 => pio0_5: ReadWrite<u32, PIO0_5::Register>),
        /// Digital I/O control for port 0 pins PIO0_6
        (0x018 => pio0_6: ReadWrite<u32, PIO0_6::Register>),
        /// Digital I/O control for port 0 pins PIO0_7
        (0x01C => pio0_7: ReadWrite<u32, PIO0_7::Register>),
        /// Digital I/O control for port 0 pins PIO0_8
        (0x020 => pio0_8: ReadWrite<u32, PIO0_8::Register>),
        /// Digital I/O control for port 0 pins PIO0_9
        (0x024 => pio0_9: ReadWrite<u32, PIO0_9::Register>),
        /// Digital I/O control for port 0 pins PIO0_10
        (0x028 => pio0_10: ReadWrite<u32, PIO0_10::Register>),
        /// Digital I/O control for port 0 pins PIO0_11
        (0x02C => pio0_11: ReadWrite<u32, PIO0_11::Register>),
        /// Digital I/O control for port 0 pins PIO0_12
        (0x030 => pio0_12: ReadWrite<u32, PIO0_12::Register>),
        /// Digital I/O control for port 0 pins PIO0_13
        (0x034 => pio0_13: ReadWrite<u32, PIO0_13::Register>),
        /// Digital I/O control for port 0 pins PIO0_14
        (0x038 => pio0_14: ReadWrite<u32, PIO0_14::Register>),
        /// Digital I/O control for port 0 pins PIO0_15
        (0x03C => pio0_15: ReadWrite<u32, PIO0_15::Register>),
        /// Digital I/O control for port 0 pins PIO0_16
        (0x040 => pio0_16: ReadWrite<u32, PIO0_16::Register>),
        /// Digital I/O control for port 0 pins PIO0_17
        (0x044 => pio0_17: ReadWrite<u32, PIO0_17::Register>),
        /// Digital I/O control for port 0 pins PIO0_18
        (0x048 => pio0_18: ReadWrite<u32, PIO0_18::Register>),
        /// Digital I/O control for port 0 pins PIO0_19
        (0x04C => pio0_19: ReadWrite<u32, PIO0_19::Register>),
        /// Digital I/O control for port 0 pins PIO0_20
        (0x050 => pio0_20: ReadWrite<u32, PIO0_20::Register>),
        /// Digital I/O control for port 0 pins PIO0_21
        (0x054 => pio0_21: ReadWrite<u32, PIO0_21::Register>),
        /// Digital I/O control for port 0 pins PIO0_22
        (0x058 => pio0_22: ReadWrite<u32, PIO0_22::Register>),
        /// Digital I/O control for port 0 pins PIO0_23
        (0x05C => pio0_23: ReadWrite<u32, PIO0_23::Register>),
        /// Digital I/O control for port 0 pins PIO0_24
        (0x060 => pio0_24: ReadWrite<u32, PIO0_24::Register>),
        /// Digital I/O control for port 0 pins PIO0_25
        (0x064 => pio0_25: ReadWrite<u32, PIO0_25::Register>),
        /// Digital I/O control for port 0 pins PIO0_26
        (0x068 => pio0_26: ReadWrite<u32, PIO0_26::Register>),
        /// Digital I/O control for port 0 pins PIO0_27
        (0x06C => pio0_27: ReadWrite<u32, PIO0_27::Register>),
        /// Digital I/O control for port 0 pins PIO0_28
        (0x070 => pio0_28: ReadWrite<u32, PIO0_28::Register>),
        /// Digital I/O control for port 0 pins PIO0_29
        (0x074 => pio0_29: ReadWrite<u32, PIO0_29::Register>),
        /// Digital I/O control for port 0 pins PIO0_30
        (0x078 => pio0_30: ReadWrite<u32, PIO0_30::Register>),
        /// Digital I/O control for port 0 pins PIO0_31
        (0x07C => pio0_31: ReadWrite<u32, PIO0_31::Register>),
        /// Digital I/O control for port 1 pins PIO1_0
        (0x080 => pio1_0: ReadWrite<u32, PIO1_0::Register>),
        /// Digital I/O control for port 1 pins PIO1_1
        (0x084 => pio1_1: ReadWrite<u32, PIO1_1::Register>),
        /// Digital I/O control for port 1 pins PIO1_2
        (0x088 => pio1_2: ReadWrite<u32, PIO1_2::Register>),
        /// Digital I/O control for port 1 pins PIO1_3
        (0x08C => pio1_3: ReadWrite<u32, PIO1_3::Register>),
        /// Digital I/O control for port 1 pins PIO1_4
        (0x090 => pio1_4: ReadWrite<u32, PIO1_4::Register>),
        /// Digital I/O control for port 1 pins PIO1_5
        (0x094 => pio1_5: ReadWrite<u32, PIO1_5::Register>),
        /// Digital I/O control for port 1 pins PIO1_6
        (0x098 => pio1_6: ReadWrite<u32, PIO1_6::Register>),
        /// Digital I/O control for port 1 pins PIO1_7
        (0x09C => pio1_7: ReadWrite<u32, PIO1_7::Register>),
        /// Digital I/O control for port 1 pins PIO1_8
        (0x0A0 => pio1_8: ReadWrite<u32, PIO1_8::Register>),
        /// Digital I/O control for port 1 pins PIO1_9
        (0x0A4 => pio1_9: ReadWrite<u32, PIO1_9::Register>),
        /// Digital I/O control for port 1 pins PIO1_10
        (0x0A8 => pio1_10: ReadWrite<u32, PIO1_10::Register>),
        /// Digital I/O control for port 1 pins PIO1_11
        (0x0AC => pio1_11: ReadWrite<u32, PIO1_11::Register>),
        /// Digital I/O control for port 1 pins PIO1_12
        (0x0B0 => pio1_12: ReadWrite<u32, PIO1_12::Register>),
        /// Digital I/O control for port 1 pins PIO1_13
        (0x0B4 => pio1_13: ReadWrite<u32, PIO1_13::Register>),
        /// Digital I/O control for port 1 pins PIO1_14
        (0x0B8 => pio1_14: ReadWrite<u32, PIO1_14::Register>),
        /// Digital I/O control for port 1 pins PIO1_15
        (0x0BC => pio1_15: ReadWrite<u32, PIO1_15::Register>),
        /// Digital I/O control for port 1 pins PIO1_16
        (0x0C0 => pio1_16: ReadWrite<u32, PIO1_16::Register>),
        /// Digital I/O control for port 1 pins PIO1_17
        (0x0C4 => pio1_17: ReadWrite<u32, PIO1_17::Register>),
        /// Digital I/O control for port 1 pins PIO1_18
        (0x0C8 => pio1_18: ReadWrite<u32, PIO1_18::Register>),
        /// Digital I/O control for port 1 pins PIO1_19
        (0x0CC => pio1_19: ReadWrite<u32, PIO1_19::Register>),
        /// Digital I/O control for port 1 pins PIO1_20
        (0x0D0 => pio1_20: ReadWrite<u32, PIO1_20::Register>),
        /// Digital I/O control for port 1 pins PIO1_21
        (0x0D4 => pio1_21: ReadWrite<u32, PIO1_21::Register>),
        /// Digital I/O control for port 1 pins PIO1_22
        (0x0D8 => pio1_22: ReadWrite<u32, PIO1_22::Register>),
        /// Digital I/O control for port 1 pins PIO1_23
        (0x0DC => pio1_23: ReadWrite<u32, PIO1_23::Register>),
        /// Digital I/O control for port 1 pins PIO1_24
        (0x0E0 => pio1_24: ReadWrite<u32, PIO1_24::Register>),
        /// Digital I/O control for port 1 pins PIO1_25
        (0x0E4 => pio1_25: ReadWrite<u32, PIO1_25::Register>),
        /// Digital I/O control for port 1 pins PIO1_26
        (0x0E8 => pio1_26: ReadWrite<u32, PIO1_26::Register>),
        /// Digital I/O control for port 1 pins PIO1_27
        (0x0EC => pio1_27: ReadWrite<u32, PIO1_27::Register>),
        /// Digital I/O control for port 1 pins PIO1_28
        (0x0F0 => pio1_28: ReadWrite<u32, PIO1_28::Register>),
        /// Digital I/O control for port 1 pins PIO1_29
        (0x0F4 => pio1_29: ReadWrite<u32, PIO1_29::Register>),
        /// Digital I/O control for port 1 pins PIO1_30
        (0x0F8 => pio1_30: ReadWrite<u32, PIO1_30::Register>),
        /// Digital I/O control for port 1 pins PIO1_31
        (0x0FC => pio1_31: ReadWrite<u32, PIO1_31::Register>),
        (0x100 => @END),
    }
}
register_bitfields![u32,
PIO0_0 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_1 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_2 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_3 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_4 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_5 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_6 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_7 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_8 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_9 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_10 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_11 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_12 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_13 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode in standard GPIO mode (EGP = 1). This bit has no effect
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Supply Selection bit.
    SSEL OFFSET(11) NUMBITS(1) [
        /// 3V3 Signaling in I2C Mode.
        _3V3SignalingInI2CMode = 0,
        /// 1V8 Signaling in I2C Mode.
        _1V8SignalingInI2CMode = 1
    ],
    /// Controls input glitch filter.
    FILTEROFF OFFSET(12) NUMBITS(1) [
        /// Filter enabled.
        FilterEnabled = 0,
        /// Filter disabled.
        FilterDisabled = 1
    ],
    /// Pull-up current source enable in I2C mode.
    ECS OFFSET(13) NUMBITS(1) [
        /// Disabled. IO is in open drain cell.
        DisabledIOIsInOpenDrainCell = 0,
        /// Enabled. Pull resistor is conencted.
        EnabledPullResistorIsConencted = 1
    ],
    /// Switch between GPIO mode and I2C mode.
    EGP OFFSET(14) NUMBITS(1) [
        /// I2C mode.
        I2CMode = 0,
        /// GPIO mode.
        GPIOMode = 1
    ],
    /// Configures I2C features for standard mode, fast mode, and Fast Mode Plus operati
    I2CFILTER OFFSET(15) NUMBITS(1) [
        /// I2C 50 ns glitch filter enabled. Typically used for Standard-mode, Fast-mode and
        FAST_MODE = 0,
        /// I2C 10 ns glitch filter enabled. Typically used for High-speed mode I2C.
        I2C10NsGlitchFilterEnabledTypicallyUsedForHighSpeedModeI2C = 1
    ]
],
PIO0_14 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode in standard GPIO mode (EGP = 1). This bit has no effect
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Supply Selection bit.
    SSEL OFFSET(11) NUMBITS(1) [
        /// 3V3 Signaling in I2C Mode.
        _3V3SignalingInI2CMode = 0,
        /// 1V8 Signaling in I2C Mode.
        _1V8SignalingInI2CMode = 1
    ],
    /// Controls input glitch filter.
    FILTEROFF OFFSET(12) NUMBITS(1) [
        /// Filter enabled.
        FilterEnabled = 0,
        /// Filter disabled.
        FilterDisabled = 1
    ],
    /// Pull-up current source enable in I2C mode.
    ECS OFFSET(13) NUMBITS(1) [
        /// Disabled. IO is in open drain cell.
        DisabledIOIsInOpenDrainCell = 0,
        /// Enabled. Pull resistor is conencted.
        EnabledPullResistorIsConencted = 1
    ],
    /// Switch between GPIO mode and I2C mode.
    EGP OFFSET(14) NUMBITS(1) [
        /// I2C mode.
        I2CMode = 0,
        /// GPIO mode.
        GPIOMode = 1
    ],
    /// Configures I2C features for standard mode, fast mode, and Fast Mode Plus operati
    I2CFILTER OFFSET(15) NUMBITS(1) [
        /// I2C 50 ns glitch filter enabled. Typically used for Standard-mode, Fast-mode and
        FAST_MODE = 0,
        /// I2C 10 ns glitch filter enabled. Typically used for High-speed mode I2C.
        I2C10NsGlitchFilterEnabledTypicallyUsedForHighSpeedModeI2C = 1
    ]
],
PIO0_15 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_16 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_17 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_18 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_19 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_20 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_21 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_22 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_23 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO0_24 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_25 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_26 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_27 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_28 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_29 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_30 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO0_31 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO1_0 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO1_1 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_2 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_3 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_4 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_5 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_6 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_7 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_8 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO1_9 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO1_10 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_11 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_12 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_13 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_14 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO1_15 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_16 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_17 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_18 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_19 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ],
    /// Analog switch input control.
    ASW OFFSET(10) NUMBITS(1) [
        /// For pins PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0 and PIO1_9,
        VALUE0 = 0,
        /// For all pins except PIO0_9, PIO0_11, PIO0_12, PIO0_15, PIO0_18, PIO0_31, PIO1_0
        VALUE1 = 1
    ]
],
PIO1_20 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_21 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_22 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_23 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_24 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_25 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_26 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_27 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_28 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_29 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_30 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
],
PIO1_31 [
    /// Selects pin function.
    FUNC OFFSET(0) NUMBITS(4) [
        /// Alternative connection 0.
        AlternativeConnection0 = 0,
        /// Alternative connection 1.
        AlternativeConnection1 = 1,
        /// Alternative connection 2.
        AlternativeConnection2 = 2,
        /// Alternative connection 3.
        AlternativeConnection3 = 3,
        /// Alternative connection 4.
        AlternativeConnection4 = 4,
        /// Alternative connection 5.
        AlternativeConnection5 = 5,
        /// Alternative connection 6.
        AlternativeConnection6 = 6,
        /// Alternative connection 7.
        AlternativeConnection7 = 7
    ],
    /// Selects function mode (on-chip pull-up/pull-down resistor control).
    MODE OFFSET(4) NUMBITS(2) [
        /// Inactive. Inactive (no pull-down/pull-up resistor enabled).
        InactiveInactiveNoPullDownPullUpResistorEnabled = 0,
        /// Pull-down. Pull-down resistor enabled.
        PullDownPullDownResistorEnabled = 1,
        /// Pull-up. Pull-up resistor enabled.
        PullUpPullUpResistorEnabled = 2,
        /// Repeater. Repeater mode.
        RepeaterRepeaterMode = 3
    ],
    /// Driver slew rate.
    SLEW OFFSET(6) NUMBITS(1) [
        /// Standard-mode, output slew rate is slower. More outputs can be switched simultan
        StandardModeOutputSlewRateIsSlowerMoreOutputsCanBeSwitchedSimultaneously = 0,
        /// Fast-mode, output slew rate is faster. Refer to the appropriate specific device
        FAST = 1
    ],
    /// Input polarity.
    INVERT OFFSET(7) NUMBITS(1) [
        /// Disabled. Input function is not inverted.
        DisabledInputFunctionIsNotInverted = 0,
        /// Enabled. Input is function inverted.
        EnabledInputIsFunctionInverted = 1
    ],
    /// Select Digital mode.
    DIGIMODE OFFSET(8) NUMBITS(1) [
        /// Disable digital mode. Digital input set to 0.
        DisableDigitalModeDigitalInputSetTo0 = 0,
        /// Enable Digital mode. Digital input is enabled.
        EnableDigitalModeDigitalInputIsEnabled = 1
    ],
    /// Controls open-drain mode.
    OD OFFSET(9) NUMBITS(1) [
        /// Normal. Normal push-pull output
        NormalNormalPushPullOutput = 0,
        /// Open-drain. Simulated open-drain output (high drive disabled).
        OpenDrainSimulatedOpenDrainOutputHighDriveDisabled = 1
    ]
]
];
pub(crate) const IOCON_BASE: StaticRef<IoconRegisters> =
    unsafe { StaticRef::new(0x50001000 as *const IoconRegisters) };

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Function {
    GPIO = 0,
    Alt1 = 1,
    Alt2 = 2,
    Alt3 = 3,
    Alt4 = 4,
    Alt5 = 5,
    Alt6 = 6,
    Alt7 = 7,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Pull {
    None = 0b00,
    Down = 0b01,
    Up = 0b10,
    Repeater = 0b11,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Slew {
    Standard = 0,
    Fast = 1,
}

pub struct Config {
    pub function: Function,
    pub pull: Pull,
    pub slew: Slew,
    pub invert: bool,
    pub digital_mode: bool,
    pub open_drain: bool,
}

pub struct Iocon {
    registers: StaticRef<IoconRegisters>,
}

impl Iocon {
    pub const fn new() -> Self {
        Self {
            registers: IOCON_BASE,
        }
    }

    pub fn configure_pin(&self, pin: LPCPin, config: Config) {
        let standard_value = PIO0_0::FUNC.val(config.function as u32)
            + PIO0_0::MODE.val(config.pull as u32)
            + PIO0_0::DIGIMODE.val(config.digital_mode as u32);

        match pin {
            LPCPin::P0_0 => self.registers.pio0_0.set(standard_value.into()),
            LPCPin::P0_1 => self.registers.pio0_1.set(standard_value.into()),
            // LPCPin::P0_2 => self.registers.pio0_2.set(standard_value.into()),
            // LPCPin::P0_3 => self.registers.pio0_3.set(standard_value.into()),
            // LPCPin::P0_4 => self.registers.pio0_4.set(standard_value.into()),
            // LPCPin::P0_5 => self.registers.pio0_5.set(standard_value.into()),
            // LPCPin::P0_6 => self.registers.pio0_6.set(standard_value.into()),
            // LPCPin::P0_7 => self.registers.pio0_7.set(standard_value.into()),
            // LPCPin::P0_8 => self.registers.pio0_8.set(standard_value.into()),
            // LPCPin::P0_9 => self.registers.pio0_9.set(standard_value.into()),
            // LPCPin::P0_10 => self.registers.pio0_10.set(standard_value.into()),
            // LPCPin::P0_11 => self.registers.pio0_11.set(standard_value.into()),
            // LPCPin::P0_12 => self.registers.pio0_12.set(standard_value.into()),
            // LPCPin::P0_13 => self.registers.pio0_13.set(standard_value.into()),
            // LPCPin::P0_14 => self.registers.pio0_14.set(standard_value.into()),
            // LPCPin::P0_15 => self.registers.pio0_15.set(standard_value.into()),
            // LPCPin::P0_16 => self.registers.pio0_16.set(standard_value.into()),
            // LPCPin::P0_17 => self.registers.pio0_17.set(standard_value.into()),
            // LPCPin::P0_18 => self.registers.pio0_18.set(standard_value.into()),
            // LPCPin::P0_19 => self.registers.pio0_19.set(standard_value.into()),
            // LPCPin::P0_20 => self.registers.pio0_20.set(standard_value.into()),
            // LPCPin::P0_21 => self.registers.pio0_21.set(standard_value.into()),
            // LPCPin::P0_22 => self.registers.pio0_22.set(standard_value.into()),
            // LPCPin::P0_23 => self.registers.pio0_23.set(standard_value.into()),
            // LPCPin::P0_24 => self.registers.pio0_24.set(standard_value.into()),
            // LPCPin::P0_25 => self.registers.pio0_25.set(standard_value.into()),
            // LPCPin::P0_26 => self.registers.pio0_26.set(standard_value.into()),
            // LPCPin::P0_27 => self.registers.pio0_27.set(standard_value.into()),
            // LPCPin::P0_28 => self.registers.pio0_28.set(standard_value.into()),
            // LPCPin::P0_29 => self.registers.pio0_29.set(standard_value.into()),
            // LPCPin::P0_30 => self.registers.pio0_30.set(standard_value.into()),
            // LPCPin::P0_31 => self.registers.pio0_31.set(standard_value.into()),
            // LPCPin::P1_0 => self.registers.pio1_0.set(standard_value.into()),
            // LPCPin::P1_1 => self.registers.pio1_1.set(standard_value.into()),
            // LPCPin::P1_2 => self.registers.pio1_2.set(standard_value.into()),
            // LPCPin::P1_3 => self.registers.pio1_3.set(standard_value.into()),
            LPCPin::P1_4 => self.registers.pio1_4.set(standard_value.into()), //BLUE LED ON THE BOARD
            // LPCPin::P1_5 => self.registers.pio1_5.set(standard_value.into()),
            LPCPin::P1_6 => self.registers.pio1_6.set(standard_value.into()),
            // LPCPin::P1_7 => self.registers.pio1_7.set(standard_value.into()),
            // LPCPin::P1_8 => self.registers.pio1_8.set(standard_value.into()),
            LPCPin::P1_9 => self.registers.pio1_9.set(standard_value.into()), //USER BUTTON ON THE BOARD
            // LPCPin::P1_10 => self.registers.pio1_10.set(standard_value.into()),
            // LPCPin::P1_11 => self.registers.pio1_11.set(standard_value.into()),
            // LPCPin::P1_12 => self.registers.pio1_12.set(standard_value.into()),
            // LPCPin::P1_13 => self.registers.pio1_13.set(standard_value.into()),
            // LPCPin::P1_14 => self.registers.pio1_14.set(standard_value.into()),
            // LPCPin::P1_15 => self.registers.pio1_15.set(standard_value.into()),
            // LPCPin::P1_16 => self.registers.pio1_16.set(standard_value.into()),
            // LPCPin::P1_17 => self.registers.pio1_17.set(standard_value.into()),
            // LPCPin::P1_18 => self.registers.pio1_18.set(standard_value.into()),
            // LPCPin::P1_19 => self.registers.pio1_19.set(standard_value.into()),
            // LPCPin::P1_20 => self.registers.pio1_20.set(standard_value.into()),
            // LPCPin::P1_21 => self.registers.pio1_21.set(standard_value.into()),
            // LPCPin::P1_22 => self.registers.pio1_22.set(standard_value.into()),
            // LPCPin::P1_23 => self.registers.pio1_23.set(standard_value.into()),
            // LPCPin::P1_24 => self.registers.pio1_24.set(standard_value.into()),
            // LPCPin::P1_25 => self.registers.pio1_25.set(standard_value.into()),
            // LPCPin::P1_26 => self.registers.pio1_26.set(standard_value.into()),
            // LPCPin::P1_27 => self.registers.pio1_27.set(standard_value.into()),
            // LPCPin::P1_28 => self.registers.pio1_28.set(standard_value.into()),
            // LPCPin::P1_29 => self.registers.pio1_29.set(standard_value.into()),
            // LPCPin::P1_30 => self.registers.pio1_30.set(standard_value.into()),
            // LPCPin::P1_31 => self.registers.pio1_31.set(standard_value.into()),
            _ => {}
        }
    }

    pub fn set_pull_none(&self, pin: LPCPin) {
        let config = Config {
            function: Function::GPIO,
            pull: Pull::None,
            slew: Slew::Standard,
            invert: false,
            digital_mode: true,
            open_drain: false,
        };
        self.configure_pin(pin, config);
    }

    pub fn set_pull_up(&self, pin: LPCPin) {
        let config = Config {
            function: Function::GPIO,
            pull: Pull::Up,
            slew: Slew::Standard,
            invert: false,
            digital_mode: true,
            open_drain: false,
        };
        self.configure_pin(pin, config);
    }

    pub fn set_pull_down(&self, pin: LPCPin) {
        let config = Config {
            function: Function::GPIO,
            pull: Pull::Down,
            slew: Slew::Standard,
            invert: false,
            digital_mode: true,
            open_drain: false,
        };
        self.configure_pin(pin, config);
    }
}
