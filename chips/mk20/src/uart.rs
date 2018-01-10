//! Implementation of the MK20 UART Peripheral

use core::cell::Cell;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::hil::uart;
use core::mem;
use nvic;
use regs::uart::*;
use clock;

pub struct Uart {
    index: usize,
    registers: *mut Registers,
    client: Cell<Option<&'static uart::Client>>,
    buffer: TakeCell<'static, [u8]>,
    rx_len: Cell<usize>,
    rx_index: Cell<usize>
}

pub static mut UART0: Uart = Uart::new(0);
pub static mut UART1: Uart = Uart::new(1);
pub static mut UART2: Uart = Uart::new(2);
pub static mut UART3: Uart = Uart::new(3);
pub static mut UART4: Uart = Uart::new(4);

impl Uart {
    pub const fn new(index: usize) -> Uart {
        Uart {
            index: index,
            registers: UART_BASE_ADDRS[index],
            client: Cell::new(None),
            buffer: TakeCell::empty(),
            rx_len: Cell::new(0),
            rx_index: Cell::new(0),
        }
    }

    pub fn handle_interrupt(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        // Read byte from data register; reading S1 and D clears interrupt
        if regs.s1.is_set(S1::RDRF) {
            let datum: u8 = regs.d.get();

            // Put byte into buffer, trigger callback if buffer full
            let mut done = false;
            let mut index = self.rx_index.get();
            self.buffer.map( |buf| {
                buf[index] = datum;
                index = index + 1;
                if index >= self.rx_len.get() {
                    done = true;
                }
                self.rx_index.set(index);
            });
            if done {
                self.client.get().map(|client| {
                    match self.buffer.take() {
                        Some(buf) => client.receive_complete(buf, index, uart::Error::CommandComplete),
                        None => ()
                    }
                });
            }
        }
    }

    pub fn handle_error(&self) {
        // TODO: implement
    }

    fn set_parity(&self, parity: hil::uart::Parity) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        let (pe, pt) = match parity {
            hil::uart::Parity::None => (C1::PE::CLEAR, C1::PT::Even),
            hil::uart::Parity::Even => (C1::PE::SET, C1::PT::Even),
            hil::uart::Parity::Odd => (C1::PE::SET, C1::PT::Odd)
        };

        // This basic procedure outlined in section 59.9.3.
        // Set control register 1, which configures the parity.
        regs.c1.write(pe + pt +
                      C1::LOOPS::CLEAR +
                      C1::UARTSWAI::CLEAR +
                      C1::RSRC::CLEAR +
                      C1::M::EightBit +
                      C1::WAKE::Idle +
                      C1::ILT::AfterStop);
    }

    fn set_stop_bits(&self, stop_bits: hil::uart::StopBits) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        let stop_bits = match stop_bits {
            hil::uart::StopBits::One => BDH::SBNS::One,
            hil::uart::StopBits::Two => BDH::SBNS::Two
        };

        regs.bdh.modify(stop_bits);
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        // Baud rate generation. Note that UART0 and UART1 are sourced from the core clock, not the
        // bus clock.
        let uart_clock = match self.index {
            0 | 1 => clock::core_clock_hz(),
            _ => clock::peripheral_clock_hz()
        };

        let baud_counter: u32 = (uart_clock >> 4) / baud_rate;

        // Set the baud rate.
        regs.c4.modify(C4::BRFA.val(0));
        regs.bdh.modify(BDH::SBR.val((baud_counter >> 8) as u8));
        regs.bdl.set(baud_counter as u8);
    }

    pub fn enable_rx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c1.modify(C1::ILT::SET); // Idle after stop bit
        regs.c2.modify(C2::RE::SET);  // Enable UART reception
    }

    pub fn enable_rx_interrupts(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.rwfifo.set(1);               // Issue interrupt on each byte
        regs.c5.modify(C5::RDMAS::CLEAR); // Issue interrupt on RX data

        match self.index {
            0 => unsafe {nvic::enable(nvic::NvicIdx::UART0)},
            1 => unsafe {nvic::enable(nvic::NvicIdx::UART1)},
            2 => unsafe {nvic::enable(nvic::NvicIdx::UART2)},
            3 => unsafe {nvic::enable(nvic::NvicIdx::UART3)},
            4 => unsafe {nvic::enable(nvic::NvicIdx::UART4)},
            _ => unreachable!()
        };
        regs.c2.modify(C2::RIE::SET);     // Enable interrupts
    }

    pub fn enable_tx(&self) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.c2.modify(C2::TE::SET);
    }

    fn enable_clock(&self) {
        use sim::{clocks, Clock};
        match self.index {
            0 => clocks::UART0.enable(),
            1 => clocks::UART1.enable(),
            2 => clocks::UART2.enable(),
            3 => clocks::UART3.enable(),
            4 => clocks::UART4.enable(),
            _ => unreachable!()
        };
    }

    pub fn send_byte(&self, byte: u8) {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };

        while !regs.s1.is_set(S1::TRDE) {}
        regs.d.set(byte);
    }

    pub fn tx_ready(&self) -> bool {
        let regs: &mut Registers = unsafe { mem::transmute(self.registers) };
        regs.s1.is_set(S1::TC)
    }
}


/// Implementation of kernel::hil::UART
impl hil::uart::UART for Uart {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: uart::UARTParams) {
        self.enable_clock();

        self.set_parity(params.parity);
        self.set_stop_bits(params.stop_bits);
        self.set_baud_rate(params.baud_rate);

        self.enable_rx();
        self.enable_rx_interrupts();
        self.enable_tx();
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        // This basic procedure outlined in section 59.9.3.
        for i in 0..tx_len {
            self.send_byte(tx_data[i]);
        }

        while !self.tx_ready() {}

        self.client.get().map(move |client|
            client.transmit_complete(tx_data, uart::Error::CommandComplete)
        );
    }

    #[allow(unused_variables)]
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        let mut length = rx_len;
        if rx_len > rx_buffer.len() {
            length = rx_buffer.len();
        }

        self.buffer.put(Some(rx_buffer));
        self.rx_len.set(length);
        self.rx_index.set(0);
    }
}

interrupt_handler!(uart0_handler, UART0);
interrupt_handler!(uart1_handler, UART1);
interrupt_handler!(uart2_handler, UART2);
interrupt_handler!(uart3_handler, UART3);
interrupt_handler!(uart4_handler, UART4);
interrupt_handler!(uart0_err_handler, UART0);
interrupt_handler!(uart1_err_handler, UART1);
interrupt_handler!(uart2_err_handler, UART2);
interrupt_handler!(uart3_err_handler, UART3);
interrupt_handler!(uart4_err_handler, UART4);
