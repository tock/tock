use common::take_cell::TakeCell;
use hil::Driver;
use hil::uart::{UART, Client};


pub static mut WRITE_BUF : [u8; 1024] = [0; 1024];

pub struct UartLoop<'a, U: UART + 'a> {
    uart: &'a U,
    buffer: TakeCell<&'static mut [u8]>
}

impl<'a, U: UART> UartLoop<'a, U> {
    pub const fn new(uart: &'a U, buffer: &'static mut [u8]) -> UartLoop<'a, U> {
        UartLoop {
            uart: uart,
            buffer: TakeCell::new(buffer)
        }
    }

    pub fn initialize(&self) {
        self.uart.enable_tx();
        self.uart.enable_rx();
    }
}


impl<'a, U: UART> Client for UartLoop<'a, U> {
    fn write_done(&self, buffer: &'static mut [u8]) {
      // Write TX is done, notify appropriate app and start another
      // transaction if pending

      // Clear pin using direct MMIO
      unsafe {
        // Clear
        asm!("\
            movw r3, 0x1058    \n\
            movt r3, 0x400E    \n\
            movs r4, 0x1000    \n\
            str  r4, [r3]      \n\
            "
            :               /* output */
            :               /* input */
            : "r3", "r4"    /* clobbers */
            : "volatile"
            );

        for n in 0..5000 {
          asm!("nop" :::: "volatile");asm!("nop" :::: "volatile");
        }
        static mut bigbuf : [u8; 1000] = ['b' as u8; 1000];
        bigbuf[999] = '\n' as u8;
        bigbuf[0] = '0' as u8;
        bigbuf[9] = '\n' as u8;
        bigbuf[10] = '!' as u8;

        // Set
        asm!("\
            movw r3, 0x1054    \n\
            movt r3, 0x400E    \n\
            movs r4, 0x1000    \n\
            str  r4, [r3]      \n\
            "
            :               /* output */
            :               /* input */
            : "r3", "r4"    /* clobbers */
            : "volatile"
            );
        self.uart.send_bytes(&mut bigbuf, 1000);

        // RESULTS
        // bytes ,    ms
        //    1  ,   0.0178
        //    5  ,   0.387
        //   10  ,   0.993
        //   50  ,   5.84
        //  100  ,  11.9
        //  500  ,  60.4
        // 1000  , 121.
      }
    }

    fn read_done(&self, c: u8) {
    }
}

