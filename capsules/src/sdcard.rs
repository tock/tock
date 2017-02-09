//! Provide capsule driver for accessing an SD Card.
//! This allows initialization and block reads or writes on top of SPI

// Resources for SD Card API
// elm-chan.org/docs/mmc/mmc_e.html
// alumni.cs.ucr.edu/~amitra/sdcard/Additional/sdcard_appnote_foust.pdf
// luckyresistor.me/cat-protector/software/sdcard-2/
// http://users.ece.utexas.edu/~valvano/EE345M/SD_Physical_Layer_Spec.pdf

use core::cell::Cell;
use core::cmp;
use kernel::common::take_cell::TakeCell;

use kernel::hil;
use kernel::hil::time::Frequency;

// Buffers used for SD card transactions
// Constraints:
//  * RXBUFFER must be greater than or equal to TXBUFFER in length
//  * Both RXBUFFER and TXBUFFER must be longer  than the SD card's block size
pub static mut TXBUFFER: [u8; 1024] = [0; 1024]; //XXX: This can get MUCH smaller, but only if we fix SPI
pub static mut RXBUFFER: [u8; 1024] = [0; 1024]; //XXX: I think this is going to have to stay huge

//XXX: FOR TESTING
pub static mut CLIENT_BUFFER: [u8; 1024] = [0; 1024];

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
    //WaitForWritesBusy { count: u32 },
}

// SD Card types
const CT_MMC: u8 = 0x01;
const CT_SD1: u8 = 0x02;
const CT_SD2: u8 = 0x04;
const CT_BLOCK: u8 = 0x08;

pub trait SDCardClient {
    fn card_detection_changed(&self, installed: bool);
    fn init_done(&self, block_size: u32, total_size: u64);
    fn read_done(&self, data: &'static mut [u8], len: usize);
    fn write_done(&self, buffer: &'static mut [u8]);
    fn error(&self, error: u32); //XXX: How do we handle errors with this interface?
}

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
}

impl<'a, A: hil::time::Alarm + 'a> SDCard<'a, A> {
    pub fn new(spi: &'a hil::spi::SpiMasterDevice,
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

        if self.is_initialized.get() {
            // device is already initialized
            self.set_spi_fast_mode();
        } else {
            // device is still being initialized
            self.set_spi_slow_mode();
        }

        // check that write buffer is long enough
        if write_buffer.len() < 8+recv_len {
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
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
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
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
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
                } else {
                    panic!("Error in {:?}. R1: {}", self.state.get(), r1);
                }

                // Read CSD register
                // Note that the receive length needs to be increased here
                //  to capture the 16-byte register (plus some slack)
                self.state.set(State::InitComplete);
                self.send_command(SDCmd::CMD9, 0x0, write_buffer, read_buffer, 28);
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
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
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
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
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
                    panic!("Error in {:?}. R1: {}", self.state.get(), r1);
                }
            }

            State::InitComplete => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    let mut total_size: u64 = 0;

                    // find CSD register value
                    for i in 0..read_buffer.len() {
                        if read_buffer[i] == 0xFE && (i+11 < read_buffer.len()) {
                            // get total size from CSD
                            if (read_buffer[i+1] & 0xC0) == 0x00 {
                                // CSD version 1.0
                                let c_size = (((read_buffer[i+7] & 0x03) as u32) << 10) |
                                             (((read_buffer[i+8] & 0xFF) as u32) <<  2) |
                                             (((read_buffer[i+9] & 0xC0) as u32) >>  6);
                                let c_size_mult = (((read_buffer[i+10] & 0x03) as u32) << 1) |
                                                  (((read_buffer[i+11] & 0x80) as u32) >> 7);
                                let read_bl_len = (read_buffer[i+6] & 0x0F) as u32;

                                let block_count = (c_size+1) * (1 << (c_size_mult+2));
                                let block_len = 1 << read_bl_len;
                                total_size = block_count as u64 * block_len as u64;
                            } else {
                                // CSD version 2.0
                                let c_size = (((read_buffer[i+8] & 0x3F) as u32) << 16) |
                                             (((read_buffer[i+9] & 0xFF) as u32) <<  8) |
                                             ((read_buffer[i+10] & 0xFF) as u32);
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

                    /*
                    // perform callback
                    self.client.map(|client| {
                        client.init_done(512, total_size);
                    });
                    panic!("Init complete: {} bytes", total_size);
                    */

                    /* TESTING
                     * WHERE
                    */
                    unsafe{self.read_blocks(&mut CLIENT_BUFFER, 0, 3);} // Read blocks from memory
                    //unsafe{self.write_blocks(&mut CLIENT_BUFFER, 0, 1);} // Write blocks to memory
                } else {
                    panic!("Error in {:?}. R1: {}", self.state.get(), r1);
                }
            }

            State::StartReadBlocks { count } => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    if count == 1 {
                        // check for data block to be ready
                        self.state.set(State::WaitReadBlock);
                        self.read_bytes(write_buffer, read_buffer, 1);
                    } else {
                        // check for data block to be ready
                        self.state.set(State::WaitReadBlocks{ count: count });
                        self.read_bytes(write_buffer, read_buffer, 1);
                    }
                } else {
                    panic!("Error in {:?}. R1: {}", self.state.get(), r1);
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
                    panic!("Error in {:?}. Read Response: {}", self.state.get(), read_buffer[0]);
                }
            }

            State::ReadBlockComplete => {
                //XXX: Block read complete, copy to client buffer and do callback
                panic!("Got a buffer!");
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
                    panic!("Error in {:?}. Read Response: {}", self.state.get(), read_buffer[0]);
                }
            }

            State::ReceivedBlock { count } => {
                //XXX: copy data to client buffer

                if count <= 1 {
                    // all blocks received. Terminate multiple read
                    self.state.set(State::ReadBlocksComplete);
                    self.send_command(SDCmd::CMD12, 0x0, write_buffer, read_buffer, 10);
                } else {
                    // check for next data block to be ready
                    self.state.set(State::WaitReadBlocks{ count: count-1 });
                    self.read_bytes(write_buffer, read_buffer, 1);
                }
            }

            State::ReadBlocksComplete => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    //XXX: read blocks complete, do callback
                    panic!("Read blockS complete");
                } else {
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
                }
            }

            State::StartWriteBlocks { count } => {
                // check response
                let (r1, _, _) = self.get_response(SDResponse::R1, read_buffer);

                if r1 == 0x00 {
                    if count == 1 {
                        // ensure that the write buffer is long enough
                        if write_buffer.len() < 515 {
                            panic!("Write buffer not long enough to store entire SD card block");
                        }

                        // set up data packet
                        write_buffer[0] = 0xFE; // Data token
                        write_buffer[513] = 0xFF; // dummy CRC
                        write_buffer[514] = 0xFF; // dummy CRC
                        //XXX: copy over data from client buffer

                        //XXX: TESTING
                        for i in 1..512 {
                            write_buffer[i] = i as u8;
                        }

                        // write data packet
                        self.state.set(State::WriteBlockResponse);
                        self.write_bytes(write_buffer, read_buffer, 515);
                    } else {
                        panic!("IMPLEMENTME");
                    }
                } else {
                    panic!("Error in {:?}. R1: {:#X}", self.state.get(), r1);
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
                    panic!("Error in {:?}. Data Response: {:#X}", self.state.get(), read_buffer[0]);
                }
            }

            State::WaitWriteBlockBusy => {
                if read_buffer[0] != 0x00 {
                    //XXX: write successfully completed do something about that
                    self.alarm_count.set(0);
                    panic!("Write complete");
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
            panic!("{:?} timed out.", self.alarm_state.get());
        } else {
            self.alarm_count.set(repeats+1);
        }

        match self.alarm_state.get() {
            AlarmState::DetectionChange => {
                if self.is_installed() {
                    panic!("SD Card installed");
                } else {
                    panic!("SD Card gone");
                }

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
                        self.state.set(State::WaitReadBlocks{ count: count });
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

    // watches SD card detect pin for changes, sends callback on change
    pub fn detect_changes(&self) {
        self.detect_pin
            .get()
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
                    self.state.set(State::InitReset);
                    self.send_command(SDCmd::CMD0, 0x0, txbuffer, rxbuffer, 10);
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

                    // convert block address to byte address for non-block
                    //  access cards
                    let mut address = sector;
                    if (self.card_type.get() & CT_BLOCK) != CT_BLOCK {
                        address *= 512;
                    }

                    //XXX: do something if count is 0 or the buffer can't hold a block
                    // is it valid to read a partial block?

                    self.state.set(State::StartReadBlocks { count: count });
                    if count == 1 {
                        self.send_command(SDCmd::CMD17, address, txbuffer, rxbuffer, 10);
                    } else {
                        self.send_command(SDCmd::CMD18, address, txbuffer, rxbuffer, 10);
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

                    // convert block address to byte address for non-block
                    //  access cards
                    let mut address = sector;
                    if (self.card_type.get() & CT_BLOCK) != CT_BLOCK {
                        address *= 512;
                    }

                    //XXX: do something if count is 0 or the buffer doesn't hold a block
                    // is it valid to write a partial block?

                    self.state.set(State::StartWriteBlocks { count: count });
                    if count == 1 {
                        self.send_command(SDCmd::CMD24, address, txbuffer, rxbuffer, 10);
                    } else {
                        panic!("Can't write multiple blocks yet");
                        /*
                        self.send_command(SDCmd::CMD18, address, txbuffer, rxbuffer, 10);
                        */
                    }

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
            //XXX: send an error callback
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


// Application driver for SD Card capsule, layers on top of SD Card capsule
pub struct SDCardDriver<'a, A: hil::time::Alarm + 'a> {
    sdcard: &'a SDCard<'a, A>,
}
