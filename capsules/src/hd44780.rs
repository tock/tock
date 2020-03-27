//! Provides userspace access to LCD connected on the board, but defined in the
//! kernel.
//!
//! The LCD must be connected as shown here, because the pins of the LCD are
//! already defined in the kernel, and modifying them means re-compiling the
//! kernel with the modifications.
//!
//! This capsule takes an alarm, an array of pins, two buffers initialized
//! to 0, and the grant through which buffers from the userspace will be sent.
//!
//! According to the HD44780 datasheet, there must be a delay between certain
//! operations on the device. Since there cannot be a delay while running on
//! kernel mode, the alarm is the best way to implement those delays. To
//! remember the state before and after each delay, the program will be a big
//! state-machine that goes through the possible states defined in the
//! LCDStatus enum. Also, since there is no way to stop the userspace while
//! waiting for the alarms to finish, the solution was to save in a buffer all
//! the commands that the userspace sent. This way, after each delay, the
//! buffer will be checked in order to see if there are any commands left to be
//! executed.
//!
//! Usage
//! -----
//!
//! Usage
//! -----
//! ```rust
//! let lcd = components::hd44780::HD44780Component::new(board_kernel, mux_alarm).finalize(
//!     components::hd44780_component_helper!(
//!         stm32f4xx::tim2::Tim2,
//!         // rs pin
//!         stm32f4xx::gpio::PinId::PF13.get_pin().as_ref().unwrap(),
//!         // en pin
//!         stm32f4xx::gpio::PinId::PE11.get_pin().as_ref().unwrap(),
//!         // data 4 pin
//!         stm32f4xx::gpio::PinId::PF14.get_pin().as_ref().unwrap(),
//!         // data 5 pin
//!         stm32f4xx::gpio::PinId::PE13.get_pin().as_ref().unwrap(),
//!         // data 6 pin
//!         stm32f4xx::gpio::PinId::PF15.get_pin().as_ref().unwrap(),
//!         // data 7 pin
//!         stm32f4xx::gpio::PinId::PG14.get_pin().as_ref().unwrap()
//!     )
//! );
//! ```
//!
//! Syscall Interface
//! -----------------
//!
//! The userspace can make two types of commands:
//! - write a buffer to the LCD, using the allow syscall
//! - any other command for display/cursor control, using the command syscall
//!
//! ### Allow
//!
//! Allow syscall is used for sending from the userspace to kernelspace a
//! buffer to be displayed on the LCD.
//!
//! #### `allow_num`
//!
//! - `0`: Send the buffer to the kernel;
//!   - `slice`: the buffer.
//!   - Return: The number of bytes that were saved to the command buffer and
//! to be written to the screen.
//!
//! ### Command
//!
//! #### `command_num`
//!
//! - `0`: Initialize the LCD using two arguments given:
//!   - `data_1`: Number of columns on the LCD.
//!   - `data_2`: Number of lines on the LCD.
//!   - Return: `SUCCESS` if the command was saved successfully in the buffer
//! for the commands, `EBUSY` otherwise (the buffer was full).
//! - `1`: Set cursor to a given position:
//!   - `data_1`: The column on which to set the cursor.
//!   - `data_2`: The line on which to set the cursor.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `2`: Home command, clears the display and sets the cursor to (0,0).
//!   - `data_1`: Unused.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `3`: Clear command, clears the display and sets the cursor to (0,0).
//!   - `data_1`: Unused.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `4`: Left_to_right command, or Right_to_left command.
//!   - `data_1`: - `0`: Left_to_right: flow the text from left to right.
//!               - `1`: Right_to_left: flow the text from right to left.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `5`: Autoscroll command or No_autoscroll command
//!   - `data_1`: - `0`: Autoscroll: 'right justify' the text from the cursor.
//!               - `1`: No_autoscroll: 'left justify' the text from the cursor.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `6`: Cursor command or No_cursor command
//!   - `data_1`: - `0`: Cursor: Turn on the underline cursor.
//!               - `1`: No_cursor: Turn off the underline cursor.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `7`: Display command or No_display command
//!   - `data_1`: - `0`: Display: Turn on the display very quickly.
//!               - `1`: No_display: Turn off the display very quickly.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `8`: Blink command or No_blink command
//!   - `data_1`: - `0`: Blink: Turn on the blinking cursor display.
//!               - `1`: No_blink: Turn off the blinking cursor display.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//! - `9`: Scroll_display_left command or Scroll_display_right command.
//!   - `data_1`: - `0`: Scroll_display_left: Scroll the display to the left
//!                         without changing the RAM.
//!               - `1`: Scroll_display_right: Scroll the display to the right
//!                         without changing the RAM.
//!   - `data_2`: Unused.
//!   - Return: `SUCCESS` if the command was saved successfully, `EBUSY`
//! otherwise.
//!
//! Author: Teona Severin <teona.severin9@gmail.com>

use crate::driver;
use core::cell::Cell;
use kernel::common::cells::TakeCell;
use kernel::hil::gpio;
use kernel::hil::time::{self, Alarm, Frequency};
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.

pub const DRIVER_NUM: usize = driver::NUM::Hd44780 as usize;

// commands
static LCD_CLEARDISPLAY: u8 = 0x01;
// static LCD_RETURNHOME: u8 = 0x02;
static LCD_ENTRYMODESET: u8 = 0x04;
static LCD_DISPLAYCONTROL: u8 = 0x08;
static LCD_CURSORSHIFT: u8 = 0x10;
static LCD_FUNCTIONSET: u8 = 0x20;
// static LCD_SETCGRAMADDR: u8 = 0x40;
static LCD_SETDDRAMADDR: u8 = 0x80;

// flags for display entry mode
// static LCD_ENTRYRIGHT: u8 = 0x00;
static LCD_ENTRYLEFT: u8 = 0x02;
static LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
static LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// flags for display on/off control
static LCD_DISPLAYON: u8 = 0x04;
// static LCD_DISPLAYOFF: u8 = 0x00;
static LCD_CURSORON: u8 = 0x02;
// static LCD_CURSOROFF: u8 = 0x00;
static LCD_BLINKON: u8 = 0x01;
static LCD_BLINKOFF: u8 = 0x00;

// flags for display/cursor shift
static LCD_DISPLAYMOVE: u8 = 0x08;
// static LCD_CURSORMOVE: u8 = 0x00;
static LCD_MOVERIGHT: u8 = 0x04;
static LCD_MOVELEFT: u8 = 0x00;

// flags for function set
static LCD_8BITMODE: u8 = 0x10;
static LCD_4BITMODE: u8 = 0x00;
static LCD_2LINE: u8 = 0x08;
static LCD_1LINE: u8 = 0x00;
static LCD_5X8DOTS: u8 = 0x00;

// command constants
const BEGIN: u8 = 130;
const SET_CURSOR: u8 = 131;
const HOME: u8 = 132;
const CLEAR: u8 = 133;
const LEFT_TO_RIGHT: u8 = 134;
const RIGHT_TO_LEFT: u8 = 144;
const AUTOSCROLL: u8 = 135;
const NO_AUTOSCROLL: u8 = 145;
const CURSOR: u8 = 136;
const NO_CURSOR: u8 = 146;
const DISPLAY: u8 = 137;
const NO_DISPLAY: u8 = 147;
const BLINK: u8 = 138;
const NO_BLINK: u8 = 148;
const SCROLL_DISPLAY_LEFT: u8 = 139;
const SCROLL_DISPLAY_RIGHT: u8 = 149;

const BUFFER_FULL: i16 = -1;
const BUFSIZE: usize = 200;
const ALLOW_BAD_VALUE: usize = BUFSIZE;

pub static mut BUFFER: [u8; BUFSIZE] = [0; BUFSIZE];
pub static mut ROW_OFFSETS: [u8; 4] = [0; 4];

#[derive(Default)]
pub struct App {
    text_buffer: Option<AppSlice<Shared, u8>>,
}

// The states the program can be in.
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
    Home,
    Printing,
    PulseLow,
    PulseHigh,
    Command,
    Clear,
}

pub struct HD44780<'a, A: Alarm<'a>> {
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
    lcd_after_pulse_status: Cell<LCDStatus>,
    lcd_after_command_status: Cell<LCDStatus>,
    lcd_after_delay_status: Cell<LCDStatus>,
    command_to_finish: Cell<u8>,
}

impl<'a, A: Alarm<'a>> HD44780<'a, A> {
    pub fn new(
        rs_pin: &'a dyn gpio::Pin,
        en_pin: &'a dyn gpio::Pin,
        data_4_pin: &'a dyn gpio::Pin,
        data_5_pin: &'a dyn gpio::Pin,
        data_6_pin: &'a dyn gpio::Pin,
        data_7_pin: &'a dyn gpio::Pin,
        command_buffer: &'static mut [u8],
        row_offsets: &'static mut [u8],
        alarm: &'a A,
        grant: Grant<App>,
    ) -> HD44780<'a, A> {
        rs_pin.make_output();
        en_pin.make_output();
        data_4_pin.make_output();
        data_5_pin.make_output();
        data_6_pin.make_output();
        data_7_pin.make_output();
        HD44780 {
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
            lcd_after_pulse_status: Cell::new(LCDStatus::Idle),
            lcd_after_command_status: Cell::new(LCDStatus::Idle),
            lcd_after_delay_status: Cell::new(LCDStatus::Idle),
            command_to_finish: Cell::new(0),
        }
    }

    /* set_rows sets initializing parameters for the communication.
     *
     * Example:
     *  self.set_rows(0x00, 0x40, 0x00+col, 0x40+col);
     */
    fn set_rows(&self, row0: u8, row1: u8, row2: u8, row3: u8) -> ReturnCode {
        self.row_offsets.map(|buffer| {
            buffer[0] = row0;
            buffer[1] = row1;
            buffer[2] = row2;
            buffer[3] = row3;
        });
        ReturnCode::SUCCESS
    }

    /* handle_commands calls the bring_to_0() function and then starts
     * executing the first command saved in the buffer.
     *
     * Example:
     *  self.handle_commands();
     */
    fn handle_commands(&self) -> ReturnCode {
        self.bring_to_0();
        if self.lcd_status.get() == LCDStatus::Idle {
            let offset = self.command_offset.get() as usize;
            if offset < self.command_len.get() as usize {
                self.command_buffer.map(|buffer| {
                    let current = buffer[offset];
                    match current {
                        BEGIN => {
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

                        SET_CURSOR => {
                            let col_number: u8 = buffer[offset + 1];
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
                        }

                        CLEAR => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.lcd_clear(LCDStatus::Idle);
                        }

                        HOME => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.lcd_home(LCDStatus::Idle);
                        }

                        LEFT_TO_RIGHT => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode
                                .set(self.display_mode.get() | LCD_ENTRYLEFT);
                            self.command_to_finish
                                .replace(LCD_ENTRYMODESET | self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        RIGHT_TO_LEFT => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode
                                .set(self.display_mode.get() & !LCD_ENTRYLEFT);
                            self.command_to_finish
                                .replace(LCD_ENTRYMODESET | self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        AUTOSCROLL => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode
                                .set(self.display_mode.get() | LCD_ENTRYSHIFTINCREMENT);
                            self.command_to_finish
                                .replace(LCD_ENTRYMODESET | self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        NO_AUTOSCROLL => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_mode
                                .set(self.display_mode.get() & !LCD_ENTRYSHIFTINCREMENT);
                            self.command_to_finish
                                .replace(LCD_ENTRYMODESET | self.display_mode.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        CURSOR => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control
                                .set(self.display_control.get() | LCD_CURSORON);
                            self.command_to_finish
                                .replace(LCD_DISPLAYCONTROL | self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        NO_CURSOR => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control
                                .set(self.display_control.get() & !LCD_CURSORON);
                            self.command_to_finish
                                .replace(LCD_DISPLAYCONTROL | self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        DISPLAY => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control
                                .set(self.display_control.get() | LCD_DISPLAYON);
                            self.command_to_finish
                                .replace(LCD_DISPLAYCONTROL | self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        NO_DISPLAY => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control
                                .set(self.display_control.get() & !LCD_DISPLAYON);
                            self.command_to_finish
                                .replace(LCD_DISPLAYCONTROL | self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        BLINK => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control
                                .set(self.display_control.get() | LCD_BLINKON);
                            self.command_to_finish
                                .replace(LCD_DISPLAYCONTROL | self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        NO_BLINK => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.display_control
                                .set(self.display_control.get() & !LCD_BLINKON);
                            self.command_to_finish
                                .replace(LCD_DISPLAYCONTROL | self.display_control.get());
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        SCROLL_DISPLAY_LEFT => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.command_to_finish
                                .replace(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT);
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        SCROLL_DISPLAY_RIGHT => {
                            self.command_offset.replace((offset + 1) as u8);
                            self.command_to_finish
                                .replace(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT);
                            self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
                        }

                        _ => {
                            // if offset < 111 {
                            //     debug!("{} {}/{}", current - 48, offset, self.command_len.get());
                            // }
                            self.rs_pin.set();
                            self.command_to_finish.replace(current);
                            self.command_offset.replace((offset + 1) as u8);
                            self.write_4_bits(
                                self.command_to_finish.get() >> 4,
                                LCDStatus::Printing,
                            );
                        }
                    }
                });
            }
        }
        ReturnCode::SUCCESS
    }

    /* bring_to_0 checks if there are any commands already executed and not
     * deleted from the buffer and deletes them; after that, the offset will
     * be always 0 and the length will be the number of commands not executed.
     *
     * Example:
     *  self.bring_to_0();
     */
    fn bring_to_0(&self) {
        let index = self.command_offset.get() as usize;
        let len = self.command_len.get() as usize;

        if index < len {
            self.command_buffer.map(|buffer| {
                for i in index..len {
                    buffer[i - index] = buffer[i];
                }
                self.command_len.replace((len - index) as u8);
                self.command_offset.replace(0);
            });
        }
    }

    /* pulse function starts executing the toggle needed by the device after
     * each write operation, according to the HD44780 datasheet, figure 26,
     * toggle that will be continued in the fired() function.
     *
     * As argument, there is :
     *  - the status of the program after the process of pulse is done
     *
     * Example:
     *  self.pulse(LCDStatus::Idle);
     */
    fn pulse(&self, after_pulse_status: LCDStatus) {
        self.lcd_after_pulse_status.set(after_pulse_status);
        self.en_pin.clear();
        self.set_delay(500, LCDStatus::PulseLow);
    }

    /* write_4_bits will either set or clear each data_pin according to the
     * value to be written on the device.
     *
     * As arguments, there are:
     *  - the value to be written
     *  - the next status of the program after writing the value
     *
     * Example:
     *  self.write_4_bits(27, LCDStatus::Idle);
     */
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

    /* lcd_display will call lcd_command with certain arguments for the display
     * initialization.
     *
     * As argument, there is:
     *  - the status of the program after setting the display
     *
     * Example:
     *  self.lcd_display(LCDStatus::Idle);
     */
    fn lcd_display(&self, next_state: LCDStatus) {
        self.command_to_finish
            .set(LCD_DISPLAYCONTROL | self.display_control.get());
        self.lcd_command(LCD_DISPLAYCONTROL | self.display_control.get(), next_state);
    }

    /* lcd_command is the main funcion that communicates with the device, and
     * sends certain values received as arguments to the device (through
     * write_4_bits function). Due to the delays, the funcion is continued in
     * the fired() function.
     *
     * As arguments, there are:
     *  - the value to be sent to the device
     *  - the next status of the program after sending the value
     *
     * Example:
     *  self.lcd_command(LCD_CLEARDISPLAY, LCDStatus::Clear);
     */
    fn lcd_command(&self, value: u8, next_state: LCDStatus) {
        self.lcd_after_command_status.set(next_state);
        self.command_to_finish.set(value);
        self.rs_pin.clear();
        self.write_4_bits(value >> 4, LCDStatus::Command);
    }

    /* lcd_clear clears the lcd and brings the cursor at position (0,0).
     *
     * As argument, there is:
     *  - the status of the program after clearing the display
     *
     * Example:
     *  self.clear(LCDStatus::Idle);
     */
    fn lcd_clear(&self, next_state: LCDStatus) {
        self.lcd_after_delay_status.set(next_state);
        self.lcd_command(LCD_CLEARDISPLAY, LCDStatus::Clear);
    }

    /* lcd_home clears the lcd and brings the cursor at position (0,0),
     * as lcd_clear.
     *
     * As argument, there is:
     *  - the status of the program after returning to home
     *
     * Example:
     *  self.home(LCDStatus::Idle);
     */
    fn lcd_home(&self, next_state: LCDStatus) {
        self.lcd_after_delay_status.set(next_state);
        self.lcd_command(LCD_CLEARDISPLAY, LCDStatus::Home);
    }

    /* set_delay sets an alarm and saved the next state after that.
     *
     * As argument, there are:
     *  - the duration of the alarm:
     *      - 10 means 100 ms
     *      - 100 means 10 ms
     *      - 500 means 2 ms
     *  - the status of the program after the alarm fires
     *
     * Example:
     *  self.set_delay(10, LCDStatus::Idle);
     */
    fn set_delay(&self, timer: u32, next_status: LCDStatus) {
        self.lcd_status.set(next_status);
        self.alarm.set_alarm(
            self.alarm
                .now()
                .wrapping_add(<A::Frequency>::frequency() / timer),
        )
    }

    /* check_buffer checks if there is enough space available on the buffer
     * for the last command sent from userspace.
     *
     * Example: self.check_buffer();
     */
    fn check_buffer(&self, to_check: usize) -> i16 {
        let current_len = self.command_len.get() as usize;
        if current_len > 197 {
            // debug!("current_len from check_buffer {}", current_len);
        }
        if current_len >= BUFSIZE {
            return BUFFER_FULL;
        }
        if current_len + to_check > BUFSIZE {
            return (BUFSIZE - current_len) as i16;
        } else {
            return to_check as i16;
        }
    }
}

impl<'a, A: Alarm<'a>> Driver for HD44780<'a, A> {
    /* Send a buffer to be displayed on the LCD device. The buffer is fully
     * saved in the command buffer if there are enough empty slots left, or
     * partially saved until the buffer gets full.
     *
     *  * As arguments, there are:
     *  - the driver number
     *  - the allow command number
     *  - pointer to the buffer to be sent
     *
     * Return: SuccessWithValue - number of bytes written in the buffer
     *         ENOSUPPORT - the allow_num is not 1, so the syscall was mistaken
     *
     * Example:
     *
     * // save a Begin command with 16 and 1 as arguments
     * char buffer[128];
     * int ret = allow(DRIVER_LCD_NUM, 1, (void *) buffer, 0);
     */
    fn allow(
        &self,
        appid: AppId,
        allow_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        let mut ret = 0;
        let mut partial: bool = false;
        match allow_num {
            0 => self
                .apps
                .enter(appid, |app, _| {
                    if let Some(ref s) = slice {
                        /* check to see how many empty slots are and how much
                         * can be written to the buffer
                         */
                        ret = self.check_buffer(s.len());
                        match ret {
                            BUFFER_FULL => {
                                self.handle_commands();
                                return ReturnCode::SuccessWithValue {
                                    value: ALLOW_BAD_VALUE,
                                };
                            }
                            _ => {
                                if ret != s.len() as i16 {
                                    partial = true;
                                }
                            }
                        }
                        /* go through the buffer received and save in the command
                         * buffer the values to be displayed on the device (those
                         * values are lower than 128 and will be saved exactly as
                         * they are)
                         */
                        let mut leng = self.command_len.get() as usize;
                        self.command_buffer.map(|buffer| {
                            for byte in s.iter() {
                                if partial == true {
                                    if leng >= BUFSIZE {
                                        break;
                                    }
                                }
                                buffer[leng] = *byte;
                                leng += 1;
                            }
                        });
                        self.command_len.replace(leng as u8);
                    };
                    app.text_buffer = slice;
                    self.handle_commands();
                    ReturnCode::SuccessWithValue {
                        value: ret as usize,
                    }
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /* Save any setup command in the command buffer. The commands are detailed
     * at the beginning of the capsule and saved in the buffer according to the
     * arguments. All the commands sent through the `command` syscall will be
     * saved in the buffer using values bigger than 128.
     *
     * As arguments, there are:
     *  - the driver number
     *  - the command number, that defines what command was sent
     *  - two optional arguments that are used when needed
     *
     * Return: SUCCES - the command was saved successfully
     *         EBUSY - there are no empty slots in the buffer for the command
     *
     * Example:
     * #define DRIVER_LCD_NUM 0x80005
     *
     * // save a Begin command with 16 and 1 as arguments
     * int ret = command(DRIVER_LCD_NUM, 0, 16, 1);
     */
    fn command(&self, command_num: usize, data_1: usize, data_2: usize, _: AppId) -> ReturnCode {
        /* check to see how many slots we need in the command buffer for the
         * request
         */
        let mut to_check: usize = 1;
        if command_num == 0 || command_num == 1 {
            to_check = 3;
        }
        let ret = self.check_buffer(to_check);
        /* return EBUSY if there is no space in the buffer */
        if ret == BUFFER_FULL || ret != to_check as i16 {
            self.handle_commands();
            return ReturnCode::EBUSY;
        }
        match command_num {
            /* Save a Begin command and the two arguments */
            0 => {
                let mut index = self.command_len.get() as usize;
                self.command_buffer.map(|buffer| {
                    buffer[index] = BEGIN;
                    index += 1;
                    buffer[index] = data_1 as u8;
                    index += 1;
                    buffer[index] = data_2 as u8;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();

                ReturnCode::SUCCESS
            }

            /* Save a Set_cursor command and the two arguments */
            1 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = SET_CURSOR;
                    index += 1;
                    buffer[index] = data_1 as u8;
                    index += 1;
                    buffer[index] = data_2 as u8;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Home command */
            2 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = HOME;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Clear command */
            3 => {
                let mut index = self.command_len.get() as usize;

                self.command_buffer.map(|buffer| {
                    buffer[index] = CLEAR;
                    index += 1;
                });
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Left_to_right or Right_to_left command */
            4 => {
                let mut index = self.command_len.get() as usize;
                match data_1 {
                    /* Left_to_right */
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = LEFT_TO_RIGHT;
                            index += 1;
                        });
                    }

                    /* Right_to_left */
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = RIGHT_TO_LEFT;
                            index += 1;
                        });
                    }

                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save an Autoscroll or a No_autoscroll command */
            5 => {
                let mut index = self.command_len.get() as usize;
                match data_1 {
                    /* Autoscroll */
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = AUTOSCROLL;
                            index += 1;
                        });
                    }

                    /* No_autoscroll */
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = NO_AUTOSCROLL;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Cursor or a No_cursor command */
            6 => {
                let mut index = self.command_len.get() as usize;
                match data_1 {
                    /* Cursor */
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = CURSOR;
                            index += 1;
                        });
                    }

                    /* No_cursor */
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = NO_CURSOR;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Display or a No_display command */
            7 => {
                let mut index = self.command_len.get() as usize;
                match data_1 {
                    /* Display */
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = DISPLAY;
                            index += 1;
                        });
                    }

                    /* No_display */
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = NO_DISPLAY;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Blink or a No_blink command */
            8 => {
                let mut index = self.command_len.get() as usize;
                match data_1 {
                    /* Blink */
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = BLINK;
                            index += 1;
                        });
                    }

                    /* No_blink */
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = NO_BLINK;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* Save a Scroll_display_left or a Scroll_display_right command */
            9 => {
                let mut index = self.command_len.get() as usize;
                match data_1 {
                    /* Scroll_display_left */
                    0 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = SCROLL_DISPLAY_LEFT;
                            index += 1;
                        });
                    }

                    /* Scroll_display_right */
                    1 => {
                        self.command_buffer.map(|buffer| {
                            buffer[index] = SCROLL_DISPLAY_RIGHT;
                            index += 1;
                        });
                    }
                    _ => {}
                }
                self.command_len.replace(index as u8);
                self.handle_commands();
                ReturnCode::SUCCESS
            }

            /* default */
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    /* subscribe syscall not implemented
     *
     * Returns always ENOSUPORT.
     */
    fn subscribe(
        &self,
        _subscribe_num: usize,
        _callback: Option<Callback>,
        _app_id: AppId,
    ) -> ReturnCode {
        return ReturnCode::ENOSUPPORT;
    }
}

impl<'a, A: Alarm<'a>> time::AlarmClient for HD44780<'a, A> {
    /* fired() is called after each alarm finished, and depending on the
     * current state of the program, the next step in being decided.
     */
    fn fired(&self) {
        let state = self.lcd_status.get();

        match state {
            LCDStatus::Idle => {
                self.handle_commands();
            }

            LCDStatus::Begin0 => {
                self.rs_pin.clear();
                self.en_pin.clear();

                if (self.display_function.get() & LCD_8BITMODE) == 0 {
                    self.write_4_bits(0x03, LCDStatus::Begin0_1);
                } else {
                    self.rs_pin.clear();
                    self.lcd_command(
                        (LCD_FUNCTIONSET | self.display_function.get()) >> 4,
                        LCDStatus::Begin4,
                    );
                }
            }

            LCDStatus::Begin0_1 => {
                self.set_delay(200, LCDStatus::Begin1);
            }

            LCDStatus::Begin1 => {
                self.write_4_bits(0x03, LCDStatus::Begin1_2);
            }

            LCDStatus::Begin1_2 => {
                self.set_delay(200, LCDStatus::Begin2);
            }

            LCDStatus::Begin2 => {
                self.write_4_bits(0x03, LCDStatus::Begin2_3);
            }

            LCDStatus::Begin2_3 => {
                self.set_delay(500, LCDStatus::Begin3);
            }

            LCDStatus::Begin3 => {
                self.write_4_bits(0x02, LCDStatus::Begin9);
            }

            LCDStatus::Begin4 => {
                self.command_to_finish
                    .set(LCD_FUNCTIONSET | self.display_function.get());
                self.lcd_command(
                    LCD_FUNCTIONSET | self.display_function.get(),
                    LCDStatus::Begin5,
                );
            }

            LCDStatus::Begin5 => self.set_delay(200, LCDStatus::Begin6),

            LCDStatus::Begin6 => {
                self.lcd_command(
                    LCD_FUNCTIONSET | self.display_function.get(),
                    LCDStatus::Begin7,
                );
            }

            LCDStatus::Begin7 => {
                self.set_delay(500, LCDStatus::Begin8);
            }

            LCDStatus::Begin8 => {
                self.lcd_command(
                    LCD_FUNCTIONSET | self.display_function.get(),
                    LCDStatus::Begin9,
                );
            }

            LCDStatus::Begin9 => {
                self.command_to_finish
                    .set(LCD_FUNCTIONSET | self.display_function.get());
                self.lcd_command(
                    LCD_FUNCTIONSET | self.display_function.get(),
                    LCDStatus::Begin10,
                );
            }

            LCDStatus::Begin10 => {
                self.display_control
                    .set(LCD_DISPLAYON | LCD_CURSORON | LCD_BLINKOFF);
                self.lcd_display(LCDStatus::Begin11);
            }

            LCDStatus::Begin11 => {
                self.lcd_clear(LCDStatus::Begin12);
            }

            LCDStatus::Begin12 => {
                self.display_mode
                    .set(LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT);
                self.command_to_finish
                    .set(LCD_ENTRYMODESET | self.display_mode.get());
                self.lcd_command(self.command_to_finish.get(), LCDStatus::Idle);
            }

            LCDStatus::Clear => {
                self.set_delay(500, self.lcd_after_delay_status.get());
            }

            LCDStatus::Home => {
                self.set_delay(500, self.lcd_after_delay_status.get());
            }

            LCDStatus::Printing => {
                self.write_4_bits(self.command_to_finish.get(), LCDStatus::Idle);
            }

            LCDStatus::PulseLow => {
                self.en_pin.set();
                self.set_delay(500, LCDStatus::PulseHigh);
            }

            LCDStatus::Command => {
                self.write_4_bits(
                    self.command_to_finish.get(),
                    self.lcd_after_command_status.get(),
                );
            }

            LCDStatus::PulseHigh => {
                self.en_pin.clear();
                self.set_delay(500, self.lcd_after_pulse_status.get());
            }
        }
    }
}
