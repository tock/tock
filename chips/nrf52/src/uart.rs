use chip;
use core::cell::Cell;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil::uart;
use nvic;
use peripheral_interrupts::NvicIdx;
use pinmux::Pinmux;

const BUF_SIZE: usize = 48;
static mut BUF: [u8; BUF_SIZE] = [0; BUF_SIZE];

#[repr(C, packed)]
pub struct Registers {
    pub task_startrx: VolatileCell<u32>,            // 0x000-0x004
    pub task_stoprx: VolatileCell<u32>,             // 0x004-0x008
    pub task_starttx: VolatileCell<u32>,            // 0x008-0x00c
    pub task_stoptx: VolatileCell<u32>,             // 0x00c-0x010
    _reserved1: [u32; 7],                           // 0x010-0x02c
    pub task_flush_rx: VolatileCell<u32>,           // 0x02c-0x030
    _reserved2: [u32; 52],                          // 0x030-0x100
    pub event_cts: VolatileCell<u32>,               // 0x100-0x104
    pub event_ncts: VolatileCell<u32>,              // 0x104-0x108
    _reserved3: [u32; 2],                           // 0x108-0x110
    pub event_endrx: VolatileCell<u32>,             // 0x110-0x114
    _reserved4: [u32; 3],                           // 0x114-0x120
    pub event_endtx: VolatileCell<u32>,             // 0x120-0x124
    pub event_error: VolatileCell<u32>,             // 0x124-0x128
    _reserved6: [u32; 7],                           // 0x128-0x144
    pub event_rxto: VolatileCell<u32>,              // 0x144-0x148
    _reserved7: [u32; 1],                           // 0x148-0x14C
    pub event_rxstarted: VolatileCell<u32>,         // 0x14C-0x150
    pub event_txstarted: VolatileCell<u32>,         // 0x150-0x154
    _reserved8: [u32; 1],                           // 0x154-0x158
    pub event_txstopped: VolatileCell<u32>,         // 0x158-0x15c
    _reserved9: [u32; 41],                          // 0x15c-0x200
    pub shorts: VolatileCell<u32>,                  // 0x200-0x204
    _reserved10: [u32; 64],                         // 0x204-0x304
    pub intenset: VolatileCell<u32>,                // 0x304-0x308
    pub intenclr: VolatileCell<u32>,                // 0x308-0x30C
    _reserved11: [u32; 93],                         // 0x30C-0x480
    pub errorsrc: VolatileCell<u32>,                // 0x480-0x484
    _reserved12: [u32; 31],                         // 0x484-0x500
    pub enable: VolatileCell<u32>,                  // 0x500-0x504
    _reserved13: [u32; 1],                          // 0x504-0x508
    pub pselrts: VolatileCell<Pinmux>,              // 0x508-0x50c
    pub pseltxd: VolatileCell<Pinmux>,              // 0x50c-0x510
    pub pselcts: VolatileCell<Pinmux>,              // 0x510-0x514
    pub pselrxd: VolatileCell<Pinmux>,              // 0x514-0x518
    _reserved14: [u32; 3],                          // 0x518-0x524
    pub baudrate: VolatileCell<u32>,                // 0x524-0x528
    _reserved15: [u32; 3],                          // 0x528-0x534
    pub rxd_ptr: VolatileCell<u32>,                 // 0x534-0x538
    pub rxd_maxcnt: VolatileCell<u32>,              // 0x538-0x53c
    pub rxd_amount: VolatileCell<u32>,              // 0x53c-0x540
    _reserved16: [u32; 1],                          // 0x540-0x544
    pub txd_ptr: VolatileCell<u32>,                 // 0x544-0x548
    pub txd_maxcnt: VolatileCell<u32>,              // 0x548-0x54c
    pub txd_amount: VolatileCell<u32>,              // 0x54c-0x550
    _reserved17: [u32; 7],                          // 0x550-0x56C
    pub config: VolatileCell<u32>,                  // 0x56C-0x570
    //_reserved18: [u32; 675],
    //pub power: VolatileCell<u32>,
}

pub const UART_BASE: u32 = 0x40002000;

pub struct UART {
    regs: *const Registers,
    client: Cell<Option<&'static uart::Client>>,
    buffer: TakeCell<'static, [u8]>,
    len: Cell<usize>,
    index: Cell<usize>,
}

#[derive(Copy, Clone)]
pub struct UARTParams {
    pub baud_rate: u32,
}

pub static mut UART0: UART = UART::new();

// This UART implementation uses pins 5-8:
//   pin 5: RTS
//   pin 6: TX
//   pin 7 CTS
//   pin 8: RX

impl UART {
    pub const fn new() -> UART {
        UART {
            regs: UART_BASE as *mut Registers,
            client: Cell::new(None),
            buffer: TakeCell::empty(),
            len: Cell::new(0),
            index: Cell::new(0),
        }
    }

    pub fn configure(&self, tx: Pinmux, rx: Pinmux, cts: Pinmux, rts: Pinmux) {
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

    pub fn enable(&self) {
        let regs = unsafe { &*self.regs };
        regs.enable.set(8);
    }

    pub fn enable_nvic(&self) {
        nvic::enable(NvicIdx::UART0);
    }

    pub fn disable_nvic(&self) {
        nvic::disable(NvicIdx::UART0);
    }

    pub fn enable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(1 << 3 as u32);
    }

    #[inline(never)]
    #[no_mangle]
    pub fn enable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenset.set(1 << 8 as u32);
    }

    pub fn disable_rx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(1 << 3 as u32);
    }

    #[inline(never)]
    #[no_mangle]
    pub fn disable_tx_interrupts(&self) {
        let regs = unsafe { &*self.regs };
        regs.intenclr.set(1 << 8 as u32);
    }

    #[inline(never)]
    #[no_mangle]
    pub fn handle_interrupt(&mut self) {
        let regs = unsafe { &*self.regs };
        let tx = regs.event_endtx.get() != 0;

        if tx {
            if self.index.get() == self.len.get() {
                regs.event_endtx.set(0 as u32);
                regs.task_stoptx.set(1 as u32);
                // Signal client write done
                self.client.get().map(|client| {
                    self.buffer.take().map(|buffer| {
                        client.transmit_complete(buffer, uart::Error::CommandComplete);
                    });
                });
            }
            //FIXME: add support for longer than messages than BUF_SIZE
        }
    }

    #[inline(never)]
    #[no_mangle]
    pub unsafe fn send_byte(&self, byte: u8) {
        let regs = &*self.regs;

        self.index.set(1);
        self.len.set(1);

        regs.event_endtx.set(0);
        BUF[0] = byte;
        regs.txd_ptr.set((&BUF as *const u8) as u32);
        regs.txd_maxcnt.set(1);
        regs.task_starttx.set(1);

        self.enable_tx_interrupts();
        self.enable_nvic();
    }

    pub fn tx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_endtx.get() & 0b1 != 0
    }

    fn rx_ready(&self) -> bool {
        let regs = unsafe { &*self.regs };
        regs.event_endrx.get() & 0b1 != 0
    }
}

impl uart::UART for UART {
    fn set_client(&self, client: &'static uart::Client) {
        self.client.set(Some(client));
    }

    fn init(&self, params: uart::UARTParams) {
        self.enable();
        self.set_baud_rate(params.baud_rate);
    }

    #[inline(never)]
    #[no_mangle]
    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        let regs = unsafe { &*self.regs };

        if tx_len == 0 {
            return;
        }

        // copy data to buffer
        // FIXME: move to a function and this can crash with a bigger buffer than BUF_SIZE
        for (i, c) in tx_data.as_ref()[0..tx_len].iter().enumerate() {
            unsafe { BUF[i] = *c; }
        }

        self.buffer.replace(tx_data);

        self.len.set(tx_len);
        if tx_len > BUF_SIZE {
            self.index.set(BUF_SIZE);
        }
        else {
            self.index.set(tx_len);
        }

        regs.event_endtx.set(0);
        // assign pointer the address of first byte of the buffer to transmit
        unsafe { regs.txd_ptr.set((&BUF as *const u8) as u32); }
        // length of the buffer to transmit
        regs.txd_maxcnt.set(tx_len as u32);
        regs.task_starttx.set(1);
        self.enable_tx_interrupts();
        self.enable_nvic();
    }

    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        let regs = unsafe { &*self.regs };
        regs.task_startrx.set(1);
        let mut i = 0;
        while i < rx_len {
            while !self.rx_ready() {}
            rx_buffer[i] = regs.rxd_ptr.get() as u8;
            i += 1;
        }
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn UART0_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::UART0);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::UART0);
}
