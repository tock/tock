/// Driver for the FM25CL FRAM chip (http://www.cypress.com/part/fm25cl64b-dg)

use core::cell::Cell;
use core::cmp;

use kernel::common::take_cell::TakeCell;
use kernel::hil;
use kernel::{AppId, AppSlice, Callback, Driver, Shared};


pub static mut TXBUFFER: [u8; 512] = [0; 512];
pub static mut RXBUFFER: [u8; 512] = [0; 512];

pub static mut KERNEL_TXBUFFER: [u8; 512] = [0; 512];
pub static mut KERNEL_RXBUFFER: [u8; 512] = [0; 512];

const SPI_SPEED: u32 = 4000000;

#[allow(dead_code)]
enum Opcodes {
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    ReadStatusRegister = 0x05,
    WriteStatusRegister = 0x01,
    ReadMemory = 0x03,
    WriteMemory = 0x02,
}

#[derive(Clone,Copy,PartialEq)]
enum State {
    Idle,

    /// Simple read states
    ReadStatus,

    /// Write to the FRAM
    WriteEnable,
    WriteMemory,

    /// Read from the FRAM
    ReadMemory,
}


pub trait FM25CLClient {
    fn status(&self, status: u8);
    fn read(&self, data: &'static mut [u8], len: usize);
    fn done(&self, buffer: &'static mut [u8]);
}

pub struct FM25CL<'a> {
    spi: &'a hil::spi::SPIMasterDevice,
    state: Cell<State>,
    txbuffer: TakeCell<&'static mut [u8]>,
    rxbuffer: TakeCell<&'static mut [u8]>,
    client: TakeCell<&'static FM25CLClient>,
    client_buffer: TakeCell<&'static mut [u8]>, // Store buffer and state for passing back to client
    client_write_address: Cell<u16>,
    client_write_len: Cell<u16>,
}

impl<'a> FM25CL<'a> {
    pub fn new(spi: &'a hil::spi::SPIMasterDevice,
               txbuffer: &'static mut [u8],
               rxbuffer: &'static mut [u8])
               -> FM25CL<'a> {
        // setup and return struct
        FM25CL {
            spi: spi,
            state: Cell::new(State::Idle),
            txbuffer: TakeCell::new(txbuffer),
            rxbuffer: TakeCell::new(rxbuffer),
            client: TakeCell::empty(),
            client_buffer: TakeCell::empty(),
            client_write_address: Cell::new(0),
            client_write_len: Cell::new(0),
        }
    }

    pub fn set_client<C: FM25CLClient>(&self, client: &'static C, ) {
        self.client.replace(client);
    }

    /// Setup SPI for this chip
    fn configure_spi(&self) {
        self.spi.configure(hil::spi::ClockPolarity::IdleLow, hil::spi::ClockPhase::SampleLeading, SPI_SPEED);
    }

    pub fn read_status(&self) {
        self.configure_spi();

        self.txbuffer.take().map(|txbuffer| {
            self.rxbuffer.take().map(move |rxbuffer| {
                txbuffer[0] = Opcodes::ReadStatusRegister as u8;

                // Use 4 bytes instead of the required 2 because that works better
                // with DMA for some reason.
                self.spi.read_write_bytes(txbuffer, Some(rxbuffer), 4);
                // self.spi.read_write_bytes(txbuffer, Some(rxbuffer), 2);
                self.state.set(State::ReadStatus);
            });
        });
    }

    pub fn write(&self, address: u16, buffer: &'static mut [u8], len: u16) {
        self.configure_spi();

        self.txbuffer.take().map(move |txbuffer| {

            txbuffer[0] = Opcodes::WriteEnable as u8;

            let write_len = cmp::min(txbuffer.len(), len as usize);

            // Need to save the buffer passed to us so we can give it back.
            self.client_buffer.replace(buffer);
            // Also save address and len for the actual write.
            self.client_write_address.set(address);
            self.client_write_len.set(write_len as u16);

            self.state.set(State::WriteEnable);
            self.spi.read_write_bytes(txbuffer, None, 1);
        });
    }

    pub fn read(&self, address: u16, buffer: &'static mut [u8], len: u16) {
        self.configure_spi();

        self.txbuffer.take().map(|txbuffer| {
            self.rxbuffer.take().map(move |rxbuffer| {
                txbuffer[0] = Opcodes::ReadMemory as u8;
                txbuffer[1] = ((address >> 8) & 0xFF) as u8;
                txbuffer[2] = (address & 0xFF) as u8;

                // Save the user buffer for later
                self.client_buffer.replace(buffer);

                let read_len = cmp::min(rxbuffer.len()-3, len as usize);

                self.state.set(State::ReadMemory);
                self.spi.read_write_bytes(txbuffer, Some(rxbuffer), read_len+3);
            });
        });
    }
}

impl<'a> hil::spi::SpiMasterClient for FM25CL<'a> {
    fn read_write_done(&self, write_buffer: &'static mut [u8], read_buffer: Option<&'static mut [u8]>, len: usize) {

        match self.state.get() {
            State::ReadStatus => {
                self.state.set(State::Idle);

                // Put back buffers that we got back from SPI layer.
                self.txbuffer.replace(write_buffer);

                read_buffer.map(|read_buffer| {
                    let status = read_buffer[1];

                    // Also replace this buffer
                    self.rxbuffer.replace(read_buffer);

                    self.client.map(|client| {
                        client.status(status);
                    });
                });
            },
            State::WriteEnable => {
                self.state.set(State::WriteMemory);

                self.client_buffer.map(move |buffer| {
                    write_buffer[0] = Opcodes::WriteMemory as u8;
                    write_buffer[1] = ((self.client_write_address.get() >> 8) & 0xFF) as u8;
                    write_buffer[2] = (self.client_write_address.get() & 0xFF) as u8;

                    let write_len = cmp::min(write_buffer.len(), self.client_write_len.get() as usize);

                    for i in 0..write_len {
                        write_buffer[(i+3) as usize] = buffer[i as usize];
                    }

                    self.spi.read_write_bytes(write_buffer, read_buffer, write_len+3);
                });
            },
            State::WriteMemory => {
                self.state.set(State::Idle);

                // Replace these buffers
                self.txbuffer.replace(write_buffer);
                read_buffer.map(|read_buffer| {
                    self.rxbuffer.replace(read_buffer);
                });

                // Call done with the write() buffer
                self.client_buffer.take().map(move |buffer| {
                    self.client.map(move |client| {
                        client.done(buffer);
                    });
                });
            },
            State::ReadMemory => {
                self.state.set(State::Idle);

                // Replace the TX buffer
                self.txbuffer.replace(write_buffer);

                read_buffer.map(|read_buffer| {
                    self.client_buffer.take().map(move |buffer| {
                        let read_len = cmp::min(buffer.len(), len);

                        for i in 0..(read_len-3) {
                            buffer[i] = read_buffer[i+3];
                        }

                        self.rxbuffer.replace(read_buffer);

                        self.client.map(move |client| {
                            client.read(buffer, read_len);
                        });
                    });
                });
            },
            _ => {}
        }
    }
}

/// Holds buffers and whatnot that the application has passed us.
struct AppState {
    callback: Cell<Option<Callback>>,
    read_buffer: TakeCell<AppSlice<Shared, u8>>,
    write_buffer: TakeCell<AppSlice<Shared, u8>>,
}

/// Default implementation of the FM25CL driver that provides a Driver
/// interface for providing access to applications.
pub struct FM25CLDriver<'a> {
    fm25cl: &'a FM25CL<'a>,
    app_state: TakeCell<AppState>,
    kernel_read: TakeCell<&'static mut [u8]>,
    kernel_write: TakeCell<&'static mut [u8]>,
}

impl<'a> FM25CLDriver<'a> {
    pub fn new(fm25: &'a FM25CL, write_buf: &'static mut [u8], read_buf: &'static mut [u8]) -> FM25CLDriver<'a> {
        FM25CLDriver {
            fm25cl: fm25,
            app_state: TakeCell::empty(),
            kernel_read: TakeCell::new(read_buf),
            kernel_write: TakeCell::new(write_buf),
        }
    }
}

impl<'a> FM25CLClient for FM25CLDriver<'a> {
    fn status(&self, status: u8) {
        self.app_state.map(|app_state| {
            app_state.callback.get().map(|mut cb| {
                cb.schedule(0, status as usize, 0);
            });
        });
    }

    fn read(&self, data: &'static mut [u8], len: usize) {
        self.app_state.map(|app_state| {
            let mut read_len: usize = 0;

            app_state.read_buffer.map(move |read_buffer| {
                read_len = cmp::min(read_buffer.len(), len);

                let d = &mut read_buffer.as_mut()[0..(read_len as usize)];
                for (i, c) in data[0..read_len].iter().enumerate() {
                    d[i] = *c;
                }

                self.kernel_read.replace(data);
            });

            app_state.callback.get().map(|mut cb| {
                cb.schedule(1, read_len, 0);
            });
        });
    }

    fn done(&self, buffer: &'static mut [u8]) {
        self.kernel_write.replace(buffer);

        self.app_state.map(|app_state| {
            app_state.callback.get().map(|mut cb| {
                cb.schedule(2, 0, 0);
            });
        });
    }
}

impl<'a> Driver for FM25CLDriver<'a> {
    fn allow(&self, _appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> isize {
        match allow_num {
            // Pass read buffer in from application
            0 => {
                let appst = match self.app_state.take() {
                    None => {
                        AppState {
                            callback: Cell::new(None),
                            read_buffer: TakeCell::new(slice),
                            write_buffer: TakeCell::empty(),
                        }
                    }
                    Some(appst) => {
                        appst.read_buffer.replace(slice);
                        appst
                    }
                };
                self.app_state.replace(appst);
                0
            }
            // Pass write buffer in from application
            1 => {
                let appst = match self.app_state.take() {
                    None => {
                        AppState {
                            callback: Cell::new(None),
                            write_buffer: TakeCell::new(slice),
                            read_buffer: TakeCell::empty(),
                        }
                    }
                    Some(appst) => {
                        appst.write_buffer.replace(slice);
                        appst
                    }
                };
                self.app_state.replace(appst);
                0
            }
            _ => -1,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> isize {
        match subscribe_num {
            0 => {
                let appst = match self.app_state.take() {
                    None => {
                        AppState {
                            callback: Cell::new(Some(callback)),
                            write_buffer: TakeCell::empty(),
                            read_buffer: TakeCell::empty(),
                        }
                    }
                    Some(appst) => {
                        appst.callback.set(Some(callback));
                        appst
                    }
                };
                self.app_state.replace(appst);
                0
            }

            // default
            _ => -1,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> isize {
        match command_num {
            // get status
            0 => {
                self.fm25cl.read_status();
                0
            }

            // read
            1 => {
                let address = (data & 0xFFFF) as u16;
                let len = (data >> 16) & 0xFFFF;

                self.kernel_read.take().map(|kernel_read| {
                    let read_len = cmp::min(len, kernel_read.len());

                    self.fm25cl.read(address, kernel_read, read_len as u16);
                });
                0
            }

            // write
            2 => {
                let address = (data & 0xFFFF) as u16;
                let len = ((data >> 16) & 0xFFFF) as usize;

                self.app_state.map(|app_state| {
                    app_state.write_buffer.map(|write_buffer| {
                        self.kernel_write.take().map(|kernel_write| {
                            // Check bounds for write length
                            let buf_len = cmp::min(write_buffer.len(), kernel_write.len());
                            let write_len = cmp::min(buf_len, len);

                            let d = &mut write_buffer.as_mut()[0..write_len];
                            for (i, c) in kernel_write[0..write_len].iter_mut().enumerate() {
                                *c = d[i];
                            }

                            self.fm25cl.write(address, kernel_write, write_len as u16);
                        });
                    });
                });
                0
            }

            // default
            _ => -1,
        }
    }
}
