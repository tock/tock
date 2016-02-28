///! A dummy I2C client

use sam4l::i2c;
use hil;

struct DummyI2CClient;

static mut I2C_Client : DummyI2CClient = DummyI2CClient;

impl hil::i2c::I2CClient for DummyI2CClient {
    fn command_complete(&self, _buffer: &'static mut [u8]) {
        panic!("Command complete!!");

    }

    fn command_error(&self, _buffer: &'static mut [u8], error: hil::i2c::Error) {
        println!("Expected error {}", error);
    }
}

pub fn i2c_dummy_test(_dev: &'static mut i2c::I2CDevice) {
    use hil::i2c::I2C;

    static mut DATA : [u8; 2] = [0; 2];

    let dev = unsafe { &mut i2c::I2C2 };

    let i2c_client = unsafe { &I2C_Client };
    dev.set_client(i2c_client);
    dev.enable();

    println!("Try writing to a non-existent I2C device");
    dev.write(0x1, i2c::START | i2c::STOP, unsafe { &mut DATA} );
}

