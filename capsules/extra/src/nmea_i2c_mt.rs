// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! A capsule for I2C NMEA. This is used to read NMEA sentences from a GPS/GNSS
//! device connected via I2C.
//!
//! This has been tested against a MediaTek MT3333 device.
//! It will work with any MediaTek GNSS outputing NMEA over I2C. It is expected to
//! work with any GNSS device that reports NMEA strings when reading from an
//! I2C address.

use crate::nmea::{NmeaClient, NmeaDevice};
use crate::nmea_i2c_mt::i2c::I2CClient;
use core::cell::Cell;
use core::str;
use kernel::hil::i2c::{self, I2CDevice};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

// We can only read a small-ish number of bytes at a time
// otherwise the GNSS device will crash and hold the I2C
// clock low. 32 is the maximum we can read
pub const I2C_BUFFER_LEN: usize = 24;

pub const NMEA_BUFFER_LEN: usize = 128;

pub struct I2cNmea<'a, I: I2CDevice> {
    sentence_buffer: TakeCell<'static, [u8]>,
    i2c_buffer: TakeCell<'static, [u8]>,
    nmea_offset: Cell<usize>,
    i2c: &'a I,
    client: OptionalCell<&'a dyn NmeaClient>,
    write: Cell<(bool, usize)>,
}

impl<'a, I: I2CDevice> I2cNmea<'a, I> {
    pub fn new(i2c: &'a I, i2c_buffer: &'static mut [u8]) -> Self {
        I2cNmea {
            sentence_buffer: TakeCell::empty(),
            i2c_buffer: TakeCell::new(i2c_buffer),
            nmea_offset: Cell::new(0),
            i2c,
            client: OptionalCell::empty(),
            write: Cell::new((false, 0)),
        }
    }
}

impl<'a, I: I2CDevice> NmeaDevice<'a> for I2cNmea<'a, I> {
    fn set_client(&self, client: &'a dyn NmeaClient) {
        self.client.set(client);
    }

    fn read_sentence(
        &self,
        sentence: &'static mut [u8],
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        match self.i2c_buffer.take() {
            Some(buffer) => {
                let i2c_buf_len = buffer.len();
                self.write.set((false, 0));

                match self.i2c.read(buffer, i2c_buf_len) {
                    Ok(()) => {
                        self.sentence_buffer.replace(sentence);
                        Ok(())
                    }
                    Err((e, buf)) => {
                        self.i2c_buffer.replace(buf);
                        Err((e.into(), sentence))
                    }
                }
            }
            None => Err((ErrorCode::BUSY, sentence)),
        }
    }

    fn write_sentence(
        &self,
        buffer: &'static mut [u8],
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        self.write.set((true, length));

        match self.i2c.write(buffer, length) {
            Ok(()) => Ok(()),
            Err((e, buf)) => Err((e.into(), buf)),
        }
    }
}

impl<'a, I: I2CDevice> I2CClient for I2cNmea<'a, I> {
    fn command_complete(&self, buffer: &'static mut [u8], status: Result<(), i2c::Error>) {
        if self.write.get().0 {
            if let Err(e) = status {
                self.client.map(|call| {
                    call.callback(buffer, 0, Err(e.into()));
                });
            } else {
                self.client.map(|call| {
                    call.callback(buffer, self.write.get().1, Ok(()));
                });
            }
        } else {
            self.sentence_buffer.take().map(|nmea_buf| {
                let i2c_buf_len = buffer.len();

                if let Err(e) = status {
                    self.nmea_offset.set(0);
                    self.i2c_buffer.replace(buffer);

                    self.client.map(|call| {
                        call.callback(nmea_buf, 0, Err(e.into()));
                    });

                    return;
                }

                let string = match str::from_utf8(buffer) {
                    Ok(utf8) => utf8,
                    Err(_e) => {
                        self.nmea_offset.set(0);
                        self.i2c_buffer.replace(buffer);

                        self.client.map(|call| {
                            call.callback(nmea_buf, 0, Err(ErrorCode::NOSUPPORT));
                        });
                        return;
                    }
                };

                let mut nmea_offset = self.nmea_offset.get();

                if nmea_offset == 0 {
                    // We have no existing data
                    if let Some(location) = string.find('$') {
                        // We have no existing data and found the start
                        // of a new sentence

                        // Copy the data after the `$`
                        let size = i2c_buf_len - location;
                        nmea_buf[nmea_offset..(nmea_offset + size)]
                            .copy_from_slice(&buffer[location..]);
                        nmea_offset += size;
                    }

                    // Otherwise this is the middle of the sentence and we have no
                    // header so just try again
                } else {
                    if let Some(location) = string.find('$') {
                        // This includes the end of the current sentence
                        // and the start of a new sentence

                        if (nmea_offset + location) > nmea_buf.len() {
                            // We will overflow our buffer, just drop the packet and try again

                            nmea_offset = 0;

                            // Copy the rest of the data
                            let size = i2c_buf_len - location;
                            nmea_buf[nmea_offset..(nmea_offset + size)]
                                .copy_from_slice(&buffer[location..]);
                            nmea_offset += size;

                            self.nmea_offset.set(nmea_offset);
                            self.sentence_buffer.replace(nmea_buf);

                            if let Err((e, buf)) = self.i2c.read(buffer, i2c_buf_len) {
                                self.i2c_buffer.replace(buf);

                                self.client.map(|call| {
                                    call.callback(
                                        self.sentence_buffer.take().unwrap(),
                                        0,
                                        Err(e.into()),
                                    );
                                });
                            }
                            return;
                        }

                        // First copy the remainder of the sentence to the buffer
                        nmea_buf[nmea_offset..(nmea_offset + location)]
                            .copy_from_slice(&buffer[0..location]);
                        nmea_offset += location;

                        // Now parse the sentence
                        match str::from_utf8(&nmea_buf[0..nmea_offset]) {
                            Ok(utf8) => utf8,
                            Err(_e) => {
                                self.nmea_offset.set(0);
                                self.i2c_buffer.replace(buffer);

                                self.client.map(|call| {
                                    call.callback(nmea_buf, 0, Err(ErrorCode::NOSUPPORT));
                                });
                                return;
                            }
                        };

                        // At this point we have a sentence with a `$` at the start and we
                        // just hit the next `$`.
                        // We report it back to the caller.
                        // We loose the rest of the data we just read though

                        self.i2c_buffer.replace(buffer);
                        self.nmea_offset.set(0);

                        self.client.map(|call| {
                            call.callback(nmea_buf, nmea_offset, Ok(()));
                        });

                        return;
                    } else {
                        if (nmea_offset + i2c_buf_len) > nmea_buf.len() {
                            // We will overflow our buffer, just drop the packet and try again
                            nmea_offset = 0;
                        } else {
                            // This is the middle of a sentence, copy the entire string to our buffer
                            nmea_buf[nmea_offset..(nmea_offset + i2c_buf_len)]
                                .copy_from_slice(buffer);
                            nmea_offset += i2c_buf_len;
                        }
                    }
                }

                self.sentence_buffer.replace(nmea_buf);
                self.nmea_offset.set(nmea_offset);

                if let Err((e, buf)) = self.i2c.read(buffer, i2c_buf_len) {
                    self.i2c_buffer.replace(buf);

                    self.client.map(|call| {
                        call.callback(self.sentence_buffer.take().unwrap(), 0, Err(e.into()));
                    });
                }
            });
        }
    }
}
