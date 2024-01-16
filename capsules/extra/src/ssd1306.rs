use kernel::{
    utilities::cells::{OptionalCell, TakeCell},
    ErrorCode,
};

use kernel::deferred_call::{
    DeferredCall, DeferredCallClient
};

use crate::bus::{self, Bus, BusWidth};
use kernel::hil::display::{
    Align, FrameBuffer, FrameBufferClient, FrameBufferSetup, GraphicsFrame, GraphicsMode,
    PixelFormat, Point, Rotation, Screen, ScreenClient, Tile,
};
use core::cell::Cell;

pub const SLAVE_ADDRESS_WRITE: u8 = 0b0111100;
pub const SLAVE_ADDRESS_READ: u8 = 0b0111101;
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 64;
pub const BUFFER_PADDING: usize = 1;
pub const TILE_SIZE: u16 = 8;
pub const I2C_ADDR: usize = 0x3D;
pub const WRITE_COMMAND: u8 = 0x40;
pub const CONTROL_COMMAND: u8 = 0x00;

pub const BUFFER_SIZE: usize = 64;
pub const SEQUENCE_BUFFER_SIZE: usize = 32;
#[derive(PartialEq, Copy, Clone)]
pub struct ScreenCommand {
    pub id: CommandId,
    pub parameters: Option<&'static [u8]>,
}

#[repr(u8)]
pub enum MemoryAddressing {
    Page = 0x10,
    Horizontal = 0x00,
    Vertical = 0x01,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ScreenStatus {
    Idle,
    Init,
    Error(ErrorCode),
    SendCommand,
    SendCommandId(bool, usize),
    SendCommandArguments(usize),
    SendCommandArgumentsDone,
    WriteData,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ScreenAsyncCommand {
    Idle,
    AsyncFrameBufferCommand(Result<(), kernel::ErrorCode>),
    AsyncScreenCommand(Result<(), kernel::ErrorCode>),
    Write(Result<(), kernel::ErrorCode>),
}

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum CommandId {
    /* Fundamental Commands */
    // Contrast Control - 2 bytes command
    // 2nd byte: contrast step - 0 <-> 255
    SetContrastControl = 0x81,

    // Entire Display ON
    // 0xA4 - output follows RAM content
    // 0xA5 - output ignores RAM content
    EntireDisplay = 0xA4,

    // Set Normal Display - 0xA6
    // 0/1 in RAM - OFF/ON in display panel
    // Set Inverse Display - 0xA7
    // 0/1 in RAM - ON?OFF in display panel
    SetNormalDisplayOn = 0xA6,
    SetNormalDisplayOff = 0xA7,

    // Set Display Off - 0xAE
    // Set Display On - 0xAF
    SetDisplayOff = 0xAE,
    SetDisplayOn = 0xAF,

    /* Addressing Settings */
    // Set Lower Column - 0x00 <-> 0x0F
    SetLowerColumn = 0x00,

    // Set Higher Column - 0x10 <-> 0x1F
    SetHigherColumn = 0x10,

    // Set Memory Addressing Mode - 2 bytes command
    // 2nd byte: MemoryAddressing enum
    SetMemoryMode = 0x20,

    // Set Column Address - 3 bytes command
    // 2nd byte: column start address (0-127)
    // 3rd byte: column end address (0-127)
    SetColumnAddr = 0x21,

    // Set Page Address - 3 bytes command
    // 2nd byte: page start address (0-7)
    // 3rd byte: page end address (0-7)
    SetPageAddr = 0x22,

    // Set Page Start Address for MemoryAddressing::Page - 0xB0 <-> 0xB7
    SetPageStart = 0xB0,

    /* Hardware Configuration */
    // Set Display Start Line - 0x40 <-> 0x7F
    SetDisplayStart = 0x40,

    // Set Segment Re-map
    // column address 0 -> SEG0 - 0xA0
    // column address 127 -> SEG0 - 0xA1
    SetSegmentRemap0 = 0xA0,
    SetSegmentRemap127 = 0xA1,

    // Set Multiplex Radio - 2 bytes command
    // 2nd byte: mux - 16 <-> 64
    SetMultiplexRadio = 0xA8,

    // Set COM Output Scan Direction
    // from COM0 -> COM[multiplex radio - 1] - 0xC0
    // from COM[multiplex radio - 1] -> COM0 - 0xC8
    SetCOMOutputScanAsc = 0xC0,
    SetCOMOutputScanDes = 0xC8,

    // Set Display Offset - 2 bytes command
    // 2nd byte: the vertical shift - 0 <->63
    SetDisplayOffset = 0xD3,

    // Set COM Pins - 2 bytes command
    // 2nd byte: - bit 4: 0 -> seq COM pin configuration
    //                    1 -> reset + alternative pin configuration
    //           - bit 5: 0 -> reset + disable COM left/right remap
    //                    1 -> enable COM left/right remap
    SetCOMPins = 0xDA,

    /* Timing & Driving Scheme Setting */
    // Set Display Clock Divide Ratio + Oscillator Freq - 2 bytes command
    // 2nd byte: - bits 3:0 -> divide ratio (D) of the display clocks (DCLK)
    //                         D = bits 3:0 + 1
    //           - bits 7:4 -> adds to the oscillator frequency
    SetDisplayClockDivideRatio = 0xD5,

    // Set Pre-charge period - 2 bytes command
    // 2nd byte: - bits 3:0 -> phase 1 period: 1 <-> 15
    //           - bits 7:4 -> phase 2 period: 1 <-> 15
    SetPreChargePeriod = 0xD9,

    // Set Vcomh Deselect Level - 2 bytes command
    // 2nd byte: bits 6:4 - 000b (0,65), 010b (0,77), 020b (0,83)
    SetVcomhDeselect = 0xDB,

    // Nop
    Nop = 0xE3,
    Write = 0xE4,

    /* Scrolling Commands */
    // Continous Horizontal Scroll Setup - 7 bytes commands
    // 2nd, 6th and 7th bytes: dummy bytes
    // 3rd byte: start page address - 0 <-> 7
    // 4th byte: set time interval between each scroll step in frame freq
    //             000b -> 5 | 001b -> 64 | 010b -> 128 | 011b -> 256
    //             100b -> 3 | 101b -> 4  | 110b -> 25  | 111b -> 2
    // 5th byte: end page address - 0 <-> 7 (>= 3rd byte)
    ContHorizontalScrollRight = 0x26,
    ContHorizontalScrollLeft = 0x27,

    // Continous Horizontal & Vertical Scroll Setup - 6 bytes commands
    // 2nd byte: dummy byte
    // 3rd byte: start page address - 0 <-> 7
    // 4th byte: set time interval between each scroll step in frame freq
    //             000b -> 5 | 001b -> 64 | 010b -> 128 | 011b -> 256
    //             100b -> 3 | 101b -> 4  | 110b -> 25  | 111b -> 2
    // 5th byte: end page address - 0 <-> 7 (>= 3rd byte)
    // 6th byte: vertical scroll offset - 0 <-> 63
    ContVertHorizontalScrollRight = 0x29,
    ContVertHorizontalScrollLeft = 0x2A,

    // Deactivate Scrolling that is configured by one of the last 4 commands
    DeactivateScrolling = 0x2E,

    // Activate Scrolling that is configured by one of the last 4 commands
    // Overwrites the previously configured setup
    ActivateScrolling = 0x2F,

    // Set Vertical Scroll Area - 3 bytes command
    // 2nd byte: number of rows in top fixed area
    // 3rd byte: number of rows in scroll area
    SetVerticalScroll = 0xA3,

    /* Charge Pump Settings */
    // Charge Pump Command - 2 bytes command
    // 2nd byte: - 0x14 - enable (followed by 0xAF - display on)
    //           - 0x10 - disable
    ChargePump = 0x8D,
}

const SSD1306_INIT_SEQ: [ScreenCommand; 20] = [
    ScreenCommand {
        id: CommandId::SetDisplayOff,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::SetDisplayClockDivideRatio,
        parameters: Some(&[CONTROL_COMMAND, 0x80]),
    },
    ScreenCommand {
        id: CommandId::SetMultiplexRadio,
        parameters: Some(&[CONTROL_COMMAND, HEIGHT as u8 - 1]),
    },
    ScreenCommand {
        id: CommandId::SetDisplayOffset,
        parameters: Some(&[CONTROL_COMMAND, 0x00]),
    },
    ScreenCommand {
        id: CommandId::SetDisplayStart,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::ChargePump,
        parameters: Some(&[CONTROL_COMMAND, 0x14]),
    },
    ScreenCommand {
        id: CommandId::SetMemoryMode,
        parameters: Some(&[CONTROL_COMMAND, MemoryAddressing::Horizontal as u8]),
    },
    ScreenCommand {
        id: CommandId::SetSegmentRemap127,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::SetCOMOutputScanDes,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::SetCOMPins,
        parameters: Some(&[CONTROL_COMMAND, 0x12]),
    },
    ScreenCommand {
        id: CommandId::SetContrastControl,
        parameters: Some(&[CONTROL_COMMAND, 0xCF]),
    },
    ScreenCommand {
        id: CommandId::SetPreChargePeriod,
        parameters: Some(&[CONTROL_COMMAND, 0xF1]),
    },
    ScreenCommand {
        id: CommandId::SetVcomhDeselect,
        parameters: Some(&[CONTROL_COMMAND, 0x40]),
    },
    ScreenCommand {
        id: CommandId::EntireDisplay,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::SetNormalDisplayOn,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::DeactivateScrolling,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::SetDisplayOn,
        parameters: None,
    },
    ScreenCommand {
        id: CommandId::SetPageAddr,
        parameters: Some(&[CONTROL_COMMAND, 0x00, 0xFF]),
    },
    ScreenCommand {
        id: CommandId::SetColumnAddr,
        parameters: Some(&[CONTROL_COMMAND, 0x00, WIDTH as u8 - 1]),
    },
    ScreenCommand {
        id: CommandId::Write,
        parameters: None,
    },
    // ScreenCommand {
    //     id: CommandId::SetPageAddr,
    //     parameters: Some(&[CONTROL_COMMAND, 0x02, 0x5]),
    // },
    // ScreenCommand {
    //     id: CommandId::SetColumnAddr,
    //     parameters: Some(&[CONTROL_COMMAND, 0xA, WIDTH as u8 - 0x10]),
    // },
    // ScreenCommand {
    //     id: CommandId::Write,
    //     parameters: None,
    // },
    // ScreenCommand {
    //     id: CommandId::Write,
    //     parameters: None,
    // }
];

pub struct SSD1306<'a, B: Bus<'a>> {
    bus: &'a B,
    status: Cell<ScreenStatus>,
    async_status: Cell<ScreenAsyncCommand>,

    rotation: Cell<Rotation>,
    graphics_mode: Cell<GraphicsMode>,
    origin: Cell<Point>,
    tile: Cell<Tile>,

    app_write_buffer: TakeCell<'static, [u8]>,
    bus_write_buffer: TakeCell<'static, [u8]>,
    aux_write_buffer: TakeCell<'static, [u8]>,
    write_buffer_len: Cell<usize>,
    write_buffer_position: Cell<usize>,

    command_sequence: TakeCell<'static, [ScreenCommand]>,
    command_sequence_length: Cell<usize>,
    command_sequence_position: Cell<usize>,
    command_arguments: TakeCell<'static, [u8]>,

    initialization_complete: Cell<bool>,
    screen_client: OptionalCell<&'static dyn ScreenClient>,
    frame_buffer_client: OptionalCell<&'static dyn FrameBufferClient>,

    deferred_caller: DeferredCall,

    initial_write: Cell<bool>,
    invert: Cell<bool>,
}

impl<'a, B: Bus<'a>> SSD1306<'a, B> {
    pub fn new(
        bus: &'a B,
        command_sequence: &'static mut [ScreenCommand],
        command_arguments: &'static mut [u8],
        app_write_buffer: &'static mut [u8],
        bus_write_buffer: &'static mut [u8],
        aux_write_buffer: &'static mut [u8],
        deferred_caller: DeferredCall,
    ) -> SSD1306<'a, B> {
        SSD1306 {
            bus,
            status: Cell::new(ScreenStatus::Idle),
            async_status: Cell::new(ScreenAsyncCommand::Idle),
            rotation: Cell::new(Rotation::Normal),
            origin: Cell::new(Point { x: 0, y: 0 }),
            tile: Cell::new(Tile {
                align: Align {
                    horizontal: 0,
                    vertical: 0,
                },
                size: GraphicsFrame {
                    width: TILE_SIZE,
                    height: TILE_SIZE,
                },
            }),
            graphics_mode: Cell::new(GraphicsMode {
                frame: GraphicsFrame {
                    width: WIDTH as u16,
                    height: HEIGHT as u16,
                },
                pixel_format: PixelFormat::Mono,
            }),
            app_write_buffer: TakeCell::new(app_write_buffer),
            bus_write_buffer: TakeCell::new(bus_write_buffer),
            aux_write_buffer: TakeCell::new(aux_write_buffer),
            write_buffer_len: Cell::new(0),
            write_buffer_position: Cell::new(0),
            command_sequence: TakeCell::new(command_sequence),
            command_sequence_length: Cell::new(0),
            command_sequence_position: Cell::new(0),
            command_arguments: TakeCell::new(command_arguments),
            initialization_complete: Cell::new(false),
            screen_client: OptionalCell::empty(),
            frame_buffer_client: OptionalCell::empty(),
            initial_write: Cell::new(false),
            deferred_caller: deferred_caller,
            invert: Cell::new(false),
        }
    }

    pub fn init(&self) -> Result<(), ErrorCode> {
        if self.status.get() == ScreenStatus::Idle {
            self.status.set(ScreenStatus::Init);
            self.do_next_op();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    // todo remove pub fn initialize_callback_handle(&self, handle: DeferredCallHandle) {
    //     self.handle.replace(handle);
    // }

    pub fn do_next_op(&self) {
        match self.status.get() {
            ScreenStatus::Init => {
                self.status.set(ScreenStatus::Idle);
                if let Err(err) = self.prepare_init_sequence() {
                    self.status.set(ScreenStatus::Error(err));
                } else {
                    self.command_sequence_position.set(0);
                    self.status.set(ScreenStatus::SendCommand);
                }
                self.do_next_op();
            }
            ScreenStatus::Idle => {}
            ScreenStatus::Error(err) => {
                panic!("{:?}", err);
            }
            ScreenStatus::SendCommand => {
                let position = self.command_sequence_position.get();
                if position < self.command_sequence_length.get() {
                    self.command_sequence.map_or_else(
                        || panic!("ssd1306: do next op has no command sequence buffer"),
                        |command_sequence| {
                            self.send_command(command_sequence[position]);
                        },
                    )
                } else {
                    // todo commands done
                    self.status.set(ScreenStatus::Idle);
                    if !self.initialization_complete.get() {
                        self.initialization_complete.set(true);
                        self.command_sequence_position.set(0);
                        self.command_sequence_length.set(0);
                    } else {
                        self.do_next_op();
                    }
                }
            }
            ScreenStatus::SendCommandId(arguments, len) => {
                if arguments {
                    self.status.set(ScreenStatus::SendCommandArguments(len));
                    self.do_next_op();
                } else {
                    self.command_sequence_position
                        .set(self.command_sequence_position.get() + 1);
                    self.status.set(ScreenStatus::SendCommand);
                    self.do_next_op();
                }
            }
            ScreenStatus::SendCommandArguments(len) => {
                self.send_arguments(len);
            }
            ScreenStatus::SendCommandArgumentsDone => {
                self.command_sequence_position
                    .set(self.command_sequence_position.get() + 1);
                self.status.set(ScreenStatus::SendCommand);
                self.do_next_op();
            }
            ScreenStatus::WriteData => {
                self.prepare_write_buffer();
                self.status.set(ScreenStatus::SendCommand);
                self.do_next_op();
            }
        }
    }

    fn send_arguments(&self, len: usize) {
        self.command_arguments.take().map_or_else(
            || panic!("ssd1306: send argument has no command arguments buffer"),
            |arguments| {
                self.status.set(ScreenStatus::SendCommandArgumentsDone);
                let _ = self.bus.write(BusWidth::Bits8, arguments, len);
            },
        );
    }

    fn prepare_write_buffer(&self) {
        self.bus_write_buffer.map_or_else(
            || panic!("write function has no write buffer"),
            |bus_write_buffer| {
                bus_write_buffer[0] = WRITE_COMMAND;

                self.aux_write_buffer.map_or_else(
                    || panic!("write function has no app write buffer"),
                    |app_write_buffer| {
                        let mut app_buf_index = 0;
                        let GraphicsMode {
                            frame: GraphicsFrame { width, height },
                            pixel_format: _,
                        } = self.graphics_mode.get();
                        let Point { x, y } = self.origin.get();
                        for h in y..y + height {
                            for _l in x..x + width {
                                for index in 0..8 {
                                    let bit = (app_write_buffer[app_buf_index]
                                        & (1 << (7 - index)))
                                        >> (7 - index);
                                    let buffer_index = (app_buf_index % (width as usize / 8)) * 8
                                        + h as usize * WIDTH
                                        + index
                                        + x as usize;
                                    let bit_index = ((app_buf_index % width as usize)
                                        / (width as usize / 8))
                                        as u8;

                                    if bit == 0 {
                                        bus_write_buffer[buffer_index] &= !(1 << bit_index);
                                    } else if bit == 1 {
                                        bus_write_buffer[buffer_index] |= 1 << bit_index;
                                    }
                                }
                                app_buf_index += 1;
                            }
                        }
                    },
                );
            },
        );
    }

    fn send_command(&self, cmd: ScreenCommand) {
        if cmd.id == CommandId::Write {
            self.bus_write_buffer.take().map_or_else(
                || panic!("ssd1306: send_command has no write buffer"),
                |buffer| {
                    buffer[0] = WRITE_COMMAND;
                    if !self.initial_write.get() {
                        self.initial_write.set(true);
                        let GraphicsMode {
                            frame: GraphicsFrame { width, height },
                            pixel_format: _,
                        } = self.graphics_mode.get();
                        self.write_buffer_len.set((height * width) as usize / 8);
                        for i in 0..self.write_buffer_len.get() {
                            buffer[i + 1] = 0x00;
                        }
                    }
                    for i in 1..buffer.len() {
                        buffer[i] = !buffer[i];
                    }
                    self.status.set(ScreenStatus::SendCommandId(false, 0));
                    let _ = self.bus.write(BusWidth::Bits8, buffer, buffer.len());
                },
            )
        } else {
            let _ = self.bus.set_addr(BusWidth::Bits16LE, cmd.id as usize);
            let new_state = if let Some(params) = cmd.parameters {
                self.populate_arguments_buffer(cmd);
                ScreenStatus::SendCommandId(true, params.len())
            } else {
                ScreenStatus::SendCommandId(false, 0)
            };
            self.status.set(new_state);
        }
    }

    fn populate_arguments_buffer(&self, cmd: ScreenCommand) {
        self.command_arguments.map_or_else(
            || panic!("ssd1306 populate arguments has no command arguments buffer"),
            |command_buffer| {
                if let Some(parameters) = cmd.parameters {
                    for (i, param) in parameters.iter().enumerate() {
                        command_buffer[i] = *param;
                    }
                }
            },
        )
    }

    fn prepare_init_sequence(&self) -> Result<(), ErrorCode> {
        if self.status.get() == ScreenStatus::Idle {
            self.command_sequence.map_or_else(
                || panic!("ssd1306: init sequence has no command sequence buffer"),
                |command_sequence| {
                    if SSD1306_INIT_SEQ.len() <= command_sequence.len() {
                        self.command_sequence_length.set(SSD1306_INIT_SEQ.len());
                        for (i, cmd) in SSD1306_INIT_SEQ.iter().enumerate() {
                            command_sequence[i] = *cmd;
                        }
                        Ok(())
                    } else {
                        Err(ErrorCode::NOMEM)
                    }
                },
            )
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn prepare_write_sequence(&self) -> Result<(), ErrorCode> {
        if self.status.get() == ScreenStatus::Idle {
            self.command_sequence.map_or_else(
                || panic!("ssd1306: write sequence has no command sequence buffer"),
                |command_sequence| {
                    command_sequence[0] = ScreenCommand {
                        id: CommandId::SetPageAddr,
                        parameters: Some(&[CONTROL_COMMAND, 0x00, 0xFF]),
                    };
                    command_sequence[1] = ScreenCommand {
                        id: CommandId::SetColumnAddr,
                        parameters: Some(&[CONTROL_COMMAND, 0x00, WIDTH as u8 - 1]),
                    };
                    command_sequence[2] = ScreenCommand {
                        id: CommandId::Write,
                        parameters: None,
                    };
                    self.command_sequence_length.set(3);
                    self.command_sequence_position.set(0);
                    Ok(())
                },
            )
        } else {
            Err(ErrorCode::BUSY)
        }
    }
}

impl<'a, B: Bus<'a>> bus::Client for SSD1306<'a, B> {
    fn command_complete(
        &self,
        buffer: Option<&'static mut [u8]>,
        _len: usize,
        status: Result<(), kernel::ErrorCode>,
    ) {
        if let Some(buf) = buffer {
            if self.status.get() == ScreenStatus::SendCommandArgumentsDone {
                self.command_arguments.replace(buf);
            } else {
                // write command complete
                self.bus_write_buffer.replace(buf);
                self.frame_buffer_client.map_or_else(
                    || panic!("ssd1306: do next op has no screen client"),
                    |frame_buffer_client| {
                        // callback
                        self.write_buffer_len.replace(0);
                        self.write_buffer_position.replace(0);
                        frame_buffer_client.command_complete(status);
                    },
                );
            }
        }

        if let Err(err) = status {
            self.status.set(ScreenStatus::Error(err));
        }

        self.do_next_op();
    }
}

impl<'a, B: Bus<'a>> FrameBuffer<'static> for SSD1306<'a, B> {
    fn get_mode(&self) -> GraphicsMode {
        self.graphics_mode.get()
    }

    fn get_tile_format(&self) -> kernel::hil::display::Tile {
        self.tile.get()
    }

    fn set_write_frame(&self, origin: Point, size: GraphicsFrame) -> Result<(), ErrorCode> {
        if !self.initialization_complete.get() {
            Err(ErrorCode::OFF)
        } else if self.status.get() == ScreenStatus::Idle {
            let mut current_mode = self.graphics_mode.get();
            if (origin.x + size.width > WIDTH as u16) || (origin.y + size.height > HEIGHT as u16) {
                return Err(ErrorCode::INVAL);
            }
            current_mode.frame.height = size.height / 8;
            current_mode.frame.width = size.width;
            self.graphics_mode.replace(current_mode);
            self.origin.replace(Point {
                x: origin.x,
                y: origin.y / 8,
            });
            self.async_status
                .set(ScreenAsyncCommand::AsyncFrameBufferCommand(Ok(())));
            // todo remove self.handle.map(|handle| self.deferred_caller.set(*handle));
            self.deferred_caller.set();
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn get_buffer_padding(&self) -> (usize, usize) {
        (BUFFER_PADDING, 0)
    }

    fn write(
        &self,
        buffer: &'static mut [u8],
        len: usize,
        reset_position: bool,
    ) -> Result<(), ErrorCode> {
        let ret = if !self.initialization_complete.get() {
            Err(ErrorCode::OFF)
        } else if self.status.get() == ScreenStatus::Idle {
            if reset_position {
                self.write_buffer_position.set(0);
                self.write_buffer_len.set(0);
            }
            self.app_write_buffer.replace(buffer);
            let mut status = Ok(());
            self.app_write_buffer.map_or_else(
                || panic!("write has no app buffer"),
                |app_buffer| {
                    self.aux_write_buffer.map_or_else(
                        || panic!("write has no aux buffer"),
                        |aux_buffer| {
                            let current_position = self.write_buffer_position.get();
                            if len + current_position > aux_buffer.len() {
                                status = Err(ErrorCode::INVAL);
                            } else {
                                aux_buffer[current_position..(len + current_position)]
                                    .copy_from_slice(
                                        &app_buffer[BUFFER_PADDING..(len + BUFFER_PADDING)],
                                    );
                                self.write_buffer_position.replace(len + current_position);
                            }
                            self.write_buffer_len.set(len + current_position);
                        },
                    );
                },
            );
            status
        } else {
            Err(ErrorCode::BUSY)
        };
        self.async_status.set(ScreenAsyncCommand::Write(ret));
        // todo removeself.handle.map(|handle| self.deferred_caller.set(*handle));
        self.deferred_caller.set();
        ret
    }

    fn flush(&self) -> Result<(), ErrorCode> {
        if !self.initialization_complete.get() {
            Err(ErrorCode::OFF)
        } else if self.status.get() == ScreenStatus::Idle {
            if self.bus_write_buffer.is_none() {
                Err(ErrorCode::NOSUPPORT)
            } else {
                if let Err(err) = self.prepare_write_sequence() {
                    self.status.set(ScreenStatus::Error(err));
                } else {
                    self.status.set(ScreenStatus::WriteData);
                }
                self.do_next_op();
                Ok(())
            }
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn set_client(&self, client: Option<&'static dyn FrameBufferClient>) {
        if let Some(client) = client {
            self.frame_buffer_client.set(client);
        } else {
            self.frame_buffer_client.clear();
        }
    }
}

impl<'a, B: Bus<'a>> Screen<'static> for SSD1306<'a, B> {
    fn get_rotation(&self) -> Rotation {
        self.rotation.get()
    }

    fn set_client(&self, client: Option<&'static dyn ScreenClient>) {
        if let Some(client) = client {
            self.screen_client.set(client);
        } else {
            self.screen_client.clear();
        }
    }

    fn set_brightness(&self, _brightness: u16) -> Result<(), kernel::ErrorCode> {
        self.async_status
            .set(ScreenAsyncCommand::AsyncScreenCommand(Ok(())));
        // todo remove self.handle.map(|handle| self.deferred_caller.set(*handle));
        self.deferred_caller.set();
        Ok(())
    }

    fn set_power(&self, _enabled: bool) -> Result<(), kernel::ErrorCode> {
        self.async_status
            .set(ScreenAsyncCommand::AsyncScreenCommand(Ok(())));
        // todo remove self.handle.map(|handle| self.deferred_caller.set(*handle));
        self.deferred_caller.set();
        Ok(())
    }

    fn set_invert(&self, enabled: bool) -> Result<(), kernel::ErrorCode> {
        self.invert.replace(enabled);
        self.async_status
            .set(ScreenAsyncCommand::AsyncScreenCommand(Ok(())));
        //todo remove self.handle.map(|handle| self.deferred_caller.set(*handle));
        self.deferred_caller.set();
        Ok(())
    }

    fn set_rotation(&self, rotation: Rotation) -> Result<(), ErrorCode> {
        self.rotation.set(rotation);
        // todo update origin and graphics mode
        self.async_status
            .set(ScreenAsyncCommand::AsyncScreenCommand(Ok(())));
        //todo remove self.handle.map(|handle| self.deferred_caller.set(*handle));
        self.deferred_caller.set();
        Ok(())
    }
}

impl<'a, B: Bus<'a>> FrameBufferSetup<'static> for SSD1306<'a, B> {
    fn set_mode(&self, mode: GraphicsMode) -> Result<(), ErrorCode> {
        self.graphics_mode.replace(mode);
        Ok(())
    }

    fn get_num_supported_modes(&self) -> usize {
        1
    }

    fn get_supported_mode(&self, index: usize) -> Option<GraphicsMode> {
        match index {
            0 => Some(self.graphics_mode.get()),
            _ => None,
        }
    }
}

impl<'a, B: Bus<'a>> DeferredCallClient for SSD1306<'a, B> {
    fn handle_deferred_call(&self) {
        match self.async_status.get() {
            ScreenAsyncCommand::Idle => panic!("Received dynamic call without a caller"),
            ScreenAsyncCommand::AsyncScreenCommand(res) => {
                self.screen_client.map_or_else(
                    || panic!("ssd1306: dynamic deferred call client has no screen setup client"),
                    |screen_client| {
                        self.async_status.set(ScreenAsyncCommand::Idle);
                        screen_client.command_complete(res);
                    },
                );
            }
            ScreenAsyncCommand::AsyncFrameBufferCommand(res) => {
                self.frame_buffer_client.map_or_else(
                    || panic!("ssd1306: dynamic deferred call client has no frame buffer client"),
                    |frame_buffer_client| {
                        self.async_status.set(ScreenAsyncCommand::Idle);
                        frame_buffer_client.command_complete(res);
                    },
                );
            }
            ScreenAsyncCommand::Write(res) => {
                self.frame_buffer_client.map_or_else(
                    || panic!("ssd1306: dynamic deferred call client has no frame buffer client"),
                    |frame_buffer_client| {
                        self.app_write_buffer.take().map_or_else(
                            || panic!("ssd1306: dynamic deferred call has no app write buffer"),
                            |buffer| {
                                self.async_status.set(ScreenAsyncCommand::Idle);
                                frame_buffer_client.write_complete(buffer, res);
                            },
                        );
                    },
                );
            }
        }
    }

    fn register(&'static self) {
        self.deferred_caller.register(self);
    }
}
