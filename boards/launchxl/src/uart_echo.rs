use kernel::common::cells::MapCell;
use kernel::hil::uart;
use kernel::hil::uart::{Client, UART};
/*
    Add the snippet below to main if you want to enable echo

    // Create a virtual device for echo test
    let echo_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    echo_uart.setup();
    let echo = static_init!(
            uart_echo::UartEcho<UartDevice>,
            uart_echo::UartEcho::new(
                echo_uart, 
                &mut uart_echo::OUT_BUF,
                &mut uart_echo::IN_BUF,
            )
        );
    hil::uart::UART::set_client(echo_uart, echo);
    echo.initialize();
*/

const DEFAULT_BAUD: u32 = 115200;

const MAX_PAYLOAD: usize = 1;

pub static mut OUT_BUF: [u8; MAX_PAYLOAD * 2] = [0; MAX_PAYLOAD + 1];
pub static mut IN_BUF: [u8; MAX_PAYLOAD] = [0; MAX_PAYLOAD];

pub struct UartEcho<U: 'static + UART> {
    uart: &'static U,
    baud: u32,
    tx_buf: MapCell<&'static mut [u8]>,
    rx_buf: MapCell<&'static mut [u8]>,
}

impl<U: 'static + UART> UartEcho<U> {
    pub fn new(
        uart: &'static U,
        tx_buf: &'static mut [u8],
        rx_buf: &'static mut [u8],
    ) -> UartEcho<U> {
        uart.configure(uart::UARTParameters {
            baud_rate: DEFAULT_BAUD,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
        UartEcho {
            uart: &uart,
            baud: DEFAULT_BAUD,
            tx_buf: MapCell::new(tx_buf),
            rx_buf: MapCell::new(rx_buf),
        }
    }

    pub fn initialize(&self) {
        self.rx_buf.take().map(|buf| {
            self.uart.receive(buf, MAX_PAYLOAD);
        });
    }
}

impl<U: 'static + UART> Client for UartEcho<U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        self.tx_buf.put(buffer);
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, _error: uart::Error) {
        // copy into tx buf
        let mut added_carraige_returns = 0;
        for n in 0..rx_len {
            self.tx_buf.map(|buf| {
                buf[n + added_carraige_returns] = buffer[n];
                if buffer[n] == b'\r' {
                    buf[n + 1] = b'\n';
                    added_carraige_returns = 1;
                }
            });
        }
        // give buffer back to uart
        self.uart.receive(buffer, MAX_PAYLOAD);

        // output on uart
        self.tx_buf
            .take()
            .map(|buf| self.uart.transmit(buf, rx_len + added_carraige_returns));
    }
}
