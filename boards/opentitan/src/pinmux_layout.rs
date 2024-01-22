// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2023.

use earlgrey::pinmux_config::{EarlGreyPinmuxConfig, INPUT_NUM, OUTPUT_NUM};
use earlgrey::registers::top_earlgrey::{PinmuxInsel, PinmuxOutsel};

type In = PinmuxInsel;
type Out = PinmuxOutsel;

pub enum BoardPinmuxLayout {}

/// Implementations of Pinmux initial board configurations.
/// Defined Pinmux layout is designed for CW310 FPGA board
/// and is compatible with Hyperdebug test board IO layout.
/// In feature we should add layouts compatible with other
/// OpenTitan boards.
/// Source of true:
/// <OPENTITAN_TREE/hw/top_earlgrey/data/pins_cw310_hyperdebug.xdc>
impl EarlGreyPinmuxConfig for BoardPinmuxLayout {
    /// Array of input selector initial configurations
    #[rustfmt::skip]
    const INPUT: &'static [PinmuxInsel; INPUT_NUM] = &[
        In::Ioa2,         // GpioGpio0
        In::Ioa3,         // GpioGpio1
        In::Ioa6,         // GpioGpio2
        In::Iob0,         // GpioGpio3
        In::Iob1,         // GpioGpio4
        In::Iob2,         // GpioGpio5
        In::Iob3,         // GpioGpio6
        In::Iob6,         // GpioGpio7
        In::Iob7,         // GpioGpio8
        In::Iob8,         // GpioGpio9
        In::Ioc0,         // GpioGpio10
        In::Ioc1,         // GpioGpio11
        In::Ioc2,         // GpioGpio12
        In::Ioc5,         // GpioGpio13
        In::Ioc6,         // GpioGpio14
        In::Ioc7,         // GpioGpio15
        In::Ioc8,         // GpioGpio16
        In::Ioc9,         // GpioGpio17
        In::Ioc10,        // GpioGpio18
        In::Ioc11,        // GpioGpio19
        In::Ioc12,        // GpioGpio20
        In::Ior0,         // GpioGpio21
        In::Ior1,         // GpioGpio22
        In::Ior2,         // GpioGpio23
        In::Ior3,         // GpioGpio24
        In::Ior4,         // GpioGpio25
        In::Ior5,         // GpioGpio26
        In::Ior6,         // GpioGpio27
        In::Ior7,         // GpioGpio28
        In::Ior10,        // GpioGpio29
        In::Ior11,        // GpioGpio30
        In::Ior12,        // GpioGpio31
        In::Ioa7,         // I2c0Sda
        In::Ioa8,         // I2c0Scl
        In::Iob10,        // I2c1Sda
        In::Iob9,         // I2c1Scl
        In::Iob11,        // I2c2Sda
        In::Iob12,        // I2c2Scl
        In::ConstantZero, // SpiHost1Sd0
        In::ConstantZero, // SpiHost1Sd1
        In::ConstantZero, // SpiHost1Sd2
        In::ConstantZero, // SpiHost1Sd3
        In::Ioa0,         // Uart0Rx
        In::Ioa4,         // Uart1Rx
        In::Iob4,         // Uart2Rx
        In::Ioc3,         // Uart3Rx
        In::ConstantZero, // SpiDeviceTpmCsb
        In::ConstantZero, // FlashCtrlTck
        In::ConstantZero, // FlashCtrlTms
        In::ConstantZero, // FlashCtrlTdi
        In::ConstantZero, // SysrstCtrlAonAcPresent
        In::ConstantZero, // SysrstCtrlAonKey0In
        In::ConstantZero, // SysrstCtrlAonKey1In
        In::ConstantZero, // SysrstCtrlAonKey2In
        In::ConstantZero, // SysrstCtrlAonPwrbIn
        In::ConstantZero, // SysrstCtrlAonLidOpen
        In::ConstantZero, // UsbdevSense
    ];

    /// Array representing configgurations of pinmux output selector
    #[rustfmt::skip]
    const OUTPUT: &'static [PinmuxOutsel; OUTPUT_NUM] = &[
        // __________  BANK IOA __________
        Out::ConstantHighZ, // Ioa0 (CW310Hyp Uart_RX / CW310 SAM3X)
        Out::Uart3Tx,       // Ioa1 (CW310Hyp Uart_Tx / CW310 SAM3x)
        Out::GpioGpio0,     // Ioa2
        Out::GpioGpio1,     // Ioa3
        Out::ConstantHighZ, // Ioa4
        Out::Uart1Tx,       // Ioa5
        Out::GpioGpio2,     // Ioa6
        Out::I2c0Sda,       // Ioa7 I2C0_TPM_SDA
        Out::I2c0Scl,       // Ioa8 I2C0_TPM_SCL
        // __________ BANK IOB __________
        Out::GpioGpio3,     // Iob0 SPI_HOST_CS
        Out::GpioGpio4,     // Iob1 SPI_HOST_DI
        Out::GpioGpio5,     // Iob2 SPI_HOST_DO
        Out::GpioGpio6,     // Iob3 SPI_HOST_CLK
        Out::ConstantHighZ, // Iob4 UART2_RX
        Out::Uart2Tx,       // Iob5 UART2_TX
        Out::GpioGpio7,     // Iob6
        Out::GpioGpio8,     // Iob7
        Out::GpioGpio9,     // Iob8
        Out::I2c1Scl,       // Iob9  I2C1_SCL
        Out::I2c1Sda,       // Iob10 I2C1_SDA
        Out::I2c2Sda,       // Iob11 I2C2_SDA
        Out::I2c2Scl,       // Iob12 I2C2_SCL
        // __________ BANK IOC __________
        Out::GpioGpio10,    // Ioc0
        Out::GpioGpio11,    // Ioc1
        Out::GpioGpio12,    // Ioc2
        Out::ConstantHighZ, // Ioc3 UART3_RX
        Out::Uart0Tx,       // Ioc4 UART3_TX
        Out::ConstantHighZ, // Ioc5 (TAP STRAP 1)
        Out::GpioGpio14,    // Ioc6
        Out::GpioGpio15,    // Ioc7
        Out::ConstantHighZ, // Ioc8 (TAP STRAP 0)
        Out::GpioGpio17,    // Ioc9
        Out::GpioGpio18,    // Ioc10
        Out::GpioGpio19,    // Ioc11
        Out::GpioGpio20,    // Ioc12
        // __________ BANK IOR __________
        Out::GpioGpio21,    // Ior0
        Out::GpioGpio22,    // Ior1
        Out::GpioGpio23,    // Ior2
        Out::GpioGpio24,    // Ior3
        Out::GpioGpio25,    // Ior4
        Out::GpioGpio26,    // Ior5
        Out::GpioGpio27,    // Ior6
        Out::GpioGpio28,    // Ior7
        // DIO CW310_hyp       Ior8
        // DIO CW310_hyp       Ior9
        Out::GpioGpio29,    // Ior10
        Out::GpioGpio30,    // Ior11
        Out::GpioGpio31,    // Ior12
        Out::ConstantHighZ, // Ior13
    ];
}
