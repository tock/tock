//! Provides userspace access to LEDs on a board.
//!
//! This allows for much more cross platform controlling of LEDs without having
//! to know which of the GPIO pins exposed across the syscall interface are
//! LEDs.
//!
//! This capsule takes an array of pins and the polarity of the LED (active high
//! or active low). This allows the board to configure how the underlying GPIO
//! must be controlled to turn on and off LEDs, such that the syscall driver
//! interface can be agnostic to the LED polarity.
//!
//! Usage
//! -----
//!
//! ```rust
//! let led_pins = static_init!(
//!     [(&'static sam4l::gpio::GPIOPin, capsules::led::ActivationMode); 3],
//!     [(&sam4l::gpio::PA[13], capsules::led::ActivationMode::ActiveLow),   // Red
//!      (&sam4l::gpio::PA[15], capsules::led::ActivationMode::ActiveLow),   // Green
//!      (&sam4l::gpio::PA[14], capsules::led::ActivationMode::ActiveLow)]); // Blue
//! let led = static_init!(
//!     capsules::led::LED<'static, sam4l::gpio::GPIOPin>,
//!     capsules::led::LED::new(led_pins));
//! ```
//!
//! Syscall Interface
//! -----------------
//!
//! - Stability: 2 - Stable
//!
//! ### Command
//!
//! All LED operations are synchronous, so this capsule only uses the `command`
//! syscall.
//!
//! #### `command_num`
//!
//! - `0`: Return the number of LEDs on this platform.
//!   - `data`: Unused.
//!   - Return: Number of LEDs.
//! - `1`: Turn the LED on.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.
//! - `2`: Turn the LED off.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.
//! - `3`: Toggle the on/off state of the LED.
//!   - `data`: The index of the LED. Starts at 0.
//!   - Return: `SUCCESS` if the LED index was valid, `EINVAL` otherwise.

use kernel::hil::gpio;
use kernel::hil::time::{self, Alarm, Frequency};
// use std::thread::sleep;
// use std::ops::BitOrAssign;
use kernel::{AppId, Driver, ReturnCode, debug, Shared, Grant, AppSlice, capabilities, create_capability};
use core::cell::Cell;
use kernel::common::cells::{TakeCell};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lcd1602 as usize;

static LOW: u8 = 0;
static HIGH: u8 = 1;

// commands
static LCD_CLEARDISPLAY: u8 = 0x01;
// static LCD_RETURNHOME: u8 = 0x02;
static LCD_ENTRYMODESET: u8 = 0x04;
static LCD_DISPLAYCONTROL: u8 = 0x08;
// static LCD_CURSORSHIFT: u8 = 0x10;
static LCD_FUNCTIONSET: u8 = 0x20;
// static LCD_SETCGRAMADDR: u8 = 0x40;
static LCD_SETDDRAMADDR: u8 = 0x80;

// flags for display entry mode
// static LCD_ENTRYRIGHT: u8 = 0x00;
static LCD_ENTRYLEFT: u8 = 0x02;
// static LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
static LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// flags for display on/off control
static LCD_DISPLAYON: u8 =  0x04;
// static LCD_DISPLAYOFF: u8 =  0x00;
static LCD_CURSORON: u8 =  0x02;
// static LCD_CURSOROFF: u8 =  0x00;
static LCD_BLINKON: u8 =  0x01;
static LCD_BLINKOFF: u8 =  0x00;

// flags for display/cursor shift
// static LCD_DISPLAYMOVE: u8 = 0x08;
// static LCD_CURSORMOVE: u8 = 0x00;
// static LCD_MOVERIGHT: u8 = 0x04;
// static LCD_MOVELEFT: u8 = 0x00;

// flags for function set
static LCD_8BITMODE: u8 = 0x10;
static LCD_4BITMODE: u8 = 0x00;
static LCD_2LINE: u8 = 0x08;
static LCD_1LINE: u8 = 0x00;
// static LCD_5X10DOTS: u8 = 0x04;
static LCD_5X8DOTS: u8 = 0x00;

pub static mut BUFFER: [u8; 700] = [0; 700];
pub static mut ROW_OFFSETS: [u8; 4] = [0; 4];

#[derive(Default)]
pub struct App {
    text_buffer: Option<AppSlice<Shared, u8>>, 
}
#[derive(Copy, Clone, PartialEq)]
enum LCDStatus {
    Idle,
    Begin0,
    Begin0_1,
    Begin1,
    Begin1_2,
    Begin2,
    Begin2_3,
    Begin3,
    Begin4,
    Begin5,
    Begin5_6,
    Begin6,
    Begin7,
    Begin7_8,
    Begin8,
    Begin9,
    Begin10,
    Begin11,
    Begin12,
    Begin13,
    Begin14,
    Begin15,
    Begin16,
    Begin17,
    Begin18,
    Other,
    Printing0,
    Printing1,

    Pulse0,
    Pulse1,
}

pub struct LCD<'a, A: Alarm<'a>> {
    rs_pin: &'a dyn gpio::Pin,
    en_pin: &'a dyn gpio::Pin,
    data_4_pin: &'a dyn gpio::Pin,
    data_5_pin: &'a dyn gpio::Pin,
    data_6_pin: &'a dyn gpio::Pin,
    data_7_pin: &'a dyn gpio::Pin,
    display_function: Cell<u8>,
    display_control: Cell<u8>,
    display_mode: Cell<u8>,
    num_lines: Cell<u8>,
    current_method: Cell<i8>,
    current_counter: Cell<i8>,
    row_offsets: TakeCell<'static, [u8]>,
    command_buffer: TakeCell<'static, [u8]>,
    command_len: Cell<u8>,
    command_offset: Cell<u8>,
    alarm: &'a A,
    apps: Grant<App>,
    lcd_status: Cell<LCDStatus>,
    lcd_prev_status: Cell<LCDStatus>,
}

impl<'a, A: Alarm<'a>> LCD<'a, A> {
    pub fn new(
        alarm: &'a A,
        rs_pin: &'a dyn gpio::Pin,
        en_pin: &'a dyn gpio::Pin,
        data_4_pin: &'a dyn gpio::Pin,
        data_5_pin: &'a dyn gpio::Pin,
        data_6_pin: &'a dyn gpio::Pin,
        data_7_pin: &'a dyn gpio::Pin,
        command_buffer: &'static mut [u8],
        row_offsets: &'static mut [u8],
        board_kernel: &'static kernel::Kernel,
        grant: Grant<App>,
    ) -> LCD<'a, A> {
        // Make all pins output and off
        rs_pin.make_output();
        en_pin.make_output();
        data_4_pin.make_output();
        data_5_pin.make_output();
        data_6_pin.make_output();
        data_7_pin.make_output();
        LCD {
            rs_pin: rs_pin,
            en_pin: en_pin,
            data_4_pin: data_4_pin,
            data_5_pin: data_5_pin,
            data_6_pin: data_6_pin,
            data_7_pin: data_7_pin,
            display_function: Cell::new (LCD_4BITMODE | LCD_1LINE | LCD_5X8DOTS),
            display_control: Cell::new (0),
            display_mode: Cell::new(0),
            num_lines: Cell::new(0),
            current_method: Cell::new(0),
            current_counter: Cell::new(0),
            row_offsets: TakeCell::new(row_offsets),
            command_buffer: TakeCell::new(command_buffer),
            command_len: Cell::new(0),
            command_offset: Cell::new(0),
            alarm: alarm,
            apps: grant,
            lcd_status: Cell::new(LCDStatus::Idle),
            lcd_prev_status: Cell::new(LCDStatus::Idle),
        }
    }

  
    // pub fn start(&self, timer: u32) {
    //     // debug!(" {} ", <A::Frequency>::frequency()/timer);
        
    //     debug! ("aici?");
    // }

    fn set_rows(&self, row0: u8, row1: u8, row2: u8, row3: u8) -> ReturnCode {
        self.row_offsets.map(|buffer| {
            buffer[0] = row0;
            buffer[1] = row1;
            buffer[2] = row2;
            buffer[3] = row3;
        });
        ReturnCode::SUCCESS
    }

    

    fn handle_commands(&self) -> ReturnCode {
        // debug! ("am fost apelati");
        if self.lcd_status.get() == LCDStatus::Idle {
            let len = self.command_len.get() as usize;
            let mut offset = self.command_offset.get() as usize;
            // debug! ("len {} offset {} ", len, offset);
            self.command_buffer.map(|buffer| {
                while offset < len {
                    // debug! ("{} offset {} len", offset, len);
                    // for i in 0..len {
                    //     debug! ("pe poz {} aici avem {}", i as usize, buffer[i as usize]);
                    // }
                    let current = buffer[offset];

                    match current {
                        // begin
                        130 => {
                            let cols = buffer[offset + 1];
                            let lines = buffer[offset + 2];

                            if lines > 1 {
                                self.display_function.replace(self.display_function.get() | LCD_2LINE);
                            }

                            self.num_lines.replace(lines);
                            self.set_rows(0x00, 0x40, 0x00 + cols, 0x40 + cols);
    
                            self.lcd_prev_status.replace(LCDStatus::Idle);
                            self.lcd_status.replace(LCDStatus::Begin0);

                            self.alarm.set_alarm(
                                self.alarm.now().wrapping_add(<A::Frequency>::frequency()/10)
                            );
                            break;
                        },

                        131 => {
                            let mut col_number: u8 = buffer[offset + 1];
                            let mut line_number: u8 = buffer[offset + 2];

                            if line_number >= 4 {
                                line_number = 3;
                            }

                            if line_number >= self.num_lines.get() {
                                line_number = self.num_lines.get() - 1;
                            }

                            self.lcd_prev_status.replace(LCDStatus::Other);
                            self.rs_pin.clear();

                            let mut value: u8 = 0;
                            self.row_offsets.map(|buffer| {
                                value = buffer[self.num_lines.get() as usize];
                            });
                            self.command_offset.replace((offset + 3) as u8);
                            self.write_4_bits(LCD_SETDDRAMADDR | (col_number + value));
                            break;
                        },

                         _ => {
                            // command cu HIGH
                            debug!("avem de scris {} ", current >> 4);
                            self.rs_pin.set();
                            self.lcd_prev_status.replace(LCDStatus::Printing0);
                            
                            // offset = offset + 1;
                            // self.command_offset.replace(offset as u8);
                            
                            self.write_4_bits(current >> 4);
                            break;
                        }
                    }
                }
                offset += 1;
            });
        }
        ReturnCode::SUCCESS
    }
   
    fn pulse(&self) {
        // if self.lcd_prev_status.get() == LCDStatus::Begin12 {
        //     debug! ("in pulse, urmeaza pulse0");
        // }
        self.en_pin.clear();
        self.lcd_status.replace(LCDStatus::Pulse0);

        self.alarm.set_alarm(
            self.alarm.now().wrapping_add(<A::Frequency>::frequency()/500)
        )
    }

    fn write_4_bits(&self, value: u8) {
        // if self.lcd_prev_status.get() == LCDStatus::Begin12 {
        //     debug! ("in write_4_bits");
        // }
        debug! ("printam {} ", value);
        if (value >> 0) & 0x01 != 0 {
            self.data_4_pin.set();
        } else {
            self.data_4_pin.clear();
        }

        if (value >> 1) & 0x01 != 0 {
            self.data_5_pin.set();
        } else {
            self.data_5_pin.clear();
        }

        if (value >> 2) & 0x01 != 0 {
            self.data_6_pin.set();
        } else {
            self.data_6_pin.clear();
        }

        if (value >> 3) & 0x01 != 0 {
            self.data_7_pin.set();
        } else {
            self.data_7_pin.clear();
        }

        self.pulse();
    }

    fn set_delay(&self, timer: u32) {
        self.alarm.set_alarm(
            self.alarm.now().wrapping_add(<A::Frequency>::frequency()/timer)
        )
    }
}

impl<'a, A:Alarm<'a>> Driver for LCD<'a, A> {
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    if let Some(ref s) = slice {
                        // debug! ("am primit {:?}", s.len());
                        let mut leng = self.command_len.get() as usize;
                       
                        self.command_buffer.map(|buffer| {
                            for byte in s.iter() {
                                buffer[leng] = *byte;
                                leng += 1;
                            }
                        });
                        self.command_len.replace(leng as u8);
                    }; 
                    app.text_buffer = slice;
                   
                    // debug! (" am pus pana acum {} chestii in vector/900", self.command_len.get());
                    self.handle_commands();
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
    
    fn command(&self, command_num: usize, data: usize, data_3: usize, _: AppId) -> ReturnCode {
        // debug! (" col {}", data);
        // debug! (" row {}", data_3);
        match command_num {
            // begin
            0 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = 130;
                    index += 1;
                    buffer[index] = data as u8;
                    index += 1;
                    buffer[index] = data_3 as u8;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
               
                ReturnCode::SUCCESS              
            }

            1 => {
                ReturnCode::SUCCESS
            },

            // set cursor
            5 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = 131;
                    index += 1;
                    buffer[index] = data as u8;
                    index += 1;
                    buffer[index] = data_3 as u8;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
                
                ReturnCode::SUCCESS
            }

            // print number
            6 => {
                ReturnCode::SUCCESS
            }

            // print string
            4 => {
                ReturnCode::SUCCESS
            }

            // Home
            2 => {
                ReturnCode::SUCCESS
            }

            // clear
            3 => {
                ReturnCode::SUCCESS
            }

            // Display/No
            7 => {
                match data {
                    // Display
                    0 => {

                    }

                    // no display
                    1 => {

                    }

                    _ => {}
                }
                ReturnCode::SUCCESS
            }

            // Blink/No
            8 => {
                match data {
                    // Blink
                    0 => {

                    }

                    // no blink
                    1 => {

                    }

                    _ => {}
                }
                ReturnCode::SUCCESS
            }
            
            // Cursor/No
            9 => {
                // Cursor
                match data {
                    0 => {

                    }
    
                    // no cursor
                    1 => {
    
                    }
    
                    _ => {}
                }
                ReturnCode::SUCCESS
            }

            // Scroll Left/Right
            10 => {
                match data {
                    // scroll left
                    0 => {

                    }

                    // scroll right
                    1 => {

                    }

                    _ => {}
                }
                ReturnCode::SUCCESS
            }

            // Left/Right to Right/Left
            11 => {
                // Left to Right
                match data {
                    0 => {

                    }
    
                    // Right to Left
                    1 => {
    
                    }
    
                    _ => {}
                }
                ReturnCode::SUCCESS
            }

            // Autoscroll/No
            12 => {
                match data {
                     // Autoscroll
                0 => {

                }

                // no autoscroll
                1 => {

                }

                _ => {}
                }    
                ReturnCode::SUCCESS
            }
            // default
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for LCD<'a, A> {
    fn fired(&self) {
        let state = self.lcd_status.get();

        match state {
            LCDStatus::Idle => {
                // debug!("in idle");
                self.handle_commands();
            },

            LCDStatus::Begin0 => {
                // debug!("in begin0");
                self.rs_pin.clear();
                self.en_pin.clear();


                if (self.display_function.get() & LCD_8BITMODE) == 0 {
                    self.lcd_prev_status.replace(LCDStatus::Begin0);
                    
                    self.write_4_bits(0x03);
                } else {
                    self.lcd_prev_status.replace(LCDStatus::Begin4);
                    self.rs_pin.clear();

                    self.write_4_bits((LCD_FUNCTIONSET | self.display_function.get()) >> 4);
                }
            },

            LCDStatus::Begin0_1 => {
                // debug!("in begin0_1");
                self.lcd_prev_status.replace(LCDStatus::Begin0_1);
                self.lcd_status.replace(LCDStatus::Begin1);

                self.set_delay(200);
            },

            LCDStatus::Begin1 => {
                // debug!("in begin1");
                self.lcd_prev_status.replace(LCDStatus::Begin1);
                self.write_4_bits(0x03);
            },

            LCDStatus::Begin1_2 => {
                // debug!("in begin1_2");
                self.lcd_prev_status.replace(LCDStatus::Begin1_2);
                self.lcd_status.replace(LCDStatus::Begin2);

                self.set_delay(200);
            }, 

            LCDStatus::Begin2 => {
                // debug!("in begin2");
                self.lcd_prev_status.replace(LCDStatus::Begin2);
                self.write_4_bits(0x03);
            },

            LCDStatus::Begin2_3 => {
                // debug!("in begin2_3");
                self.lcd_prev_status.replace(LCDStatus::Begin2_3);
                self.lcd_status.replace(LCDStatus::Begin3);

                self.set_delay(500);
            }, 

            LCDStatus::Begin3 => {
                // debug!("in begin3");
                self.lcd_prev_status.replace(LCDStatus::Begin3);
                self.write_4_bits(0x02);
                
            },

            LCDStatus::Begin5 => {
                // debug!("in begin5");
                self.lcd_prev_status.replace(LCDStatus::Begin5);
                self.write_4_bits((LCD_FUNCTIONSET | self.display_function.get()) >> 4);
            },

            LCDStatus::Begin5_6 => {
                // debug!("in begin5_6");
                self.lcd_prev_status.replace(LCDStatus::Begin5_6);
                self.lcd_status.replace(LCDStatus::Begin6);

                self.set_delay(200);
            },

            LCDStatus::Begin6 => {
                // debug!("in begin6");
                self.lcd_prev_status.replace(LCDStatus::Begin6);

                self.rs_pin.clear();
                self.write_4_bits((LCD_FUNCTIONSET | self.display_function.get()) >> 4);
            },

            LCDStatus::Begin7 => {
                // debug!("in begin7");
                self.lcd_prev_status.replace(LCDStatus::Begin7);
                self.write_4_bits(LCD_FUNCTIONSET | self.display_function.get());
            },

            LCDStatus::Begin7_8 => {
                // debug!("in begin7_8");
                self.lcd_prev_status.replace(LCDStatus::Begin7_8);
                self.lcd_status.replace(LCDStatus::Begin8);

                self.set_delay(500);
                // self.alarm.set_alarm(
                //     self.alarm.now().wrapping_add(<A::Frequency>::frequency()/500);
                // )
            },

            LCDStatus::Begin8 => {
                // debug!("in begin8");
                self.lcd_prev_status.replace(LCDStatus::Begin8);

                self.rs_pin.clear();
                self.write_4_bits((LCD_FUNCTIONSET | self.display_function.get()) >> 4);
            },

            LCDStatus::Begin9 => {
                // debug!("in begin9");
                self.lcd_prev_status.replace(LCDStatus::Begin9);
                self.write_4_bits(LCD_FUNCTIONSET | self.display_function.get());
            }

            LCDStatus::Begin10 => {
                // debug!("in begin10");
                self.lcd_prev_status.replace(LCDStatus::Begin10);

                self.rs_pin.clear();  
                self.write_4_bits((LCD_FUNCTIONSET | self.display_function.get()) >> 4);
                
            },

            LCDStatus::Begin11 => {
                // debug!("in begin11");
                self.lcd_prev_status.replace(LCDStatus::Begin11);
                self.write_4_bits(LCD_FUNCTIONSET | self.display_function.get());
            },

            LCDStatus::Begin12 => {
                // debug!("in begin12");
                self.lcd_prev_status.replace(LCDStatus::Begin12);
                self.rs_pin.clear();

                self.write_4_bits((LCD_DISPLAYON | LCD_CURSORON | LCD_BLINKOFF | LCD_DISPLAYCONTROL) >> 4);
            },

            LCDStatus::Begin13 => {
                // debug!("in begin13");
                self.lcd_prev_status.replace(LCDStatus::Begin13);
                self.write_4_bits(LCD_DISPLAYON | LCD_CURSORON | LCD_BLINKON | LCD_DISPLAYCONTROL);
            },

            LCDStatus::Begin14 => {
                // debug!("in begin14");
                self.lcd_prev_status.replace(LCDStatus::Begin14);
                self.rs_pin.clear();

                self.write_4_bits(LCD_CLEARDISPLAY >> 4);
            },

            LCDStatus::Begin15 => {
                // debug!("in begin15");
                self.lcd_prev_status.replace(LCDStatus::Begin15);
                self.write_4_bits(LCD_CLEARDISPLAY);
            },

            LCDStatus::Begin16 => {
                // debug!("in begin16");
                self.lcd_prev_status.replace(LCDStatus::Begin16);
                self.lcd_status.replace(LCDStatus::Begin18);
   
                self.set_delay(500);
                // self.alarm.set_alarm(
                //     self.alarm.now().wrapping_add(<A::Frequency>::frequency()/500);
                // )
            },

            LCDStatus::Begin17 => {
                // debug!("in begin17");
                self.lcd_prev_status.replace(LCDStatus::Begin17);

                self.rs_pin.clear();  
                self.write_4_bits((LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT | LCD_ENTRYMODESET) >> 4);
            }

            LCDStatus::Begin18 => {
                // debug!("in begin18");
                self.lcd_prev_status.replace(LCDStatus::Begin18);

                self.write_4_bits(LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT | LCD_ENTRYMODESET);
            },

            LCDStatus::Printing1 => {
                let offset = self.command_offset.get() as usize;
                let mut current = 0; 
                self.command_buffer.map(|buffer| {
                    current = buffer[offset];
                });
                // debug! ("{}", current);
                debug! ("avem de printat a doua oara {}", current & 15);
                self.lcd_prev_status.replace(LCDStatus::Printing1);
                self.write_4_bits(current & 15);
            }

            LCDStatus::Pulse0 => {
                // debug!("in pulse0");
                self.en_pin.set();

                self.lcd_status.replace(LCDStatus::Pulse1);

                self.set_delay(500);
                // self.alarm.set_alarm(
                //     self.alarm.now().wrapping_add(<A::Frequency>::frequency()/500);
                // )
            },

            LCDStatus::Pulse1 => {
                // debug!("in pulse1");
                
                let prev_state = self.lcd_prev_status.get();

                match prev_state {
                    LCDStatus::Begin0 => {
                        self.lcd_status.replace(LCDStatus::Begin0_1);
                    }, 

                    LCDStatus::Begin1 => {
                        self.lcd_status.replace(LCDStatus::Begin1_2);
                    }, 

                    LCDStatus::Begin2 => {
                        self.lcd_status.replace(LCDStatus::Begin2_3);
                    }, 

                    LCDStatus::Begin3 => {
                        self.lcd_status.replace(LCDStatus::Begin10);
                    },

                    LCDStatus::Begin4 => {
                        self.lcd_status.replace(LCDStatus::Begin5);
                    },

                    LCDStatus::Begin5 => {
                        self.lcd_status.replace(LCDStatus::Begin5_6);
                    },

                    LCDStatus::Begin6 => {
                        self.lcd_status.replace(LCDStatus::Begin7);
                    },

                    LCDStatus::Begin7 => {
                        self.lcd_status.replace(LCDStatus::Begin7_8);
                    },

                    LCDStatus::Begin8 => {
                        self.lcd_status.replace(LCDStatus::Begin9);
                    },

                    LCDStatus::Begin9 => {
                        self.lcd_status.replace(LCDStatus::Begin10);
                    }

                    LCDStatus::Begin10 => {
                        self.lcd_status.replace(LCDStatus::Begin11);
                    },

                    LCDStatus::Begin11 => {
                        self.lcd_status.replace(LCDStatus::Begin12);
                    },
                    
                    LCDStatus::Begin12 => {
                        // debug! ("ajung vreodata aici?");
                        self.lcd_status.replace(LCDStatus::Begin13);
                    },

                    LCDStatus::Begin13 => {
                        self.lcd_status.replace(LCDStatus::Begin14);
                    },

                    LCDStatus::Begin14 => {
                        self.lcd_status.replace(LCDStatus::Begin15);
                    },

                    LCDStatus::Begin15 => {
                        self.lcd_status.replace(LCDStatus::Begin16);
                    },

                    LCDStatus::Begin16 => {
                        self.lcd_status.replace(LCDStatus::Begin17);
                    },

                    LCDStatus::Begin17 => {
                        self.lcd_status.replace(LCDStatus::Begin18);
                    },

                    LCDStatus::Begin18 => {
                        self.lcd_status.replace(LCDStatus::Idle);
                        self.command_offset.replace(self.command_offset.get() + 3);
                    },

                    LCDStatus::Other => {
                        self.lcd_status.replace(LCDStatus::Idle);
                    },

                    LCDStatus::Printing0 => {
                        self.lcd_status.replace(LCDStatus::Printing1);
                    },

                    LCDStatus::Printing1 => {
                        self.lcd_status.replace(LCDStatus::Idle);
                        self.command_offset.replace(self.command_offset.get() + 1);
                    }

                    _ => {}
                }
                self.en_pin.clear();
                self.set_delay(500);
                // self.alarm.set_alarm(
                //     self.alarm.now().wrapping_add(<A::Frequency>::frequency()/500);
                // )

            },

            _ => {}

        }
        // self.lcd_status.replace(LCDStatus::Idle);
        // self.handle_commands();
    }
}

