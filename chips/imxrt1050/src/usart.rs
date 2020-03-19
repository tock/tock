use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::registers::{register_bitfields, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;
use kernel::ClockInterface;
use kernel::ReturnCode;

use cortex_m_semihosting::{hprint, hprintln};
use crate::ccm;

/// Universal synchronous asynchronous receiver transmitter
#[repr(C)]
struct UsartRegisters {

}

// register_bitfields![];

// const USART2_BASE: StaticRef<UsartRegisters> =
//     unsafe { StaticRef::new(0x40004400 as *const UsartRegisters) };
// const USART3_BASE: StaticRef<UsartRegisters> =
//     unsafe { StaticRef::new(0x40004800 as *const UsartRegisters) };

// #[allow(non_camel_case_types)]
// #[derive(Copy, Clone, PartialEq)]
// enum USARTStateRX {
//     Idle,
//     DMA_Receiving,
// }

// #[allow(non_camel_case_types)]
// #[derive(Copy, Clone, PartialEq)]
// enum USARTStateTX {
//     Idle,
//     DMA_Transmitting,
//     Transfer_Completing, // DMA finished, but not all bytes sent
// }

pub struct Usart<'a> {
    // clock: UsartClock,
    tx_client: OptionalCell<&'a dyn hil::uart::TransmitClient>,
    tx_buffer: TakeCell<'static, [u8]>  
}

pub static mut USART_SEMIHOSTING: Usart = Usart::new();


impl Usart<'a> {
    const fn new() -> Usart<'a> {
        Usart {
            // clock: clock,
            tx_client: OptionalCell::empty(),
            tx_buffer: TakeCell::empty(),
        }
    }

    pub fn is_enabled_clock(&self) -> bool {
        // self.clock.is_enabled()
        true
    }

    pub fn enable_clock(&self) {
        // self.clock.enable();
    }

    pub fn disable_clock(&self) {
        // self.clock.disable();
    }

    // According to section 25.4.13, we need to make sure that USART TC flag is
    // set before disabling the DMA TX on the peripheral side.
    pub fn handle_interrupt(&self) {
        // self.clear_transmit_complete();
        // self.disable_transmit_complete_interrupt();

        // // Ignore if USARTStateTX is in some other state other than
        // // Transfer_Completing.
        // if self.usart_tx_state.get() == USARTStateTX::Transfer_Completing {
        //     self.disable_tx();
        //     self.usart_tx_state.set(USARTStateTX::Idle);

        //     // get buffer
        //     let buffer = self.tx_dma.map_or(None, |tx_dma| tx_dma.return_buffer());
        //     let len = self.tx_len.get();
        //     self.tx_len.set(0);

        //     // alert client
        // }
    }


    // for use by panic in io.rs
    pub fn send_byte(&self, byte: u8) {
        // loop till TXE (Transmit data register empty) becomes 1
        // while !self.registers.sr.is_set(SR::TXE) {}

        // self.registers.dr.set(byte.into());
        hprint!("{}", char::from(byte)).unwrap();
    }
}

impl hil::uart::Transmit<'a> for Usart<'a> {
    fn set_transmit_client(&self, client: &'a dyn hil::uart::TransmitClient) {
        self.tx_client.set(client);
    }

    fn transmit_buffer(
        &self,
        tx_data: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {

        for byte in tx_data.iter() {
            self.send_byte(*byte);
        }

        hprintln!("[usart] Ba am trimis datele").unwrap();
        self.tx_buffer.put(Some(tx_data));
        self.tx_client.map(|client| {
            if let Some(buf) = self.tx_buffer.take() {
                client.transmitted_buffer(buf, tx_len, ReturnCode::SUCCESS);
            }
        });
        (ReturnCode::SUCCESS, None)
    }

    fn transmit_word(&self, _word: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn transmit_abort(&self) -> ReturnCode {
        ReturnCode::SUCCESS
    }
}

impl hil::uart::Configure for Usart<'a> {
    fn configure(&self, params: hil::uart::Parameters) -> ReturnCode {
        // if params.baud_rate != 115200
        //     || params.stop_bits != hil::uart::StopBits::One
        //     || params.parity != hil::uart::Parity::None
        //     || params.hw_flow_control != false
        //     || params.width != hil::uart::Width::Eight
        // {
        //     panic!(
        //         "Currently we only support uart setting of 115200bps 8N1, no hardware flow control"
        //     );
        // }

        // // Configure the word length - 0: 1 Start bit, 8 Data bits, n Stop bits
        // self.registers.cr1.modify(CR1::M::CLEAR);

        // // Set the stop bit length - 00: 1 Stop bits
        // self.registers.cr2.modify(CR2::STOP.val(0b00 as u32));

        // // Set no parity
        // self.registers.cr1.modify(CR1::PCE::CLEAR);

        // // Set the baud rate. By default OVER8 is 0 (oversampling by 16) and
        // // PCLK1 is at 16Mhz. The desired baud rate is 115.2KBps. So according
        // // to Table 149 of reference manual, the value for BRR is 8.6875
        // // DIV_Fraction = 0.6875 * 16 = 11 = 0xB
        // // DIV_Mantissa = 8 = 0x8
        // self.registers.brr.modify(BRR::DIV_Fraction.val(0xB as u32));
        // self.registers.brr.modify(BRR::DIV_Mantissa.val(0x8 as u32));

        // // Enable transmit block
        // self.registers.cr1.modify(CR1::TE::SET);

        // // Enable receive block
        // self.registers.cr1.modify(CR1::RE::SET);

        // // Enable USART
        // self.registers.cr1.modify(CR1::UE::SET);

        ReturnCode::SUCCESS
    }
}

impl hil::uart::Receive<'a> for Usart<'a> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        // self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::SUCCESS, None)
    }

    fn receive_word(&self) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn receive_abort(&self) -> ReturnCode {
        ReturnCode::EBUSY
    }
}

impl hil::uart::UartData<'a> for Usart<'a> {}
impl hil::uart::Uart<'a> for Usart<'a> {}