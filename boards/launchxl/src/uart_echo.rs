use kernel::common::cells::MapCell;
use kernel::hil::uart;
use kernel::ReturnCode;

/*
    #############################################
    // Add the snippet below to main if you want to enable echo on UART1 and UART2
    // Create a virtual device for echo test
    let echo0_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    echo0_uart.setup();
    let echo0 = static_init!(
            uart_echo::UartEcho<UartDevice, UartDevice>,
            uart_echo::UartEcho::new(
                echo0_uart,
                echo0_uart,
                &mut uart_echo::OUT_BUF0,
                &mut uart_echo::IN_BUF0,
            )
        );

    hil::uart::UART::set_client(echo0_uart, echo0);
    echo0.initialize();

    // Directly hook up UART1 for echo test
    let echo1 = static_init!(
            uart_echo::UartEcho<cc26x2::uart::UART, cc26x2::uart::UART>,
            uart_echo::UartEcho::new(
                &cc26x2::uart::UART1,
                &cc26x2::uart::UART1,
                &mut uart_echo::OUT_BUF1,
                &mut uart_echo::IN_BUF1,
            )
        );
    hil::uart::UART::set_client(&cc26x2::uart::UART1, echo1);
    cc26x2::uart::UART1.initialize();
    cc26x2::uart::UART1.configure(uart_echo::UART_PARAMS);
    echo1.initialize();

    #############################################
    // Add the snipper below to main if you want to criss-cross TX/RX of UART0/1
    // Create a virtual device for echo test
    let echo0_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    echo0_uart.setup();
    let echo0 = static_init!(
            uart_echo::UartEcho<UartDevice, cc26x2::uart::UART>,
            uart_echo::UartEcho::new(
                echo0_uart,
                &cc26x2::uart::UART1,
                &mut uart_echo::OUT_BUF0,
                &mut uart_echo::IN_BUF0,
            )
        );
    hil::uart::UART::set_client(echo0_uart, echo0);
    cc26x2::uart::UART1.set_rx_client(echo0);

    echo0.initialize();

    // Create a virtual device for echo test
    let echo1_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    echo1_uart.setup();
    let echo1 = static_init!(
            uart_echo::UartEcho<cc26x2::uart::UART, UartDevice>,
            uart_echo::UartEcho::new(
                &cc26x2::uart::UART1,
                echo1_uart,
                &mut uart_echo::OUT_BUF1,
                &mut uart_echo::IN_BUF1,
            )
        );
    cc26x2::uart::UART1.set_tx_client(echo1);
    hil::uart::UART::set_client(echo1_uart, echo1);

    cc26x2::uart::UART1.initialize();
    cc26x2::uart::UART1.configure(uart_echo::UART_PARAMS);
    echo1.initialize();
*/

const DEFAULT_BAUD: u32 = 115200;

const MAX_PAYLOAD: usize = 1;

pub const UART_PARAMS: uart::Parameters = uart::Parameters {
    baud_rate: DEFAULT_BAUD,
    stop_bits: uart::StopBits::One,
    parity: uart::Parity::None,
    hw_flow_control: false,
    width: uart::Width::Eight,
};

pub static mut OUT_BUF0: [u8; MAX_PAYLOAD * 2] = [0; MAX_PAYLOAD * 2];
pub static mut IN_BUF0: [u8; MAX_PAYLOAD] = [0; MAX_PAYLOAD];
pub static mut OUT_BUF1: [u8; MAX_PAYLOAD * 2] = [0; MAX_PAYLOAD * 2];
pub static mut IN_BUF1: [u8; MAX_PAYLOAD] = [0; MAX_PAYLOAD];

// just in case you want to mix and match UART types (eg: one is muxed, one is direct)
pub struct UartEcho<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>> {
    uart_tx: &'static UTx,
    uart_rx: &'static URx,
    baud: u32,
    tx_buf: MapCell<&'static mut [u8]>,
    rx_buf: MapCell<&'static mut [u8]>,
}

impl<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>> UartEcho<UTx, URx> {
    pub fn new(
        uart_tx: &'static UTx,
        uart_rx: &'static URx,
        tx_buf: &'static mut [u8],
        rx_buf: &'static mut [u8],
    ) -> UartEcho<UTx, URx> {
        assert!(
            tx_buf.len() > rx_buf.len(),
            "UartEcho has improperly sized buffers"
        );
        UartEcho {
            uart_tx: &uart_tx,
            uart_rx: &uart_rx,
            baud: DEFAULT_BAUD,
            tx_buf: MapCell::new(tx_buf),
            rx_buf: MapCell::new(rx_buf),
        }
    }

    pub fn initialize(&self) {
        self.rx_buf.take().map(|buf| {
            self.uart_rx.receive_buffer(buf, MAX_PAYLOAD);
        });
    }
}

impl<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>> uart::TransmitClient for UartEcho<UTx, URx> {
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _len: usize, _rcode: ReturnCode) {
        self.tx_buf.put(buffer);
    }
}


impl<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>> uart::ReceiveClient for UartEcho<UTx, URx> {
    fn received_buffer(&self, buffer: &'static mut [u8], rx_len: usize, _rcode: ReturnCode,  _error: uart::Error) {
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
        self.uart_rx.receive_buffer(buffer, MAX_PAYLOAD);

        // output on uart
        self.tx_buf
            .take()
            .map(|buf| self.uart_tx.transmit_buffer(buf, rx_len + added_carraige_returns));
    }
}
