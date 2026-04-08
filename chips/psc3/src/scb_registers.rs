// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Infineon Technologies AG 2026.

//! SCB (Serial Communication Block) registers and bitfields.
//! In seperate file to avoid too large files.

use kernel::utilities::{
    registers::{register_bitfields, register_structs, ReadWrite},
    StaticRef,
};

register_structs! {
    /// Serial Communications Block (SPI/UART/I2C)
    pub ScbRegisters {
        /// Generic control
        (0x000 => pub ctrl: ReadWrite<u32, CTRL::Register>),
        /// Generic status
        (0x004 => pub status: ReadWrite<u32>),
        /// Command/response control
        (0x008 => pub cmd_resp_ctrl: ReadWrite<u32, CMD_RESP_CTRL::Register>),
        /// Command/response status
        (0x00C => pub cmd_resp_status: ReadWrite<u32, CMD_RESP_STATUS::Register>),
        (0x010 => _reserved0),
        /// SPI control
        (0x020 => pub spi_ctrl: ReadWrite<u32, SPI_CTRL::Register>),
        /// SPI status
        (0x024 => pub spi_status: ReadWrite<u32, SPI_STATUS::Register>),
        /// SPI transmitter control
        (0x028 => pub spi_tx_ctrl: ReadWrite<u32, SPI_TX_CTRL::Register>),
        /// SPI receiver control
        (0x02C => pub spi_rx_ctrl: ReadWrite<u32, SPI_RX_CTRL::Register>),
        (0x030 => _reserved1),
        /// UART control
        (0x040 => pub uart_ctrl: ReadWrite<u32, UART_CTRL::Register>),
        /// UART transmitter control
        (0x044 => pub uart_tx_ctrl: ReadWrite<u32, UART_TX_CTRL::Register>),
        /// UART receiver control
        (0x048 => pub uart_rx_ctrl: ReadWrite<u32, UART_RX_CTRL::Register>),
        /// UART receiver status
        (0x04C => pub uart_rx_status: ReadWrite<u32>),
        /// UART flow control
        (0x050 => pub uart_flow_ctrl: ReadWrite<u32, UART_FLOW_CTRL::Register>),
        (0x054 => _reserved2),
        /// I2C control
        (0x060 => pub i2c_ctrl: ReadWrite<u32, I2C_CTRL::Register>),
        /// I2C status
        (0x064 => pub i2c_status: ReadWrite<u32, I2C_STATUS::Register>),
        /// I2C master command
        (0x068 => pub i2c_m_cmd: ReadWrite<u32, I2C_M_CMD::Register>),
        /// I2C slave command
        (0x06C => pub i2c_s_cmd: ReadWrite<u32, I2C_S_CMD::Register>),
        /// I2C configuration
        (0x070 => pub i2c_cfg: ReadWrite<u32, I2C_CFG::Register>),
        /// I2C stretch control
        (0x074 => pub i2c_stretch_ctrl: ReadWrite<u32>),
        /// I2C stretch status
        (0x078 => pub i2c_stretch_status: ReadWrite<u32, I2C_STRETCH_STATUS::Register>),
        (0x07C => _reserved3),
        /// I2C control for High-Speed mode
        (0x080 => pub i2c_ctrl_hs: ReadWrite<u32, I2C_CTRL_HS::Register>),
        (0x084 => _reserved4),
        /// Transmitter control
        (0x200 => pub tx_ctrl: ReadWrite<u32, TX_CTRL::Register>),
        /// Transmitter FIFO control
        (0x204 => pub tx_fifo_ctrl: ReadWrite<u32, TX_FIFO_CTRL::Register>),
        /// Transmitter FIFO status
        (0x208 => pub tx_fifo_status: ReadWrite<u32, TX_FIFO_STATUS::Register>),
        (0x20C => _reserved5),
        /// Transmitter FIFO write
        (0x240 => pub tx_fifo_wr: ReadWrite<u32, TX_FIFO_WR::Register>),
        (0x244 => _reserved6),
        /// Receiver control
        (0x300 => pub rx_ctrl: ReadWrite<u32, RX_CTRL::Register>),
        /// Receiver FIFO control
        (0x304 => pub rx_fifo_ctrl: ReadWrite<u32, RX_FIFO_CTRL::Register>),
        /// Receiver FIFO status
        (0x308 => pub rx_fifo_status: ReadWrite<u32, RX_FIFO_STATUS::Register>),
        (0x30C => _reserved7),
        /// Slave address and mask
        (0x310 => pub rx_match: ReadWrite<u32, RX_MATCH::Register>),
        (0x314 => _reserved8),
        /// Receiver FIFO read
        (0x340 => pub rx_fifo_rd: ReadWrite<u32, RX_FIFO_RD::Register>),
        /// Receiver FIFO read silent
        (0x344 => pub rx_fifo_rd_silent: ReadWrite<u32, RX_FIFO_RD_SILENT::Register>),
        (0x348 => _reserved9),
        /// Active clocked interrupt signal
        (0xE00 => pub intr_cause: ReadWrite<u32, INTR_CAUSE::Register>),
        (0xE04 => _reserved10),
        /// Externally clocked I2C interrupt request
        (0xE80 => pub intr_i2c_ec: ReadWrite<u32, INTR_I2C_EC::Register>),
        (0xE84 => _reserved11),
        /// Externally clocked I2C interrupt mask
        (0xE88 => pub intr_i2c_ec_mask: ReadWrite<u32, INTR_I2C_EC_MASK::Register>),
        /// Externally clocked I2C interrupt masked
        (0xE8C => pub intr_i2c_ec_masked: ReadWrite<u32, INTR_I2C_EC_MASKED::Register>),
        (0xE90 => _reserved12),
        /// Externally clocked SPI interrupt request
        (0xEC0 => pub intr_spi_ec: ReadWrite<u32, INTR_SPI_EC::Register>),
        (0xEC4 => _reserved13),
        /// Externally clocked SPI interrupt mask
        (0xEC8 => pub intr_spi_ec_mask: ReadWrite<u32, INTR_SPI_EC_MASK::Register>),
        /// Externally clocked SPI interrupt masked
        (0xECC => pub intr_spi_ec_masked: ReadWrite<u32, INTR_SPI_EC_MASKED::Register>),
        (0xED0 => _reserved14),
        /// Master interrupt request
        (0xF00 => pub intr_m: ReadWrite<u32, INTR_M::Register>),
        /// Master interrupt set request
        (0xF04 => pub intr_m_set: ReadWrite<u32, INTR_M_SET::Register>),
        /// Master interrupt mask
        (0xF08 => pub intr_m_mask: ReadWrite<u32, INTR_M_MASK::Register>),
        /// Master interrupt masked request
        (0xF0C => pub intr_m_masked: ReadWrite<u32, INTR_M_MASKED::Register>),
        (0xF10 => _reserved15),
        /// Slave interrupt request
        (0xF40 => pub intr_s: ReadWrite<u32, INTR_S::Register>),
        /// Slave interrupt set request
        (0xF44 => pub intr_s_set: ReadWrite<u32, INTR_S_SET::Register>),
        /// Slave interrupt mask
        (0xF48 => pub intr_s_mask: ReadWrite<u32, INTR_S_MASK::Register>),
        /// Slave interrupt masked request
        (0xF4C => pub intr_s_masked: ReadWrite<u32, INTR_S_MASKED::Register>),
        (0xF50 => _reserved16),
        /// Transmitter interrupt request
        (0xF80 => pub intr_tx: ReadWrite<u32, INTR_TX::Register>),
        /// Transmitter interrupt set request
        (0xF84 => pub intr_tx_set: ReadWrite<u32, INTR_TX_SET::Register>),
        /// Transmitter interrupt mask
        (0xF88 => pub intr_tx_mask: ReadWrite<u32, INTR_TX_MASK::Register>),
        /// Transmitter interrupt masked request
        (0xF8C => pub intr_tx_masked: ReadWrite<u32, INTR_TX_MASKED::Register>),
        (0xF90 => _reserved17),
        /// Receiver interrupt request
        (0xFC0 => pub intr_rx: ReadWrite<u32, INTR_RX::Register>),
        /// Receiver interrupt set request
        (0xFC4 => pub intr_rx_set: ReadWrite<u32, INTR_RX_SET::Register>),
        /// Receiver interrupt mask
        (0xFC8 => pub intr_rx_mask: ReadWrite<u32, INTR_RX_MASK::Register>),
        /// Receiver interrupt masked request
        (0xFCC => pub intr_rx_masked: ReadWrite<u32, INTR_RX_MASKED::Register>),
        (0xFD0 => @END),
    }
}
register_bitfields![u32,
pub CTRL [
    /// N/A
    OVS OFFSET(0) NUMBITS(4) [],
    /// This field specifies the clocking for the address matching (I2C) or slave selection detection logic (SPI)
    /// '0': Internally clocked mode
    /// '1': Externally clocked mode
    ///
    /// In internally clocked mode, the serial interface protocols run off the SCB clock. In externally clocked mode, the serial interface protocols run off the clock as provided by the serial interface.
    ///
    /// The clocking for the rest of the logic is determined by CTRL.EC_OP_MODE.
    ///
    /// Externally clocked mode is only used for synchronous serial interface protocols (SPI and I2C) in slave mode. In SPI mode, only Motorola submode (all Motorola modes: 0, 1, 2, 3) is supported.
    ///
    /// In UART mode this field should be '0'.
    EC_AM_MODE OFFSET(8) NUMBITS(1) [],
    /// This field specifies the clocking for the SCB block
    /// '0': Internally clocked mode
    /// '1': externally clocked mode
    ///  In internally clocked mode, the serial interface protocols run off the SCB clock. In externally clocked mode, the serial interface protocols run off the clock as provided by the serial interface.
    ///
    /// Externally clocked operation mode is only used for synchronous serial interface protocols (SPI and I2C) in slave mode AND EZ mode. In SPI mode, only Motorola submode (all Motorola modes: 0, 1, 2, 3) is supported. The maximum SPI slave, EZ mode bitrate is 48 Mbps (transmission and IO delays outside the IP will degrade the effective bitrate).
    ///
    /// In UART mode this field should be '0'.
    EC_OP_MODE OFFSET(9) NUMBITS(1) [],
    /// Non EZ mode ('0') or EZ mode ('1').
    /// In EZ mode, a meta protocol is applied to the serial interface protocol. This meta protocol adds meaning to the data frames transferred by the serial interface protocol: a data frame can represent a memory address, a write memory data element or a read memory data element. EZ mode is only used for synchronous serial interface protocols: SPI and I2C. In SPI mode, only Motorola submode (all Motorola modes: 0, 1, 2, 3) is supported and the transmitter should use continuous data frames; i.e. data frames not separated by slave deselection. This mode is only applicable to slave functionality. In EZ mode, the slave can read from and write to an addressable memory structure of 32 bytes. In EZ mode, data frames should 8-bit in size and should be transmitted and received with the Most Significant Bit (MSB) first.
    ///
    /// In UART mode this field should be '0'.
    EZ_MODE OFFSET(10) NUMBITS(1) [],
    /// N/A
    CMD_RESP_MODE OFFSET(12) NUMBITS(1) [],
    /// N/A
    MEM_WIDTH OFFSET(14) NUMBITS(2) [
        /// 8-bit FIFO data elements.
        /// This mode provides the biggest amount of FIFO entries, but  TX_CTRL.DATA_WIDTH and RX_CTRL.DATA_WIDTH are restricted to [0, 7].
        BYTE = 0,
        /// 16-bit FIFO data elements.
        /// TX_CTRL.DATA_WIDTH and RX_CTRL.DATA_WIDTH are restricted to [0, 15].
        HALFWORD = 1,
        /// 32-bit FIFO data elements.
        /// This mode provides the smallest amount of FIFO entries, but TX_CTRL.DATA_WIDTH and RX_CTRL.DATA_WIDTH can be in a range of [0, 31].
        WORD = 2,
        /// N/A
        RSVD = 3
    ],
    /// Determines whether a received matching address is accepted in the RX FIFO ('1') or not ('0').
    ///
    /// In I2C mode, this field is used to allow the slave to put the received slave address or general call address in the RX FIFO. Note that a received matching address is put in the RX FIFO when this bit is '1' for both I2C read and write transfers.
    ///
    /// In multi-processor UART receiver mode, this field is used to allow the receiver to put the received address in the RX FIFO. Note: non-matching addresses are never put in the RX FIFO.
    ADDR_ACCEPT OFFSET(16) NUMBITS(1) [],
    /// Only used in externally clocked mode. If the externally clocked logic and the internal CPU accesses to EZ memory coincide/collide, this bit determines whether the CPU access should block and result in bus wait states ('BLOCK is 1') or not (BLOCK is '0'). IF BLOCK is '0' and the accesses collide, CPU read operations return 0xffff:ffff and CPU write operations are ignored. Colliding accesses are registered as interrupt causes: INTR_TX.BLOCKED and INTR_RX.BLOCKED.
    BLOCK OFFSET(17) NUMBITS(1) [],
    /// N/A
    MODE OFFSET(24) NUMBITS(2) [
        /// Inter-Integrated Circuits (I2C) mode.
        I2C = 0,
        /// Serial Peripheral Interface (SPI) mode.
        SPI = 1,
        /// Universal Asynchronous Receiver/Transmitter (UART) mode.
        UART = 2
    ],
    /// EC_ACCESS is used to enable I2CS_EC or SPIS_EC access to internal EZ memory.
    /// 1: enable clk_scb
    /// 0: disable clk_scb
    ///
    /// Before going to deepsleep this field should be set to 1.
    /// when waking up from DeepSleep power mode, and PLL is locked (clk_scb is at expected frequency), this filed should be set to 0.
    EC_ACCESS OFFSET(28) NUMBITS(1) [],
    /// SCB block is enabled ('1') or not ('0'). The proper order in which to initialize SCB is as follows:
    /// - Program protocol specific information using SPI_CTRL, UART_CTRL (and UART_TX_CTRL and UART_RX_CTRL) or I2C_CTRL registers. This includes selection of a submode, master/slave functionality and transmitter/receiver functionality when applicable.
    /// - Program generic transmitter (TX_CTRL) and receiver (RX_CTRL) information. This includes enabling of the transmitter and receiver functionality.
    /// - Program transmitter FIFO (TX_FIFO_CTRL) and receiver FIFO (RX_FIFO_CTRL) information.
    /// - Program CTRL register to enable SCB, select the specific operation mode and oversampling factor.
    /// Generally when this block is enabled, no control information should be changed. Changes should be made AFTER disabling this block, e.g. to modify the operation mode (from I2C to SPI) or to go from externally to internally clocked. The change takes effect after the block is re-enabled. Note that disabling the block will cause re-initialization of the design and associated state is lost (e.g. FIFO content).
    ///
    /// Specific to SPI master case,  when SCB is idle,  below registers can be changed without disabling SCB block,
    ///       TX_CTRL
    ///       TX_FIFO_CTRL
    ///       RX_CTRL
    ///       RX_FIFO_CTRL
    ///       SPI_CTRL.SSEL,
    ENABLED OFFSET(31) NUMBITS(1) []
],
pub STATUS [
    /// Indicates whether the externally clocked logic is potentially accessing the EZ memory (this is only possible in EZ mode). This bit can be used by SW to determine whether it is safe to issue a SW access to the EZ memory (without bus wait states (a blocked SW access) or bus errors being generated). Note that the INTR_TX.BLOCKED and INTR_RX.BLOCKED interrupt causes are used to indicate whether a SW access was actually blocked by externally clocked logic.
    EC_BUSY OFFSET(0) NUMBITS(1) []
],
pub CMD_RESP_CTRL [
    /// I2C/SPI read base address for CMD_RESP mode. At the start of a read transfer this BASE_RD_ADDR is copied to CMD_RESP_STATUS.CURR_RD_ADDR. This field should not be modified during ongoing bus transfers.
    BASE_RD_ADDR OFFSET(0) NUMBITS(9) [],
    /// I2C/SPI write base address for CMD_RESP mode. At the start of a write transfer this BASE_WR_ADDR is copied to CMD_RESP_STATUS.CURR_WR_ADDR. This field should not be modified during ongoing bus transfers.
    BASE_WR_ADDR OFFSET(16) NUMBITS(9) []
],
pub CMD_RESP_STATUS [
    /// I2C/SPI read current address for CMD_RESP mode. HW increments the field after a read access to the memory buffer. However, when the last memory buffer address is reached, the address is NOT incremented (but remains at the maximum memory buffer address).
    ///
    /// The field is used to determine how many bytes have been read (# bytes = CURR_RD_ADDR - CMD_RESP_CTRL.BASE_RD_ADDR).
    ///
    /// This field is reliable when there is no bus transfer. This field is potentially unreliable when there is a ongoing bus transfer, i.e. when CMD_RESP_EC_BUSY is '0', the field is reliable.
    CURR_RD_ADDR OFFSET(0) NUMBITS(9) [],
    /// I2C/SPI write current address for CMD_RESP mode. HW increments the field after a write access to the memory buffer. However, when the last memory buffer address is reached, the address is NOT incremented (but remains at the maximum memory buffer address).
    ///
    /// The field is used to determine how many bytes have been written (# bytes = CURR_WR_ADDR - CMD_RESP_CTRL.BASE_WR_ADDR).
    ///
    /// This field is reliable when there is no bus transfer. This field is potentially unreliable when there is a ongoing bus transfer, i.e. when CMD_RESP_EC_BUSY is '0', the field is reliable.
    CURR_WR_ADDR OFFSET(16) NUMBITS(9) [],
    /// Indicates whether there is an ongoing bus transfer to the IP.
    /// '0': no ongoing bus transfer.
    /// '1': ongoing bus transfer.
    ///
    /// For SPI, the field is '1' when slave mode is selected.
    ///
    /// For I2C, the field is set to '1' at a I2C START/RESTART. In case of an address match, the  field is set to '0' on a I2C STOP. In case of NO address match, the field is set to '0' after the failing address match.
    CMD_RESP_EC_BUS_BUSY OFFSET(30) NUMBITS(1) [],
    /// N/A
    CMD_RESP_EC_BUSY OFFSET(31) NUMBITS(1) []
],
pub SPI_CTRL [
    /// Continuous SPI data transfers enabled ('1') or not ('0'). This field is used in master mode. In slave mode, both continuous and non-continuous SPI data transfers are supported independent of this field.
    ///
    /// When continuous transfers are enabled individual data frame transfers are not necessarily separated by slave deselection (as indicated by the level or pulse on the SELECT line): if the TX FIFO has multiple data frames, data frames are send out without slave deselection.
    ///
    /// When continuous transfers are not enabled individual data frame transfers are always separated by slave deselection: independent of the availability of TX FIFO data frames, data frames are sent out with slave deselection.
    SSEL_CONTINUOUS OFFSET(0) NUMBITS(1) [],
    /// Only used in SPI Texas Instruments' submode.
    ///
    /// When '1', the data frame start indication is a pulse on the Slave SELECT line that precedes the transfer of the first data frame bit.
    ///
    /// When '0', the data frame start indication is a pulse on the Slave SELECT line that coincides with the transfer of the first data frame bit.
    SELECT_PRECEDE OFFSET(1) NUMBITS(1) [],
    /// N/A
    CPHA OFFSET(2) NUMBITS(1) [],
    /// N/A
    CPOL OFFSET(3) NUMBITS(1) [],
    /// Changes the SCLK edge on which MISO is captured in master mode, or MOSI is captured in slave mode.
    ///
    /// When '0', the default applies,
    /// for Motorola as determined by CPOL and CPHA,
    /// for Texas Instruments on the falling edge of SCLK(CPOL is '0' and CPHA is '1'),
    /// for National Semiconductors on the rising edge of SCLK(CPOL is '0' and CPHA is '0').
    ///
    /// When '1', the alternate clock edge is used (which comes half a SPI SCLK period later).
    /// for master, applicable to all Motorola, TI and National Semiconductors flavors, and CPOL/CPHA timing mdoes.
    /// for slave, applicable to Motorola flavor only, and CPHA=0 timing modes only, and internally-clocked mode only.
    ///
    /// Late sampling addresses the round trip delay associated with transmitting SCLK from the master to the slave and transmitting MISO from the slave to the master.
    LATE_SAMPLE OFFSET(4) NUMBITS(1) [],
    /// N/A
    SCLK_CONTINUOUS OFFSET(5) NUMBITS(1) [],
    /// N/A
    SSEL_POLARITY0 OFFSET(8) NUMBITS(1) [],
    /// N/A
    SSEL_POLARITY1 OFFSET(9) NUMBITS(1) [],
    /// N/A
    SSEL_POLARITY2 OFFSET(10) NUMBITS(1) [],
    /// N/A
    SSEL_POLARITY3 OFFSET(11) NUMBITS(1) [],
    /// N/A
    SSEL_SETUP_DEL OFFSET(12) NUMBITS(1) [],
    /// N/A
    SSEL_HOLD_DEL OFFSET(13) NUMBITS(1) [],
    /// N/A
    SSEL_INTER_FRAME_DEL OFFSET(14) NUMBITS(1) [],
    /// Local loopback control (does NOT affect the information on the pins). Only used in master mode. Not used in National Semiconductors submode.
    /// '0': No local loopback
    /// '1': the SPI master MISO line is connected to the SPI master MOSI line. In other words, in loopback mode the SPI master receives on MISO what it transmits on MOSI.
    LOOPBACK OFFSET(16) NUMBITS(1) [],
    /// N/A
    MODE OFFSET(24) NUMBITS(2) [
        /// SPI Motorola submode. In master mode, when not transmitting data (SELECT is inactive), SCLK is stable at CPOL. In slave mode, when not selected, SCLK is ignored; i.e. it can be either stable or clocking. In master mode, when there is no data to transmit (TX FIFO is empty), SELECT is inactive.
        SPI_MOTOROLA = 0,
        /// SPI Texas Instruments submode. In master mode, when not transmitting data, SCLK is stable at '0'. In slave mode, when not selected, SCLK is ignored; i.e. it can be either stable or clocking. In master mode, when there is no data to transmit (TX FIFO is empty), SELECT is inactive; i.e. no pulse is generated.
        SPI_TI = 1,
        /// SPI National Semiconductors submode. In master mode, when not transmitting data, SCLK is stable at '0'. In slave mode, when not selected, SCLK is ignored; i.e. it can be either stable or clocking. In master mode, when there is no data to transmit (TX FIFO is empty), SELECT is inactive.
        SPI_NS = 2
    ],
    /// Selects one of the four incoming/outgoing SPI slave select signals:
    /// - 0: Slave 0, SSEL[0].
    /// - 1: Slave 1, SSEL[1].
    /// - 2: Slave 2, SSEL[2].
    /// - 3: Slave 3, SSEL[3].
    /// SCB block should be disabled when changes are made to this field.
    SSEL OFFSET(26) NUMBITS(2) [],
    /// N/A
    MASTER_MODE OFFSET(31) NUMBITS(1) []
],
pub SPI_STATUS [
    /// SPI bus is busy. The bus is considered busy ('1') during an ongoing transaction. For Motorola and National submodes, the busy bit is '1', when the slave selection is activated. For TI submode, the busy bit is '1' from the time the preceding/coinciding slave select is activated for the first transmitted data frame, till the last MOSI/MISO bit of the last data frame is transmitted.
    BUS_BUSY OFFSET(0) NUMBITS(1) [],
    /// Indicates whether the externally clocked logic is potentially accessing the EZ memory and/or updating BASE_ADDR or CURR_ADDR (this is only possible in EZ mode). This bit can be used by SW to determine whether BASE_ADDR and CURR_ADDR are reliable.
    SPI_EC_BUSY OFFSET(1) NUMBITS(1) [],
    /// SPI current EZ address. Current address pointer. This field is only reliable in internally clocked mode. In externally clocked mode the field may be unreliable (during an ongoing transfer when SPI_EC_BUSY is '1'), as clock domain synchronization is not performed in the design.
    CURR_EZ_ADDR OFFSET(8) NUMBITS(8) [],
    /// SPI base EZ address. Address as provided by a SPI write transfer. This field is only reliable in internally clocked mode. In externally clocked mode the field may be unreliable, as clock domain synchronization is not performed in the design.
    BASE_EZ_ADDR OFFSET(16) NUMBITS(8) []
],
pub SPI_TX_CTRL [
    /// Parity bit. When '0', the transmitter generates an even parity. When '1', the transmitter generates an odd parity.
    PARITY OFFSET(4) NUMBITS(1) [],
    /// Parity generation enabled ('1') or not ('0').
    PARITY_ENABLED OFFSET(5) NUMBITS(1) [],
    /// SPI master MOSI output level when SELECT output inactive,
    /// 0: retain the level of last data bit
    /// 1: change to high,
    ///    (MOSI level is high, before the first data bit time, and after data bit time, defined SSEL/SCLK driving edge with CPOL/CPHA)
    MOSI_IDLE_HIGH OFFSET(16) NUMBITS(1) []
],
pub SPI_RX_CTRL [
    /// Parity bit. When '0', the receiver expects an even parity. When '1', the receiver expects an odd parity.
    PARITY OFFSET(4) NUMBITS(1) [],
    /// Parity checking enabled ('1') or not ('0').
    PARITY_ENABLED OFFSET(5) NUMBITS(1) [],
    /// Behavior when a parity check fails. When '0', received data is send to the RX FIFO. When '1', received data is dropped and lost.
    DROP_ON_PARITY_ERROR OFFSET(8) NUMBITS(1) []
],
pub UART_CTRL [
    /// Local loopback control (does NOT affect the information on the pins).
    /// 0: Loopback is not enabled
    /// 1: UART_TX is connected to UART_RX. UART_RTS is connected to UART_CTS.
    /// This allows a SCB UART transmitter to communicate with its receiver counterpart.
    LOOPBACK OFFSET(16) NUMBITS(1) [],
    /// N/A
    MODE OFFSET(24) NUMBITS(2) [
        /// Standard UART submode.
        StandardUARTSubmode = 0,
        /// SmartCard (ISO7816) submode. Support for negative acknowledgement (NACK) on the receiver side and retransmission on the transmitter side.
        UART_SMARTCARD = 1,
        /// Infrared Data Association (IrDA) submode. Return to Zero modulation scheme.
        UART_IRDA = 2
    ]
],
pub UART_TX_CTRL [
    /// Stop bits. STOP_BITS + 1 is the duration of the stop period in terms of halve bit periods. Valid range is [1, 7]; i.e. a stop period should last at least one bit period.
    STOP_BITS OFFSET(0) NUMBITS(3) [],
    /// Parity bit. When '0', the transmitter generates an even parity. When '1', the transmitter generates an odd parity. Only applicable in standard UART and SmartCard submodes.
    PARITY OFFSET(4) NUMBITS(1) [],
    /// Parity generation enabled ('1') or not ('0'). Only applicable in standard UART submodes. In SmartCard submode, parity generation is always enabled through hardware. In IrDA submode, parity generation is always disabled through hardware
    PARITY_ENABLED OFFSET(5) NUMBITS(1) [],
    /// When '1', a data frame is retransmitted when a negative acknowledgement is received. Only applicable to the SmartCard submode.
    RETRY_ON_NACK OFFSET(8) NUMBITS(1) []
],
pub UART_RX_CTRL [
    /// Stop bits. STOP_BITS + 1 is the duration of the stop period in terms of half bit periods. Valid range is [1, 7]; i.e. a stop period should last at least one bit period.
    ///
    /// Note that in case of a stop bits error, the successive data frames may get lost as the receiver needs to resynchronize its start bit detection. The amount of lost data frames depends on both the amount of stop bits, the idle time between data frames and the data frame value.
    STOP_BITS OFFSET(0) NUMBITS(3) [],
    /// N/A
    PARITY OFFSET(4) NUMBITS(1) [],
    /// N/A
    PARITY_ENABLED OFFSET(5) NUMBITS(1) [],
    /// Inverts incoming RX line signal. Inversion is after local loopback. This functionality is intended for IrDA receiver functionality.
    POLARITY OFFSET(6) NUMBITS(1) [],
    /// Behavior when a parity check fails.
    /// When '0', received data is sent to the RX FIFO.
    /// When '1', received data is dropped and lost.
    /// Only applicable in standard UART and SmartCard submodes (negatively acknowledged SmartCard data frames may be dropped with this field).
    DROP_ON_PARITY_ERROR OFFSET(8) NUMBITS(1) [],
    /// Behavior when an error is detected in a start or stop period.
    /// When '0', received data is sent to the RX FIFO.
    ///  When '1', received data is dropped and lost.
    DROP_ON_FRAME_ERROR OFFSET(9) NUMBITS(1) [],
    /// N/A
    MP_MODE OFFSET(10) NUMBITS(1) [],
    /// Only applicable in standard UART submode. When '1', the receiver performs break detection and baud rate detection on the incoming data. First, break detection counts the amount of bit periods that have a line value of '0'. BREAK_WIDTH specifies the minimum required amount of bit periods. Successful break detection sets the INTR_RX.BREAK_DETECT interrupt cause to '1'. Second, baud rate detection counts the amount of peripheral clock periods that are use to receive the synchronization byte (0x55; least significant bit first). The count is available through UART_RX_STATUS.BR_COUNTER. Successful baud rate detection sets the INTR_RX.BAUD_DETECT interrupt cause to '1' (BR_COUNTER is reliable). This functionality is used to synchronize/refine the receiver clock to the transmitter clock. The receiver software can use the BR_COUNTER value to set the right IP clock (from the programmable clock IP) to guarantee successful receipt of the first LIN data frame (Protected Identifier Field) after the synchronization byte.
    LIN_MODE OFFSET(12) NUMBITS(1) [],
    /// N/A
    SKIP_START OFFSET(13) NUMBITS(1) [],
    /// N/A
    HDRXEN OFFSET(14) NUMBITS(1) [],
    /// N/A
    BREAK_WIDTH OFFSET(16) NUMBITS(4) [],
    /// N/A
    BREAK_LEVEL OFFSET(24) NUMBITS(1) []
],
pub UART_RX_STATUS [
    /// Amount of SCB clock periods that constitute the transmission of a 0x55 data frame (sent least significant bit first) as determined by the receiver. BR_COUNTER / 8 is the amount of SCB clock periods that constitute a bit period. This field has valid data when INTR_RX.BAUD_DETECT is set to '1'.
    BR_COUNTER OFFSET(0) NUMBITS(12) []
],
pub UART_FLOW_CTRL [
    /// Trigger level. When the receiver FIFO has less entries than the amount of this field, a Ready To Send (RTS) output signal is activated. By setting this field to '0', flow control is effectively disabled (may be useful for debug purposes).
    TRIGGER_LEVEL OFFSET(0) NUMBITS(8) [],
    /// Polarity of the RTS output signal:
    /// '0': RTS is active low;
    /// '1': RTS is active high;
    ///
    /// During SCB reset (Hibernate system power mode), RTS output signal is '1'. This represents an inactive state assuming an active low polarity.
    RTS_POLARITY OFFSET(16) NUMBITS(1) [],
    /// Polarity of the CTS input signal
    /// '0': CTS is active low ;
    /// '1': CTS is active high;
    CTS_POLARITY OFFSET(24) NUMBITS(1) [],
    /// Enable use of CTS input signal by the UART transmitter:
    /// '0': Disabled. The UART transmitter ignores the CTS input signal and transmits when a data frame is available for transmission in the TX FIFO or the TX shift register.
    /// '1': Enabled. The UART transmitter uses CTS input signal to qualify the transmission of data. It transmits when CTS input signal is active and a data frame is available for transmission in the TX FIFO or the TX shift register.
    ///
    /// If UART_CTRL.LOOPBACK is '1', the CTS input signal is driven by the RTS output signal locally in SCB (both signals are subjected to signal polarity changes are indicated by RTS_POLARITY and CTS_POLARITY).
    CTS_ENABLED OFFSET(25) NUMBITS(1) []
],
I2C_CTRL [
    /// Serial I2C interface high phase oversampling factor. HIGH_PHASE_OVS + 1 SCB clock periods constitute the high phase of a bit period. The valid range is [5, 15] with input signal median filtering and [4, 15] without input signal median filtering.
    ///
    /// The field is only used in master mode. In slave mode, the field is NOT used. However, there is a frequency requirement for the SCB clock wrt. the regular interface (IF) high time to guarantee functional correct behavior. With input signal median filtering, the IF high time should be >= 6 SCB clock cycles and <= 16 SCB clock cycles. Without input signal median filtering, the IF high time should be >= 5 SCB clock cycles and <= 16 SCB clock cycles.
    HIGH_PHASE_OVS OFFSET(0) NUMBITS(4) [],
    /// Serial I2C interface low phase oversampling factor. LOW_PHASE_OVS + 1 SCB clock periods constitute the low phase of a bit period. The valid range is [7, 15] with input signal median filtering and [6, 15] without input signal median filtering.
    ///
    /// The field is only used in master mode. In slave mode, the field is NOT used. However, there is a frequency requirement for the SCB clock wrt. the regular (no stretching) interface (IF) low time to guarantee functionally correct behavior. With input signal median filtering, the IF low time should be >= 8 SCB clock cycles and <= 16 IP clock cycles. Without input signal median filtering, the IF low time should be >= 7 SCB clock cycles and <= 16 SCB clock cycles.
    ///
    /// in slave mode, this field is used to define number of clk_scb cycles for tSU-DAT timing (from ACK/NACK/data ready, to SCL rising edge (released from I2C slave clock stretching))
    LOW_PHASE_OVS OFFSET(4) NUMBITS(4) [],
    /// N/A
    M_READY_DATA_ACK OFFSET(8) NUMBITS(1) [],
    /// N/A
    M_NOT_READY_DATA_NACK OFFSET(9) NUMBITS(1) [],
    /// N/A
    S_GENERAL_IGNORE OFFSET(11) NUMBITS(1) [],
    /// N/A
    S_READY_ADDR_ACK OFFSET(12) NUMBITS(1) [],
    /// N/A
    S_READY_DATA_ACK OFFSET(13) NUMBITS(1) [],
    /// This field is used during an address match or general call address in internally clocked mode
    /// Only used when:
    ///  - EC_AM_MODE is '0', EC_OP_MODE is '0', S_GENERAL_IGNORE is '0] and non EZ mode.
    /// Functionality is as follows:
    /// - 1: a received (matching) slave address is immediately NACK'd when the receiver FIFO is full.
    /// - 0: clock stretching is performed (till the receiver FIFO is no longer full).
    ///
    /// For externally clocked logic (EC_AM is '1') on an address match or general call address (and S_GENERAL_IGNORE is '0'). Only used when (NOT used when EC_AM is '1' and EC_OP is '1' and address match and EZ mode):
    /// - EC_AM is '1' and EC_OP is '0'.
    /// - EC_AM is '1' and general call address match.
    /// - EC_AM is '1' and non EZ mode.
    /// Functionality is as follows:
    /// - 1: a received (matching or general) slave address is always immediately NACK'd. There are two possibilities:
    ///        1). the SCB clock is available (in Active system power mode) and it handles the rest of the current transfer. In this case the I2C master will not observe the NACK.
    ///        2).SCB clock is not present (in DeepSleep system power mode). In this case the I2C master will observe the NACK and may retry the transfer in the future (which gives the internally clocked logic the time to wake up from DeepSleep system power mode).
    /// - 0: clock stretching is performed (till the SCB clock is available). The logic will handle the ongoing transfer as soon as the clock is enabled.
    S_NOT_READY_ADDR_NACK OFFSET(14) NUMBITS(1) [],
    /// Only used when:
    /// - non EZ mode
    /// Functionality is as follows:
    /// - 1: a received data element byte the slave is immediately NACK'd when the receiver FIFO is full.
    /// - 0: clock stretching is performed (till the receiver FIFO is no longer full).
    S_NOT_READY_DATA_NACK OFFSET(15) NUMBITS(1) [],
    /// Local loopback control (does NOT affect the information on the pins). Only applicable in master/slave mode.
    /// When '0', no loopback
    /// When '1', loopback is enabled internally in the peripheral, and as a result unaffected by other I2C devices. This allows a SCB I2C peripheral to address itself.
    LOOPBACK OFFSET(16) NUMBITS(1) [],
    /// N/A
    SLAVE_MODE OFFSET(30) NUMBITS(1) [],
    /// N/A
    MASTER_MODE OFFSET(31) NUMBITS(1) []
],
I2C_STATUS [
    /// I2C bus is busy. The bus is considered busy ('1'), from the time a START is detected or from the time the SCL line is '0'. The bus is considered idle ('0'), from the time a STOP is detected. If SCB block is disabled, BUS_BUSY is '0'. After enabling the block, it takes time for the BUS_BUSY to detect a busy bus. This time is the maximum high time of the SCL line. For a 100 kHz interface frequency, this maximum high time may last roughly 5 us (half a bit period).
    ///
    /// For single master systems, BUS_BUSY does not have to be used to detect an idle bus before a master starts a transfer using I2C_M_CMD.M_START (no bus collisions).
    ///
    /// For multi-master systems, BUS_BUSY can be used to detect an idle bus before a master starts a transfer using I2C_M_CMD.M_START_ON_IDLE (to prevent bus collisions).
    BUS_BUSY OFFSET(0) NUMBITS(1) [],
    /// N/A
    I2C_EC_BUSY OFFSET(1) NUMBITS(1) [],
    /// N/A
    I2CS_IC_BUSY OFFSET(2) NUMBITS(1) [],
    /// N/A
    S_READ OFFSET(4) NUMBITS(1) [],
    /// N/A
    M_READ OFFSET(5) NUMBITS(1) [],
    /// N/A
    CURR_EZ_ADDR OFFSET(8) NUMBITS(8) [],
    /// N/A
    BASE_EZ_ADDR OFFSET(16) NUMBITS(8) [],
    /// N/A
    HS_MODE OFFSET(24) NUMBITS(1) []
],
I2C_M_CMD [
    /// When '1', transmit a START or REPEATED START. Whether a START or REPEATED START is transmitted depends on the state of the master state machine. A START is only transmitted when the master state machine is in the default state. A REPEATED START is transmitted when the master state machine is not in the default state, but is working on an ongoing transaction. The REPEATED START can only be transmitted after a NACK or ACK has been received for a transmitted data element or after a NACK has been transmitted for a received data element. When this action is performed, the hardware sets this field to '0'.
    M_START OFFSET(0) NUMBITS(1) [],
    /// When '1', transmit a START as soon as the bus is idle (I2C_STATUS.BUS_BUSY is '0', note that BUSY has a default value of '0'). For bus idle detection the hardware relies on STOP detection. As a result, bus idle detection is only functional after at least one I2C bus transfer has been detected on the bus (default/reset value of BUSY is '0') . A START is only transmitted when the master state machine is in the default state. When this action is performed, the hardware sets this field to '0'.
    M_START_ON_IDLE OFFSET(1) NUMBITS(1) [],
    /// When '1', attempt to transmit an acknowledgement (ACK). When this action is performed, the hardware sets this field to '0'.
    M_ACK OFFSET(2) NUMBITS(1) [],
    /// for I2C master, the NACKed byte should be properly received. it write  the data byte, before ACK/NACK decision.
    ///
    /// When '1', attempt to transmit a negative acknowledgement (NACK).
    /// if the reciever FIFO is full (the received data byte cannot be written), it stretch SCL(extend SCL low phase) until the receiver FIFO changes to not full, to write the last byte, then send out NACK.
    ///
    /// When this action is performed, the hardware sets this field to '0'.
    M_NACK OFFSET(3) NUMBITS(1) [],
    /// When '1', attempt to transmit a STOP. When this action is performed, the hardware sets this field to '0'.
    ///  I2C_M_CMD.M_START has a higher priority than this command: in situations where both a STOP and a REPEATED START could be transmitted, M_START takes precedence over M_STOP.
    M_STOP OFFSET(4) NUMBITS(1) []
],
I2C_S_CMD [
    /// When '1', attempt to transmit an acknowledgement (ACK). When this action is performed, the hardware sets this field to '0'. In EZ mode, this field should be set to '0' (it is only to be used in non EZ mode).
    S_ACK OFFSET(0) NUMBITS(1) [],
    /// When '1', attempt to transmit a negative acknowledgement (NACK). When this action is performed, the hardware sets this field to '0'.  In EZ mode, this field should be set to '0' (it is only to be used in non EZ mode). This command has a higher priority than I2C_S_CMD.S_ACK, I2C_CTRL.S_READY_ADDR_ACK or I2C_CTRL.S_READY_DATA_ACK.
    S_NACK OFFSET(1) NUMBITS(1) [],
    /// When '1', attempt to send ones when TX_FIFO is empty.
    ///
    /// Once hardware starts to send ones, it will continue send ones until NACK is received, regardless of TX_FIFO status (even if new data is written into TX_FIFO).
    ///
    /// This bit is used to avoid stretching SCL, which is not expected for some master devices.
    S_TX_ONES_ON_EMPTY OFFSET(2) NUMBITS(1) [],
    /// When '1', attempt to stretch SCL at time t1, SCL falling edge after 'START, Master-code, NACK' pattern is detected.
    ///
    /// When I2C_CTRL.HS_ENABLED is set, it should be set; after wakeup from DeepSleep power mode, it should also be set.
    ///
    /// When INTR_S.I2C_HS_ENTER triggers, firmware configure clk_scb to meet I2C Hs-mode timing requirements, then firmware can clear this bit.
    S_STRETCH_HS OFFSET(8) NUMBITS(1) []
],
I2C_CFG [
    /// Trim settings for the 50ns glitch filter on the SDA input. Default setting meets the I2C glitch rejections specs. Programmability available if required
    SDA_IN_FILT_TRIM OFFSET(0) NUMBITS(2) [],
    /// Enable for 50ns glitch filter on SDA input
    /// '0': 0 ns.
    /// '1: 50 ns (filter enabled).
    SDA_IN_FILT_SEL OFFSET(4) NUMBITS(1) [],
    /// Trim settings for the 50ns glitch filter on the SDA input. Default setting meets the I2C glitch rejections specs. Programmability available if required
    SCL_IN_FILT_TRIM OFFSET(8) NUMBITS(2) [],
    /// Enable for 50ns glitch filter on SCL input
    /// '0': 0 ns.
    /// '1: 50 ns (filter enabled).
    SCL_IN_FILT_SEL OFFSET(12) NUMBITS(1) [],
    /// Trim settings for the 50ns delay filter on SDA output used to guarantee tHD_DAT I2C parameter. Default setting meets the I2C spec. Programmability available if required
    SDA_OUT_FILT0_TRIM OFFSET(16) NUMBITS(2) [],
    /// Trim settings for the 50ns delay filter on SDA output used to guarantee tHD_DAT I2C parameter. Default setting meets the I2C spec. Programmability available if required
    SDA_OUT_FILT1_TRIM OFFSET(18) NUMBITS(2) [],
    /// Trim settings for the 50ns delay filter on SDA output used to guarantee tHD_DAT I2C parameter. Default setting meets the I2C spec. Programmability available if required
    SDA_OUT_FILT2_TRIM OFFSET(20) NUMBITS(2) [],
    /// Selection of cumulative filter delay on SDA output to meet tHD_DAT parameter
    /// '0': 0 ns.
    /// '1': 50 ns (filter 0 enabled).
    /// '2': 100 ns (filters 0 and 1 enabled).
    /// '3': 150 ns (filters 0, 1 and 2 enabled).
    SDA_OUT_FILT_SEL OFFSET(28) NUMBITS(2) []
],
I2C_STRETCH_CTRL [
    /// N/A
    STRETCH_THRESHOLD OFFSET(0) NUMBITS(4) []
],
I2C_STRETCH_STATUS [
    /// N/A
    STRETCH_COUNT OFFSET(0) NUMBITS(4) [],
    /// N/A
    STRETCH_DETECTED OFFSET(4) NUMBITS(1) [],
    /// N/A
    SYNC_DETECTED OFFSET(5) NUMBITS(1) [],
    /// N/A
    STRETCHING OFFSET(8) NUMBITS(1) []
],
I2C_CTRL_HS [
    /// N/A
    HOVS_HS OFFSET(0) NUMBITS(4) [],
    /// N/A
    LOVS_HS OFFSET(4) NUMBITS(4) [],
    /// N/A
    HS_ENABLED OFFSET(31) NUMBITS(1) []
],
pub TX_CTRL [
    /// Dataframe width, depending on CTRL.MEM_WIDTH.
    /// DATA_WIDTH + 1 is the amount of bits in a transmitted data frame.
    /// This number does not include start, parity and stop bits.
    /// For UART mode, the valid range is [3, 8].
    /// For SPI, the valid range is [3, 31].
    /// For I2C the only valid value is 7.
    /// In EZ mode (for both SPI and I2C), the only valid value is 7.
    DATA_WIDTH OFFSET(0) NUMBITS(5) [],
    /// Least significant bit first ('0') or most significant bit first ('1'). For I2C, this field should be '1'.
    MSB_FIRST OFFSET(8) NUMBITS(1) [],
    /// Each IO cell 'xxx' has two associated IP output signals 'xxx_out_en' and 'xxx_out'.
    /// '0': Normal operation mode. Typically, this operation mode is used for IO cells that are connected to (board) wires/lines that are driven by a single IO cell. In this operation mode, for an IO cell 'xxx' that is used as an output, the 'xxx_out_en' output enable signal is typically constant '1' the 'xxx_out' output is the outputted value. In other words, in normal operation mode, the 'xxx_out' output is used to control the IO cell output value: 'xxx_out' is '0' to drive an IO cell output value of '0' and 'xxx_out' is '1' to drive an IO cell output value of '1'.
    /// '1': Open drain operation mode. Typically this operation mode is used for IO cells that are connected to (board) wires/lines that are driven by multiple IO cells (possibly on multiple chips). In this operation mode, for and IO cell 'xxx' that is used as an output, the 'xxx_out_en' output controls the outputted value. Typically, open drain operation mode drives low/'0' and the 'xxx_out' output is constant '1'. In other words, in open drain operation mode, the 'xxx_out_en' output is used to control the IO cell output value: in drive low/'0' mode: 'xxx_out_en' is '1' (drive enabled) to drive an IO cell output value of '0' and 'xxx_out_en' is '1' (drive disabled) to not drive an IO cell output value (another IO cell can drive the wire/line or a pull up results in a wire/line value '1').
    ///
    /// The open drain mode is supported for:
    /// - UART mode, 'uart_tx' IO cell.
    /// - SPI mode, 'spi_miso' IO cell.
    ///
    /// this bit is not applicable to I2C mode, 'i2c_scl' and 'i2c_sda' IO cells.
    OPEN_DRAIN OFFSET(16) NUMBITS(1) [],
    /// Each IO cell 'xxx' has two associated IP output signals 'xxx_out_en' and 'xxx_out'.
    /// '0': Normal operation mode. Typically, this operation mode is used for IO cells that are connected to (board) wires/lines that are driven by a single IO cell. In this operation mode, for an IO cell 'xxx' that is used as an output, the 'xxx_out_en' output enable signal is typically constant '1' the 'xxx_out' output is the outputted value. In other words, in normal operation mode, the 'xxx_out' output is used to control the IO cell output value: 'xxx_out' is '0' to drive an IO cell output value of '0' and 'xxx_out' is '1' to drive an IO cell output value of '1'.
    ///
    /// '1': Open drain operation mode. Typically this operation mode is used for IO cells that are connected to (board) wires/lines that are driven by multiple IO cells (possibly on multiple chips). In this operation mode, for and IO cell 'xxx' that is used as an output, the 'xxx_out_en' output controls the outputted value. Typically, open drain operation mode drives low/'0' and the 'xxx_out' output is constant '1'. In other words, in open drain operation mode, the 'xxx_out_en' output is used to control the IO cell output value: in drive low/'0' mode: 'xxx_out_en' is '1' (drive enabled) to drive an IO cell output value of '0' and 'xxx_out_en' is '1' (drive disabled) to not drive an IO cell output value (another IO cell can drive the wire/line or a pull up results in a wire/line value '1').
    ///
    /// this bit is applicable to I2C SCL only.
    /// I2C SDA always work in open-drain mode.
    ///
    /// this is not applicable to M0S8, which does not need special control in SCB for open-drain drive mode.
    OPEN_DRAIN_SCL OFFSET(17) NUMBITS(1) []
],
pub TX_FIFO_CTRL [
    /// Trigger level. When the transmitter FIFO has less entries than the number of this field, a transmitter trigger event INTR_TX.TRIGGER is generated.
    TRIGGER_LEVEL OFFSET(0) NUMBITS(8) [],
    /// When '1', the transmitter FIFO and transmitter shift register are cleared/invalidated. Invalidation will last for as long as this field is '1'. If a quick clear/invalidation is required, the field should be set to '1' and be followed by a set to '0'. If a clear/invalidation is required for an extended time period, the field should be set to '1' during the complete time period.
    CLEAR OFFSET(16) NUMBITS(1) [],
    /// When '1', hardware reads from the transmitter FIFO do not remove FIFO entries. Freeze will not advance the TX FIFO read pointer.
    FREEZE OFFSET(17) NUMBITS(1) []
],
pub TX_FIFO_STATUS [
    /// Amount of entries in the transmitter FIFO. The value of this field ranges from 0 to FF_DATA_NR (EZ_DATA_NR/2).
    USED OFFSET(0) NUMBITS(9) [],
    /// Indicates whether the TX shift registers holds a valid data frame ('1') or not ('0'). The shift register can be considered the top of the TX FIFO (the data frame is not included in the USED field of the TX FIFO). The shift register is a working register and holds the data frame that is currently transmitted (when the protocol state machine is transmitting a data frame) or the data frame that is transmitted next (when the protocol state machine is not transmitting a data frame).
    SR_VALID OFFSET(15) NUMBITS(1) [],
    /// FIFO read pointer: FIFO location from which a data frame is read by the hardware.
    RD_PTR OFFSET(16) NUMBITS(8) [],
    /// FIFO write pointer: FIFO location at which a new data frame is written.
    WR_PTR OFFSET(24) NUMBITS(8) []
],
pub TX_FIFO_WR [
    /// Data frame written into the transmitter FIFO. Behavior is similar to that of a PUSH operation. Note that when CTRL.MEM_WIDTH is '0', only DATA[7:0] are used and when CTRL.MEM_WIDTH is '1', only DATA[15:0] are used.
    ///
    /// A write to a full TX FIFO sets INTR_TX.OVERFLOW to '1'.
    DATA OFFSET(0) NUMBITS(32) []
],
pub RX_CTRL [
    /// Dataframe width, depending on CTRL.MEM_WIDTH.
    /// DATA_WIDTH + 1 is the expected amount of bits in received data frame.
    /// This number does not include start, parity and stop bits.
    /// For UART mode, the valid range is [3, 8].
    /// For SPI, the valid range is [3, 31].
    /// For I2C the only valid value is 7.
    /// In EZ mode (for both SPI and I2C), the only valid value is 7.
    DATA_WIDTH OFFSET(0) NUMBITS(5) [],
    /// Least significant bit first ('0') or most significant bit first ('1'). For I2C, this field should be '1'.
    MSB_FIRST OFFSET(8) NUMBITS(1) [],
    /// Median filter. When '1', a digital 3 taps median filter is performed on input interface lines. This filter should reduce the susceptibility to errors. However, its requires higher oversampling values. For UART IrDA submode, this field should always be '1'.
    MEDIAN OFFSET(9) NUMBITS(1) []
],
pub RX_FIFO_CTRL [
    /// Trigger level. When the receiver FIFO has more entries than the number of this field, a receiver trigger event INTR_RX.TRIGGER is generated.
    TRIGGER_LEVEL OFFSET(0) NUMBITS(8) [],
    /// When '1', the receiver FIFO and receiver shift register are cleared/invalidated. Invalidation will last for as long as this field is '1'. If a quick clear/invalidation is required, the field should be set to '1' and be followed by a set to '0'. If a clear/invalidation is required for an extended time period, the field should be set to '1' during the complete time period.
    CLEAR OFFSET(16) NUMBITS(1) [],
    /// When '1', hardware writes to the receiver FIFO have no effect. Freeze will not advance the RX FIFO write pointer.
    FREEZE OFFSET(17) NUMBITS(1) []
],
pub RX_FIFO_STATUS [
    /// Amount of entries in the receiver FIFO. The value of this field ranges from 0 to FF_DATA_NR  (EZ_DATA_NR/2).
    USED OFFSET(0) NUMBITS(9) [],
    /// Indicates whether the RX shift registers holds a (partial) valid data frame ('1') or not ('0'). The shift register can be considered the bottom of the RX FIFO (the data frame is not included in the USED field of the RX FIFO). The shift register is a working register and holds the data frame that is currently being received (when the protocol state machine is receiving a data frame).
    SR_VALID OFFSET(15) NUMBITS(1) [],
    /// FIFO read pointer: FIFO location from which a data frame is read.
    RD_PTR OFFSET(16) NUMBITS(8) [],
    /// FIFO write pointer: FIFO location at which a new data frame is written by the hardware.
    WR_PTR OFFSET(24) NUMBITS(8) []
],
pub RX_MATCH [
    /// N/A
    ADDR OFFSET(0) NUMBITS(8) [],
    /// Slave device address mask. This field is a mask that specifies which of the slave address bits take part in the matching. MATCH = ((ADDR & MASK) == ('slave address' & MASK)).
    MASK OFFSET(16) NUMBITS(8) []
],
pub RX_FIFO_RD [
    /// Data read from the receiver FIFO. Reading a data frame will remove the data frame from the FIFO; i.e. behavior is similar to that of a POP operation. Note that when CTRL.MEM_WIDTH is '0', only DATA[7:0] are used and when CTRL.MEM_WIDTH is '1', only DATA[15:0] are used
    ///
    /// A read from an empty RX FIFO sets INTR_RX.UNDERFLOW to '1'.
    ///
    /// When this register is read through the debugger, the data frame will not be removed from the FIFO. Similar in operation to RX_FIFO_RD_SILENT
    DATA OFFSET(0) NUMBITS(32) []
],
pub RX_FIFO_RD_SILENT [
    /// Data read from the receiver FIFO. Reading a data frame will NOT remove the data frame from the FIFO; i.e. behavior is similar to that of a PEEK operation. Note that when CTRL.MEM_WIDTH is '0', only DATA[7:0] are used and when CTRL.MEM_WIDTH is '1', only DATA[15:0] are used
    ///
    /// A read from an empty RX FIFO sets INTR_RX.UNDERFLOW to '1'.
    DATA OFFSET(0) NUMBITS(32) []
],
pub INTR_CAUSE [
    /// Master interrupt active ('interrupt_master'): INTR_M_MASKED != 0.
    M OFFSET(0) NUMBITS(1) [],
    /// Slave interrupt active ('interrupt_slave'): INTR_S_MASKED != 0.
    S OFFSET(1) NUMBITS(1) [],
    /// Transmitter interrupt active ('interrupt_tx'): INTR_TX_MASKED != 0.
    TX OFFSET(2) NUMBITS(1) [],
    /// Receiver interrupt active ('interrupt_rx'): INTR_RX_MASKED != 0.
    RX OFFSET(3) NUMBITS(1) [],
    /// Externally clock I2C interrupt active ('interrupt_i2c_ec'): INTR_I2C_EC_MASKED != 0.
    I2C_EC OFFSET(4) NUMBITS(1) [],
    /// Externally clocked SPI interrupt active ('interrupt_spi_ec'): INTR_SPI_EC_MASKED != 0.
    SPI_EC OFFSET(5) NUMBITS(1) []
],
INTR_I2C_EC [
    /// Wake up request. Active on incoming slave request (with address match).
    ///
    /// Only used when CTRL.EC_AM_MODE is '1'.
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    /// STOP detection. Activated on the end of a every transfer (I2C STOP).
    ///
    /// Only available for a slave request with an address match, in EZ and CMD_RESP modes, when CTRL.EC_OP_MODE is '1'.
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    /// STOP detection after a write transfer occurred. Activated on the end of a write transfer (I2C STOP). This event is an indication that a buffer memory location has been written to. For EZ mode: a  transfer that only writes the base address does NOT activate this event.
    ///
    /// Only available for a slave request with an address match, in EZ and CMD_RESP modes, when CTRL.EC_OP_MODE is '1'.
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    /// STOP detection after a read transfer occurred. Activated on the end of a read transfer (I2C STOP). This event is an indication that a buffer memory location has been read from.
    ///
    /// Only available for a slave request with an address match, in EZ and CMD_RESP modes, when CTRL.EC_OP_MODE is '1'.
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_I2C_EC_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
INTR_I2C_EC_MASKED [
    /// Logical and of corresponding request and mask bits.
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
pub INTR_SPI_EC [
    /// Wake up request. Active on incoming slave request when externally clocked selection is '1'.
    ///
    /// Only used when CTRL.EC_AM_MODE is '1'.
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    /// STOP detection. Activated on the end of a every transfer (SPI deselection).
    ///
    /// Only available in EZ and CMD_RESP mode and when CTRL.EC_OP_MODE is '1'.
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    /// STOP detection after a write transfer occurred. Activated on the end of a write transfer (SPI deselection). This event is an indication that a buffer memory location has been written to. For EZ mode: a  transfer that only writes the base address does NOT activate this event.
    ///
    /// Only used in EZ and CMD_RESP modes and when CTRL.EC_OP_MODE is '1'.
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    /// STOP detection after a read transfer occurred. Activated on the end of a read transfer (SPI deselection). This event is an indication that a buffer memory location has been read from.
    ///
    /// Only used in EZ and CMD_RESP modes and when CTRL.EC_OP_MODE is '1'.
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
pub INTR_SPI_EC_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
pub INTR_SPI_EC_MASKED [
    /// Logical and of corresponding request and mask bits.
    WAKE_UP OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EZ_STOP OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EZ_WRITE_STOP OFFSET(2) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EZ_READ_STOP OFFSET(3) NUMBITS(1) []
],
pub INTR_M [
    /// I2C master lost arbitration: the value driven by the master on the SDA line is not the same as the value observed on the SDA line.
    ///
    /// The Firmware should clear the TX FIFO, to re-do this transfer.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// I2C master negative acknowledgement. Set to '1', when the master receives a NACK (typically after the master transmitted the slave address or TX data).
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// I2C master acknowledgement. Set to '1', when the master receives a ACK (typically after the master transmitted the slave address or TX data).
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// I2C master STOP. Set to '1', when the master has transmitted a STOP.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// I2C master bus error (unexpected detection of START or STOP condition).
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// SPI master transfer done event: all data frames in the transmit FIFO are sent, the transmit FIFO is empty (both TX FIFO and transmit shifter register are empty), and SPI select output pin is deselected.
    SPI_DONE OFFSET(9) NUMBITS(1) [],
    /// entered I2C Hs-mode, at time t1, SCL falling edge after 'START, 8-bit master code (0000_1XXX), NACK' sequence.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// exited I2C Hs-mode, after STOP detection.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_M_SET [
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    SPI_DONE OFFSET(9) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_M_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    SPI_DONE OFFSET(9) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_M_MASKED [
    /// Logical and of corresponding request and mask bits.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    SPI_DONE OFFSET(9) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_S [
    /// I2C slave lost arbitration: the value driven on the SDA line is not the same as the value observed on the SDA line (while the SCL line is '1'). This should not occur, it represents erroneous I2C bus behavior. In case of lost arbitration, the I2C slave state machine aborts the ongoing transfer. The Firmware may decide to clear the TX and RX FIFOs in case of this error.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// N/A
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// N/A
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// N/A
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    /// N/A
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// N/A
    I2C_START OFFSET(5) NUMBITS(1) [],
    /// N/A
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    /// N/A
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    /// N/A
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// N/A
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    /// N/A
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    /// N/A
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) [],
    /// N/A
    I2C_RESTART OFFSET(16) NUMBITS(1) [],
    /// N/A
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// N/A
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_S_SET [
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_START OFFSET(5) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_RESTART OFFSET(16) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_S_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_START OFFSET(5) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_RESTART OFFSET(16) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_S_MASKED [
    /// Logical and of corresponding request and mask bits.
    I2C_ARB_LOST OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_NACK OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_ACK OFFSET(2) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_WRITE_STOP OFFSET(3) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_STOP OFFSET(4) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_START OFFSET(5) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_ADDR_MATCH OFFSET(6) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_GENERAL OFFSET(7) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_BUS_ERROR OFFSET(8) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    SPI_EZ_WRITE_STOP OFFSET(9) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    SPI_EZ_STOP OFFSET(10) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    SPI_BUS_ERROR OFFSET(11) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_RESTART OFFSET(16) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_HS_ENTER OFFSET(24) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    I2C_HS_EXIT OFFSET(25) NUMBITS(1) []
],
pub INTR_TX [
    /// N/A
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// N/A
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    /// N/A
    EMPTY OFFSET(4) NUMBITS(1) [],
    /// N/A
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Attempt to read from an empty TX FIFO. This happens when SCB is ready to transfer data and EMPTY is '1'.
    ///
    /// Only used in FIFO mode.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// SW cannot get access to the EZ memory (EZ data access), due to an externally clocked EZ access. This may happen when STATUS.EC_BUSY is '1'.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// N/A
    UART_NACK OFFSET(8) NUMBITS(1) [],
    /// N/A
    UART_DONE OFFSET(9) NUMBITS(1) [],
    /// N/A
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
pub INTR_TX_SET [
    /// Write with '1' to set corresponding bit in interrupt request register.
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    EMPTY OFFSET(4) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    UART_NACK OFFSET(8) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    UART_DONE OFFSET(9) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt request register.
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
pub INTR_TX_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    EMPTY OFFSET(4) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    UART_NACK OFFSET(8) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    UART_DONE OFFSET(9) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
pub INTR_TX_MASKED [
    /// Logical and of corresponding request and mask bits.
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    NOT_FULL OFFSET(1) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    EMPTY OFFSET(4) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    UART_NACK OFFSET(8) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    UART_DONE OFFSET(9) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    UART_ARB_LOST OFFSET(10) NUMBITS(1) []
],
pub INTR_RX [
    /// N/A
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// N/A
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    /// N/A
    FULL OFFSET(3) NUMBITS(1) [],
    /// N/A
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// N/A
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// SW cannot get access to the EZ memory (EZ_DATA accesses), due to an externally clocked EZ access. This may happen when STATUS.EC_BUSY is '1'.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// N/A
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    /// N/A
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    /// N/A
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    /// N/A
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
],
pub INTR_RX_SET [
    /// Write with '1' to set corresponding bit in interrupt request register.
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    FULL OFFSET(3) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    /// Write with '1' to set corresponding bit in interrupt status register.
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
],
pub INTR_RX_MASK [
    /// Mask bit for corresponding bit in interrupt request register.
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    FULL OFFSET(3) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    /// Mask bit for corresponding bit in interrupt request register.
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
],
pub INTR_RX_MASKED [
    /// Logical and of corresponding request and mask bits.
    TRIGGER OFFSET(0) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    NOT_EMPTY OFFSET(2) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    FULL OFFSET(3) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    OVERFLOW OFFSET(5) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    UNDERFLOW OFFSET(6) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    BLOCKED OFFSET(7) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    FRAME_ERROR OFFSET(8) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    PARITY_ERROR OFFSET(9) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    BAUD_DETECT OFFSET(10) NUMBITS(1) [],
    /// Logical and of corresponding request and mask bits.
    BREAK_DETECT OFFSET(11) NUMBITS(1) []
]
];
pub const SCB3_BASE: StaticRef<ScbRegisters> =
    unsafe { StaticRef::new(0x42860000 as *const ScbRegisters) };
