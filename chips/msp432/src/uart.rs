//! UART

use crate::usci::{self, UsciARegisters};
use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ReturnCode;

pub static mut UART0: Uart<'static> = Uart::new(usci::USCI_A0_BASE);

const DEFAULT_CLOCK_FREQ_HZ: u32 = 12_000_000;

struct BaudFraction {
    frac: f32,
    reg_val: u8,
}

#[rustfmt::skip]
// Table out of the datahseet correct the baudrate
const BAUD_FRACTIONS: &'static [BaudFraction; 36] = &[
    BaudFraction { frac: 0.0000, reg_val: 0x00 },
    BaudFraction { frac: 0.0529, reg_val: 0x01 },
    BaudFraction { frac: 0.0715, reg_val: 0x02 },
    BaudFraction { frac: 0.0835, reg_val: 0x04 },
    BaudFraction { frac: 0.1001, reg_val: 0x08 },
    BaudFraction { frac: 0.1252, reg_val: 0x10 },
    BaudFraction { frac: 0.1430, reg_val: 0x20 },
    BaudFraction { frac: 0.1670, reg_val: 0x11 },
    BaudFraction { frac: 0.2147, reg_val: 0x21 },
    BaudFraction { frac: 0.2224, reg_val: 0x22 },
    BaudFraction { frac: 0.2503, reg_val: 0x44 },
    BaudFraction { frac: 0.3000, reg_val: 0x25 },
    BaudFraction { frac: 0.3335, reg_val: 0x49 },
    BaudFraction { frac: 0.3575, reg_val: 0x4A },
    BaudFraction { frac: 0.3753, reg_val: 0x52 },
    BaudFraction { frac: 0.4003, reg_val: 0x92 },
    BaudFraction { frac: 0.4286, reg_val: 0x53 },
    BaudFraction { frac: 0.4378, reg_val: 0x55 },
    BaudFraction { frac: 0.5002, reg_val: 0xAA },
    BaudFraction { frac: 0.5715, reg_val: 0x6B },
    BaudFraction { frac: 0.6003, reg_val: 0xAD },
    BaudFraction { frac: 0.6254, reg_val: 0xB5 },
    BaudFraction { frac: 0.6432, reg_val: 0xB6 },
    BaudFraction { frac: 0.6667, reg_val: 0xD6 },
    BaudFraction { frac: 0.7001, reg_val: 0xB7 },
    BaudFraction { frac: 0.7147, reg_val: 0xBB },
    BaudFraction { frac: 0.7503, reg_val: 0xDD },
    BaudFraction { frac: 0.7861, reg_val: 0xED },
    BaudFraction { frac: 0.8004, reg_val: 0xEE },
    BaudFraction { frac: 0.8333, reg_val: 0xBF },
    BaudFraction { frac: 0.8464, reg_val: 0xDF },
    BaudFraction { frac: 0.8572, reg_val: 0xEF },
    BaudFraction { frac: 0.8751, reg_val: 0xF7 },
    BaudFraction { frac: 0.9004, reg_val: 0xFB },
    BaudFraction { frac: 0.9170, reg_val: 0xFD },
    BaudFraction { frac: 0.9288, reg_val: 0xFE },
];

pub struct Uart<'a> {
    registers: StaticRef<UsciARegisters>,
    clock_frequency: u32,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,

    tx_buffer: TakeCell<'static, [u8]>,
    tx_len: Cell<usize>,
    tx_index: Cell<usize>,
}

impl Uart<'a> {
    pub(crate) const fn new(regs: StaticRef<UsciARegisters>) -> Uart<'a> {
        Uart {
            registers: regs,
            clock_frequency: DEFAULT_CLOCK_FREQ_HZ,
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
            tx_len: Cell::new(0),
            tx_index: Cell::new(0),
        }
    }

    pub fn transmit_sync(&self, data: &[u8]) {
        for b in data.iter() {
            while self.registers.statw.is_set(usci::UCAxSTATW::UCBUSY) {}
            self.registers.txbuf.set(*b as u16);
        }
    }
}

impl<'a> hil::uart::UartData<'a> for Uart<'a> {}
impl<'a> hil::uart::Uart<'a> for Uart<'a> {}

impl hil::uart::Configure for Uart<'a> {
    fn configure(&self, params: hil::uart::Parameters) -> ReturnCode {
        // Disable module
        let regs = self.registers;
        regs.ctlw0.modify(usci::UCAxCTLW0::UCSWRST::SET);

        // Setup module to UART mode
        regs.ctlw0.modify(usci::UCAxCTLW0::UCMODE::UARTMode);

        // Setup clock-source to SMCLK
        regs.ctlw0.modify(usci::UCAxCTLW0::UCSSEL::SMCLK);

        // Setup word-length
        match params.width {
            hil::uart::Width::Eight => regs.ctlw0.modify(usci::UCAxCTLW0::UC7BIT::CLEAR),
            hil::uart::Width::Seven => regs.ctlw0.modify(usci::UCAxCTLW0::UC7BIT::SET),
            hil::uart::Width::Six => {
                panic!("UART: width of 6 bit is not supported by this hardware!")
            }
        }

        // Setup stop bits
        if params.stop_bits == hil::uart::StopBits::One {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCSPB::CLEAR);
        } else {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCSPB::SET);
        }

        // Setup parity
        if params.parity == hil::uart::Parity::None {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCPEN::CLEAR);
        } else {
            regs.ctlw0.modify(usci::UCAxCTLW0::UCPEN::SET);
            if params.parity == hil::uart::Parity::Even {
                regs.ctlw0.modify(usci::UCAxCTLW0::UCPAR::SET);
            } else {
                regs.ctlw0.modify(usci::UCAxCTLW0::UCPAR::CLEAR);
            }
        }

        // Setup baudrate, all the calculation from the datasheet p. 915
        let n = (self.clock_frequency / params.baud_rate) as u16;
        let n_float = (self.clock_frequency as f32) / (params.baud_rate as f32);
        let frac_part = n_float - (n as f32);
        if n > 16 {
            // Oversampling is enabled
            regs.brw.set(n >> 4); // equals n / 16
            let ucbrf = (n_float / 16.0f32 - ((n >> 4) as f32) * 16.0f32) as u16;
            regs.mctlw
                .modify(usci::UCAxMCTLW::UCBRF.val(ucbrf) + usci::UCAxMCTLW::UCOS16::SET);
        } else {
            // No oversampling
            regs.brw.set(n);
            regs.mctlw.modify(usci::UCAxMCTLW::UCOS16::CLEAR);
        }

        // Look for the closest calibration value
        // According to the datasheet not the closest value should be taken but the next smaller one
        let mut ucbrs = BAUD_FRACTIONS[0].reg_val;
        for val in BAUD_FRACTIONS.iter() {
            if val.frac > frac_part {
                break;
            }
            ucbrs = val.reg_val;
        }
        regs.mctlw.modify(usci::UCAxMCTLW::UCBRS.val(ucbrs as u16));

        // Enable module
        regs.ctlw0.modify(usci::UCAxCTLW0::UCSWRST::CLEAR);

        // Enable receive interrupt
        // self.registers.ie.modify(usci::UCAxIE::UCRXIE::SET);

        ReturnCode::SUCCESS
    }
}

impl<'a> hil::uart::Transmit<'a> for Uart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::FAIL, Some(tx_buffer))
    }

    fn transmit_word(&self, word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

impl<'a> hil::uart::Receive<'a> for Uart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::FAIL, Some(rx_buffer))
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}
