//! UART driver, cc26x2 family
use gpio;
use ioc;
use kernel;
use kernel::common::cells::OptionalCell;
use kernel::common::registers::{ReadOnly, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::gpio::Pin;
use kernel::hil::uart;
use kernel::ReturnCode;
use prcm;

const MCU_CLOCK: u32 = 48_000_000;

#[repr(C)]
struct UartRegisters {
    dr: ReadWrite<u32>,
    rsr_ecr: ReadWrite<u32>,
    _reserved0: [u32; 0x4],
    fr: ReadOnly<u32, Flags::Register>,
    _reserved1: [u32; 0x2],
    ibrd: ReadWrite<u32, IntDivisor::Register>,
    fbrd: ReadWrite<u32, FracDivisor::Register>,
    lcrh: ReadWrite<u32, LineControl::Register>,
    ctl: ReadWrite<u32, Control::Register>,
    ifls: ReadWrite<u32>,
    imsc: ReadWrite<u32, Interrupts::Register>,
    ris: ReadOnly<u32, Interrupts::Register>,
    mis: ReadOnly<u32, Interrupts::Register>,
    icr: WriteOnly<u32, Interrupts::Register>,
    dmactl: ReadWrite<u32>,
}

pub static mut UART0: UART = UART::new();

register_bitfields![
    u32,
    Control [
        UART_ENABLE OFFSET(0) NUMBITS(1) [],
        TX_ENABLE OFFSET(8) NUMBITS(1) [],
        RX_ENABLE OFFSET(9) NUMBITS(1) []
    ],
    LineControl [
        FIFO_ENABLE OFFSET(4) NUMBITS(1) [],
        WORD_LENGTH OFFSET(5) NUMBITS(2) [
            Len5 = 0x0,
            Len6 = 0x1,
            Len7 = 0x2,
            Len8 = 0x3
        ]
    ],
    IntDivisor [
        DIVISOR OFFSET(0) NUMBITS(16) []
    ],
    FracDivisor [
        DIVISOR OFFSET(0) NUMBITS(6) []
    ],
    Flags [
        TX_FIFO_FULL OFFSET(5) NUMBITS(1) []
    ],
    Interrupts [
        ALL_INTERRUPTS OFFSET(0) NUMBITS(12) []
    ]
];

const UART_BASE: StaticRef<UartRegisters> =
    unsafe { StaticRef::new(0x40001000 as *const UartRegisters) };

pub struct UART {
    registers: StaticRef<UartRegisters>,
    client: OptionalCell<&'static uart::Client>,
    tx_pin: OptionalCell<u8>,
    rx_pin: OptionalCell<u8>,
}

impl UART {
    const fn new() -> UART {
        UART {
            registers: UART_BASE,
            client: OptionalCell::empty(),
            tx_pin: OptionalCell::empty(),
            rx_pin: OptionalCell::empty(),
        }
    }

    /// Initialize the UART hardware.
    ///
    /// This function needs to be run before the UART module is used.
    pub fn initialize_and_set_pins(&self, tx_pin: u8, rx_pin: u8) {
        self.tx_pin.set(tx_pin);
        self.rx_pin.set(rx_pin);
        self.power_and_clock();
        self.disable_interrupts();
    }

    fn configure(&self, params: kernel::hil::uart::UARTParameters) -> ReturnCode {
        // These could probably be implemented, but are currently ignored, so
        // throw an error.
        if params.stop_bits != kernel::hil::uart::StopBits::One {
            return ReturnCode::ENOSUPPORT;
        }
        if params.parity != kernel::hil::uart::Parity::None {
            return ReturnCode::ENOSUPPORT;
        }
        if params.hw_flow_control != false {
            return ReturnCode::ENOSUPPORT;
        }

        self.tx_pin.map_or(ReturnCode::EOFF, |tx_pin| {
            self.rx_pin.map_or(ReturnCode::EOFF, |rx_pin| {
                unsafe {
                    // Make sure the TX pin is output/high before assigning it to UART control
                    // to avoid falling edge glitches
                    gpio::PORT[*tx_pin as usize].make_output();
                    gpio::PORT[*tx_pin as usize].set();

                    // Map UART signals to IO pin
                    ioc::IOCFG[*tx_pin as usize].enable_uart_tx();
                    ioc::IOCFG[*rx_pin as usize].enable_uart_rx();
                }

                // Disable the UART before configuring
                self.disable();

                self.set_baud_rate(params.baud_rate);

                // Set word length
                let regs = &*self.registers;
                regs.lcrh.write(LineControl::WORD_LENGTH::Len8);

                self.fifo_enable();

                // Enable UART, RX and TX
                regs.ctl.write(
                    Control::UART_ENABLE::SET + Control::RX_ENABLE::SET + Control::TX_ENABLE::SET,
                );

                ReturnCode::SUCCESS
            })
        })
    }

    fn power_and_clock(&self) {
        prcm::Power::enable_domain(prcm::PowerDomain::Serial);
        while !prcm::Power::is_enabled(prcm::PowerDomain::Serial) {}
        prcm::Clock::enable_uart();
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        // Fractional baud rate divider
        let div = (((MCU_CLOCK * 8) / baud_rate) + 1) / 2;
        // Set the baud rate
        let regs = &*self.registers;
        regs.ibrd.write(IntDivisor::DIVISOR.val(div / 64));
        regs.fbrd.write(FracDivisor::DIVISOR.val(div % 64));
    }

    fn fifo_enable(&self) {
        let regs = &*self.registers;
        regs.lcrh.modify(LineControl::FIFO_ENABLE::SET);
    }

    fn fifo_disable(&self) {
        let regs = &*self.registers;
        regs.lcrh.modify(LineControl::FIFO_ENABLE::CLEAR);
    }

    fn disable(&self) {
        self.fifo_disable();
        let regs = &*self.registers;
        regs.ctl.modify(
            Control::UART_ENABLE::CLEAR + Control::TX_ENABLE::CLEAR + Control::RX_ENABLE::CLEAR,
        );
    }

    fn disable_interrupts(&self) {
        // Disable all UART interrupts
        let regs = &*self.registers;
        regs.imsc.modify(Interrupts::ALL_INTERRUPTS::CLEAR);
        // Clear all UART interrupts
        regs.icr.write(Interrupts::ALL_INTERRUPTS::SET);
    }

    /// Clears all interrupts related to UART.
    pub fn handle_interrupt(&self) {
        let regs = &*self.registers;
        // Clear interrupts
        regs.icr.write(Interrupts::ALL_INTERRUPTS::SET);
    }

    /// Transmits a single byte if the hardware is ready.
    pub fn send_byte(&self, c: u8) {
        // Wait for space in FIFO
        while !self.tx_ready() {}
        // Put byte in data register
        let regs = &*self.registers;
        regs.dr.set(c as u32);
    }

    /// Checks if there is space in the transmit fifo queue.
    pub fn tx_ready(&self) -> bool {
        let regs = &*self.registers;
        !regs.fr.is_set(Flags::TX_FIFO_FULL)
    }
}

impl kernel::hil::uart::UART for UART {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(client);
    }

    fn configure(&self, params: kernel::hil::uart::UARTParameters) -> ReturnCode {
        self.configure(params)
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        if tx_len == 0 {
            return;
        }

        for i in 0..tx_len {
            self.send_byte(tx_data[i]);
        }

        self.client.map(move |client| {
            client.transmit_complete(tx_data, kernel::hil::uart::Error::CommandComplete);
        });
    }

    #[allow(unused)]
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {}

    fn abort_receive(&self) {
        unimplemented!()
    }
}
