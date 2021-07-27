//! UART driver.

use core::cell::Cell;
use kernel::ErrorCode;

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

pub const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x6000_0000 as *const UartRegisters) };

register_structs! {
    pub UartRegisters {
        (0x000 => fifo: ReadWrite<u32, FIFO::Register>),
        (0x004 => int_raw: ReadWrite<u32, INT::Register>),
        (0x008 => int_st: ReadWrite<u32, INT::Register>),
        (0x00C => int_ena: ReadWrite<u32, INT::Register>),
        (0x010 => int_clr: ReadWrite<u32, INT::Register>),
        (0x014 => clkdiv: ReadWrite<u32, CLKDIV::Register>),
        (0x018 => autobaud: ReadWrite<u32, AUTOBAUD::Register>),
        (0x01C => status: ReadWrite<u32, STATUS::Register>),
        (0x020 => conf0: ReadWrite<u32, CONF0::Register>),
        (0x024 => conf1: ReadWrite<u32, CONF1::Register>),
        (0x028 => lowpulse: ReadWrite<u32, LOWPULSE::Register>),
        (0x02C => highpulse: ReadWrite<u32, HIGHPULSE::Register>),
        (0x030 => rxd_cnt: ReadWrite<u32, RXD_CNT::Register>),
        (0x034 => flow_config: ReadWrite<u32, FLOW_CONFIG::Register>),
        (0x038 => sleep_conf: ReadWrite<u32, SLEEP_CONF::Register>),
        (0x03C => swfc_conf: ReadWrite<u32, SWFC_CONF::Register>),
        (0x040 => idle_conf: ReadWrite<u32, IDLE_CONF::Register>),
        (0x044 => rs485_conf: ReadWrite<u32, RS485_CONF::Register>),
        (0x048 => at_cmd_precnt: ReadWrite<u32, AT_CMD_PRECNT::Register>),
        (0x04C => at_cmd_postcnt: ReadWrite<u32, AT_CMD_POSTCNT::Register>),
        (0x050 => at_cmd_gaptout: ReadWrite<u32, AT_CMD_GAPTOUT::Register>),
        (0x054 => at_cmd_char: ReadWrite<u32, AT_CMD_CHAR::Register>),
        (0x058 => mem_conf: ReadWrite<u32, MEM_CONF::Register>),
        (0x05C => _reserved0),
        (0x064 => mem_cnt_status: ReadWrite<u32, MEM_CNT_STATUS::Register>),
        (0x068 => pospulse: ReadWrite<u32, POSPULSE::Register>),
        (0x06C => negpulse: ReadWrite<u32, NEGPULSE::Register>),
        (0x070 => @END),
    }
}

register_bitfields![u32,
    FIFO [
        RXFIFO_RD_BYTE OFFSET(0) NUMBITS(8) [],
    ],
    INT [
        RXFIFO_FULL_INT OFFSET(0) NUMBITS(1) [],
        TXFIFO_EMPTY_INT OFFSET(1) NUMBITS(1) [],
        PARITY_ERR_INT OFFSET(2) NUMBITS(1) [],
        FRM_ERR_INT OFFSET(3) NUMBITS(1) [],
        RXFIFO_OVF_INT OFFSET(4) NUMBITS(1) [],
        DSR_CHG_INT OFFSET(5) NUMBITS(1) [],
        CTS_CHG_INT OFFSET(6) NUMBITS(1) [],
        BRK_DET_INT OFFSET(7) NUMBITS(1) [],
        RXFIFO_TOUT_INT OFFSET(8) NUMBITS(1) [],
        SW_XON_INT OFFSET(9) NUMBITS(1) [],
        SW_XOFF_INT OFFSET(10) NUMBITS(1) [],
        GLITCH_DET_INT OFFSET(11) NUMBITS(1) [],
        TX_BRK_DONE_INT OFFSET(12) NUMBITS(1) [],
        TX_BRK_IDLE_DONE_INT OFFSET(13) NUMBITS(1) [],
        TX_DONE_INT OFFSET(14) NUMBITS(1) [],
        RS485_PARITY_ERR_INT OFFSET(15) NUMBITS(1) [],
        RS485_FRM_ERR_INT OFFSET(16) NUMBITS(1) [],
        RS485_CLASH_INT OFFSET(17) NUMBITS(1) [],
        AT_CMD_CHAR_DET_INT OFFSET(18) NUMBITS(1) [],
    ],
    CLKDIV [
        CLKDIV OFFSET(0) NUMBITS(20) [],
        CLKDIV_FRAG OFFSET(20) NUMBITS(4) [],
    ],
    AUTOBAUD [
        AUTOBAUD_EN OFFSET(0) NUMBITS(1) [],
        GLITCH_FILT OFFSET(8) NUMBITS(8) [],
    ],
    STATUS [
        RXFIFO_CNT OFFSET(0) NUMBITS(8) [],
        ST_URX_OUT OFFSET(8) NUMBITS(4) [],
        DSRN OFFSET(13) NUMBITS(1) [],
        CTSN OFFSET(14) NUMBITS(1) [],
        RXD OFFSET(15) NUMBITS(1) [],
        TXFIFO_CNT OFFSET(16) NUMBITS(8) [],
        ST_UTX_OUT OFFSET(24) NUMBITS(4) [],
        DTRN OFFSET(29) NUMBITS(1) [],
        RTSN OFFSET(30) NUMBITS(1) [],
        TXD OFFSET(31) NUMBITS(1) [],
    ],
    CONF0 [
        PARITY OFFSET(0) NUMBITS(1) [],
        PARITY_EN OFFSET(1) NUMBITS(1) [],
        BIT_NUM OFFSET(2) NUMBITS(2) [],
        STOP_BIT_NUM OFFSET(4) NUMBITS(2) [],
        SW_RTS OFFSET(6) NUMBITS(1) [],
        SW_DTR OFFSET(7) NUMBITS(1) [],
        TXD_BRK OFFSET(8) NUMBITS(1) [],
        IRDA_DPLX OFFSET(9) NUMBITS(1) [],
        IRDA_TX_EN OFFSET(10) NUMBITS(1) [],
        IRDA_WCTL OFFSET(11) NUMBITS(1) [],
        IRDA_TX_INV OFFSET(12) NUMBITS(1) [],
        IRDA_RX_INV OFFSET(13) NUMBITS(1) [],
        LOOPBACK OFFSET(14) NUMBITS(1) [],
        TX_FLOW_EN OFFSET(15) NUMBITS(1) [],
        IRDA_EN OFFSET(16) NUMBITS(1) [],
        RXFIFO_RST OFFSET(17) NUMBITS(1) [],
        TXFIFO_RST OFFSET(18) NUMBITS(1) [],
        RXD_INV OFFSET(19) NUMBITS(1) [],
        CTS_INV OFFSET(20) NUMBITS(1) [],
        DSR_INV OFFSET(21) NUMBITS(1) [],
        TXD_INV OFFSET(22) NUMBITS(1) [],
        RTS_INV OFFSET(23) NUMBITS(1) [],
        DTR_INV OFFSET(24) NUMBITS(1) [],
        TICK_REF_ALWAYS_ON OFFSET(27) NUMBITS(1) [],
    ],
    CONF1 [
        RXFIFO_FULL_THRHD OFFSET(0) NUMBITS(7) [],
        TXFIFO_EMPTY_THRHD OFFSET(8) NUMBITS(6) [],
        RX_FLOW_THRHD OFFSET(16) NUMBITS(6) [],
        RX_FLOW_EN OFFSET(23) NUMBITS(1) [],
        RX_TOUT_THRHD OFFSET(24) NUMBITS(7) [],
        RX_TOUT_EN OFFSET(31) NUMBITS(1) [],
    ],
    LOWPULSE [
        LOWPULSE_MIN_CNT OFFSET(0) NUMBITS(20) [],
    ],
    HIGHPULSE [
        HIGHPULSE_MIN_CNT OFFSET(0) NUMBITS(20) []
    ],
    RXD_CNT [
        RXD_EDGE_CNT OFFSET(0) NUMBITS(10) [],
    ],
    FLOW_CONFIG [
        SW_FLOW_CON_EN OFFSET(0) NUMBITS(1) [],
        XONOFF_DEL OFFSET(1) NUMBITS(1) [],
        FORCE_XON OFFSET(2) NUMBITS(1) [],
        FORCE_XOFF OFFSET(3) NUMBITS(1) [],
        SEND_XON OFFSET(4) NUMBITS(1) [],
        SEND_XOFF OFFSET(5) NUMBITS(1) [],
    ],
    SLEEP_CONF [
        ACTIVE_THRESHOLD OFFSET(0) NUMBITS(10) [],
    ],
    SWFC_CONF [
        XON_THRESHOLD OFFSET(0) NUMBITS(8) [],
        XOFF_THRESHOLD OFFSET(8) NUMBITS(8) [],
        XON_CHAR OFFSET(16) NUMBITS(8) [],
        XOFF_CHAR OFFSET(24) NUMBITS(8) [],
    ],
    IDLE_CONF [
        RX_IDLE_THRHD OFFSET(0) NUMBITS(10) [],
        TX_IDLE_NUM OFFSET(10) NUMBITS(10) [],
        TX_BRK_NUM OFFSET(20) NUMBITS(8) [],
    ],
    RS485_CONF [
        RS485_EN OFFSET(0) NUMBITS(1) [],
        DL0_EN OFFSET(1) NUMBITS(1) [],
        DL1_EN OFFSET(2) NUMBITS(1) [],
        RS485TX_RX_EN OFFSET(3) NUMBITS(1) [],
        RS485RXBY_TX_EN OFFSET(4) NUMBITS(1) [],
        RS485_RX_DLY_NUM OFFSET(5) NUMBITS(1) [],
        RS485_TX_DLY_NUM OFFSET(6) NUMBITS(4) [],
    ],
    AT_CMD_PRECNT [
        PRE_IDLE_NUM OFFSET(0) NUMBITS(24) [],
    ],
    AT_CMD_POSTCNT [
        POST_IDLE_NUM OFFSET(0) NUMBITS(24) [],
    ],
    AT_CMD_GAPTOUT [
        RX_GAP_TOUT OFFSET(0) NUMBITS(24) [],
    ],
    AT_CMD_CHAR [
        AT_CMD_CHAR_REG OFFSET(0) NUMBITS(8) [],
        CHAR_NUM OFFSET(8) NUMBITS(8) [],
    ],
    MEM_CONF [
        MEM_PD OFFSET(0) NUMBITS(1) [],
        RX_SIZE OFFSET(3) NUMBITS(4) [],
        TX_SIZE OFFSET(7) NUMBITS(4) [],
        RX_FLOW_THRHD_H3 OFFSET(15) NUMBITS(3) [],
        RX_TOUT_THRHD_H3 OFFSET(18) NUMBITS(3) [],
        XON_THRESHOLD_H2 OFFSET(21) NUMBITS(2) [],
        XOFF_THRESHOLD_H2 OFFSET(23) NUMBITS(2) [],
        RX_MEM_FULL_THRHD OFFSET(25) NUMBITS(3) [],
        TX_MEM_EMPTY_THRHD OFFSET(28) NUMBITS(3) [],
    ],
    MEM_CNT_STATUS [
        RX_MEM_CNT OFFSET(0) NUMBITS(3) [],
        TX_MEM_CNT OFFSET(3) NUMBITS(3) [],
    ],
    POSPULSE [
        POSEDGE_MIN_CNT OFFSET(0) NUMBITS(20) [],
    ],
    NEGPULSE [
        NEGEDGE_MIN_CNT OFFSET(0) NUMBITS(20) [],
    ],
];

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,

    rx_buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UartParams {
    pub baud_rate: u32,
}

impl<'a> Uart<'a> {
    pub const fn new(base: StaticRef<UartRegisters>) -> Uart<'a> {
        Uart {
            registers: base,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
            rx_buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
        }
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.int_ena.modify(
            INT::TXFIFO_EMPTY_INT::SET
                + INT::TX_BRK_DONE_INT::SET
                + INT::TX_BRK_IDLE_DONE_INT::SET
                + INT::TX_DONE_INT::SET,
        );
    }

    pub fn disable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.int_clr.modify(
            INT::TXFIFO_EMPTY_INT::SET
                + INT::TX_BRK_DONE_INT::SET
                + INT::TX_BRK_IDLE_DONE_INT::SET
                + INT::TX_DONE_INT::SET,
        );
        regs.int_ena.modify(
            INT::TXFIFO_EMPTY_INT::CLEAR
                + INT::TX_BRK_DONE_INT::CLEAR
                + INT::TX_BRK_IDLE_DONE_INT::CLEAR
                + INT::TX_DONE_INT::CLEAR,
        );
    }

    fn enable_rx_interrupt(&self) {
        let regs = self.registers;

        regs.int_ena.modify(
            INT::RXFIFO_FULL_INT::SET + INT::RXFIFO_OVF_INT::SET + INT::RXFIFO_TOUT_INT::SET,
        );
    }

    pub fn disable_rx_interrupt(&self) {
        let regs = self.registers;

        regs.int_clr.modify(
            INT::RXFIFO_FULL_INT::SET + INT::RXFIFO_OVF_INT::SET + INT::RXFIFO_TOUT_INT::SET,
        );
        regs.int_ena.modify(
            INT::RXFIFO_FULL_INT::CLEAR + INT::RXFIFO_OVF_INT::CLEAR + INT::RXFIFO_TOUT_INT::CLEAR,
        );
    }

    fn tx_progress(&self) {
        let regs = self.registers;
        let idx = self.tx_index.get();
        let len = self.tx_len.get();

        if idx < len {
            // If we are going to transmit anything, we first need to enable the
            // TX interrupt. This ensures that we will get an interrupt, where
            // we can either call the callback from, or continue transmitting
            // bytes.
            self.enable_tx_interrupt();

            // Read from the transmit buffer and send bytes to the UART hardware
            // until either the buffer is empty or the UART hardware is full.
            self.tx_buffer.map(|tx_buf| {
                let tx_len = len - idx;

                for i in 0..tx_len {
                    if regs.status.read(STATUS::TXFIFO_CNT) >= 127 {
                        break;
                    }
                    let tx_idx = idx + i;
                    regs.fifo
                        .write(FIFO::RXFIFO_RD_BYTE.val(tx_buf[tx_idx] as u32));
                    self.tx_index.set(tx_idx + 1)
                }
            });
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let intrs = regs.int_st.extract();

        if intrs.is_set(INT::TXFIFO_EMPTY_INT) {
            self.disable_tx_interrupt();

            if self.tx_index.get() == self.tx_len.get() {
                // We sent everything to the UART hardware, now from an
                // interrupt callback we can issue the callback.
                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|tx_buf| {
                        client.transmitted_buffer(tx_buf, self.tx_len.get(), Ok(()));
                    });
                });
            } else {
                // We have more to transmit, so continue in tx_progress().
                self.tx_progress();
            }
        }
    }

    pub fn transmit_sync(&self, bytes: &[u8]) {
        let regs = self.registers;
        for b in bytes.iter() {
            while regs.status.read(STATUS::TXFIFO_CNT) > 8 {}
            regs.fifo.write(FIFO::RXFIFO_RD_BYTE.val(*b as u32));
        }
    }
}

impl hil::uart::Configure for Uart<'_> {
    fn configure(&self, _params: hil::uart::Parameters) -> Result<(), ErrorCode> {
        // Disable all interrupts for now
        self.disable_rx_interrupt();
        self.disable_tx_interrupt();

        Ok(())
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if tx_len == 0 || tx_len > tx_data.len() {
            Err((ErrorCode::SIZE, tx_data))
        } else if self.tx_buffer.is_some() {
            Err((ErrorCode::BUSY, tx_data))
        } else {
            // Save the buffer so we can keep sending it.
            self.tx_buffer.replace(tx_data);
            self.tx_len.set(tx_len);
            self.tx_index.set(0);

            self.tx_progress();
            Ok(())
        }
    }

    fn transmit_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn transmit_word(&self, _word: u32) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}

/* UART receive is not implemented yet, mostly due to a lack of tests avaliable */
impl<'a> hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
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

        self.enable_rx_interrupt();

        self.rx_buffer.replace(rx_buffer);
        self.rx_len.set(rx_len);

        Ok(())
    }

    fn receive_abort(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn receive_word(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }
}
