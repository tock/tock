use kernel::common::cells::MapCell;
use kernel::hil::uart;
use kernel::ReturnCode;

/*
    #############################################
    // Add the snippet below to main if you want to enable echo on UART0
    // Create a virtual device for echo test
    let echo0 = static_init!(
            uart_echo::UartEcho<cc26x2::uart::UART, cc26x2::uart::UART>,
            uart_echo::UartEcho::new(
                &cc26x2::uart::UART0,
                &cc26x2::uart::UART0,
                &mut uart_echo::OUT_BUF0,
                &mut uart_echo::IN_BUF0,
            )
        );
    hil::uart::Transmit::set_transmit_client(&cc26x2::uart::UART0, echo0);
    hil::uart::Receive::set_receive_client(&cc26x2::uart::UART0, echo0);
    // change initialize to false to test buffer mode
    echo0.initialize(true);

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

use kernel::debug;

impl<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>>
    UartEcho<UTx, URx>
{
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

    pub fn initialize(&self, word_mode: bool) {
        if word_mode {
            debug!("putting self as receive word");
            self.uart_rx.receive_word();
        } else {
            self.rx_buf.take().map(|buf| {
                self.uart_rx.receive_buffer(buf, MAX_PAYLOAD);
            });
        }
    }
}

impl<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>>
    uart::TransmitClient for UartEcho<UTx, URx>
{
    fn transmitted_buffer(&self, buffer: &'static mut [u8], _len: usize, _rcode: ReturnCode) {
        self.tx_buf.put(buffer);
    }
}

impl<UTx: 'static + uart::Transmit<'static>, URx: 'static + uart::Receive<'static>>
    uart::ReceiveClient for UartEcho<UTx, URx>
{
    fn received_word(&self, word: u32, _rval: ReturnCode, _error: uart::Error) {
        if word == b'\r' as u32 {
            self.tx_buf.take().map(|buf| {
                buf[0] = b'\r';
                buf[1] = b'\n';
                // output on uart
                self.uart_tx.transmit_buffer(buf, 2)
            });
        // if no buffer, these bytes will be dropped
        } else {
            self.uart_tx.transmit_word(word);
        }
        self.uart_rx.receive_word();
    }

    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        _rcode: ReturnCode,
        _error: uart::Error,
    ) {
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
        self.tx_buf.take().map(|buf| {
            self.uart_tx
                .transmit_buffer(buf, rx_len + added_carraige_returns)
        });
    }
}
