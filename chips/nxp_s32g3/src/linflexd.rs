// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! LINFlexD UART driver for NXP S32G3.
//!
//! Register definitions and bitfields are taken from S32G3 RM §49.5.
//!
//! Only UART mode is implemented (not LIN). The hardware supports 8N1, 9-bit,
//! 16-bit, and 17-bit frames with configurable parity and stop bits. This
//! driver uses buffer mode (not FIFO) with interrupt-driven TX/RX.

use core::cell::Cell;

use kernel::{
    deferred_call::{DeferredCall, DeferredCallClient},
    hil::uart::{self, Configure, Parity, Receive, StopBits, Transmit, Width},
    utilities::{
        cells::{OptionalCell, TakeCell},
        registers::{
            interfaces::{ReadWriteable, Readable, Writeable},
            register_bitfields, register_structs, ReadWrite,
        },
        StaticRef,
    },
    ErrorCode,
};

/// LINFlexD_0
pub const LF0_BASE: StaticRef<LinFlexDRegisters> =
    unsafe { StaticRef::new(0x401C_8000 as *const LinFlexDRegisters) };

/// LINFlexD_1
pub const LF1_BASE: StaticRef<LinFlexDRegisters> =
    unsafe { StaticRef::new(0x401C_C000 as *const LinFlexDRegisters) };

// See RM §49.5.1 for the full register map.
register_structs! {
    pub LinFlexDRegisters {
        /// LIN Control Register 1: provides control bits to configure LINFlexD features
        (0x000 => pub lincr1:  ReadWrite<u32, LINCR1::Register>),
        /// LIN Interrupt Enable Register: controls enabling of interrupts
        (0x004 => pub linier:  ReadWrite<u32, LINIER::Register>),
        /// LIN Status Register: indicates the current state of the LINFlexD module
        (0x008 => pub linsr:   ReadWrite<u32, LINSR::Register>),
        /// LIN Error Status Register: indicates various LIN error conditions
        (0x00C => pub linesr:  ReadWrite<u32, LINESR::Register>),
        /// UART Mode Control Register: provides control bits to configure UART features
        (0x010 => pub uartcr:  ReadWrite<u32, UARTCR::Register>),
        /// UART Mode Status Register: indicates the status and errors of UART operations
        (0x014 => pub uartsr:  ReadWrite<u32, UARTSR::Register>),
        /// LIN Time-Out Control Status Register: contains control and status bits for timeout
        (0x018 => pub lintcsr: ReadWrite<u32, LINTCSR::Register>),
        /// LIN Output Compare Register: contains the value to be compared with the timeout counter
        (0x01C => pub linocr:  ReadWrite<u32, LINOCR::Register>),
        /// LIN Time-Out Control Register: contains the header and response timeout durations
        (0x020 => pub lintocr: ReadWrite<u32, LINTOCR::Register>),
        /// LIN Fractional Baud Rate Register: decides the fractional part of the baud rate
        (0x024 => pub linfbrr: ReadWrite<u32, LINFBRR::Register>),
        /// LIN Integer Baud Rate Register: decides the integer part of the baud rate
        (0x028 => pub linibrr: ReadWrite<u32, LINIBRR::Register>),
        /// LIN Checksum Field Register: consists of checksum bits calculated or programmed
        (0x02C => pub lincfr:  ReadWrite<u32, LINCFR::Register>),
        (0x030 => pub lincr2:  ReadWrite<u32, LINCR2::Register>),
        /// Buffer Identifier Register: provides information about the transaction identifier
        (0x034 => pub bidr:    ReadWrite<u32, BIDR::Register>),
        /// Buffer Data Register Least Significant: parts DATA0 to DATA3 of the 8-byte buffer
        (0x038 => pub bdrl:    ReadWrite<u32, BDRL::Register>),
        /// Buffer Data Register Most Significant: parts DATA4 to DATA7 of the 8-byte buffer
        (0x03C => pub bdrm:    ReadWrite<u32, BDRM::Register>),
        (0x040 => _reserved0),
        (0x044 => _reserved1),
        (0x048 => _reserved2),
        /// Global Control Register: provides global configurations for both LIN and UART modes
        (0x04C => pub gcr:     ReadWrite<u32, GCR::Register>),
        /// UART Preset Timeout Register: contains the preset value of the timeout in UART mode
        (0x050 => pub uartpto:  ReadWrite<u32, UARTPTO::Register>),
        /// UART Current Timeout Register: contains the current timeout value in UART mode
        (0x054 => pub uartcto:  ReadWrite<u32, UARTCTO::Register>),
        /// DMA Tx Enable Register: enables the DMA TX interface
        (0x058 => pub dmatxe:  ReadWrite<u32, DMATXE::Register>),
        /// DMA Rx Enable Register: enables the DMA RX interface
        (0x05C => pub dmarxe:  ReadWrite<u32, DMARXE::Register>),
        (0x060 => @END),
    }
}

register_bitfields![u32,
    // RM §49.5.2
    LINCR1 [
        /// Initialization Mode Request: write 1 to request LINFlexD to enter Initialization mode
        INIT   OFFSET(0)  NUMBITS(1) [],
        /// Sleep Mode Request: write 1 to request LINFlexD to enter Sleep mode
        SLEEP  OFFSET(1)  NUMBITS(1) [],
        /// Receiver Buffer Locked mode: lock receiver buffer against overrun
        RBLM   OFFSET(2)  NUMBITS(1) [],
        /// Slave Mode Sync Break Length: select 10-bit or 11-bit break length for slave
        SSBL   OFFSET(3)  NUMBITS(1) [],
        /// Master Mode Enable: select Master (1) or Slave (0) mode
        MME    OFFSET(4)  NUMBITS(1) [],
        /// Loop Back mode: enable (1) or disable (0) loop back test mode
        LBKM   OFFSET(5)  NUMBITS(1) [],
        /// Master Break Length: choose length of sync break generated by master
        MBL    OFFSET(8)  NUMBITS(4) [],
        /// Auto Wakeup: sleep bit cleared automatically when wakeup flag is set
        AUTOWU OFFSET(12) NUMBITS(1) [],
        /// Checksum Field Disable: no checksum field is sent in the frame
        CFD    OFFSET(14) NUMBITS(1) [],
        /// Checksum Calculation Disable: disable hardware checksum calculation
        CCD    OFFSET(15) NUMBITS(1) [],
        /// LIN State Capture Enable on Bit Error: capture state to LINSR[LINS] on bit error
        NLSE   OFFSET(16) NUMBITS(1) []
    ],

    // RM §49.5.3
    LINIER [
        /// Header Received Interrupt Enable: generate interrupt when header reception complete (HRF set)
        HRIE  OFFSET(0)  NUMBITS(1) [],
        /// Data Transmitted Interrupt Enable: generate interrupt when data transmission complete (DTF set)
        DTIE  OFFSET(1)  NUMBITS(1) [],
        /// Data Reception Complete Interrupt Enable: generate interrupt when data reception complete (DRF set)
        DRIE  OFFSET(2)  NUMBITS(1) [],
        /// Timeout Interrupt Enable: generate interrupt when timeout occurs in UART mode (TO set)
        TOIE  OFFSET(3)  NUMBITS(1) [],
        /// Wakeup Interrupt Enable: generate interrupt when wakeup flag is set (WUF set)
        WUIE  OFFSET(5)  NUMBITS(1) [],
        /// LIN State Interrupt Enable: generate interrupt when entering specific LIN states (for debugging)
        LSIE  OFFSET(6)  NUMBITS(1) [],
        /// Buffer Overrun Error Interrupt Enable: generate interrupt when buffer overrun occurs (BOF set)
        BOIE  OFFSET(7)  NUMBITS(1) [],
        /// Frame Error Interrupt Enable: generate interrupt when framing error occurs (FEF set)
        FEIE  OFFSET(8)  NUMBITS(1) [],
        /// Header Error Interrupt Enable: generate interrupt when sync field/delimiter/parity error occurs (LINESR flags set)
        HEIE  OFFSET(11) NUMBITS(1) [],
        /// Checksum Error Interrupt Enable: generate interrupt when checksum error occurs (CEF set)
        CEIE  OFFSET(12) NUMBITS(1) [],
        /// Bit Error Interrupt Enable: generate interrupt when bit error occurs (BEF set)
        BEIE  OFFSET(13) NUMBITS(1) [],
        /// Output Compare Interrupt Enable: generate interrupt when counter matches compare value (OCF set)
        OCIE  OFFSET(14) NUMBITS(1) [],
        /// Stuck at Zero Interrupt Enable: generate interrupt when stuck at zero timeout occurs (SZF set)
        SZIE  OFFSET(15) NUMBITS(1) []
    ],

    // RM §49.5.4
    LINSR [
        /// Header Received Flag: set when header reception is completed
        HRF   OFFSET(0)  NUMBITS(1) [],
        /// Data Transmission Completed Flag: set when data transmission is completed
        DTF   OFFSET(1)  NUMBITS(1) [],
        /// Data Reception Completed Flag: set when data reception is completed
        DRF   OFFSET(2)  NUMBITS(1) [],
        /// Wakeup Flag: set by hardware when a falling edge is detected on Rx pin in sleep
        WUF   OFFSET(5)  NUMBITS(1) [],
        /// Receiver Data Input: reflects the current logical value of Rx pin
        RDI   OFFSET(6)  NUMBITS(1) [],
        /// Receiver Busy: indicates that a reception is ongoing in slave mode
        RXBUSY OFFSET(7) NUMBITS(1) [],
        /// Data Reception Buffer Not Empty: set when first response byte is stored in BDRL
        DRBNE OFFSET(8)  NUMBITS(1) [],
        /// Release Message Buffer: release message buffer and indicates data ready for software
        RMB   OFFSET(9)  NUMBITS(1) [],
        /// LIN State: indicates current state of LINFlexD internal state machine
        LINS  OFFSET(12) NUMBITS(4) [
            // LINFlexD state machine states — LINSR[LINS] field values (RM §49.5.4).
            SLEEP = 0b0000,
            INIT  = 0b0001,
            IDLE  = 0b0010
        ],
        /// Receive Data Byte Count: contains the number of bytes currently in RX buffer
        RDC   OFFSET(16) NUMBITS(3) []
    ],
    // RM §49.5.5
    LINESR [
        /// Noise Flag: set when noise is detected in the received character
        NF    OFFSET(0)  NUMBITS(1) [],
        /// Buffer Overrun Flag: set when new byte received and RMB is not cleared
        BOF   OFFSET(7)  NUMBITS(1) [],
        /// Framing Error Flag: set when invalid stop bit is detected
        FEF   OFFSET(8)  NUMBITS(1) [],
        /// ID Parity Error Flag: set when parity error in received identifier occurs
        IDPEF OFFSET(9) NUMBITS(1) [],
        /// Sync Delimiter Error Flag: set when received sync delimiter is less than 1 bit time
        SDEF  OFFSET(10) NUMBITS(1) [],
        /// Sync Field Error Flag: set when received sync field byte is inconsistent
        SFEF  OFFSET(11) NUMBITS(1) [],
        /// Checksum Error Flag: set when received checksum does not match hardware calculation
        CEF   OFFSET(12) NUMBITS(1) [],
        /// Bit Error Flag: set when transmitted bit differs from monitored bit on bus
        BEF   OFFSET(13) NUMBITS(1) [],
        /// Output Compare Flag: set when timeout counter matches compare register value
        OCF   OFFSET(14) NUMBITS(1) [],
        /// Stuck At Zero Flag: set when dominant level persists for 100 bit times
        SZF   OFFSET(15) NUMBITS(1) []
    ],

    // RM §49.5.6
    UARTCR [
        /// UART Mode: select UART mode (1) or LIN mode (0)
        UART   OFFSET(0)  NUMBITS(1) [],
        /// Word Length 0: works with WL1 to configure word length
        WL0    OFFSET(1)  NUMBITS(1) [],
        /// Parity Control Enable: enable parity transmission and check
        PCE    OFFSET(2)  NUMBITS(1) [],
        /// Parity Control 0: works with PC1 to configure parity type (even/odd/0/1)
        PC0    OFFSET(3)  NUMBITS(1) [],
        /// Transmitter Enable: enables the transmitter
        TxEn   OFFSET(4)  NUMBITS(1) [],
        /// Receiver Enable: enables the receiver
        RxEn   OFFSET(5)  NUMBITS(1) [],
        /// Parity Control 1: works with PC0 to configure parity type
        PC1    OFFSET(6)  NUMBITS(1) [],
        /// Word Length 1: works with WL0 to configure word length
        WL1    OFFSET(7)  NUMBITS(1) [],
        /// Tx FIFO/Buffer Mode: select FIFO mode (1) or Buffer mode (0) for transmitter
        TFBM   OFFSET(8)  NUMBITS(1) [],
        /// Rx FIFO/Buffer Mode: select FIFO mode (1) or Buffer mode (0) for receiver
        RFBM   OFFSET(9)  NUMBITS(1) [],
        /// Transmitter Data Field Length/TX FIFO Counter: number of bytes to transmit in buffer mode
        TDFL   OFFSET(13) NUMBITS(3) [],
        /// Reception Data Field Length/RX FIFO Counter: number of bytes to receive in buffer mode
        RDFL   OFFSET(10) NUMBITS(3) [],
        /// Stop Bits in UART Reception Mode: configure expected stop bits (1, 2, or 3)
        SBUR   OFFSET(17) NUMBITS(2) [],
        /// Disable Timeout in UART mode: disable/reset timeout timer depending on frame count
        DTU_PCETX OFFSET(19) NUMBITS(1) [],
        /// Number of expected frames: configures number of expected frames in UART reception
        NEF    OFFSET(20) NUMBITS(3) [],
        /// Reduced Over Sampling Enable: enables user-programmable reduced oversampling
        ROSE   OFFSET(23) NUMBITS(1) [],
        /// Over Sampling Rate: configures number of samples taken per bit when ROSE is enabled
        OSR    OFFSET(24) NUMBITS(4) [],
        /// Configurable Sample Point: decides sample point during reduced oversampling
        CSP    OFFSET(28) NUMBITS(3) [],
        /// Monitor Idle State: controls what UARTCTO monitors (idle line vs received bits)
        MIS    OFFSET(31) NUMBITS(1) []
    ],

    // RM §49.5.7
    UARTSR [
        /// Noise Flag: set when noise is detected in received character (same as LINESR[NF])
        NF      OFFSET(0)  NUMBITS(1) [],
        /// Data Transmission Completed / TX FIFO Full: indicates Tx completion or FIFO status
        DTFTFF  OFFSET(1)  NUMBITS(1) [],
        /// Data Reception Completed / RX FIFO Empty: indicates Rx completion or FIFO status
        DRFRFE  OFFSET(2)  NUMBITS(1) [],
        /// Timeout: set when a UART timeout occurs
        TO      OFFSET(3)  NUMBITS(1) [],
        /// Receive FIFO Not Empty: set when at least one byte is in Rx FIFO (FIFO mode only)
        RFNE    OFFSET(4)  NUMBITS(1) [],
        /// Wakeup Flag: set on falling edge of RX pin in sleep mode
        WUF     OFFSET(5)  NUMBITS(1) [],
        /// Receiver Data Input: reflects the current value of the Rx pin
        RDI     OFFSET(6)  NUMBITS(1) [],
        /// Buffer Overrun Flag: set when receiver buffer/FIFO overrun occurs
        BOF     OFFSET(7)  NUMBITS(1) [],
        /// Framing Error Flag: set when invalid stop bit is detected
        FEF     OFFSET(8)  NUMBITS(1) [],
        /// Release Message Buffer: same as LINSR[RMB], data ready for software
        RMB     OFFSET(9)  NUMBITS(1) [],
        /// Parity Error Flag: indicates which byte has parity error in UART buffer mode
        PE      OFFSET(10) NUMBITS(4) [],
        /// Output Compare Flag: set when timeout counter matches compare register value
        OCF     OFFSET(14) NUMBITS(1) [],
        /// Stuck At Zero Flag: set when RX pin is dominant for 100 bit times
        SZF     OFFSET(15) NUMBITS(1) []
    ],

    // RM §49.5.8
    LINTCSR [
        /// Counter Value: reflects the current value of the timeout counter
        CNT   OFFSET(0)  NUMBITS(8) [],
        /// Time-out counter enable: enables the timeout counter
        TOCE  OFFSET(8)  NUMBITS(1) [],
        /// Idle on timeout: reset LIN state machine to Idle on timeout event (MODE=0 only)
        IOT   OFFSET(9)  NUMBITS(1) [],
        /// Time-out counter mode: select output compare mode (1) or LIN mode (0)
        MODE  OFFSET(10) NUMBITS(1) []
    ],

    // RM §49.5.9
    LINOCR [
        /// Output compare value 1: compare value for slave mode match
        OC1 OFFSET(0) NUMBITS(8) [],
        /// Output compare value 2: compare value for master mode match
        OC2 OFFSET(8) NUMBITS(8) []
    ],

    // RM §49.5.10
    LINTOCR [
        /// Header timeout value: header timeout duration in bit times (slave mode only)
        HTO OFFSET(0) NUMBITS(7) [],
        /// Response timeout value: response timeout duration in bit times per byte
        RTO OFFSET(8) NUMBITS(4) []
    ],

    // RM §49.5.11
    LINFBRR [
        /// Fractional Baud rates: decides the fractional part of the baud rate divisor
        FBR OFFSET(0) NUMBITS(4) []
    ],

    // RM §49.5.12
    LINIBRR [
        /// Integer Baud rates: decides the integer part of the baud rate divisor
        IBR OFFSET(0) NUMBITS(20) []
    ],

    // RM §49.5.13
    LINCFR [
        /// Checksum bits: read-only calculated checksum, or read/write programmed checksum
        CF OFFSET(0) NUMBITS(8) []
    ],

    // RM §49.5.14
    LINCR2 [
        /// Header Transmission Request: request transmission of the LIN header
        HTRQ  OFFSET(8)  NUMBITS(1) [],
        /// Abort Request: request abort of current transmission
        ABRQ  OFFSET(9)  NUMBITS(1) [],
        /// Data Transmission Request: request transmission of the LIN data field
        DTRQ  OFFSET(10) NUMBITS(1) [],
        /// Data Discard request: discard incoming response data and move to Idle
        DDRQ  OFFSET(11) NUMBITS(1) [],
        /// Wakeup Generate Request: generate a wakeup pulse on the LIN bus
        WURQ  OFFSET(12) NUMBITS(1) [],
        /// Idle on Identifier Parity Error: reset LIN state machine on identifier parity error
        IOPE  OFFSET(13) NUMBITS(1) [],
        /// Idle on Bit Error: reset LIN state machine on bit error
        IOBE  OFFSET(14) NUMBITS(1) [],
        /// Two Bit delimiter bit: select break delimiter length of 2 bits (1) or 1 bit (0)
        TBDE  OFFSET(15) NUMBITS(1) []
    ],

    // RM §49.5.15
    BIDR [
        /// Identifier: identifier part of the identifier field without parity
        ID  OFFSET(0)  NUMBITS(6) [],
        /// Classic Checksum: select Classic Checksum (1) or Enhanced Checksum (0)
        CCS OFFSET(8)  NUMBITS(1) [],
        /// Direction: select Transmit from buffer (1) or Receive into buffer (0)
        DIR OFFSET(9)  NUMBITS(1) [],
        /// Data Field Length: number of bytes in data field minus 1
        DFL OFFSET(10) NUMBITS(3) []
    ],

    // RM §49.5.16
    BDRL [
        /// Data Byte 0: data byte 0 of the data field
        DATA0 OFFSET(0)  NUMBITS(8) [],
        /// Data Byte 1: data byte 1 of the data field
        DATA1 OFFSET(8)  NUMBITS(8) [],
        /// Data Byte 2: data byte 2 of the data field
        DATA2 OFFSET(16) NUMBITS(8) [],
        /// Data Byte 3: data byte 3 of the data field
        DATA3 OFFSET(24) NUMBITS(8) []
    ],

    // RM §49.5.17
    BDRM [
        /// Data Byte 4: data byte 4 of the data field
        DATA4 OFFSET(0)  NUMBITS(8) [],
        /// Data Byte 5: data byte 5 of the data field
        DATA5 OFFSET(8)  NUMBITS(8) [],
        /// Data Byte 6: data byte 6 of the data field
        DATA6 OFFSET(16) NUMBITS(8) [],
        /// Data Byte 7: data byte 7 of the data field
        DATA7 OFFSET(24) NUMBITS(8) []
    ],

    // RM §49.5.18
    GCR [
        /// Soft reset: executes a soft reset of the LINFlexD controller FSMs/FIFOs/registers
        SR    OFFSET(0)  NUMBITS(1) [],
        /// STOP mode: request to put LINFlexD in a low power, power-down state
        STOP  OFFSET(1)  NUMBITS(1) [],
        /// Received data level inversion selection: enables inversion of received payload data
        RDLIS OFFSET(2)  NUMBITS(1) [],
        /// Transmit data level inversion selection: enables inversion of transmitted payload data
        TDLIS OFFSET(3)  NUMBITS(1) [],
        /// Received data first bit MSB: configure first bit of received payload as MSB (1) or LSB (0)
        RDFBM OFFSET(4)  NUMBITS(1) [],
        /// Transmit data first bit MSB: configure first bit of transmitted payload as MSB (1) or LSB (0)
        TDFBM OFFSET(5)  NUMBITS(1) []
    ],

    // RM §49.5.19
    UARTPTO [
        /// Preset Timeout: preset value of the UART timeout counter
        PTO OFFSET(0) NUMBITS(12) []
    ],

    // RM §49.5.20
    UARTCTO [
        /// Current Timeout: read-only current value of the UART timeout counter
        CTO OFFSET(0) NUMBITS(12) []
    ],

    // RM §49.5.21
    DMATXE [
        /// DMA Tx channel enable: enables the DMA Tx channel
        DTE0 OFFSET(0) NUMBITS(1) []
    ],

    // RM §49.5.22
    DMARXE [
        /// DMA Rx channel enable: enables the DMA Rx channel
        DRE0 OFFSET(0) NUMBITS(1) []
    ]
];

// Maximum iterations for spin-wait loops (e.g. waiting for INIT mode, TX drain).
// Units: bare loop iterations (register read + compare + branch).
// At the pre-PLL M7 clock (FIRC = 48 MHz) each MMIO read takes ~10 cycles,
// so 200 000 iterations ≈ 40 ms — far above the hardware's sub-microsecond
// state-machine transition time.  Exceeding this limit returns `false`; the
// caller should propagate an error rather than silently continuing.
const HW_POLL_MAX: u32 = 200_000;

// ---------------------------------------------------------------------------
// Deferred-call state for TX/RX abort notifications
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq)]
enum TxState {
    Idle,
    Transmitting,
    Aborted(Result<(), ErrorCode>),
}

#[derive(Copy, Clone, PartialEq)]
enum RxState {
    Idle,
    Receiving,
    Aborted(Result<(), ErrorCode>, uart::Error),
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// LINFlexD UART instance bound to an MMIO base address.
///
/// Construct with [`LinFlexD::new`] and place the instance in
/// `static` storage via `static_init!`.
pub struct LinFlexD<'a> {
    registers: StaticRef<LinFlexDRegisters>,
    tx_client: OptionalCell<&'a dyn uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn uart::ReceiveClient>,
    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>,
    tx_state: Cell<TxState>,
    rx_state: Cell<RxState>,
    deferred_call: DeferredCall,
    /// LIN_BAUD_CLK input frequency in Hz used to compute IBR/FBR.
    /// Defaults to FIRC (48 MHz) — caller must update via
    /// [`set_input_clock_hz`] after switching MC_CGM_0 mux 8.
    input_clock_hz: Cell<u32>,
}

impl LinFlexD<'_> {
    // ---------------------------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------------------------

    /// Create a new `LinFlexD` bound to `base`.  The instance is inert until
    /// [`configure`](Hil::uart::Configure::configure) succeeds.
    pub fn new(base: StaticRef<LinFlexDRegisters>) -> Self {
        Self {
            registers: base,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
            tx_state: Cell::new(TxState::Idle),
            rx_state: Cell::new(RxState::Idle),
            deferred_call: DeferredCall::new(),
            input_clock_hz: Cell::new(48_000_000),
        }
    }

    /// Create the LF0 instance.
    pub fn new_lf0() -> Self {
        Self::new(LF0_BASE)
    }

    /// Create the LF1 instance.
    pub fn new_lf1() -> Self {
        Self::new(LF1_BASE)
    }

    /// Update the LIN_BAUD_CLK input frequency used to compute UART baud
    /// divisors. Call **before** [`Configure::configure`] when the underlying
    /// MC_CGM_0 mux 8 source has been switched away from FIRC (the
    /// power-on default this driver assumes).
    pub fn set_input_clock_hz(&self, hz: u32) {
        self.input_clock_hz.set(hz);
    }

    // ---------------------------------------------------------------------------
    // Private helpers
    // ---------------------------------------------------------------------------

    /// Busy-wait until LINSR[LINS] == INIT mode.
    /// RM does not talk about how long it takes to enter init mode after setting LINCR1[INIT]
    /// NXP driver does the poll. We assume it is needed to ensure state machine is correctly reset before we proceed with configuration.
    /// Kept inline because it is a short loop body called once per configure.
    #[inline]
    fn wait_for_init_mode(&self) -> bool {
        let regs = self.registers;
        for _ in 0..HW_POLL_MAX {
            if regs.linsr.read(LINSR::LINS) == LINSR::LINS::Value::INIT as u32 {
                return true;
            }
        }
        false
    }

    fn disable_tx_interrupt(&self) {
        self.registers.linier.modify(LINIER::DTIE::CLEAR);
    }

    fn disable_rx_interrupt(&self) {
        self.registers
            .linier
            .modify(LINIER::DRIE::CLEAR + LINIER::TOIE::CLEAR + LINIER::BOIE::CLEAR);
    }

    // ---------------------------------------------------------------------------
    // Interrupt-driven TX progress
    // ---------------------------------------------------------------------------

    fn tx_progress(&self) {
        let regs = self.registers;
        let idx = self.tx_index.get();
        let len = self.tx_len.get();
        if idx >= len {
            return;
        }
        self.tx_buffer.map(|tx_buf| {
            regs.bdrl.write(BDRL::DATA0.val(tx_buf[idx] as u32));
        });
        self.tx_index.set(idx + 1);
    }

    fn is_transmitting(&self) -> bool {
        self.tx_state.get() == TxState::Transmitting
    }

    /// Called by the platform interrupt handler.
    ///
    /// Clears relevant status flags and drives TX/RX progress.
    // TODO: implement DMA support
    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let uartsr = regs.uartsr.extract();
        let linier = regs.linier.extract();
        // ---- TX completion ---------------------------------------------------
        if uartsr.is_set(UARTSR::DTFTFF) && linier.is_set(LINIER::DTIE) {
            // Clear the flag (W1C).
            regs.uartsr.write(UARTSR::DTFTFF::SET);
            self.disable_tx_interrupt();

            if self.tx_index.get() >= self.tx_len.get() {
                self.tx_state.set(TxState::Idle);
                // All bytes transmitted; notify the client.
                self.tx_client.map(|client| {
                    if self
                        .tx_buffer
                        .take()
                        .map(|buf| {
                            client.transmitted_buffer(buf, self.tx_len.get(), Ok(()));
                        })
                        .is_none()
                    {
                        // if no buffer was taken, means we were sending a single word, so notify with transmitted_word callback
                        client.transmitted_word(Ok(()));
                    }
                });
            } else {
                // More data to send.
                self.tx_progress();
                // Re-enable the TX interrupt for the next DTF.
                regs.linier.modify(LINIER::DTIE::SET);
            }
        }

        // ---- RX completion / timeout ----------------------------------------
        if uartsr.is_set(UARTSR::DRFRFE) {
            // DRF means the programmed number of bytes (RDFL+1) arrived.
            // TO means the timeout counter expired; the last byte is still
            // readable from BDRL.
            if linier.is_set(LINIER::DRIE) {
                self.disable_rx_interrupt();
                self.rx_state.set(RxState::Idle);

                // Copy received bytes into the client buffer. In UART buffer
                // mode the receive buffer is BDRM (RM §49.4.4.7 Table 331:
                // "Read Byte4-5-6-7 / BUFFER → OK"); bytes 0..3 are BDRM
                // DATA4..DATA7. BDRL is the TX buffer and must not be read here.
                let idx = self.rx_index.get();
                let len = self.rx_len.get();

                if idx < len {
                    self.rx_buffer.map(|rx_buf| {
                        let written = read_rx_bytes(regs.bdrm.get(), &mut rx_buf[idx..len]);
                        self.rx_index.set(idx + written);
                    });

                    // Release the message buffer so hardware can accept more data.
                    regs.uartsr.write(UARTSR::RMB::SET);
                }

                if self.rx_index.get() >= self.rx_len.get() {
                    // Complete; notify the client.
                    self.rx_client.map(|client| {
                        self.rx_buffer.take().map(|buf| {
                            client.received_buffer(
                                buf,
                                self.rx_len.get(),
                                Ok(()),
                                uart::Error::None,
                            );
                        });
                    });
                } else {
                    // More bytes expected; restart reception.
                    regs.uartsr.write(UARTSR::RMB::SET);
                    regs.linier
                        .modify(LINIER::DRIE::SET + LINIER::TOIE::SET + LINIER::BOIE::SET);
                }
            }
        }

        // ---- Timeout --------------------------------------------------------
        if uartsr.is_set(UARTSR::TO) && linier.is_set(LINIER::TOIE) {
            // Timeout fires when the inter-byte gap exceeds UARTPTO.
            // In most cases this indicates the sender is done.
            regs.uartsr.write(UARTSR::TO::SET);
            // Deliver whatever we have accumulated.
            let rx_len = self.rx_index.get();
            if rx_len > 0 {
                self.disable_rx_interrupt();
                self.rx_state.set(RxState::Idle);
                self.rx_client.map(|client| {
                    self.rx_buffer.take().map(|buf| {
                        client.received_buffer(buf, rx_len, Ok(()), uart::Error::None);
                    });
                });
            }
        }

        // ---- Buffer overrun --------------------------------------------------
        if uartsr.is_set(UARTSR::BOF) && linier.is_set(LINIER::BOIE) {
            regs.uartsr.write(UARTSR::BOF::SET);
            self.rx_state.set(RxState::Idle);
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|buf| {
                    client.received_buffer(
                        buf,
                        self.rx_index.get(),
                        Ok(()),
                        uart::Error::OverrunError,
                    );
                });
            });
        }
    }

    // ---------------------------------------------------------------------------
    // Early / Panic: polling helpers
    //
    // These can be used for early bring-up before interrupts are configured
    // Those are also used for panic handler.
    // ---------------------------------------------------------------------------

    /// Transmit one byte, polling for completion.
    ///
    /// Returns `true` on success, `false` if the byte was lost (poll budget
    /// exhausted or hardware not configured).
    pub fn putc(&self, byte: u8) -> bool {
        putc_poll(&self.registers, byte)
    }

    /// Transmit all bytes in `s`.
    pub fn puts(&self, s: &str) {
        self.transmit_sync(s.as_bytes())
    }

    /// Transmit all bytes in `bytes` synchronously, spinning until each byte drains.
    ///
    /// # PANIC-PATH ONLY
    /// Busy-waits per byte (bounded by `HW_POLL_MAX`; WCET ≈ 40 ms × `bytes.len()`
    /// at 48 MHz FIRC).  **Only call from the panic handler or other contexts where
    /// the kernel scheduler is permanently stopped.  Never call at runtime.**
    pub fn transmit_sync(&self, bytes: &[u8]) {
        for b in bytes.iter() {
            self.putc(*b);
        }
    }
}

// ---------------------------------------------------------------------------
// Shared polling TX primitive
// ---------------------------------------------------------------------------

/// Transmit one byte over the given LINFlexD instance by polling UARTSR[DTFTFF].
///
/// Returns `true` on success, `false` if the poll budget expired (byte lost).
/// Used by `LinFlexD::putc` for synchronous polling output.
#[inline(never)]
fn putc_poll(regs: &LinFlexDRegisters, byte: u8) -> bool {
    // Acknowledge any stale DTF flag before writing.
    regs.uartsr.write(UARTSR::DTFTFF::SET);

    // Writing DATA0 triggers TX immediately.
    regs.bdrl.write(BDRL::DATA0.val(byte as u32));

    // Wait for the hardware to signal TX completion.
    for _ in 0..HW_POLL_MAX {
        if regs.uartsr.is_set(UARTSR::DTFTFF) {
            regs.uartsr.write(UARTSR::DTFTFF::SET);
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// DeferredCallClient — deferred TX/RX abort notifications
// ---------------------------------------------------------------------------

impl DeferredCallClient for LinFlexD<'_> {
    fn register(&'static self) {
        self.deferred_call.register(self);
    }

    fn handle_deferred_call(&self) {
        if let TxState::Aborted(rcode) = self.tx_state.get() {
            self.tx_state.set(TxState::Idle);
            self.tx_client.map(|client| {
                if self
                    .tx_buffer
                    .take()
                    .map(|buf| {
                        client.transmitted_buffer(buf, self.tx_len.get(), rcode);
                    })
                    .is_none()
                {
                    client.transmitted_word(rcode);
                }
            });
        }

        if let RxState::Aborted(rcode, error) = self.rx_state.get() {
            self.rx_state.set(RxState::Idle);
            self.rx_client.map(|client| {
                self.rx_buffer.take().map(|buf| {
                    client.received_buffer(buf, self.rx_index.get(), rcode, error);
                });
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Configure trait
// ---------------------------------------------------------------------------

impl Configure for LinFlexD<'_> {
    /// Configure the LINFlexD for UART operation with the specified parameters.
    /// Note that we only support a subset of possible configurations for now:
    /// - 8 data bits
    /// - No parity
    /// - 1 stop bit
    /// - No hardware flow control
    /// - Baud rates of 115200 and 921600 (at 48 MHz FIRC)
    ///
    /// # INIT-ONLY
    /// Contains hardware spin-waits bounded by `HW_POLL_MAX` (WCET ≈ 40 ms at
    /// 48 MHz FIRC — the hardware protocol requires polling LINSR[LINS] until
    /// INIT mode is acknowledged before any register can be written).
    /// **Must only be called during board initialisation, before `kernel_loop()`.**
    /// Runtime reconfiguration is prohibited — see safety manual §UART-INIT.
    fn configure(&self, params: kernel::hil::uart::Parameters) -> Result<(), ErrorCode> {
        let regs = self.registers;

        // We only support 8N1 with no flow control for now, which covers the common use cases for console output.
        match (
            params.width,
            params.parity,
            params.stop_bits,
            params.hw_flow_control,
        ) {
            (Width::Eight, Parity::None, StopBits::One, false) => {}
            _ => return Err(ErrorCode::NOSUPPORT),
        }
        // --- Step 1: Enter INIT mode -----------------------------------------
        // Clear SLEEP, set INIT (RM §49.4.2.1.1).
        regs.lincr1
            .write(LINCR1::SLEEP::CLEAR + LINCR1::INIT::SET + LINCR1::MME::CLEAR);

        // Poll for hardware acknowledgment. (see `wait_for_init_mode` for explanation on why this is busy polling).
        if !self.wait_for_init_mode() {
            return Err(ErrorCode::BUSY);
        }

        // --- Step 2: UART mode enable ----------------------------------------
        // Must set UART bit before writing any UART-only registers (RM §49.4.2.1.1).
        regs.uartcr.write(UARTCR::UART::SET);

        // --- Step 3: Clear all status flags ----------------------------------
        // W1C registers.  UARTPTO=0 sets UARTSR[TO] immediately after reset,
        // so clear it before enabling RX.
        regs.uartsr.write(
            UARTSR::DTFTFF::SET
                + UARTSR::DRFRFE::SET
                + UARTSR::TO::SET
                + UARTSR::BOF::SET
                + UARTSR::FEF::SET
                + UARTSR::RMB::SET,
        );

        // --- Step 4: Baud rate divisors (INIT-mode only) ----------------------
        // RM §49.4.2.14:
        //   baud = LIN_CLK / (16 × LDIV)
        //   LDIV = IBR + FBR/16
        //
        // Solve in 16ths to avoid floats:
        //   ldiv16 = 16 × LDIV = round(clk_hz / baud)
        //   IBR    = ldiv16 / 16   (truncate)
        //   FBR    = ldiv16 % 16
        // FBR is 4 bits (0..15); IBR is 20 bits (1..0xFFFFF).
        let clk_hz = self.input_clock_hz.get();
        let baud = params.baud_rate;
        if baud == 0 || clk_hz == 0 {
            return Err(ErrorCode::INVAL);
        }
        let ldiv16 = (clk_hz + baud / 2) / baud;
        let ibr = ldiv16 / 16;
        let fbr = ldiv16 % 16;
        if ibr == 0 || ibr > 0xF_FFFF {
            return Err(ErrorCode::NOSUPPORT);
        }

        regs.linfbrr.write(LINFBRR::FBR.val(fbr));
        regs.linibrr.write(LINIBRR::IBR.val(ibr));

        // --- Step 5: GCR — 1 stop bit, LSB-first, no inversion, soft-reset off -
        regs.gcr.write(GCR::SR::CLEAR + GCR::STOP::CLEAR);

        // --- Step 6: UARTPTO must be non-zero --------------------------------
        // Zero immediately asserts UARTSR[TO] on some S32G3 revisions.
        // 0x80 ≈ 128 baud periods ≈ 1.1 ms at 115200.
        regs.uartpto.write(UARTPTO::PTO.val(0x80));

        // --- Step 7: Word length, parity, stop bits ---------------------------
        // RM §49.5.6 / §49.4.4:
        //   Width:  WL1=0 WL0=1 → 8 bits  (default)
        //   Parity: PCE=0 → no parity   (default)
        //   Stop:   GCR[STOP]=0 + SBUR=0 → 1 stop bit (default)
        //
        // TODO: support 9-bit, 16-bit, 17-bit word lengths when params.width is
        //       Width::Six or Width::Seven (maps to 8N1 still).
        // TODO: support 2 stop bits via SBUR when params.stop_bits == StopBits::Two.

        regs.uartcr
            .modify(UARTCR::WL1::CLEAR + UARTCR::WL0::SET + UARTCR::PCE::CLEAR);

        // --- Step 8: Buffer mode (not FIFO) ---------------------------------
        regs.uartcr
            .modify(UARTCR::TFBM::CLEAR + UARTCR::RFBM::CLEAR);

        // --- Step 9: TDFL/RDFL — single byte per frame in buffer mode ---------
        regs.uartcr
            .modify(UARTCR::TDFL.val(0) + UARTCR::RDFL.val(0));

        // --- Step 10: Enable TX and RX ----------------------------------------
        regs.uartcr.modify(UARTCR::TxEn::SET + UARTCR::RxEn::SET);

        // --- Step 11: Exit INIT → Normal mode -------------------------------
        // Keep SLEEP=0, INIT=0.  MME was already cleared above.
        regs.lincr1
            .write(LINCR1::SLEEP::CLEAR + LINCR1::INIT::CLEAR + LINCR1::MME::CLEAR);

        // --- Step 12: Disable all interrupts until TX/RX are configured -------
        regs.linier.write(
            LINIER::HRIE::CLEAR
                + LINIER::DTIE::CLEAR
                + LINIER::DRIE::CLEAR
                + LINIER::TOIE::CLEAR
                + LINIER::WUIE::CLEAR
                + LINIER::LSIE::CLEAR
                + LINIER::BOIE::CLEAR
                + LINIER::FEIE::CLEAR
                + LINIER::HEIE::CLEAR
                + LINIER::CEIE::CLEAR
                + LINIER::BEIE::CLEAR
                + LINIER::OCIE::CLEAR
                + LINIER::SZIE::CLEAR,
        );

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Transmit trait
// ---------------------------------------------------------------------------

impl<'a> Transmit<'a> for LinFlexD<'a> {
    fn set_transmit_client(&self, client: &'a dyn uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 || tx_len > tx_data.len() {
            return Err((ErrorCode::SIZE, tx_data));
        }
        if self.is_transmitting() {
            return Err((ErrorCode::BUSY, tx_data));
        }
        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_index.set(0);
        self.tx_state.set(TxState::Transmitting);

        // Enable transmitter and kick off the first batch.
        let regs = self.registers;
        regs.uartcr.modify(UARTCR::TxEn::SET);

        // Arm the TX completion interrupt.
        regs.linier.modify(LINIER::DTIE::SET);
        self.tx_progress();

        Ok(())
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        if self.is_transmitting() {
            return Err(ErrorCode::BUSY);
        }
        // ensure we send only one word
        self.tx_len.set(0);
        self.tx_index.set(0);
        let regs = self.registers;
        regs.uartcr.modify(UARTCR::TxEn::SET);
        // Arm the TX completion interrupt.
        regs.linier.modify(LINIER::DTIE::SET);
        regs.bdrl.write(BDRL::DATA0.val(word));
        self.tx_state.set(TxState::Transmitting);
        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        if self.tx_state.get() != TxState::Transmitting {
            return Ok(());
        }
        self.disable_tx_interrupt();
        self.tx_state.set(TxState::Aborted(Err(ErrorCode::CANCEL)));
        self.deferred_call.set();
        Err(ErrorCode::BUSY)
    }
}

// ---------------------------------------------------------------------------
// Receive trait
// ---------------------------------------------------------------------------

impl<'a> Receive<'a> for LinFlexD<'a> {
    fn set_receive_client(&self, client: &'a dyn uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if rx_len == 0 || rx_len > rx_buffer.len() || rx_len > 4 {
            return Err((ErrorCode::SIZE, rx_buffer));
        }
        if self.rx_state.get() == RxState::Receiving {
            return Err((ErrorCode::BUSY, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);
        self.rx_state.set(RxState::Receiving);

        // Configure RDFL to match requested byte count (max 4 in buffer mode).
        // TODO: for >4 bytes chain multiple receive_buffer calls or use FIFO mode.
        let rdfl = rx_len as u32 - 1;
        let regs = self.registers;

        // Update TDFL/RDFL while in Normal mode (allowed per RM §49.5.6).
        regs.uartcr.modify(UARTCR::RDFL.val(rdfl));

        // Clear stale flags and enable receiver.
        regs.uartsr
            .write(UARTSR::RMB::SET + UARTSR::DRFRFE::SET + UARTSR::TO::SET);
        regs.uartcr.modify(UARTCR::RxEn::SET);

        // Arm RX completion and timeout interrupts.
        regs.linier
            .modify(LINIER::DRIE::SET + LINIER::TOIE::SET + LINIER::BOIE::SET);

        Ok(())
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        // TODO: implement word receive state machine
        Err(ErrorCode::NOSUPPORT)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        if self.rx_state.get() != RxState::Receiving {
            return Ok(());
        }
        self.disable_rx_interrupt();
        self.rx_state.set(RxState::Aborted(
            Err(ErrorCode::CANCEL),
            uart::Error::Aborted,
        ));
        self.deferred_call.set();
        Err(ErrorCode::BUSY)
    }
}

// ---------------------------------------------------------------------------
// Early-boot synchronous debug output
// ---------------------------------------------------------------------------

/// Synchronous early-boot trace output via LINFLEXD_1 (LF1).
///
/// `debug!()` is interrupt/deferred-call driven and only drains once the
/// kernel main loop starts. Anything that hangs before `kernel_loop()` (PLL
/// lock waits, MC_CGM mux switches, fault handlers, …) can otherwise look like
/// a silent freeze.
///
/// Assumes LF1 is already configured. Writes bytes directly using polling TX —
/// no interrupts, no deferred calls, no constructed `LinFlexD` instance.
///
/// Usage:
///
/// ```rust,ignore
/// use nxp_s32g3::trace_sync;
/// trace_sync!("[CLK] step={} val=0x{:08x}", step, val);
/// ```
pub mod early_debug {
    use core::fmt::{self, Write};

    use kernel::utilities::registers::interfaces::{Readable, Writeable};

    use super::{BDRL, HW_POLL_MAX, UARTSR};

    struct TraceWriter;

    pub fn udelay(ms: u64) {
        // At the pre-PLL M7 clock (FIRC = 48 MHz)
        let mut count = 0;
        let target_count = ms * 48 / 3; // measured ~3 cycles per loop iteration
        while count < target_count {
            core::hint::spin_loop();
            count += 1;
        }
    }
    fn putc(byte: u8) {
        let regs = &*super::LF0_BASE;
        regs.uartsr.write(UARTSR::DTFTFF::SET);
        regs.bdrl.write(BDRL::DATA0.val(byte as u32));
        // Poll TX-complete flag. WCET ≈ 40 ms at 48 MHz FIRC (HW_POLL_MAX
        // iterations × ~3 cycles/loop). No per-iteration delay — this is the
        // panic path and the scheduler is permanently stopped.
        for _ in 0..HW_POLL_MAX {
            if regs.uartsr.is_set(UARTSR::DTFTFF) {
                regs.uartsr.write(UARTSR::DTFTFF::SET);
                break;
            }
        }
    }

    impl Write for TraceWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for &byte in s.as_bytes().iter() {
                putc(byte);
            }
            Ok(())
        }
    }

    /// Format and write synchronously. Used by the `trace_sync!` macro.
    #[doc(hidden)]
    pub fn write_fmt(args: fmt::Arguments<'_>) {
        let _ = TraceWriter.write_fmt(args);
        putc(b'\r');
        putc(b'\n');
    }
}

/// Format-and-print synchronously over LF1, bypassing the deferred debug
/// infrastructure. Intended for early-boot bring-up and fault handlers.
#[macro_export]
macro_rules! trace_sync {
    ($($arg:tt)*) => {
        $crate::linflexd::early_debug::write_fmt(format_args!($($arg)*))
    };
}

/// Extract the UART receive bytes from the BDRM register word.
///
/// In UART **buffer** mode the receive buffer is the 4-byte BDRM register:
/// RM §49.4.4.7 Table 331 lists "Read Byte4-5-6-7 / BUFFER → OK" as the only
/// valid receive reads, i.e. received bytes 0..3 map to BDRM DATA4..DATA7
/// (BDR4..BDR7). BDRL is the transmit buffer here (`putc`/`tx_progress` write
/// BDRL::DATA0), so it must NOT be used on the RX path — reading it back
/// returns stale TX data.
///
/// `bdrm` is the raw BDRM word; bytes are little-endian (DATA4 = bits 0..7).
/// Returns the number of bytes written (`dst.len()`, capped at the 4-byte
/// buffer; callers reject `rx_len > 4` in `receive_buffer`).
#[inline]
fn read_rx_bytes(bdrm: u32, dst: &mut [u8]) -> usize {
    let bytes: [u8; 4] = [
        bdrm as u8,
        (bdrm >> 8) as u8,
        (bdrm >> 16) as u8,
        (bdrm >> 24) as u8,
    ];
    let n = dst.len().min(4);
    dst[..n].copy_from_slice(&bytes[..n]);
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::deferred_call::initialize_deferred_call_state;
    use kernel::hil::uart::Receive;
    use kernel::platform::chip::ThreadIdProvider;
    enum TestThread {}
    unsafe impl ThreadIdProvider for TestThread {
        fn running_thread_id() -> usize {
            0
        }
    }
    /// Single received byte lands in BDRM::DATA4 (bits 0–7) → dst[0].
    #[test]
    fn rx_single_byte_from_bdrm_data4() {
        let mut buf = [0u8; 1];
        let bdrm = 0x0000_0042; // DATA4 = 0x42
        let n = read_rx_bytes(bdrm, &mut buf);
        assert_eq!(n, 1);
        assert_eq!(buf[0], 0x42);
    }
    /// Four received bytes map to BDRM::DATA4..DATA7 (little-endian).
    #[test]
    fn rx_four_bytes_from_bdrm() {
        let mut buf = [0u8; 4];
        let bdrm = 0x0706_0504; // DATA7=0x07, DATA6=0x06, DATA5=0x05, DATA4=0x04
        let n = read_rx_bytes(bdrm, &mut buf);
        assert_eq!(n, 4);
        assert_eq!(buf, [0x04, 0x05, 0x06, 0x07]);
    }
    /// A short destination caps the read at dst.len().
    #[test]
    fn rx_capped_by_dst_len() {
        let mut buf = [0u8; 3];
        let bdrm = 0x0706_0504;
        let n = read_rx_bytes(bdrm, &mut buf);
        assert_eq!(n, 3);
        assert_eq!(buf, [0x04, 0x05, 0x06]);
    }
    /// The 4-byte UART buffer never yields more than 4 bytes even if the
    /// destination is larger.
    #[test]
    fn rx_caps_at_four_bytes() {
        let mut buf = [0xFFu8; 6];
        let bdrm = 0x0706_0504;
        let n = read_rx_bytes(bdrm, &mut buf);
        assert_eq!(n, 4);
        assert_eq!(&buf[..4], &[0x04, 0x05, 0x06, 0x07]);
        // Tail untouched.
        assert_eq!(&buf[4..], &[0xFF, 0xFF]);
    }
    /// receive_buffer must reject rx_len > 4 with ErrorCode::SIZE rather than
    /// silently wrapping RDFL.
    #[test]
    fn receive_buffer_rejects_five_bytes() {
        initialize_deferred_call_state::<TestThread>();
        let mut backing = [0u32; 24];
        let regs = unsafe { StaticRef::new(backing.as_mut_ptr() as *const LinFlexDRegisters) };
        let lf = LinFlexD::new(regs);
        static mut BUF: [u8; 8] = [0; 8];
        let buf = unsafe { &mut BUF };
        let result = lf.receive_buffer(buf, 5);
        assert!(result.is_err());
        let (ecode, returned) = result.unwrap_err();
        assert_eq!(ecode, ErrorCode::SIZE);
        assert_eq!(returned.len(), 8);
    }
}
