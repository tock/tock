///! A dummy I2C client

use sam4l::i2c;
use hil;
use hil::i2c::I2C;
use core::cell::Cell;

// ===========================================
// Scan for I2C Slaves
// ===========================================

struct ScanClient { dev_id: Cell<u8> }

static mut SCAN_CLIENT : ScanClient = ScanClient { dev_id: Cell::new(1) };

impl hil::i2c::I2CClient for ScanClient {
    fn command_complete(&self, buffer: &'static mut [u8], error: hil::i2c::Error) {
        let mut dev_id = self.dev_id.get();

        match error {
            hil::i2c::Error::CommandComplete => println!("0x{:x}", dev_id),
            _ => {}
        }

        let dev = unsafe { &mut i2c::I2C2 };
        if dev_id < 0x7F {
            dev_id += 1;
            self.dev_id.set(dev_id);
            dev.write(dev_id, i2c::START | i2c::STOP, buffer, 1);
        } else {
            println!("Done scanning for I2C devices. Buffer len: {}", buffer.len());
        }
    }
}

pub fn i2c_scan_slaves() {
    static mut DATA : [u8; 255] = [0; 255];

    let dev = unsafe { &mut i2c::I2C2 };

    let i2c_client = unsafe { &SCAN_CLIENT };
    dev.set_client(i2c_client);
    dev.enable();

    println!("Scanning for I2C devices...");
    dev.write(i2c_client.dev_id.get(), i2c::START | i2c::STOP, unsafe { &mut DATA}, 1);
}

// ===========================================
// Test TMP006
// ===========================================

#[derive(Copy,Clone)]
enum TmpClientState {
    Enabling,
    SelectingDevIdReg,
    ReadingDevIdReg
}

struct TMP006Client { state: Cell<TmpClientState> }

static mut TMP006_CLIENT : TMP006Client =
    TMP006Client { state: Cell::new(TmpClientState::Enabling) };

impl hil::i2c::I2CClient for TMP006Client {
    fn command_complete(&self, buffer: &'static mut [u8], error: hil::i2c::Error) {
        use self::TmpClientState::*;
        println!("{}", error);

        let dev = unsafe { &mut i2c::I2C2 };

        match self.state.get() {
            Enabling => {
                buffer[0] = 0xFF as u8;
                dev.write(0x40, i2c::START | i2c::STOP, buffer, 1);
                self.state.set(SelectingDevIdReg);
            },
            SelectingDevIdReg => {
                println!("Device Id Register selected");
                dev.read(0x40, i2c::START | i2c::STOP, buffer, 2);
                self.state.set(ReadingDevIdReg);
            },
            ReadingDevIdReg => {
                let dev_id = (((buffer[0] as u16) << 8) | buffer[1] as u16) as u16;
                println!("Device Id is 0x{:x}", dev_id);
            }
        }
    }
}

pub fn i2c_tmp006_test() {
    static mut DATA : [u8; 255] = [0; 255];

    let dev = unsafe { &mut i2c::I2C2 };

    let i2c_client = unsafe { &TMP006_CLIENT };
    dev.set_client(i2c_client);
    dev.enable();

    let buf = unsafe { &mut DATA };
    println!("Enabling TMP006...");
    let config = 0x7100 | (((2 & 0x7) as u16) << 9);
    buf[0] = 0x2 as u8; // 0x2 == Configuration register
    buf[1] = ((config & 0xFF00) >> 8) as u8;
    buf[2] = (config & 0x00FF) as u8;
    dev.write(0x40, i2c::START | i2c::STOP, buf, 3);
}

