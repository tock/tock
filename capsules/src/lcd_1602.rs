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
use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::{
    capabilities, create_capability, debug, AppId, AppSlice, Driver, Grant, ReturnCode, Shared,
};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Lcd1602 as usize;

static LOW: u8 = 0;
static HIGH: u8 = 1;

// commands
static LCD_CLEARDISPLAY: u8 = 0x01;
static LCD_RETURNHOME: u8 = 0x02;
static LCD_ENTRYMODESET: u8 = 0x04;
static LCD_DISPLAYCONTROL: u8 = 0x08;
static LCD_CURSORSHIFT: u8 = 0x10;
static LCD_FUNCTIONSET: u8 = 0x20;
static LCD_SETCGRAMADDR: u8 = 0x40;
static LCD_SETDDRAMADDR: u8 = 0x80;

// flags for display entry mode
static LCD_ENTRYRIGHT: u8 = 0x00;
static LCD_ENTRYLEFT: u8 = 0x02;
static LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
static LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// flags for display on/off control
static LCD_DISPLAYON: u8 = 0x04;
static LCD_DISPLAYOFF: u8 =  0x00;
static LCD_CURSORON: u8 = 0x02;
static LCD_CURSOROFF: u8 =  0x00;
static LCD_BLINKON: u8 = 0x01;
static LCD_BLINKOFF: u8 = 0x00;

// flags for display/cursor shift
static LCD_DISPLAYMOVE: u8 = 0x08;
static LCD_CURSORMOVE: u8 = 0x00;
static LCD_MOVERIGHT: u8 = 0x04;
static LCD_MOVELEFT: u8 = 0x00;

// flags for function set
static LCD_8BITMODE: u8 = 0x10;
static LCD_4BITMODE: u8 = 0x00;
static LCD_2LINE: u8 = 0x08;
static LCD_1LINE: u8 = 0x00;
// static LCD_5X10DOTS: u8 = 0x04;
static LCD_5X8DOTS: u8 = 0x00;

pub static mut BUFFER: [u8; 200] = [0; 200];
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
    Begin6,
    Begin7,
    Begin8,
    Begin9,
    Begin10,
    Begin11,
    Begin12,
    Begin13,
    Home,
    Printing,
    PulseLow,
    PulseHigh,
    Command,
    Clear,
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
    row_offsets: TakeCell<'static, [u8]>,
    command_buffer: TakeCell<'static, [u8]>,
    command_len: Cell<u8>,
    command_offset: Cell<u8>,
    alarm: &'a A,
    apps: Grant<App>,
    lcd_status: Cell<LCDStatus>,
    lcd_prev_status: Cell<LCDStatus>,
    lcd_after_pulse_status: Cell<LCDStatus>,
    lcd_after_command_status: Cell<LCDStatus>,
    lcd_after_delay_status: Cell<LCDStatus>,
    command_to_finish: Cell<u8>,
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
            display_function: Cell::new(LCD_4BITMODE | LCD_1LINE | LCD_5X8DOTS),
            display_control: Cell::new(0),
            display_mode: Cell::new(0),
            num_lines: Cell::new(0),
            row_offsets: TakeCell::new(row_offsets),
            command_buffer: TakeCell::new(command_buffer),
            command_len: Cell::new(0),
            command_offset: Cell::new(0),
            alarm: alarm,
            apps: grant,
            lcd_status: Cell::new(LCDStatus::Idle),
            lcd_prev_status: Cell::new(LCDStatus::Idle),
            lcd_after_pulse_status: Cell::new(LCDStatus::Idle),
            lcd_after_command_status: Cell::new(LCDStatus::Idle),
            lcd_after_delay_status: Cell::new(LCDStatus::Idle),
            command_to_finish: Cell::new(0),
        }
    }

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
        self.bring_to_0();
        if self.lcd_status.get() == LCDStatus::Idle {
            let mut offset = self.command_offset.get() as usize;
            // debug! ("       Citim de pe pozitia {}", offset);
            if offset < self.command_len.get() as usize {
                self.command_buffer.map(|buffer| {
                    
                    let current = buffer[offset];
                    match current {
                        // begin
                        130 => {
                            let cols = buffer[offset + 1];
                            let lines = buffer[offset + 2];

                            if lines > 1 {
                                self.display_function
                                    .replace(self.display_function.get() | LCD_2LINE);
                            }

                            self.num_lines.replace(lines);
                            self.set_rows(0x00, 0x40, 0x00 + cols, 0x40 + cols);
                            
                            self.command_offset.replace((offset + 3) as u8);
                            self.set_delay(10, LCDStatus::Begin0);
                   
                        }

                        // set cursor
                        131 => {
                            let mut col_number: u8 = buffer[offset + 1];
                            let mut line_number: u8 = buffer[offset + 2];
                            if line_number >= 4 {
                                line_number = 3;
                            }

                            if line_number >= self.num_lines.get() {
                                line_number = self.num_lines.get() - 1;
                            }
                            
                            let mut value: u8 = 0;
                            self.row_offsets.map(|buffer| {
                                value = buffer[line_number as usize];
                            });
                            self.command_offset.replace((offset + 3) as u8);
                            self.command_to_finish
                                .replace(LCD_SETDDRAMADDR | (col_number + value));
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // clear
                        133 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.lcd_clear(LCDStatus::Idle);
                        },

                        // home
                        132 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.lcd_home(LCDStatus::Idle);
                        },

                        // left right
                        134 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode.set(self.display_mode.get() | LCD_ENTRYLEFT);
                            self.command_to_finish.replace(
                                LCD_ENTRYMODESET | 
                                self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // right left
                        144 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode.set(self.display_mode.get() & !LCD_ENTRYLEFT);
                            self.command_to_finish.replace(
                                LCD_ENTRYMODESET | 
                                self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // autoscroll
                        135 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode.set(self.display_mode.get() | LCD_ENTRYSHIFTINCREMENT);
                            self.command_to_finish.replace(
                                LCD_ENTRYMODESET | 
                                self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // no autoscroll
                        145 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode.set(self.display_mode.get() & !LCD_ENTRYSHIFTINCREMENT);
                            self.command_to_finish.replace(
                                LCD_ENTRYMODESET | 
                                self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },


                        // display
                        137 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control.set(self.display_control.get() | LCD_DISPLAYON);
                            self.command_to_finish.replace(
                                LCD_DISPLAYCONTROL | 
                                self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // no display
                        147 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control.set(self.display_control.get() & !LCD_DISPLAYON);
                            self.command_to_finish.replace(
                                LCD_DISPLAYCONTROL | 
                                self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // blink
                        138 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control.set(self.display_control.get() | LCD_BLINKON);
                            self.command_to_finish.replace(
                                LCD_DISPLAYCONTROL | 
                                self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // no blink
                        148 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control.set(self.display_control.get() & !LCD_BLINKON);
                            self.command_to_finish.replace(
                                LCD_DISPLAYCONTROL | 
                                self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // cursor
                        136 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control.set(self.display_control.get() | LCD_CURSORON);
                            self.command_to_finish.replace(
                                LCD_DISPLAYCONTROL | 
                                self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // no cursor
                        146 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control.set(self.display_control.get() & !LCD_CURSORON);
                            self.command_to_finish.replace(
                                LCD_DISPLAYCONTROL | 
                                self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // scroll left 
                        139 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.command_to_finish.replace(
                                LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT);
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },

                        // scroll right
                        149 => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.command_to_finish.replace(
                                LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT);
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        },


                        _ => {
                            // command cu HIGH
                            // debug! ("afisam {} ", current);
                            self.rs_pin.set();
                            self.command_to_finish.replace(current);
                            self.command_offset.replace((offset + 1) as u8);
                            self.write_4_bits(self.command_to_finish.get() >> 4, LCDStatus::Printing);
                        }
                    }
                });
            }
        }
        ReturnCode::SUCCESS
    }

    fn bring_to_0(&self) {
        let index = self.command_offset.get() as usize;
        let len = self.command_len.get() as usize;

        if index < len {
            self.command_buffer.map(|buffer| {
                for i in index..len {
                    buffer[i-index] = buffer[i];
                }
                self.command_len.replace((len - index) as u8);
                self.command_offset.replace(0);
            });

        }
    }

    fn pulse(&self, after_pulse_status: LCDStatus) {
        self.lcd_after_pulse_status.set(after_pulse_status);
        self.en_pin.clear();
        self.set_delay(500, LCDStatus::PulseLow);
    }

    fn write_4_bits(&self, value: u8, next_status: LCDStatus) {
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

        self.pulse(next_status);
    }

    fn lcd_display(&self, next_state: LCDStatus) {
        // self.display_control.set(self.display_control.get() | LCD_DISPLAYON);
        self.command_to_finish.set(LCD_DISPLAYCONTROL | self.display_control.get());
        self.lcd_command(LCD_DISPLAYCONTROL | self.display_control.get(), next_state);
    }

    fn lcd_command(&self, value: u8, next_state: LCDStatus) {
        self.lcd_after_command_status.set(next_state);
        self.command_to_finish.set(value);
        self.rs_pin.clear();    
        self.write_4_bits(value >> 4, LCDStatus::Command);
    }

    fn lcd_clear(&self, next_state: LCDStatus) {
        self.lcd_after_delay_status.set(next_state);
        self.lcd_command(LCD_CLEARDISPLAY, LCDStatus::Clear);
    }

    fn lcd_home(&self, next_state: LCDStatus) {
        self.lcd_after_delay_status.set(next_state);
        // self.lcd_command(LCD_RETURNHOME, LCDStatus::Home);
    }

    fn set_delay(&self, timer: u32, next_status: LCDStatus) {
        self.lcd_status.set(next_status);
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency()/timer),
        )
    }
}

impl<'a, A: Alarm<'a>> Driver for LCD<'a, A> {
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
                    self.handle_commands();
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
    fn command(&self, command_num: usize, data: usize, data_3: usize, _: AppId) -> ReturnCode {
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
            },

            // set cursor
            1 => {
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
            },

            // Home
            2 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = 132;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // clear
            3 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = 133;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // Left/Right to Right/Left
            4 => {
                let mut index = self.command_len.get() as usize;
                match data { 
                    // left right - 134
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 134;
                            index += 1;
                        });
                    }

                    // right left - 144
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 144;
                            index += 1;
                        });
                    }

                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // autoscroll
            5 => {
                let mut index = self.command_len.get() as usize;
                match data { 
                    // autoscroll - 135
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 135;
                            index += 1;
                        });
                    }

                    // no autoscroll - 145
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 145;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // Cursor/No
            6 => {
                let mut index = self.command_len.get() as usize;
                match data { 
                    // cursor - 136
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 136;
                            index += 1;
                        });
                    }

                    // no cursor - 146
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 146;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // Display/No
            7 => {
                let mut index = self.command_len.get() as usize;
                match data { 
                    // Display - 137
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 137;
                            index += 1;
                        });
                    }

                    // no display - 147
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 147;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // Blink/No
            8 => {
                let mut index = self.command_len.get() as usize;
                match data { 
                    // Blink - 138
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 138;
                            index += 1;
                        });
                    }

                    // no blink - 148
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 148;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            },

            // Scroll Left/Right
            9 => {
                let mut index = self.command_len.get() as usize;
                match data { 
                    // scroll left - 139
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 139;
                            index += 1;
                        });
                    }

                    // scroll right - 149
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = 149;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
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
            }

            LCDStatus::Begin0 => {
                // debug!("in begin0");
                self.rs_pin.clear();
                self.en_pin.clear();

                if (self.display_function.get() & LCD_8BITMODE) == 0 {
                    self.write_4_bits(0x03, LCDStatus::Begin0_1);
                } else {
                    self.rs_pin.clear();
                    self.lcd_command(
                        (LCD_FUNCTIONSET | self.display_function.get()) >> 4,
                        LCDStatus::Begin4
                    );
                }
            }

            LCDStatus::Begin0_1 => {
                // debug!("in begin0_1");
                self.set_delay(200, LCDStatus::Begin1);
            }

            LCDStatus::Begin1 => {
                // debug!("in begin1");
                self.write_4_bits(0x03, LCDStatus::Begin1_2);
            }

            LCDStatus::Begin1_2 => {
                // debug!("in begin1_2");
                self.set_delay(200, LCDStatus::Begin2);
            }

            LCDStatus::Begin2 => {
                // debug!("in begin2");
                self.write_4_bits(0x03, LCDStatus::Begin2_3);
            }

            LCDStatus::Begin2_3 => {
                // debug! ("in begin2_3");
                self.set_delay(500, LCDStatus::Begin3);
            }

            LCDStatus::Begin3 => {
                // debug! ("in begin3");
                self.write_4_bits(0x02, LCDStatus::Begin9);
            },

            LCDStatus::Begin4 => {
                // debug! ("in begin4");
                self.command_to_finish.set(LCD_FUNCTIONSET | self.display_function.get());
                self.lcd_command(LCD_FUNCTIONSET | self.display_function.get(), LCDStatus::Begin5);
            },

            LCDStatus::Begin5 => {
                // debug!("in begin5");
                self.set_delay(200, LCDStatus::Begin6)
            }

            LCDStatus::Begin6 => {
                // debug!("in begin6");
                self.lcd_command(LCD_FUNCTIONSET | self.display_function.get(), LCDStatus::Begin7);
            }

            LCDStatus::Begin7 => {
                // debug!("in begin7");
                self.set_delay(500, LCDStatus::Begin8);
            }

            LCDStatus::Begin8 => {
                // debug!("in begin8");
                self.lcd_command(LCD_FUNCTIONSET | self.display_function.get(), LCDStatus::Begin9);
            }

            LCDStatus::Begin9 => {
                // debug!("in begin9");
                self.command_to_finish.set(LCD_FUNCTIONSET | self.display_function.get());
                self.lcd_command(LCD_FUNCTIONSET | self.display_function.get(), LCDStatus::Begin10);
            }

            LCDStatus::Begin10 => {
                // debug!("in begin10");
                self.display_control.set(LCD_DISPLAYON | LCD_CURSORON | LCD_BLINKOFF);
                self.lcd_display(LCDStatus::Begin11);
            }

            LCDStatus::Begin11 => {
                // debug!("in begin11");
                self.lcd_clear(LCDStatus::Begin12);
            }

            LCDStatus::Begin12 => {
                // debug!("in begin12");
                self.display_mode.set(LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT);
                self.command_to_finish.set(LCD_ENTRYMODESET | self.display_mode.get());
                self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
            }

            LCDStatus::Clear => {
                self.set_delay(500, self.lcd_after_delay_status.get());
            }, 

            LCDStatus::Home => {
                self.set_delay(500, self.lcd_after_delay_status.get());
            },

            LCDStatus::Printing => {
                // debug!("in other1");
                self.write_4_bits(self.command_to_finish.get(), LCDStatus::Idle);
            },

            LCDStatus::PulseLow => {
                // debug!("in pulseLow");
                self.en_pin.set();
                self.set_delay(500, LCDStatus::PulseHigh);
                
            },

            LCDStatus::Command => {
                // debug!("ajungem in command, deci e ok");
                self.write_4_bits(self.command_to_finish.get(), self.lcd_after_command_status.get());
            },

            LCDStatus::PulseHigh => {
                // debug!("in pulseHigh");
                
                self.en_pin.clear();
                self.set_delay(500, self.lcd_after_pulse_status.get());
            }

            _ => {}
        }
    }
}
