//! UART driver.

use core::cell::Cell;

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

pub static mut UART0: Uart = Uart::new(UART0_BASE);

const UART0_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4001_C000 as *const UartRegisters) };

pub static mut UART1: Uart = Uart::new(UART1_BASE);

const UART1_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x4001_D000 as *const UartRegisters) };

register_structs! {
    pub UartRegisters {
        (0x000 => dr: ReadWrite<u32, DR::Register>),
        (0x004 => rsr: ReadWrite<u32, RSR::Register>),
        (0x008 => _reserved0),
        (0x018 => fr: ReadWrite<u32, FR::Register>),
        (0x01c => _reserved1),
        (0x020 => ilpr: ReadWrite<u32, ILPR::Register>),
        (0x024 => ibrd: ReadWrite<u32, IBRD::Register>),
        (0x028 => fbrd: ReadWrite<u32, FBRD::Register>),
        (0x02c => lcrh: ReadWrite<u32, LCRH::Register>),
        (0x030 => cr: ReadWrite<u32, CR::Register>),
        (0x034 => ifls: ReadWrite<u32, IFLS::Register>),
        (0x038 => ier: ReadWrite<u32, IER::Register>),
        (0x03c => ies: ReadWrite<u32, IES::Register>),
        (0x040 => mis: ReadWrite<u32, MIS::Register>),
        (0x044 => iec: ReadWrite<u32, IEC::Register>),
        (0x048 => @END),
    }
}

register_bitfields![u32,
    DR [
        DATA OFFSET(0) NUMBITS(8) [],
        FEDATA OFFSET(8) NUMBITS(1) [],
        PEDATA OFFSET(9) NUMBITS(1) [],
        BEDATA OFFSET(10) NUMBITS(1) [],
        OEDATA OFFSET(11) NUMBITS(1) []
    ],
    RSR [
        FESTAT OFFSET(0) NUMBITS(1) [],
        PESTAT OFFSET(1) NUMBITS(1) [],
        BESTAT OFFSET(2) NUMBITS(1) [],
        OESTAT OFFSET(4) NUMBITS(1) []
    ],
    FR [
        CTS OFFSET(0) NUMBITS(1) [],
        DSR OFFSET(1) NUMBITS(1) [],
        DCD OFFSET(2) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFF OFFSET(6) NUMBITS(1) [],
        TXFE OFFSET(7) NUMBITS(1) [],
        TXBUSY OFFSET(8) NUMBITS(1) []
    ],
    ILPR [
        ILPDVSR OFFSET(0) NUMBITS(8) []
    ],
    IBRD [
        DIVINT OFFSET(0) NUMBITS(16) []
    ],
    FBRD [
        DIVFRAC OFFSET(0) NUMBITS(6) []
    ],
    LCRH [
        BRK OFFSET(0) NUMBITS(1) [],
        PEN OFFSET(1) NUMBITS(1) [],
        EPS OFFSET(2) NUMBITS(1) [],
        STP2 OFFSET(3) NUMBITS(1) [],
        FEN OFFSET(4) NUMBITS(1) [],
        WLEN OFFSET(5) NUMBITS(2) [],
        SPS OFFSET(7) NUMBITS(1) []
    ],
    CR [
        UARTEN OFFSET(0) NUMBITS(1) [],
        SIREN OFFSET(1) NUMBITS(1) [],
        SIRLP OFFSET(2) NUMBITS(1) [],
        CLKEN OFFSET(3) NUMBITS(1) [],
        CLKSEL OFFSET(4) NUMBITS(2) [
            CLK_24MHZ = 0x1,
            CLK_12MHZ = 0x2,
            CLK_6MHZ = 0x3,
            CLK_3MHZ = 0x4
        ],
        LBE OFFSET(7) NUMBITS(1) [],
        TXE OFFSET(8) NUMBITS(1) [],
        RXE OFFSET(9) NUMBITS(1) [],
        DTR OFFSET(10) NUMBITS(1) [],
        RTS OFFSET(11) NUMBITS(1) [],
        OUT1 OFFSET(12) NUMBITS(1) [],
        OUT2 OFFSET(13) NUMBITS(1) [],
        RTSEN OFFSET(14) NUMBITS(1) [],
        CTSEN OFFSET(15) NUMBITS(1) []
    ],
    IFLS [
        TXIFLSEL OFFSET(0) NUMBITS(3) [],
        RXIFLSEL OFFSET(3) NUMBITS(3) []
    ],
    IER [
        TXCMPMIM OFFSET(0) NUMBITS(1) [],
        CTSMIM OFFSET(1) NUMBITS(1) [],
        DCDMIM OFFSET(2) NUMBITS(1) [],
        DSRMIM OFFSET(3) NUMBITS(1) [],
        RXIM OFFSET(4) NUMBITS(1) [],
        TXIM OFFSET(5) NUMBITS(1) [],
        RTIM OFFSET(6) NUMBITS(1) [],
        FEIM OFFSET(7) NUMBITS(1) [],
        PEIM OFFSET(8) NUMBITS(1) [],
        BEIM OFFSET(9) NUMBITS(1) [],
        OEIM OFFSET(10) NUMBITS(1) []
    ],
    IES [
        TXCMPMIS OFFSET(0) NUMBITS(1) [],
        CTSMIS OFFSET(1) NUMBITS(1) [],
        DCDMIS OFFSET(2) NUMBITS(1) [],
        DSRMIS OFFSET(3) NUMBITS(1) [],
        RXIS OFFSET(4) NUMBITS(1) [],
        TXIS OFFSET(5) NUMBITS(1) [],
        RTIS OFFSET(6) NUMBITS(1) [],
        FEIS OFFSET(7) NUMBITS(1) [],
        PEIS OFFSET(8) NUMBITS(1) [],
        BEIS OFFSET(9) NUMBITS(1) [],
        OEIS OFFSET(10) NUMBITS(1) []
    ],
    MIS [
        TXCMPMMIS OFFSET(0) NUMBITS(1) [],
        CTSMMIS OFFSET(1) NUMBITS(1) [],
        DCDMMIS OFFSET(2) NUMBITS(1) [],
        DSRMMIS OFFSET(3) NUMBITS(1) [],
        RXMIS OFFSET(4) NUMBITS(1) [],
        TXMIS OFFSET(5) NUMBITS(1) [],
        RTMIS OFFSET(6) NUMBITS(1) [],
        FEMIS OFFSET(7) NUMBITS(1) [],
        PEMIS OFFSET(8) NUMBITS(1) [],
        BEMIS OFFSET(9) NUMBITS(1) [],
        OEMIS OFFSET(10) NUMBITS(1) []
    ],
    IEC [
        TXCMPMMIC OFFSET(0) NUMBITS(1) [],
        CTSMMIC OFFSET(1) NUMBITS(1) [],
        DCDMMIC OFFSET(2) NUMBITS(1) [],
        DSRMMIC OFFSET(3) NUMBITS(1) [],
        RXMIC OFFSET(4) NUMBITS(1) [],
        TXMIC OFFSET(5) NUMBITS(1) [],
        RTMIC OFFSET(6) NUMBITS(1) [],
        FEMIC OFFSET(7) NUMBITS(1) [],
        PEMIC OFFSET(8) NUMBITS(1) [],
        BEMIC OFFSET(9) NUMBITS(1) [],
        OEMIC OFFSET(10) NUMBITS(1) []
    ]
];

pub struct Uart<'a> {
    registers: StaticRef<UartRegisters>,
    clock_frequency: u32,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UartParams {
    pub baud_rate: u32,
}

impl Uart<'a> {
    pub const fn new(base: StaticRef<UartRegisters>) -> Uart<'a> {
        Uart {
            registers: base,
            clock_frequency: 24_000_000,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
        }
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = self.registers;

        let baud_clk = 16 * baud_rate;
        let integer_divisor = self.clock_frequency / baud_clk;
        let intermediate_long = (self.clock_frequency * 64) / baud_clk;
        let fraction_divisor_long = intermediate_long - (integer_divisor * 64);

        regs.ibrd.write(IBRD::DIVINT.val(integer_divisor));
        regs.fbrd.write(FBRD::DIVFRAC.val(fraction_divisor_long));
    }

    fn enable_tx_interrupt(&self) {
        let regs = self.registers;

        // Set TX FIFO to fire at 0
        regs.ifls.modify(IFLS::TXIFLSEL.val(0));

        regs.ier.modify(IER::TXIM::SET);
    }

    fn disable_tx_interrupt(&self) {
        let regs = self.registers;

        regs.ier.modify(IER::TXIM::CLEAR);
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
                    if regs.fr.is_set(FR::TXFF) {
                        break;
                    }
                    let tx_idx = idx + i;
                    regs.dr.write(DR::DATA.val(tx_buf[tx_idx] as u32));
                    self.tx_index.set(tx_idx + 1)
                }
            });
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let irq = regs.ies.get();

        self.disable_tx_interrupt();

        if irq & 0x20 != 0 {
            // TXRIS Interrupt
            self.disable_tx_interrupt();

            if self.tx_index.get() == self.tx_len.get() {
                // We sent everything to the UART hardware, now from an
                // interrupt callback we can issue the callback.
                self.tx_client.map(|client| {
                    self.tx_buffer.take().map(|tx_buf| {
                        client.transmitted_buffer(tx_buf, self.tx_len.get(), ReturnCode::SUCCESS);
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
            while regs.fr.is_set(FR::TXFF) {}
            regs.dr.write(DR::DATA.val(*b as u32));
        }
    }
}

impl hil::uart::UartData<'a> for Uart<'a> {}
impl hil::uart::Uart<'a> for Uart<'a> {}

impl hil::uart::Configure for Uart<'a> {
    fn configure(&self, params: hil::uart::Parameters) -> ReturnCode {
        let regs = self.registers;

        // Disable UART
        regs.cr
            .write(CR::UARTEN::CLEAR + CR::RXE::CLEAR + CR::TXE::CLEAR);

        // Enable the clocks
        regs.cr.write(CR::CLKEN::SET + CR::CLKSEL::CLK_24MHZ);

        // Set the baud rate
        self.set_baud_rate(params.baud_rate);

        // Setup the UART
        regs.cr.modify(CR::RTSEN::CLEAR + CR::CTSEN::CLEAR);
        // Enalbe FIFO
        regs.lcrh.write(LCRH::FEN::SET);
        // Set 8 data bits, no parity, 1 stop bit and no flow control
        regs.lcrh.modify(LCRH::WLEN.val(3) + LCRH::FEN::SET);

        // Enable the UART
        regs.cr
            .modify(CR::UARTEN::SET + CR::RXE::SET + CR::TXE::SET);

        ReturnCode::SUCCESS
    }
}

impl hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if tx_len == 0 || tx_len > tx_data.len() {
            (ReturnCode::ESIZE, Some(tx_data))
        } else if self.tx_buffer.is_some() {
            (ReturnCode::EBUSY, Some(tx_data))
        } else {
            // Save the buffer so we can keep sending it.
            self.tx_buffer.replace(tx_data);
            self.tx_len.set(tx_len);
            self.tx_index.set(0);

            self.tx_progress();
            (ReturnCode::SUCCESS, None)
        }
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        _rx_buffer: &'static mut [u8],
        _rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::FAIL, None)
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
