/// Author: Niklas Adolfsson <niklasadolfsson1@gmail.com>
/// Date: July 8, 2017

use core::cell::Cell;
use kernel;
use nrf5x::pinmux;
use peripheral_registers;

// this could potentially be replaced to point directly to
// the WRITE_BUFFER in capsules::console::WRITE_BUFFER
const BUF_SIZE: usize = 64;
static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

// NRF UARTE Specific
const NRF_UARTE_INTR_ENDTX: u32 = 1 << 8;
const NRF_UARTE_INTR_ENDRX: u32 = 1 << 4;
const NRF_UARTE_ENABLE: u32 = 8;

pub struct UARTE {
    regs: *const peripheral_registers::UARTE,
    client: Cell<Option<&'static kernel::hil::uart::Client>>,
    buffer: kernel::common::take_cell::TakeCell<'static, [u8]>,
    remaining_bytes: Cell<usize>,
    offset: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

pub static mut UART0: UARTE = UARTE::new();

impl UARTE {
    pub const fn new() -> UARTE {
        UARTE {
            regs: peripheral_registers::UARTE_BASE as *mut peripheral_registers::UARTE,
            client: Cell::new(None),
            buffer: kernel::common::take_cell::TakeCell::empty(),
            remaining_bytes: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    pub fn configure(&self,
                     tx: pinmux::Pinmux,
                     rx: pinmux::Pinmux,
                     cts: pinmux::Pinmux,
                     rts: pinmux::Pinmux) {
        let regs = unsafe { &*self.regs };

        regs.pseltxd.set(tx);
        regs.pselrxd.set(rx);
        regs.pselcts.set(cts);
        regs.pselrts.set(rts);
    }

    fn set_baud_rate(&self, baud_rate: u32) {
        let regs = unsafe { &*self.regs };
        match baud_rate {
            1200 => regs.baudrate.set(0x0004F000),
            2400 => regs.baudrate.set(0x0009D000),
            4800 => regs.baudrate.set(0x0013B000),
            9600 => regs.baudrate.set(0x00275000),
            14400 => regs.baudrate.set(0x003AF000),
            19200 => regs.baudrate.set(0x004EA000),
            28800 => regs.baudrate.set(0x0075C000),
            38400 => regs.baudrate.set(0x009D0000),
            57600 => regs.baudrate.set(0x00EB0000),
            76800 => regs.baudrate.set(0x013A9000),
            115200 => regs.baudrate.set(0x01D60000),
            230400 => regs.baudrate.set(0x03B00000),
            250000 => regs.baudrate.set(0x04000000),
            460800 => regs.baudrate.set(0x07400000),
            921600 => regs.baudrate.set(0x0F000000),
            1000000 => regs.baudrate.set(0x10000000),
            _ => regs.baudrate.set(0x01D60000),       //setting default to 115200
        }
    }

    fn enable(&self) {
        let regs = unsafe { &*self.regs };
        regs.enable.set(NRF_UARTE_ENABLE);
    }

    #[allow(dead_code)]
    fn enable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(NRF_UARTE_INTR_ENDRX);
    }

    fn enable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(NRF_UARTE_INTR_ENDTX);
    }

    #[allow(dead_code)]
    fn disable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(NRF_UARTE_INTR_ENDRX);
    }

    fn disable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(NRF_UARTE_INTR_ENDTX);
    }

    #[inline(never)]
    // only TX supported here
    pub fn handle_interrupt(&mut self) {
        // disable interrupts
        self.disable_tx_interrupts();

        let regs = unsafe { &*self.regs };
        let tx = regs.event_endtx.get() != 0;

        if tx {
            regs.event_endtx.set(0 as u32);
            regs.task_stoptx.set(1 as u32);
            let tx_bytes = regs.txd_amount.get() as usize;
            let rem = self.remaining_bytes.get();

            // More bytes transmitted than requested
            // Should not happen
            // FIXME: Propogate error to the UART capsule?!
            if tx_bytes > rem {
                debug!("error more bytes than requested\r\n");
                return;
            }

            self.remaining_bytes.set(rem - tx_bytes);
            self.offset.set(tx_bytes);

            if self.remaining_bytes.get() == 0 {
                // Signal client write done
                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.transmit_complete(buffer, kernel::hil::uart::Error::CommandComplete);
                    });
                });
            }
            // This has been tested however this will only occur if the UART for some reason
            // could not transmit the entire buffer
            else {
                self.set_dma_pointer_to_buffer();
                regs.task_starttx.set(1);
                self.enable_tx_interrupts();
            }
        }
    }

    pub unsafe fn send_byte(&self, byte: u8) {
        let regs = &*self.regs;

        self.remaining_bytes.set(1);
        self.offset.set(0);
        regs.event_endtx.set(0);
        BUF[0] = byte;
        self.set_dma_pointer_to_buffer();
        regs.txd_maxcnt.set(1);
        regs.task_starttx.set(1);

        self.enable_tx_interrupts();
    }

    pub fn tx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_endtx.get() & 0b1 != 0
    }

    fn set_dma_pointer_to_buffer(&self) {
        let regs = unsafe { &*self.regs };
        unsafe { regs.txd_ptr.set((&BUF[self.offset.get()] as *const u8) as u32) }
    }

    fn copy_data_to_uart_buffer(&self, tx_len: usize) {
        self.buffer.map(|buffer| for i in 0..tx_len {
            unsafe { BUF[i] = buffer[i] }
        });
    }
}

impl kernel::hil::uart::UART for UARTE {
    fn set_client(&self, client: &'static kernel::hil::uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: kernel::hil::uart::UARTParams) {
        self.enable();
        self.set_baud_rate(params.baud_rate);
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let regs = unsafe { &*self.regs };

        if tx_len == 0 {
            return;
        }

        self.remaining_bytes.set(tx_len);
        self.offset.set(0);
        self.buffer.replace(tx_data);

        // configure dma to point to the the buffer 'BUF'
        self.copy_data_to_uart_buffer(tx_len);

        self.set_dma_pointer_to_buffer();
        // configure length of the buffer to transmit

        regs.txd_maxcnt.set(tx_len as u32);
        regs.event_endtx.set(0);
        regs.task_starttx.set(1);

        self.enable_tx_interrupts();
    }

    #[allow(unused)]
    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        unimplemented!()
    }
}
