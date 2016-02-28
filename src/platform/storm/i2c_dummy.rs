///! A dummy I2C client

use sam4l::i2c;
use hil;
use core::cell::Cell;

struct DummyI2CClient { devid: Cell<u8> }

static mut I2C_Client : DummyI2CClient = DummyI2CClient { devid: Cell::new(1) };

impl hil::i2c::I2CClient for DummyI2CClient {
    fn command_complete(&self, _buffer: &'static mut [u8], error: hil::i2c::Error) {
        let mut devid = self.devid.get();

        match error {
            hil::i2c::Error::CommandComplete => println!("0x{:x}", devid),
            _ => {}
        }

        let dev = unsafe { &mut i2c::I2C2 };
        let buffer = unsafe { &mut DATA };
        if devid < 0x7F {
            devid += 1;
            self.devid.set(devid);
            dev.write(devid, i2c::START | i2c::STOP, buffer );
        }
    }
}

static mut DATA : [u8; 1] = [0; 1];

pub fn i2c_scan_slaves() {
    use hil::i2c::I2C;

    let dev = unsafe { &mut i2c::I2C2 };

    let i2c_client = unsafe { &I2C_Client };
    dev.set_client(i2c_client);
    dev.enable();

    println!("Try writing to a non-existent I2C device");
    dev.write(i2c_client.devid.get(), i2c::START | i2c::STOP, unsafe { &mut DATA} );
}

