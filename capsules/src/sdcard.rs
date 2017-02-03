//! Provide capsule driver for accessing an SD Card.
//! This allows initialization and block reads or writes on top of SPI

// Resources for SD Card API
// elm-chan.org/docs/mmc/mmc_e.html
// alumni.cs.ucr.edu/~amitra/sdcard/Additional/sdcard_appnote_foust.pdf
// luckyresistor.me/cat-protector/software/sdcard-2/

use core::cell::Cell;
use kernel::common::take_cell::TakeCell;

use kernel::hil;
use kernel::hil::time::Frequency;

pub static mut TXBUFFER: [u8; 512] = [0; 512]; //XXX: This can get MUCH smaller
pub static mut RXBUFFER: [u8; 1024] = [0; 1024]; //XXX: I think this is going to have to stay huge

//XXX: FOR TESTING
pub static mut CLIENT_BUFFER: [u8; 1024] = [0; 1024];

#[allow(dead_code)]
#[derive(Clone,Copy,Debug,PartialEq)]
enum SDCmd {
    CMD0 = 0, // Reset
    CMD1 = 1, // Generic init
    CMD8 = 8, // Check voltage range
    CMD12 = 12, // Stop multiple block read
    CMD16 = 16, // Set blocksize
    CMD17 = 17, // Read single block
    CMD18 = 18, // Read multiple blocks
    CMD55 = 55, // Next command will be application specific
    CMD58 = 58, // Read operation condition register (OCR)
    ACMD41 = 0x80 + 41, // App-specific Init
}

#[allow(dead_code)]
#[derive(Clone,Copy,Debug,PartialEq)]
enum SDResponse {
    R1,
    R2,
    R3,
    R7,
}

#[derive(Clone,Copy,Debug,PartialEq)]
enum State {
    Idle,

    SendACmd { acmd: SDCmd, arg: u32 },

    InitReset,
    InitCheckVersion,
    InitRepeatHCSInit,
    InitCheckCapacity,
    InitAppSpecificInit,
    InitRepeatAppSpecificInit,
    InitRepeatGenericInit,
    InitSetBlocksize,

    ReadBlocks { count: u32 },
}

#[derive(Clone,Copy,Debug,PartialEq)]
enum AlarmState {
    Idle,

    RepeatHCSInit,
    RepeatAppSpecificInit,
    RepeatGenericInit,

    DetectionChange,
}

// SD Card types
const CT_MMC: u8 = 0x01;
const CT_SD1: u8 = 0x02;
const CT_SD2: u8 = 0x04;
const CT_SDC: u8 = (CT_SD1 | CT_SD2);
const CT_BLOCK: u8 = 0x08;

//XXX: How do we handle errors with this interface?
pub trait SDCardClient {
    fn status(&self, status: u8);
    fn card_detection_changed(&self, installed: bool);
    fn init_done(&self, status: u8);
    fn read_done(&self, data: &'static mut [u8], len: usize);
    fn write_done(&self, buffer: &'static mut [u8]);
}

// SD Card capsule, capable of being built on top of by other kernel capsules
pub struct SDCard<'a, A: hil::time::Alarm + 'a> {
    // Note: the SpiMasterDevice used by the sdcard cannot be shared by other
    //  capsules. The current implementation of the SD card requires manually
    //  handling the chip select line, breaking virtualization on that bus
    spi: &'a hil::spi::SpiMasterDevice,
    state: Cell<State>,
    after_state: Cell<State>,

    // Note: this driver requires use of the chip select line as a GPIO pin. To
    //  do so, configure the pin function as a None and pass it into this
    //  capsule. When setting up the VirtualSpiMasterDevice: if it is on top of
    //  a SPI give it the number of CS that it would have used, if it is on top
    //  of a USART give it None. In either case, the underlying SPI will
    //  attempt to assert/deassert a CS that is not connected to an external
    //  pin
    cs_pin: &'a hil::gpio::Pin,

    alarm: &'a A,
    alarm_state: Cell<AlarmState>,

    is_initialized: Cell<bool>,
    card_type: Cell<u8>,

    detect_pin: TakeCell<&'static hil::gpio::Pin>,

    txbuffer: TakeCell<&'static mut [u8]>,
    rxbuffer: TakeCell<&'static mut [u8]>,

    client: TakeCell<&'static SDCardClient>,
    client_buffer: TakeCell<&'static mut [u8]>,
}

impl<'a, A: hil::time::Alarm + 'a> SDCard<'a, A> {
    pub fn new(spi: &'a hil::spi::SpiMasterDevice,
               cs_pin: &'static hil::gpio::Pin,
               alarm: &'a A,
               detect_pin: Option<&'static hil::gpio::Pin>,
               txbuffer: &'static mut [u8],
               rxbuffer: &'static mut [u8])
               -> SDCard<'a, A> {

        // initialize buffers
        for i in 0..txbuffer.len() {
            txbuffer[i] = 0xFF;
        }
        for i in 0..rxbuffer.len() {
            rxbuffer[i] = 0xFF;
        }

        // set up chip select pin
        cs_pin.make_output();
        cs_pin.set();

        // handle optional detect pin
        let pin_cell = detect_pin.map_or_else(|| {
                                                  // else first, pin was None
                                                  TakeCell::empty()
                                              },
                                              |pin| {
                                                  pin.make_input();
                                                  TakeCell::new(pin)
                                              });

        // setup and return struct
        SDCard {
            spi: spi,
            state: Cell::new(State::Idle),
            after_state: Cell::new(State::Idle),
            cs_pin: cs_pin,
            alarm: alarm,
            alarm_state: Cell::new(AlarmState::Idle),
            is_initialized: Cell::new(false),
            card_type: Cell::new(0x00),
            detect_pin: pin_cell,
            txbuffer: TakeCell::new(txbuffer),
            rxbuffer: TakeCell::new(rxbuffer),
            client: TakeCell::empty(),
            client_buffer: TakeCell::empty(),
        }
    }

    fn set_spi_slow_mode(&self) {
        // set to CPHA=0, CPOL=0, 400 kHZ
        self.spi.configure(hil::spi::ClockPolarity::IdleLow,
                           hil::spi::ClockPhase::SampleLeading,
                           400000);
    }

    fn set_spi_fast_mode(&self) {
        // set to CPHA=0, CPOL=0, 4 MHz
        self.spi.configure(hil::spi::ClockPolarity::IdleLow,
                           hil::spi::ClockPhase::SampleLeading,
                           4000000);
    }

    fn transaction_start(&self) {
        self.cs_pin.clear();
    }

    fn transaction_end(&self) {
        self.cs_pin.set();
    }

    fn send_command(&self,
                    cmd: SDCmd,
                    arg: u32,
                    mut write_buffer: &'static mut [u8],
                    mut read_buffer: &'static mut [u8]) {
        if self.is_initialized.get() {
            // device is already initialized
            self.set_spi_fast_mode();
        } else {
            // device is still being initialized
            self.set_spi_slow_mode();
        }

        // send dummy bytes to start
        write_buffer[0] = 0xFF;
        write_buffer[1] = 0xFF;

        // command
        if (0x80 & cmd as u8) != 0x00 {
            // application-specific command
            write_buffer[2] = 0x40 | (0x7F & cmd as u8);
        } else {
            // normal command
            write_buffer[2] = 0x40 | cmd as u8;
        }

        // argument, MSB first
        write_buffer[3] = ((arg >> 24) & 0xFF) as u8;
        write_buffer[4] = ((arg >> 16) & 0xFF) as u8;
        write_buffer[5] = ((arg >> 8) & 0xFF) as u8;
        write_buffer[6] = ((arg >> 0) & 0xFF) as u8;

        // CRC is ignored except for CMD0 and maybe CMD8
        if cmd == SDCmd::CMD8 {
            write_buffer[7] = 0x87; // valid crc for CMD8(0x1AA)
        } else {
            write_buffer[7] = 0x95; // valid crc for CMD0
        }

        // always receive 10 bytes
        let recv_len = 10;

        // append dummy bytes to transmission
        for i in 0..recv_len {
            write_buffer[8 + i] = 0xFF;
        }

        self.spi.read_write_bytes(write_buffer, Some(read_buffer), 8 + recv_len);
    }

    fn read_data_packet(&self,
                        mut write_buffer: &'static mut [u8],
                        mut read_buffer: &'static mut [u8]) {
        // needs to run a state machine

        // read a single byte to see if the Data Token is ready yet

        // read Data Block size (512) + 2 for CRC
    }

    //XXX: just for testing, needs to be combined with send_command in some way
    fn read_command(&self,
                    cmd: SDCmd,
                    arg: u32,
                    mut write_buffer: &'static mut [u8],
                    mut read_buffer: &'static mut [u8],
                    count: u32) {
        if self.is_initialized.get() {
            // device is already initialized
            self.set_spi_fast_mode();
        } else {
            // device is still being initialized
            self.set_spi_slow_mode();
        }

        // send dummy bytes to start
        write_buffer[0] = 0xFF;
        write_buffer[1] = 0xFF;

        // command
        if (0x80 & cmd as u8) != 0x00 {
            // application-specific command
            write_buffer[2] = 0x40 | (0x7F & cmd as u8);
        } else {
            // normal command
            write_buffer[2] = 0x40 | cmd as u8;
        }

        // argument, MSB first
        write_buffer[3] = ((arg >> 24) & 0xFF) as u8;
        write_buffer[4] = ((arg >> 16) & 0xFF) as u8;
        write_buffer[5] = ((arg >> 8) & 0xFF) as u8;
        write_buffer[6] = ((arg >> 0) & 0xFF) as u8;

        // CRC is ignored except for CMD0 and maybe CMD8
        if cmd == SDCmd::CMD8 {
            write_buffer[7] = 0x87; // valid crc for CMD8(0x1AA)
        } else {
            write_buffer[7] = 0x95; // valid crc for CMD0
        }

        // always receive 10 bytes, plus data packets
        let recv_len = 10 + (1 + 512 + 2) * (count as usize);

        // append dummy bytes to transmission
        // THIS IS NO LONGER WORTH IT
        // NEED SOME WAY TO USE A SINGLE TRANSMISSION BYTE
        for i in 0..10 {
            write_buffer[8 + i] = 0xFF;
        }

        // Note: when running on top of the USART, the CS line is raised 1.5
        //  bytes too early so we read an extra two bytes here.
        //  See: https://github.com/helena-project/tock/issues/274
        self.spi.read_write_bytes(write_buffer, Some(read_buffer), 8 + recv_len + 2);
    }
    fn get_response(&self, response: SDResponse, read_buffer: &[u8]) -> (u8, u8, u32) {

        let mut r1: u8 = 0xFF;
        let mut r2: u8 = 0xFF;
        let mut r3: u32 = 0xFFFFFFFF;

        // scan through read buffer for response byte
        for i in 0..read_buffer.len() {
            if (read_buffer[i] & 0x80) == 0x00 {
                r1 = read_buffer[i];

                match response {
                    SDResponse::R2 => {
                        if i + 1 < read_buffer.len() {
                            r2 = read_buffer[i + 1];
                        }
                    }
                    SDResponse::R3 | SDResponse::R7 => {
                        if i + 4 < read_buffer.len() {
                            r3 = (read_buffer[i + 1] as u32) << 24 |
                                 (read_buffer[i + 2] as u32) << 16 |
                                 (read_buffer[i + 3] as u32) << 8 |
                                 (read_buffer[i + 4] as u32);
                        }
                    }
                    _ => {}
                }

                // response found
                break;
            }
        }

        (r1, r2, r3)
    }

    fn process_state(&self,
                     mut write_buffer: &'static mut [u8],
                     mut read_buffer: &'static mut [u8],
                     _: usize) {

        match self.state.get() {
            State::SendACmd { acmd, arg } => {
                // send the application-specific command and resume the state
                //  machine
                self.transaction_end();
                self.state.set(self.after_state.get());
                self.after_state.set(State::Idle);
                self.transaction_start();
                self.send_command(acmd, arg, write_buffer, read_buffer);
            }

            State::InitReset => {
                // check response
                self.transaction_end();
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                // only continue if we are in idle state
                if r1 == 0x01 {
                    // next send Check Voltage Range command that is only valid
                    //  on SDv2 cards. This is used to check which SD card version
                    //  is installed
                    self.state.set(State::InitCheckVersion);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD8, 0x1AA, write_buffer, read_buffer);
                } else {
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
                }
            }

            State::InitCheckVersion => {
                // check response
                self.transaction_end();
                let (r1, _, r7) = self.get_response(SDResponse::R7, read_buffer);

                if r1 == 0x01 && r7 == 0x1AA {
                    // we have an SDv2 card
                    // send application-specific initialization in high capacity mode (HCS)
                    self.state.set(State::SendACmd {
                        acmd: SDCmd::ACMD41,
                        arg: 0x40000000,
                    });
                    self.after_state.set(State::InitRepeatHCSInit);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer);
                } else {
                    // we have either an SDv1 or MMCv3 card
                    // send application-specific initialization
                    self.state.set(State::SendACmd {
                        acmd: SDCmd::ACMD41,
                        arg: 0x0,
                    });
                    self.after_state.set(State::InitAppSpecificInit);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer);
                }
            }

            State::InitRepeatHCSInit => {
                // check response
                self.transaction_end();
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // card initialized
                    // check card capacity
                    self.state.set(State::InitCheckCapacity);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD58, 0x0, write_buffer, read_buffer);
                } else if r1 == 0x01 {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 10 ms
                    self.alarm_state.set(AlarmState::RepeatHCSInit);
                    let interval = (10 as u32) * <A::Frequency>::frequency() / 1000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                } else {
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
                }
            }

            State::InitCheckCapacity => {
                // check response
                self.transaction_end();
                let (r1, _, r7) = self.get_response(SDResponse::R7, read_buffer);

                if r1 == 0x00 {
                    if (r7 & 0x40000000) != 0x00 {
                        self.card_type.set(CT_SD2 | CT_BLOCK);
                    } else {
                        self.card_type.set(CT_SD2);
                    }
                } else {
                    panic!("Error in {:?}. R1: {}", self.state.get(), r1);
                }

                // Initialization complete
                self.state.set(State::Idle);
                self.is_initialized.set(true);

                // replace buffers
                self.txbuffer.replace(write_buffer);
                self.rxbuffer.replace(read_buffer);

                /* TESTING
                //XXX: Initialization complete! Do callback
                panic!("Initialization complete");
                */

                panic!("Init complete");
                //unsafe{self.read_blocks(&mut CLIENT_BUFFER, 0, 1);}

            }

            State::InitAppSpecificInit => {
                // check response
                self.transaction_end();
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 <= 0x01 {
                    // SDv1 card
                    // send application-specific initialization
                    self.card_type.set(CT_SD1);
                    self.state.set(State::SendACmd {
                        acmd: SDCmd::ACMD41,
                        arg: 0x0,
                    });
                    self.after_state.set(State::InitRepeatAppSpecificInit);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer);
                } else {
                    // MMCv3 card
                    // send generic intialization
                    self.card_type.set(CT_MMC);
                    self.state.set(State::InitRepeatGenericInit);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD1, 0x0, write_buffer, read_buffer);
                }
            }

            State::InitRepeatAppSpecificInit => {
                // check response
                self.transaction_end();
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // card initialized
                    // set blocksize to 512
                    self.state.set(State::InitSetBlocksize);
                    self.send_command(SDCmd::CMD16, 512, write_buffer, read_buffer);
                } else if r1 == 0x01 {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 10 ms
                    self.alarm_state.set(AlarmState::RepeatAppSpecificInit);
                    let interval = (10 as u32) * <A::Frequency>::frequency() / 1000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                } else {
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
                }
            }

            State::InitRepeatGenericInit => {
                // check response
                self.transaction_end();
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // card initialized
                    // set blocksize to 512
                    self.state.set(State::InitSetBlocksize);
                    self.transaction_start();
                    self.send_command(SDCmd::CMD16, 512, write_buffer, read_buffer);
                } else if r1 == 0x01 {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 10 ms
                    self.alarm_state.set(AlarmState::RepeatGenericInit);
                    let interval = (10 as u32) * <A::Frequency>::frequency() / 1000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                } else {
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
                }
            }

            State::InitSetBlocksize => {
                // check response
                self.transaction_end();
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // Initialization complete
                    self.state.set(State::Idle);
                    self.is_initialized.set(true);

                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    //XXX: Initialization complete! Do callback
                    panic!("Initialization complete");
                } else {
                    panic!("Error in {:?}. R1: {}", self.state.get(), r1);
                }
            }

            State::ReadBlocks { count } => {
                panic!("Read the block");
            }

            State::Idle => {}
        }
    }

    fn process_alarm(&self) {
        match self.alarm_state.get() {
            AlarmState::DetectionChange => {
                if self.is_installed() {
                    panic!("SD Card installed");
                } else {
                    panic!("SD Card gone");
                }

                // re-enable interrupts
                self.detect_changes();
                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::RepeatHCSInit => {
                // buffers must be available to use
                if self.txbuffer.is_none() {
                    panic!("No txbuffer available for timer");
                }
                if self.rxbuffer.is_none() {
                    panic!("No rxbuffer available for timer");
                }

                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // send application-specific initialization in high capcity mode (HCS)
                        self.state.set(State::SendACmd {
                            acmd: SDCmd::ACMD41,
                            arg: 0x40000000,
                        });
                        self.after_state.set(State::InitRepeatHCSInit);
                        self.transaction_start();
                        self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::RepeatAppSpecificInit => {
                // buffers must be available to use
                if self.txbuffer.is_none() {
                    panic!("No txbuffer available for timer");
                }
                if self.rxbuffer.is_none() {
                    panic!("No rxbuffer available for timer");
                }

                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // send application-specific initialization
                        self.state.set(State::SendACmd {
                            acmd: SDCmd::ACMD41,
                            arg: 0x0,
                        });
                        self.after_state.set(State::InitRepeatAppSpecificInit);
                        self.transaction_start();
                        self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::RepeatGenericInit => {
                // buffers must be available to use
                if self.txbuffer.is_none() {
                    panic!("No txbuffer available for timer");
                }
                if self.rxbuffer.is_none() {
                    panic!("No rxbuffer available for timer");
                }

                // check card initialization again
                self.txbuffer.take().map(|write_buffer| {
                    self.rxbuffer.take().map(move |read_buffer| {
                        // send generic initialization
                        self.state.set(State::InitRepeatGenericInit);
                        self.transaction_start();
                        self.send_command(SDCmd::CMD1, 0x0, write_buffer, read_buffer);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::Idle => {}
        }
    }

    pub fn set_client<C: SDCardClient>(&self, client: &'static C) {
        self.client.replace(client);
    }

    pub fn is_installed(&self) -> bool {
        // if there is no detect pin, assume an sd card is installed
        self.detect_pin.map_or(true, |pin| {
            // sd card detection pin is active low
            pin.read() == false
        })
    }

    pub fn detect_changes(&self) {
        self.detect_pin
            .map(|pin| { pin.enable_interrupt(0, hil::gpio::InterruptMode::EitherEdge); });
    }

    pub fn initialize(&self) {
        // if not already, set card to uninitialized again
        self.is_initialized.set(false);

        // no point in initializing if the card is not installed
        if self.is_installed() {
            // reset the SD card in order to start initializing it
            self.txbuffer.take().map(|txbuffer| {
                self.rxbuffer.take().map(move |rxbuffer| {
                    self.transaction_start();
                    self.state.set(State::InitReset);
                    self.send_command(SDCmd::CMD0, 0x0, txbuffer, rxbuffer);
                });
            });
        } else {
            panic!("No SD card is installed!");
        }
    }

    pub fn read_blocks(&self, buffer: &'static mut [u8], sector: u32, count: u32) {
        // only if initialized and installed
        if self.is_installed() && self.is_initialized.get() {
            self.txbuffer.take().map(|txbuffer| {
                self.rxbuffer.take().map(move |rxbuffer| {
                    // save the user buffer for later
                    self.client_buffer.replace(buffer);

                    //XXX: Need to figure out the LBA to BA conversion

                    self.state.set(State::ReadBlocks { count: count });
                    if count == 1 {
                        self.transaction_start();
                        self.send_command(SDCmd::CMD17, sector, txbuffer, rxbuffer);
                    } else {
                        panic!("Can't read multiple blocks yet");
                    }
                })
            });
        } else {
            panic!("Can't read block from bad sd card");
        }
    }

    pub fn write_blocks(&self, buffer: &'static mut [u8], sector: u32, count: u32) {
        // only if initialized and installed
        if self.is_installed() && self.is_initialized.get() {
            self.txbuffer.take().map(|txbuffer| {
                self.rxbuffer.take().map(move |rxbuffer| {
                    // save the user buffer for later
                    self.client_buffer.replace(buffer);

                })
            });
        } else {
            panic!("Can't read block from bad sd card");
        }
    }
}

// Handle callbacks from the SPI peripheral
impl<'a, A: hil::time::Alarm + 'a> hil::spi::SpiMasterClient for SDCard<'a, A> {
    fn read_write_done(&self,
                       mut write_buffer: &'static mut [u8],
                       read_buffer: Option<&'static mut [u8]>,
                       len: usize) {

        // unrwap so we don't have to deal with options everywhere
        read_buffer.map_or_else(|| {
                                    panic!("Didn't receive a read_buffer back");
                                },
                                move |read_buffer| {
                                    self.process_state(write_buffer, read_buffer, len);
                                });
    }
}

// Handle callbacks from the timer
impl<'a, A: hil::time::Alarm + 'a> hil::time::Client for SDCard<'a, A> {
    fn fired(&self) {
        self.process_alarm();
    }
}

// Handle callbacks from the card detection pin
impl<'a, A: hil::time::Alarm + 'a> hil::gpio::Client for SDCard<'a, A> {
    fn fired(&self, _: usize) {
        //XXX: need to do something if alarm is already in use
        //  probably just want to burn the entire state machine down

        // disable additional interrupts
        self.detect_pin.map(|pin| { pin.disable_interrupt(); });

        // run a timer for 500 ms in order to let the sd card settle
        self.alarm_state.set(AlarmState::DetectionChange);
        let interval = (500 as u32) * <A::Frequency>::frequency() / 1000;
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}


// Application driver for SD Card capsule, layers on top of SD Card capsule
pub struct SDCardDriver<'a, A: hil::time::Alarm + 'a> {
    sdcard: &'a SDCard<'a, A>,
}
