// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! LINFlexD UART driver for NXP S32G3.
//!
//! Register definitions and bitfields are taken from S32G3 RM §49.5.
//!
//! Only UART mode is implemented (not LIN). The hardware supports 8N1, 9-bit,
//! 16-bit, and 17-bit frames with configurable parity and stop bits. This
//! driver uses buffer mode (not FIFO) with interrupt-driven TX/RX.

use core::cell::Cell;

use kernel::{
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

/// LINFlexD_
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
        LINS  OFFSET(12) NUMBITS(4) [],
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
/// LINFlexD state machine states — LINSR[LINS] field values (RM §49.5.4).
mod lins {
    #[allow(dead_code)]
    pub const SLEEP: u32 = 0b0000; // not currently used but documented for completeness
    pub const INIT: u32 = 0b0001;
    #[allow(dead_code)]
    pub const IDLE: u32 = 0b0010;
}
const HW_POLL_MAX: u32 = 200_000;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// LINFlexD UART instance bound to an MMIO base address.
///
/// Construct with [`LinFlexD::new`] (a `const fn`) so instances can live in
/// `static` storage without a runtime initializer.
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
    sending: Cell<bool>,
}

impl LinFlexD<'_> {
    // ---------------------------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------------------------

    /// Create a new `LinFlexD` bound to `base`.  The instance is inert until
    /// [`configure`](Hil::uart::Configure::configure) succeeds.
    pub const fn new(base: StaticRef<LinFlexDRegisters>) -> Self {
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
            sending: Cell::new(false),
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

    // ---------------------------------------------------------------------------
    // Private helpers
    // ---------------------------------------------------------------------------

    /// Busy-wait until LINSR[LINS] == INIT mode.
    #[inline]
    fn wait_for_init_mode(&self) -> bool {
        let regs = self.registers;
        for _ in 0..HW_POLL_MAX {
            if regs.linsr.read(LINSR::LINS) == lins::INIT {
                return true;
            }
        }
        false
    }

    /// Wait for UARTSR[DTFTFF] (TX complete / TX FIFO not full) to be set.
    #[inline]
    fn wait_for_tx_done(&self) -> bool {
        let regs = self.registers;
        for _ in 0..HW_POLL_MAX {
            if regs.uartsr.is_set(UARTSR::DTFTFF) {
                return true;
            }
        }
        false
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;
        regs.linier.modify(LINIER::DTIE::CLEAR);
    }

    fn disable_rx_interrupt(&self) {
        let regs = self.registers;
        regs.linier
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
                self.sending.set(false);
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

                // Copy received bytes into the client buffer.
                // In buffer mode with RDFL=0 (1 byte expected) we read only DATA0.
                let idx = self.rx_index.get();
                let len = self.rx_len.get();

                if idx < len {
                    self.rx_buffer.map(|rx_buf| {
                        // Read up to 4 received bytes from BDRM.
                        let v = regs.bdrm.extract();
                        let mut pos = idx;
                        if pos < len {
                            rx_buf[pos] = (v.read(BDRM::DATA4) & 0xFF) as u8;
                            pos += 1;
                        }
                        if pos < len {
                            rx_buf[pos] = (v.read(BDRM::DATA5) & 0xFF) as u8;
                            pos += 1;
                        }
                        if pos < len {
                            rx_buf[pos] = (v.read(BDRM::DATA6) & 0xFF) as u8;
                            pos += 1;
                        }
                        if pos < len {
                            rx_buf[pos] = (v.read(BDRM::DATA7) & 0xFF) as u8;
                            pos += 1;
                        }

                        self.rx_index.set(pos);
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
        let regs = self.registers;

        // Acknowledge any stale DTF flag before writing.
        regs.uartsr.write(UARTSR::DTFTFF::SET);

        // Writing DATA0 triggers TX immediately.
        regs.bdrl.write(BDRL::DATA0.val(byte as u32));

        // Wait for the hardware to signal TX completion.
        if self.wait_for_tx_done() {
            regs.uartsr.write(UARTSR::DTFTFF::SET);
            true
        } else {
            false
        }
    }

    /// Transmit all bytes in `s`.
    pub fn puts(&self, s: &str) {
        self.transmit_sync(s.as_bytes())
    }

    /// Transmit all bytes in `bytes`.
    pub fn transmit_sync(&self, bytes: &[u8]) {
        for b in bytes.iter() {
            self.putc(*b);
        }
    }
}

// ---------------------------------------------------------------------------
// Configure trait
// ---------------------------------------------------------------------------

impl Configure for LinFlexD<'_> {
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

        // Poll for hardware acknowledgment.
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
        // At 48 MHz FIRC:
        //   115200: IBR=26, FBR=1  → 48_000_000/(16×26.0625) = 115 107 ≈ 115200
        //   921600: IBR=3,  FBR=4  → 48_000_000/(16×3.25)    = 923 077 ≈ 921600
        let (fbr, ibr): (u32, u32) = match params.baud_rate {
            115200 => (1, 26),
            921600 => (4, 3),
            _ => return Err(ErrorCode::NOSUPPORT),
        };

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
        regs.linier.set(0);

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
        if self.sending.get() {
            return Err((ErrorCode::BUSY, tx_data));
        }
        self.tx_buffer.replace(tx_data);
        self.tx_len.set(tx_len);
        self.tx_index.set(0);
        self.sending.set(true);

        // Enable transmitter and kick off the first batch.
        let regs = self.registers;
        regs.uartcr.modify(UARTCR::TxEn::SET);

        // Arm the TX completion interrupt.
        regs.linier.modify(LINIER::DTIE::SET);
        self.tx_progress();

        Ok(())
    }

    fn transmit_word(&self, word: u32) -> Result<(), ErrorCode> {
        if self.sending.get() {
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
        self.sending.set(true);
        Ok(())
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        // Could set LINCR2[ABRQ] here but the buffer is lost either way.
        self.tx_buffer.take();
        self.tx_len.set(0);
        self.tx_index.set(0);
        self.disable_tx_interrupt();
        Ok(())
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
        if rx_len == 0 || rx_len > rx_buffer.len() {
            return Err((ErrorCode::SIZE, rx_buffer));
        }

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);
        self.rx_index.set(0);

        // Configure RDFL to match requested byte count (max 4 in buffer mode).
        // TODO: for >4 bytes chain multiple receive_buffer calls or use FIFO mode.
        let rdfl = ((rx_len as u32 - 1) & 0b11).min(3); // RDFL is 3 bits, max value 3 = 4 bytes
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
        // TODO: implement for 9-bit / 16-bit word modes
        Err(ErrorCode::FAIL)
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        self.rx_buffer.take();
        self.rx_len.set(0);
        self.rx_index.set(0);
        self.disable_rx_interrupt();
        Ok(())
    }
}
