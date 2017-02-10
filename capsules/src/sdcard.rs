//! Provide capsule driver for accessing an SD Card.
//! This allows initialization and block reads or writes on top of SPI

// Resources for SD Card API
// elm-chan.org/docs/mmc/mmc_e.html
// alumni.cs.ucr.edu/~amitra/sdcard/Additional/sdcard_appnote_foust.pdf
// luckyresistor.me/cat-protector/software/sdcard-2/
// http://users.ece.utexas.edu/~valvano/EE345M/SD_Physical_Layer_Spec.pdf

use core::cell::Cell;
use core::cmp;
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Shared};
use kernel::common::take_cell::{MapCell, TakeCell};

use kernel::hil;
use kernel::hil::time::Frequency;

// Buffers used for SD card transactions
// Constraints:
//  * RXBUFFER must be greater than or equal to TXBUFFER in length
//  * Both RXBUFFER and TXBUFFER must be longer  than the SD card's block size
pub static mut TXBUFFER: [u8; 515] = [0; 515];
pub static mut RXBUFFER: [u8; 515] = [0; 515];

// Buffer for SD card driver
pub static mut KERNEL_BUFFER: [u8; 512] = [0; 512];

#[allow(dead_code)]
#[derive(Clone,Copy,Debug,PartialEq)]
enum SDCmd {
    CMD0 = 0, // Reset
    CMD1 = 1, // Generic init
    CMD8 = 8, // Check voltage range
    CMD9 = 9, // Read chip specific data (CSD) register
    CMD12 = 12, // Stop multiple block read
    CMD16 = 16, // Set blocksize
    CMD17 = 17, // Read single block
    CMD18 = 18, // Read multiple blocks
    CMD24 = 24, // Write single block
    CMD25 = 25, // Write multiple blocks
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
    InitComplete,

    StartReadBlocks { count: u32 },
    WaitReadBlock,
    ReadBlockComplete,
    WaitReadBlocks { count: u32 },
    ReceivedBlock { count: u32 },
    ReadBlocksComplete,

    StartWriteBlocks { count: u32 },
    WriteBlockResponse,
    WriteBlockBusy,
    WaitWriteBlockBusy,
}

#[derive(Clone,Copy,Debug,PartialEq)]
enum AlarmState {
    Idle,

    DetectionChange,

    RepeatHCSInit,
    RepeatAppSpecificInit,
    RepeatGenericInit,

    WaitForDataBlock,
    WaitForDataBlocks { count: u32 },

    WaitForWriteBusy,
}

#[derive(Clone,Copy,Debug,PartialEq)]
enum ErrorCode {
    CardStateChanged = -1,
    InitializationFailure = -2,
    ReadFailure = -3,
    WriteFailure = -4,
    TimeoutFailure = -5,
}

// SD Card types
const CT_MMC: u8 = 0x01;
const CT_SD1: u8 = 0x02;
const CT_SD2: u8 = 0x04;
const CT_BLOCK: u8 = 0x08;

// SD Card capsule, capable of being built on top of by other kernel capsules
pub struct SDCard<'a, A: hil::time::Alarm + 'a> {
    // Note: the SpiMasterDevice used by the sdcard cannot be shared by other
    //  capsules. The current implementation of the SD card requires manually
    //  handling the chip select line, breaking virtualization on that bus
    spi: &'a hil::spi::SpiMasterDevice,
    state: Cell<State>,
    after_state: Cell<State>,

    alarm: &'a A,
    alarm_state: Cell<AlarmState>,
    alarm_count: Cell<u8>,

    is_initialized: Cell<bool>,
    card_type: Cell<u8>,

    detect_pin: Cell<Option<&'static hil::gpio::Pin>>,

    txbuffer: TakeCell<'static, [u8]>,
    rxbuffer: TakeCell<'static, [u8]>,

    client: Cell<Option<&'static SDCardClient>>,
    client_buffer: TakeCell<'static, [u8]>,
    client_offset: Cell<usize>,
}

impl<'a, A: hil::time::Alarm + 'a> SDCard<'a, A> {
    pub fn new(spi: &'a hil::spi::SpiMasterDevice,
               alarm: &'a A,
               detect_pin: Option<&'static hil::gpio::Pin>,
               txbuffer: &'static mut [u8],
               rxbuffer: &'static mut [u8])
               -> SDCard<'a, A> {

        // ensure that the write buffer is long enough
        if txbuffer.len() < 515 || rxbuffer.len() < 515 {
            panic!("Buffers not long enough to store entire SD card block");
        }

        // initialize buffers
        for i in 0..txbuffer.len() {
            txbuffer[i] = 0xFF;
        }
        for i in 0..rxbuffer.len() {
            rxbuffer[i] = 0xFF;
        }

        // handle optional detect pin
        let pin = detect_pin.map_or_else(|| {
                                             // else first, pin was None
                                             None
                                         },
                                         |pin| {
                                             pin.make_input();
                                             Some(pin)
                                         });

        // setup and return struct
        SDCard {
            spi: spi,
            state: Cell::new(State::Idle),
            after_state: Cell::new(State::Idle),
            alarm: alarm,
            alarm_state: Cell::new(AlarmState::Idle),
            alarm_count: Cell::new(0),
            is_initialized: Cell::new(false),
            card_type: Cell::new(0x00),
            detect_pin: Cell::new(pin),
            txbuffer: TakeCell::new(txbuffer),
            rxbuffer: TakeCell::new(rxbuffer),
            client: Cell::new(None),
            client_buffer: TakeCell::empty(),
            client_offset: Cell::new(0),
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

    fn send_command(&self,
                    cmd: SDCmd,
                    arg: u32,
                    mut write_buffer: &'static mut [u8],
                    mut read_buffer: &'static mut [u8],
                    recv_len: usize) {
        // Note: a good default recv_len is 10 bytes. Reading too many bytes
        //  rarely matters. However, it occasionally matters a lot, so we do
        //  provided a settable recv_len

        if self.is_initialized() {
            // device is already initialized
            self.set_spi_fast_mode();
        } else {
            // device is still being initialized
            self.set_spi_slow_mode();
        }

        // check that write buffer is long enough
        if write_buffer.len() < 8 + recv_len {
            panic!("Write buffer too short to send SD card commands");
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

        // append dummy bytes to transmission
        for i in 0..recv_len {
            write_buffer[8 + i] = 0xFF;
        }

        self.spi.read_write_bytes(write_buffer, Some(read_buffer), 8 + recv_len);
    }

    fn read_bytes(&self,
                  mut write_buffer: &'static mut [u8],
                  mut read_buffer: &'static mut [u8],
                  recv_len: usize) {

        self.set_spi_fast_mode();

        // set write buffer to null transactions
        // Note: this could be optimized in the future by allowing SPI to read
        //  without a write buffer passed in
        let count = cmp::min(write_buffer.len(), recv_len);
        for i in 0..count {
            write_buffer[i] = 0xFF;
        }

        self.spi.read_write_bytes(write_buffer, Some(read_buffer), recv_len);
    }

    fn write_bytes(&self,
                   mut write_buffer: &'static mut [u8],
                   mut read_buffer: &'static mut [u8],
                   recv_len: usize) {

        self.set_spi_fast_mode();

        self.spi.read_write_bytes(write_buffer, Some(read_buffer), recv_len);
    }


    fn get_response(&self, response: SDResponse, read_buffer: &[u8]) -> (u8, u8, u32) {

        let mut r1: u8 = 0xFF;
        let mut r2: u8 = 0xFF;
        let mut r3: u32 = 0xFFFFFFFF;

        // scan through read buffer for response byte
        for i in 0..read_buffer.len() {
            if (read_buffer[i] & 0x80) == 0x00 {
                // status byte is always included
                r1 = read_buffer[i];

                match response {
                    SDResponse::R2 => {
                        // status, then read/write status. Unused in practice
                        if i + 1 < read_buffer.len() {
                            r2 = read_buffer[i + 1];
                        }
                    }
                    SDResponse::R3 | SDResponse::R7 => {
                        // status, then Operating Condition Register
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

    // updates SD card state on SPI transaction returns
    fn process_spi_states(&self,
                          mut write_buffer: &'static mut [u8],
                          mut read_buffer: &'static mut [u8],
                          _: usize) {

        match self.state.get() {
            State::SendACmd { acmd, arg } => {
                // send the application-specific command and resume the state
                //  machine
                self.state.set(self.after_state.get());
                self.after_state.set(State::Idle);
                self.send_command(acmd, arg, write_buffer, read_buffer, 10);
            }

            State::InitReset => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                // only continue if we are in idle state
                if r1 == 0x01 {
                    // next send Check Voltage Range command that is only valid
                    //  on SDv2 cards. This is used to check which SD card version
                    //  is installed
                    self.state.set(State::InitCheckVersion);
                    self.send_command(SDCmd::CMD8, 0x1AA, write_buffer, read_buffer, 10);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::InitCheckVersion => {
                // check response
                let (r1, _, r7) = self.get_response(SDResponse::R7, read_buffer);

                if r1 == 0x01 && r7 == 0x1AA {
                    // we have an SDv2 card
                    // send application-specific initialization in high capacity mode (HCS)
                    self.state.set(State::SendACmd {
                        acmd: SDCmd::ACMD41,
                        arg: 0x40000000,
                    });
                    self.after_state.set(State::InitRepeatHCSInit);
                    self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer, 10);
                } else {
                    // we have either an SDv1 or MMCv3 card
                    // send application-specific initialization
                    self.state.set(State::SendACmd {
                        acmd: SDCmd::ACMD41,
                        arg: 0x0,
                    });
                    self.after_state.set(State::InitAppSpecificInit);
                    self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer, 10);
                }
            }

            State::InitRepeatHCSInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // card initialized
                    // check card capacity
                    self.alarm_count.set(0);
                    self.state.set(State::InitCheckCapacity);
                    self.send_command(SDCmd::CMD58, 0x0, write_buffer, read_buffer, 10);
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
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::InitCheckCapacity => {
                // check response
                let (r1, _, r7) = self.get_response(SDResponse::R7, read_buffer);

                if r1 == 0x00 {
                    if (r7 & 0x40000000) != 0x00 {
                        self.card_type.set(CT_SD2 | CT_BLOCK);
                    } else {
                        self.card_type.set(CT_SD2);
                    }

                    // Read CSD register
                    // Note that the receive length needs to be increased here
                    //  to capture the 16-byte register (plus some slack)
                    self.state.set(State::InitComplete);
                    self.send_command(SDCmd::CMD9, 0x0, write_buffer, read_buffer, 28);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::InitAppSpecificInit => {
                // check response
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
                    self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer, 10);
                } else {
                    // MMCv3 card
                    // send generic intialization
                    self.card_type.set(CT_MMC);
                    self.state.set(State::InitRepeatGenericInit);
                    self.send_command(SDCmd::CMD1, 0x0, write_buffer, read_buffer, 10);
                }
            }

            State::InitRepeatAppSpecificInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // card initialized
                    // set blocksize to 512
                    self.alarm_count.set(0);
                    self.state.set(State::InitSetBlocksize);
                    self.send_command(SDCmd::CMD16, 512, write_buffer, read_buffer, 10);
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
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::InitRepeatGenericInit => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // card initialized
                    // set blocksize to 512
                    self.alarm_count.set(0);
                    self.state.set(State::InitSetBlocksize);
                    self.send_command(SDCmd::CMD16, 512, write_buffer, read_buffer, 10);
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
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::InitSetBlocksize => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // Read CSD register
                    // Note that the receive length needs to be increased here
                    //  to capture the 16-byte register (plus some slack)
                    self.state.set(State::InitComplete);
                    self.send_command(SDCmd::CMD9, 0x0, write_buffer, read_buffer, 28);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::InitComplete => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    let mut total_size: u64 = 0;

                    // find CSD register value
                    for i in 0..read_buffer.len() {
                        if read_buffer[i] == 0xFE && (i + 11 < read_buffer.len()) {
                            // get total size from CSD
                            if (read_buffer[i + 1] & 0xC0) == 0x00 {
                                // CSD version 1.0
                                let c_size = (((read_buffer[i + 7] & 0x03) as u32) << 10) |
                                             (((read_buffer[i + 8] & 0xFF) as u32) << 2) |
                                             (((read_buffer[i + 9] & 0xC0) as u32) >> 6);
                                let c_size_mult = (((read_buffer[i + 10] & 0x03) as u32) << 1) |
                                                  (((read_buffer[i + 11] & 0x80) as u32) >> 7);
                                let read_bl_len = (read_buffer[i + 6] & 0x0F) as u32;

                                let block_count = (c_size + 1) * (1 << (c_size_mult + 2));
                                let block_len = 1 << read_bl_len;
                                total_size = block_count as u64 * block_len as u64;
                            } else {
                                // CSD version 2.0
                                let c_size = (((read_buffer[i + 8] & 0x3F) as u32) << 16) |
                                             (((read_buffer[i + 9] & 0xFF) as u32) << 8) |
                                             ((read_buffer[i + 10] & 0xFF) as u32);
                                total_size = ((c_size as u64) + 1) * 512 * 1024;
                            }

                            break;
                        }
                    }

                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // initialization complete
                    self.state.set(State::Idle);
                    self.is_initialized.set(true);

                    // perform callback
                    self.client.get().map(move |client| { client.init_done(512, total_size); });
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client.get().map(move |client| {
                        client.error(ErrorCode::InitializationFailure as u32);
                    });
                }
            }

            State::StartReadBlocks { count } => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    if count <= 1 {
                        // check for data block to be ready
                        self.state.set(State::WaitReadBlock);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    } else {
                        // check for data block to be ready
                        self.state.set(State::WaitReadBlocks { count: count });
                        self.read_bytes(write_buffer, read_buffer, 1);
                    }
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client
                        .get()
                        .map(move |client| { client.error(ErrorCode::ReadFailure as u32); });
                }
            }

            State::WaitReadBlock => {
                if read_buffer[0] == 0xFE {
                    // data ready to read. Read block plus CRC
                    self.alarm_count.set(0);
                    self.state.set(State::ReadBlockComplete);
                    self.read_bytes(write_buffer, read_buffer, 512 + 2);
                } else if read_buffer[0] == 0xFF {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 1 ms
                    self.alarm_state.set(AlarmState::WaitForDataBlock);
                    let interval = (1 as u32) * <A::Frequency>::frequency() / 1000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client
                        .get()
                        .map(move |client| { client.error(ErrorCode::ReadFailure as u32); });
                }
            }

            State::ReadBlockComplete => {
                // replace buffers
                self.txbuffer.replace(write_buffer);
                self.rxbuffer.replace(read_buffer);

                // read finished, perform callback
                self.state.set(State::Idle);
                self.client_buffer.take().map(move |buffer| {
                    // copy data to user buffer
                    let read_len = cmp::min(buffer.len(), 512);
                    self.rxbuffer.map(|read_buffer| for i in 0..read_len {
                        buffer[i] = read_buffer[i];
                    });

                    // callback
                    self.client.get().map(move |client| { client.read_done(buffer, read_len); });
                });
            }

            State::WaitReadBlocks { count } => {
                if read_buffer[0] == 0xFE {
                    // data ready to read. Read block plus CRC
                    self.alarm_count.set(0);
                    self.state.set(State::ReceivedBlock { count: count });
                    self.read_bytes(write_buffer, read_buffer, 512 + 2);
                } else if read_buffer[0] == 0xFF {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 1 ms
                    self.alarm_state.set(AlarmState::WaitForDataBlocks { count: count });
                    let interval = (1 as u32) * <A::Frequency>::frequency() / 1000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client
                        .get()
                        .map(move |client| { client.error(ErrorCode::ReadFailure as u32); });
                }
            }

            State::ReceivedBlock { count } => {
                // copy block over to client buffer
                self.client_buffer.map(|buffer| {
                    let offset = self.client_offset.get();
                    let read_len = cmp::min(buffer.len(), 512 + offset);
                    for i in 0..read_len {
                        buffer[i] = read_buffer[i];
                    }
                    self.client_offset.set(offset + read_len);
                });

                if count <= 1 {
                    // all blocks received. Terminate multiple read
                    self.state.set(State::ReadBlocksComplete);
                    self.send_command(SDCmd::CMD12, 0x0, write_buffer, read_buffer, 10);
                } else {
                    // check for next data block to be ready
                    self.state.set(State::WaitReadBlocks { count: count - 1 });
                    self.read_bytes(write_buffer, read_buffer, 1);
                }
            }

            State::ReadBlocksComplete => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);

                    // read finished, perform callback
                    self.client_buffer.take().map(move |buffer| {
                        self.client.get().map(move |client| {
                            client.read_done(buffer, self.client_offset.get());
                        });
                    });
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client
                        .get()
                        .map(move |client| { client.error(ErrorCode::ReadFailure as u32); });
                }
            }

            State::StartWriteBlocks { count } => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    if count <= 1 {
                        // copy over data from client buffer
                        let remaining_bytes = self.client_buffer.map_or(512, |buffer| {
                            let write_len = cmp::min(buffer.len(), 512);

                            for i in 0..write_len {
                                write_buffer[i + 1] = buffer[i];
                            }

                            512 - write_len
                        });

                        // set a known value for remaining bytes
                        for i in 0..remaining_bytes {
                            write_buffer[i + 1] = 0xFF;
                        }

                        // set up data packet
                        write_buffer[0] = 0xFE; // Data token
                        write_buffer[513] = 0xFF; // dummy CRC
                        write_buffer[514] = 0xFF; // dummy CRC

                        // write data packet
                        self.state.set(State::WriteBlockResponse);
                        self.write_bytes(write_buffer, read_buffer, 515);
                    } else {
                        panic!("Multi-block SD card writes are unimplemented");
                    }
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client
                        .get()
                        .map(move |client| { client.error(ErrorCode::WriteFailure as u32); });
                }
            }

            State::WriteBlockResponse => {
                // Get data packet
                self.state.set(State::WriteBlockBusy);
                self.read_bytes(write_buffer, read_buffer, 1);
            }

            State::WriteBlockBusy => {
                if (read_buffer[0] & 0x1F) == 0x05 {
                    // check if sd card is busy
                    self.state.set(State::WaitWriteBlockBusy);
                    self.read_bytes(write_buffer, read_buffer, 1);
                } else {
                    // error, send callback and quit
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);
                    self.state.set(State::Idle);
                    self.alarm_state.set(AlarmState::Idle);
                    self.alarm_count.set(0);
                    self.client
                        .get()
                        .map(move |client| { client.error(ErrorCode::WriteFailure as u32); });
                }
            }

            State::WaitWriteBlockBusy => {
                if read_buffer[0] != 0x00 {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // read finished, perform callback
                    self.state.set(State::Idle);
                    self.alarm_count.set(0);
                    self.client_buffer.take().map(move |buffer| {
                        self.client.get().map(move |client| { client.write_done(buffer); });
                    });
                } else {
                    // replace buffers
                    self.txbuffer.replace(write_buffer);
                    self.rxbuffer.replace(read_buffer);

                    // try again after 1 ms
                    self.alarm_state.set(AlarmState::WaitForWriteBusy);
                    let interval = (1 as u32) * <A::Frequency>::frequency() / 1000;
                    let tics = self.alarm.now().wrapping_add(interval);
                    self.alarm.set_alarm(tics);
                }
            }

            State::Idle => {
                // receiving an event from Idle means something was killed

                // replace buffers
                self.txbuffer.replace(write_buffer);
                self.rxbuffer.replace(read_buffer);
            }
        }
    }

    // updates SD card state upon timer alarm fired
    fn process_alarm_states(&self) {
        // keep track of how many times the alarm has been called in a row
        let repeats = self.alarm_count.get();
        if repeats > 100 {
            // error, send callback and quit
            self.state.set(State::Idle);
            self.alarm_state.set(AlarmState::Idle);
            self.alarm_count.set(0);
            self.client
                .get()
                .map(move |client| { client.error(ErrorCode::TimeoutFailure as u32); });
        } else {
            self.alarm_count.set(repeats + 1);
        }

        match self.alarm_state.get() {
            AlarmState::DetectionChange => {
                // perform callback
                self.client
                    .get()
                    .map(move |client| { client.card_detection_changed(self.is_installed()); });

                // re-enable interrupts
                self.detect_changes();
                self.alarm_count.set(0);
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
                        self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer, 10);
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
                        self.send_command(SDCmd::CMD55, 0x0, write_buffer, read_buffer, 10);
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
                        self.send_command(SDCmd::CMD1, 0x0, write_buffer, read_buffer, 10);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::WaitForDataBlock => {
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
                        // wait until ready and then read data block, then done
                        self.state.set(State::WaitReadBlock);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::WaitForDataBlocks { count } => {
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
                        // wait until ready and then read data block, then done
                        self.state.set(State::WaitReadBlocks { count: count });
                        self.read_bytes(write_buffer, read_buffer, 1);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::WaitForWriteBusy => {
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
                        // check if sd card is busy
                        self.state.set(State::WaitWriteBlockBusy);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    });
                });

                self.alarm_state.set(AlarmState::Idle);
            }

            AlarmState::Idle => {
                // receiving an event from Idle means something was killed
                // do nothing
            }
        }
    }

    pub fn set_client<C: SDCardClient>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn is_installed(&self) -> bool {
        // if there is no detect pin, assume an sd card is installed
        self.detect_pin.get().map_or(true, |pin| {
            // sd card detection pin is active low
            pin.read() == false
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized.get()
    }

    // watches SD card detect pin for changes, sends callback on change
    pub fn detect_changes(&self) {
        self.detect_pin
            .get()
            .map(|pin| { pin.enable_interrupt(0, hil::gpio::InterruptMode::EitherEdge); });
    }

    pub fn initialize(&self) -> ReturnCode {
        // if not already, set card to uninitialized again
        self.is_initialized.set(false);

        // no point in initializing if the card is not installed
        if self.is_installed() {
            // reset the SD card in order to start initializing it
            self.txbuffer.take().map_or(ReturnCode::ENOMEM, |txbuffer| {
                self.rxbuffer.take().map_or(ReturnCode::ENOMEM, move |rxbuffer| {
                    self.state.set(State::InitReset);
                    self.send_command(SDCmd::CMD0, 0x0, txbuffer, rxbuffer, 10);

                    // command started successfully
                    ReturnCode::SUCCESS
                })
            })
        } else {
            // no sd card installed
            ReturnCode::EOFF
        }
    }

    pub fn read_blocks(&self, buffer: &'static mut [u8], sector: u32, count: u32) -> ReturnCode {
        // only if initialized and installed
        if self.is_installed() {
            if self.is_initialized() {
                self.txbuffer.take().map_or(ReturnCode::ENOMEM, |txbuffer| {
                    self.rxbuffer.take().map_or(ReturnCode::ENOMEM, move |rxbuffer| {
                        // save the user buffer for later
                        self.client_buffer.replace(buffer);
                        self.client_offset.set(0);

                        // convert block address to byte address for non-block
                        //  access cards
                        let mut address = sector;
                        if (self.card_type.get() & CT_BLOCK) != CT_BLOCK {
                            address *= 512;
                        }

                        self.state.set(State::StartReadBlocks { count: count });
                        if count == 1 {
                            self.send_command(SDCmd::CMD17, address, txbuffer, rxbuffer, 10);
                        } else {
                            self.send_command(SDCmd::CMD18, address, txbuffer, rxbuffer, 10);
                        }

                        // command started successfully
                        ReturnCode::SUCCESS
                    })
                })
            } else {
                // sd card not initialized
                ReturnCode::ERESERVE
            }
        } else {
            // sd card not installed
            ReturnCode::EOFF
        }
    }

    pub fn write_blocks(&self, buffer: &'static mut [u8], sector: u32, count: u32) -> ReturnCode {
        // only if initialized and installed
        if self.is_installed() {
            if self.is_initialized() {
                self.txbuffer.take().map_or(ReturnCode::ENOMEM, |txbuffer| {
                    self.rxbuffer.take().map_or(ReturnCode::ENOMEM, move |rxbuffer| {
                        // save the user buffer for later
                        self.client_buffer.replace(buffer);
                        self.client_offset.set(0);

                        // convert block address to byte address for non-block
                        //  access cards
                        let mut address = sector;
                        if (self.card_type.get() & CT_BLOCK) != CT_BLOCK {
                            address *= 512;
                        }

                        self.state.set(State::StartWriteBlocks { count: count });
                        if count == 1 {
                            self.send_command(SDCmd::CMD24, address, txbuffer, rxbuffer, 10);

                            // command started successfully
                            ReturnCode::SUCCESS
                        } else {
                            // can't write multiple blocks yet
                            //self.send_command(SDCmd::CMD18, address, txbuffer, rxbuffer, 10);
                            ReturnCode::ENOSUPPORT
                        }
                    })
                })
            } else {
                // sd card not initialized
                ReturnCode::ERESERVE
            }
        } else {
            // sd card not installed
            ReturnCode::EOFF
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
                                    self.process_spi_states(write_buffer, read_buffer, len);
                                });
    }
}

// Handle callbacks from the timer
impl<'a, A: hil::time::Alarm + 'a> hil::time::Client for SDCard<'a, A> {
    fn fired(&self) {
        self.process_alarm_states();
    }
}

// Handle callbacks from the card detection pin
impl<'a, A: hil::time::Alarm + 'a> hil::gpio::Client for SDCard<'a, A> {
    fn fired(&self, _: usize) {
        // check if there was an open transaction with the sd card
        if self.alarm_state.get() != AlarmState::Idle || self.state.get() != State::Idle {
            // something was running when this occurred. Kill the transaction and
            //  send an error callback
            self.state.set(State::Idle);
            self.alarm_state.set(AlarmState::Idle);
            self.client
                .get()
                .map(move |client| { client.error(ErrorCode::CardStateChanged as u32); });
        }

        // either the card is new or gone, in either case it isn't initialized
        self.is_initialized.set(false);

        // disable additional interrupts
        self.detect_pin.get().map(|pin| { pin.disable_interrupt(); });

        // run a timer for 500 ms in order to let the sd card settle
        self.alarm_state.set(AlarmState::DetectionChange);
        let interval = (500 as u32) * <A::Frequency>::frequency() / 1000;
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}


pub trait SDCardClient {
    fn card_detection_changed(&self, installed: bool);
    fn init_done(&self, block_size: u32, total_size: u64);
    fn read_done(&self, data: &'static mut [u8], len: usize);
    fn write_done(&self, buffer: &'static mut [u8]);
    fn error(&self, error: u32);
}

/// Holds buffers and whatnot that the application has passed us.
struct AppState {
    callback: Option<Callback>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    read_buffer: Option<AppSlice<Shared, u8>>,
}

// Application driver for SD Card capsule, layers on top of SD Card capsule
pub struct SDCardDriver<'a, A: hil::time::Alarm + 'a> {
    sdcard: &'a SDCard<'a, A>,
    app_state: MapCell<AppState>,
    kernel_buf: TakeCell<'static, [u8]>,
}

impl<'a, A: hil::time::Alarm + 'a> SDCardDriver<'a, A> {
    pub fn new(sdcard: &'a SDCard<'a, A>, kernel_buf: &'static mut [u8]) -> SDCardDriver<'a, A> {

        if kernel_buf.len() < 512 {}

        SDCardDriver {
            sdcard: sdcard,
            app_state: MapCell::empty(),
            kernel_buf: TakeCell::new(kernel_buf),
        }
    }
}

impl<'a, A: hil::time::Alarm + 'a> SDCardClient for SDCardDriver<'a, A> {
    fn card_detection_changed(&self, installed: bool) {
        self.app_state.map(|app_state| {
            app_state.callback.map(|mut cb| { cb.schedule(0, installed as usize, 0); });
        });
    }

    fn init_done(&self, block_size: u32, total_size: u64) {
        self.app_state.map(|app_state| {
            app_state.callback.map(|mut cb| {
                let size_in_kb = ((total_size >> 10) & 0xFFFFFFFF) as usize;
                cb.schedule(1, block_size as usize, size_in_kb);
            });
        });
    }

    fn read_done(&self, data: &'static mut [u8], len: usize) {
        self.kernel_buf.replace(data);
        self.app_state.map(|app_state| {

            let mut read_len: usize = 0;
            self.kernel_buf.map(|data| {
                app_state.read_buffer.as_mut().map(move |read_buffer| {
                    read_len = cmp::min(read_buffer.len(), cmp::min(data.len(), len));

                    let d = &mut read_buffer.as_mut()[0..(read_len as usize)];
                    for (i, c) in data[0..read_len].iter().enumerate() {
                        d[i] = *c;
                    }
                });
            });

            app_state.callback.map(|mut cb| { cb.schedule(2, read_len, 0); });
        });
    }

    fn write_done(&self, buffer: &'static mut [u8]) {
        self.kernel_buf.replace(buffer);

        self.app_state
            .map(|app_state| { app_state.callback.map(|mut cb| { cb.schedule(3, 0, 0); }); });
    }

    fn error(&self, error: u32) {
        self.app_state.map(|app_state| {
            app_state.callback.map(|mut cb| { cb.schedule(4, error as usize, 0); });
        });
    }
}

impl<'a, A: hil::time::Alarm + 'a> Driver for SDCardDriver<'a, A> {
    fn allow(&self, _appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            // Pass read buffer in from application
            0 => {
                if self.app_state.is_none() {
                    // create new app state
                    self.app_state.put(AppState {
                        callback: None,
                        read_buffer: Some(slice),
                        write_buffer: None,
                    });
                } else {
                    // app state exists, set read buffer
                    self.app_state.map(|appst| { appst.read_buffer = Some(slice); });
                }
                ReturnCode::SUCCESS
            }

            // Pass write buffer in from application
            1 => {
                if self.app_state.is_none() {
                    // create new app state
                    self.app_state.put(AppState {
                        callback: None,
                        read_buffer: None,
                        write_buffer: Some(slice),
                    });
                } else {
                    // app state exists, set write buffer
                    self.app_state.map(|appst| { appst.write_buffer = Some(slice); });
                }
                ReturnCode::SUCCESS
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            // Set callback
            0 => {
                if self.app_state.is_none() {
                    // create new app state
                    self.app_state.put(AppState {
                        callback: Some(callback),
                        read_buffer: None,
                        write_buffer: None,
                    });
                } else {
                    // app state exists, set callback
                    self.app_state.map(|appst| { appst.callback = Some(callback); });
                }
                ReturnCode::SUCCESS
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, data: usize, _: AppId) -> ReturnCode {
        match command_num {
            // check if present
            0 => ReturnCode::SUCCESS,

            // is_installed
            1 => {
                let value = self.sdcard.is_installed() as usize;
                ReturnCode::SuccessWithValue { value: value }
            }

            // initialize
            2 => self.sdcard.initialize(),

            // read_block
            3 => {
                self.kernel_buf.take().map_or(ReturnCode::EBUSY, |kernel_buf| {
                    self.sdcard.read_blocks(kernel_buf, data as u32, 1)
                })
            }

            // write_block
            4 => {
                self.app_state.map_or(ReturnCode::ENOMEM, |app_state| {
                    app_state.write_buffer.as_mut().map_or(ReturnCode::ENOMEM, |write_buffer| {
                        self.kernel_buf.take().map_or(ReturnCode::EBUSY, |kernel_buf| {
                            // Check bounds for write length
                            let write_len = cmp::min(write_buffer.len(),
                                                     cmp::min(kernel_buf.len(), 512));

                            // copy over data
                            let d = &mut write_buffer.as_mut()[0..write_len];
                            for (i, c) in kernel_buf[0..write_len].iter_mut().enumerate() {
                                *c = d[i];
                            }

                            self.sdcard.write_blocks(kernel_buf, data as u32, 1)
                        })
                    })
                })
            }

            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
