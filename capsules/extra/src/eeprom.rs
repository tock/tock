use core::cell::Cell;
use core::cmp;
use kernel::hil::nonvolatile_storage::{NonvolatileStorage, NonvolatileStorageClient};

use kernel::ErrorCode;
use kernel::hil::i2c::{Error, I2CClient, I2CDevice};
use kernel::utilities::cells::{OptionalCell, TakeCell};

#[derive(Copy, Clone, Debug)]
enum State {
    Idle,
    Reading(usize),
    Writing(usize),
}

pub struct EEPROM<'a> {
    i2c: &'a dyn I2CDevice,
    buffer: TakeCell<'static, [u8]>,
    client_buffer: TakeCell<'a, [u8]>,
    client: OptionalCell<&'a dyn NonvolatileStorageClient<'a>>,
    state: Cell<State>,
}


impl<'a> EEPROM<'a> {
    pub fn new(i2c: &'a dyn I2CDevice, buffer: &'static mut [u8]) -> Self {
        Self {
            i2c,
            buffer: TakeCell::new(buffer),
            client_buffer: TakeCell::empty(),
            client: OptionalCell::empty(),
            state: Cell::new(State::Idle),
        }
    }

    fn read(&self, buffer: &'a mut [u8], address: usize, length: usize) -> Result<(), ErrorCode> {
        self.i2c.enable();
        self.buffer.take().map_or(Err(ErrorCode::RESERVE), move |local_buffer| {
            local_buffer[0] = ((address >> 8) & 0x00ff) as u8;
            local_buffer[1] = (address & 0x00ff) as u8;

            let read_len = cmp::min(buffer.len(), length);

            self.client_buffer.replace(buffer);

            self.state.set(State::Reading(length));
            if let Err((error, local_buffer)) = self.i2c.write_read(local_buffer, 2, read_len) {
                self.buffer.replace(local_buffer);
                self.i2c.disable();
                Err(error.into())
            } else {
                Ok(())
            }
        })
    }

    fn write(&self, buffer: &'a mut [u8], address: usize, length: usize) -> Result<(), ErrorCode> {
        self.i2c.enable();
        self.buffer.take().map_or(Err(ErrorCode::RESERVE), move |txbuffer| {
            txbuffer[0] = ((address >> 8) & 0x00ff) as u8;
            txbuffer[1] = (address & 0x00ff) as u8;

            let write_len = cmp::min(txbuffer.len() - 2, length);
            for i in 0..write_len {
                txbuffer[(i + 2)] = buffer[i];
            }

            self.client_buffer.replace(buffer);

            self.state.set(State::Writing(length));
            if let Err((error, txbuffer)) = self.i2c.write(txbuffer, write_len + 2) {
                self.buffer.replace(txbuffer);
                self.i2c.disable();
                Err(error.into())
            } else {
                Ok(())
            }
        })
    }
}

impl<'a> I2CClient for EEPROM<'a> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), Error>) {
        match self.state.get() {
            State::Reading(read_len) => {
                self.state.set(State::Idle);
                if status.is_err() {
                    self.buffer.replace(buffer);
                    // TODO
                    return;
                }
                self.client_buffer.take().map(|client_buffer| {
                    for i in 0..read_len {
                        client_buffer[i] = buffer[i];
                    }
                    self.client.map(move |client| client.read_done(client_buffer, read_len));
                });
                self.buffer.replace(buffer);
                self.i2c.disable();
            }
            State::Writing(write_len) => {
                self.state.set(State::Idle);
                self.buffer.replace(buffer);
                if status.is_err() {
                    // TODO
                    return;
                }
                self.client_buffer.take().map(move |client_buffer| {
                    self.client.map(move |client| client.write_done(client_buffer, write_len))
                });
                self.i2c.disable();
            }
            State::Idle => {}
        }
    }
}

impl<'a> NonvolatileStorage<'a> for EEPROM<'a> {
    fn set_client(&self, client: &'a dyn NonvolatileStorageClient<'a>) {
        self.client.set(client)
    }

    fn read(&self, buffer: &'a mut [u8], address: usize, length: usize) -> Result<(), ErrorCode> {
        self.read(buffer, address, length)
    }

    fn write(&self, buffer: &'a mut [u8], address: usize, length: usize) -> Result<(), ErrorCode> {
        self.write(buffer, address, length)
    }
}