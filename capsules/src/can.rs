
use core::mem::size_of;

use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::processbuffer::{ReadableProcessBuffer, WriteableProcessBuffer};
use kernel::syscall::{CommandReturn, SyscallDriver};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ProcessId;
use kernel::{debug, ErrorCode};
use kernel::hil::can;

use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::Can as usize;
pub const BYTE4_MASK: usize = 0xff000000;
pub const BYTE3_MASK: usize = 0xff0000;
pub const BYTE2_MASK: usize = 0xff00;
pub const BYTE1_MASK: usize = 0xff;

pub static mut CAN_TX_BUF: [u8; can::STANDARD_CAN_PACKET_SIZE] = [0; can::STANDARD_CAN_PACKET_SIZE];
pub static mut CAN_RX_BUF: [u8; 128] = [0; 128];

pub struct CanCapsule<'a, Can: can::Can> {
    // CAN driver
    can: &'a Can,

    // CAN buffers
    can_tx: TakeCell<'static, [u8; can::STANDARD_CAN_PACKET_SIZE]>,
    can_rx: TakeCell<'static, [u8]>,

    // App logic
    error_occured: OptionalCell<bool>,

    // App
    apps: Grant<App, UpcallCount<5>, AllowRoCount<1>, AllowRwCount<1>>,
    appid: OptionalCell<ProcessId>,
    state_changed_callback: OptionalCell<(can::State, u8)>,
}

pub struct App<> {
    index: usize,
    lost_messages: u32,
}

impl<> Default for App<> {
    fn default() -> Self {
        App {
            index: 0,
            lost_messages: 0,
        }
    }
}

impl<'a, Can: can::Can> CanCapsule<'a, Can> {
    pub fn new(
        can: &'a Can,
        grant: Grant<App, UpcallCount<5>, AllowRoCount<1>, AllowRwCount<1>>,
        can_tx: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE],
        can_rx: &'static mut [u8; 128],
    ) -> CanCapsule<'a, Can> {
        CanCapsule {
            can,
            can_tx: TakeCell::new(can_tx),
            can_rx: TakeCell::new(can_rx),
            error_occured: OptionalCell::empty(),
            apps: grant,
            appid: OptionalCell::empty(),
            state_changed_callback: OptionalCell::empty(),
        }
    }

    fn schedule_callback(&self, callback_number: usize, data1: usize, data2: usize, data3: usize) {
        self.appid.map(|appid| {
            let _ = self.apps.enter(*appid, |_app, kernel_data| {
                kernel_data.schedule_upcall(callback_number, (data1, data2, data3)).ok();
            });
        });
    }

    pub fn process_send_command(&self, processid: &mut ProcessId, id: can::Id, length: usize) -> Result<(), ErrorCode>{
        self.apps
            .enter(*processid, |_, kernel_data| {  
                kernel_data
                    .get_readonly_processbuffer(0)
                    .map(|buffer_ref| {
                        buffer_ref.enter(|buffer| {
                            debug!("do we get here???");
                            self.can_tx.take().map(|dest_buffer| {
                                for i in 0..length {
                                    dest_buffer[i] = buffer[i].get();
                                }
                                match self.can.send(id, dest_buffer, length) {
                                    Ok(_) => Ok(()),
                                    Err((err, buf)) => {
                                        self.can_tx.replace(buf);
                                        Err(err)
                                    },
                                }
                            }).unwrap_or(Err(ErrorCode::NOMEM))
                        }).unwrap_or_else(|err| err.into())
                    }).unwrap_or_else(|err| err.into())
            }).unwrap_or_else(|err| err.into())
    }
}

impl<'a, Can: can::Can> SyscallDriver for CanCapsule<'a, Can> {
    fn command(
        &self,
        command_num: usize,
        arg1: usize,
        arg2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        match command_num {
            0 => CommandReturn::success_u32(self.can.receive_fifo_count() as u32),

            1 => {
                let time_segment1 = (arg1 & BYTE4_MASK) >> 24;
                let time_segment2 = (arg1 & BYTE3_MASK) >> 16;
                let sjw = (arg1 & BYTE2_MASK) >> 8;
                let brp = arg1 & BYTE1_MASK;
                match self.can.set_bit_timing(can::BitTiming {
                    segment1: time_segment1 as u8,
                    segment2: time_segment2 as u8,
                    sync_jump_width: sjw as u32,
                    baud_rate_prescaler: brp as u32,
                }) {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
            }

            2 => {
                let operation_mode = match arg1 {
                    0 => can::OperationMode::Loopback,
                    1 => can::OperationMode::Monitoring,
                    2 => can::OperationMode::Freeze,
                    _ => can::OperationMode::Normal,
                };
                match self.can.set_operation_mode(operation_mode) {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
            }

            4 => {
                self.appid.set(appid);
                self.state_changed_callback.set((can::State::Running, 0));
                match self.can.enable() {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
            }

            5 => {
                self.appid.set(appid);
                self.state_changed_callback.set((can::State::Disabled, 0));
                match self.can.disable() {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
                
            }

            6 => {
                self.appid.set(appid);
                // send message with 16-bit identifier
                let id = can::Id::Standard(arg1 as u16);
                self.appid.map_or(CommandReturn::failure(ErrorCode::BUSY), |processid| {
                    match self.process_send_command(processid, id, arg2) {
                        Ok(_) => CommandReturn::success(),
                        Err(err) => return CommandReturn::failure(err),
                    }
                })
            }

            7 => {
                self.appid.set(appid);
                // send message with 32-bit identifier
                let id = can::Id::Extended(arg1 as u32);
                self.appid.map_or(CommandReturn::failure(ErrorCode::BUSY), |processid| {
                    match self.process_send_command(processid, id, arg2) {
                        Ok(_) => CommandReturn::success(),
                        Err(err) => CommandReturn::failure(err),
                    }
                })
            }

            8 => {
                self.appid.set(appid);
                self.can_rx.take().map(|dest_buffer| {
                    self.apps.enter(appid, |_, kernel| {
                        match kernel.get_readwrite_processbuffer(0).map(|buffer_ref| {
                            buffer_ref.enter(|buffer| {
                                if buffer.len() >= 16 + size_of::<u32>() {
                                    Ok(())
                                } else {
                                    Err(ErrorCode::SIZE)
                                }
                            }).unwrap_or_else(|err| err.into())
                        }).unwrap_or_else(|err| err.into()) {
                            Ok(_) => {
                                match self.can.start_receive_process(dest_buffer) {
                                    Ok(_) => CommandReturn::success(),
                                    Err((err, _)) => CommandReturn::failure(err),
                                }
                            },
                            Err(err) => CommandReturn::failure(err.into()),
                        }
                    }).unwrap_or_else(|err| err.into())
                }).unwrap_or(CommandReturn::failure(ErrorCode::NOMEM))
            }

            9 => {
                self.appid.set(appid);
                match self.can.stop_receive() {
                    Ok(_) => CommandReturn::success(),
                    Err(err) => CommandReturn::failure(err),
                }
            }
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }

    fn allocate_grant(&self, process_id: ProcessId) -> Result<(), kernel::process::Error> {
        self.apps.enter(process_id, |_, _| {})
    }
}

impl<'a, Can: can::Can> can::ControllerClient for CanCapsule<'a, Can> {
    fn state_changed(&self, state: can::State) {
        // debug!("[status_changed] callback");
        match self.state_changed_callback.map(|capsule_state| -> (can::State, u8) {
            capsule_state.1 = capsule_state.1 + 1;
            if state != capsule_state.0 {
                self.error_occured.set(true);
            }
            *capsule_state
        }) {
            Some(state) => self.state_changed_callback.set(state),
            None => {
                if self.error_occured.is_some() {
                    match state {
                        can::State::Running | can::State::Error(_) => {
                            self.schedule_callback(2, ErrorCode::FAIL as usize, 0, 0)
                        },
                        can::State::Disabled => unreachable!(),
                    }
                }
            }
        }
    }

    fn enabled(&self, status: Result<can::State, ErrorCode>) {
        // debug!("[enabled] callback");
        match status {
            Ok(peripheral_state) => {
                match self.state_changed_callback.take() {
                    Some(mut driver_state) => {
                        match driver_state.0 {
                            can::State::Running => {
                                driver_state.1 = driver_state.1 + 1;
                                if driver_state.1 == 2 && peripheral_state == driver_state.0 {
                                    self.schedule_callback(0, 0, 0, 0);
                                } else {
                                    self.schedule_callback(0, ErrorCode::FAIL as usize, 0, 0);
                                }
                            }
                            can::State::Disabled => {
                                self.schedule_callback(0, ErrorCode::OFF as usize, 0, 0);
                            }
                            can::State::Error(err) => {
                                self.schedule_callback(0, err as usize, 0, 0);
                            }
                        }
                    },
                    None => todo!(),
                };
            }
            Err(err) => {
                self.schedule_callback(0, err as usize, 0, 0);
            },
        }
        
        }

    fn disabled(&self, status: Result<(), ErrorCode>) {
        match status {
            Ok(()) => {
                match self.state_changed_callback.take() {
                    Some(mut driver_state) => {
                        match driver_state.0 {
                            can::State::Disabled => {
                                driver_state.1 = driver_state.1 + 1;
                                if driver_state.1 == 2 {
                                    self.schedule_callback(1, 0, 0, 0);
                                } else {
                                    self.schedule_callback(1, ErrorCode::FAIL as usize, 0, 0);
                                }
                            }
                            can::State::Running => {
                                self.schedule_callback(1, ErrorCode::OFF as usize, 0, 0);
                            }
                            can::State::Error(err) => {
                                self.schedule_callback(1, err as usize, 0, 0);
                            }
                        }
                    },
                    None => todo!(),
                };
            },
            Err(err) => {
                self.schedule_callback(0, err as usize, 0, 0);
            },
        }
    }
}

impl<'a, Can: can::Can> can::TransmitClient<{ can::STANDARD_CAN_PACKET_SIZE }> for CanCapsule<'a, Can> {
    fn transmit_complete(&self, status: Result<(), can::Error>, buffer: &'static mut [u8; can::STANDARD_CAN_PACKET_SIZE]) {
        match status {
            Ok(()) => {
                self.can_tx.replace(buffer);
                self.schedule_callback(2, 0, 0, 0)
            },
            Err(err) => {
                self.schedule_callback(2, err as usize, 0, 0);
            },
        }
        
    }
}

impl<'a, Can: can::Can> can::ReceiveClient for CanCapsule<'a, Can> {
    fn message_received(&self, id: can::Id, buffer: &mut [u8], len: usize, status: Result<(), can::Error>) {
        let mut new_buffer = false;
        let mut shared_len = 0;
        match status {
            Ok(_) => {
                match self.appid.map_or(Err(ErrorCode::NOMEM), |processid| {
                    self.apps
                        .enter(*processid, |app_data, kernel_data| {
                            kernel_data
                                .get_readwrite_processbuffer(0 as usize)
                                .map(|buffer_ref| {
                                    buffer_ref.mut_enter(|user_buffer| {
                                        shared_len = user_buffer.len();
                                        // copy buffer in the grant buffer
                                        if user_buffer[0].get() == 0 {
                                            // debug!("new buffer");
                                            new_buffer = true;
                                            app_data.index = size_of::<u32>();
                                        }
                                        user_buffer[0].set(user_buffer[0].get() + 1);
                                        // debug!("[message_received] received {:?} on buffer {}", &buffer[0..recv_len], 0);
                                        if app_data.index + len > user_buffer.len() {
                                            app_data.lost_messages = app_data.lost_messages + 1;
                                            Err(ErrorCode::SIZE)
                                        } else {
                                            let r = user_buffer[app_data.index..app_data.index + len].copy_from_slice_or_err(&buffer[0..len]); 
                                            if r.is_ok() {
                                                app_data.index = app_data.index + len;
                                            }
                                            r
                                        }
                                        
                                    }).unwrap_or_else(|err| err.into())
                                }).unwrap_or_else(|err| err.into())
                        }).unwrap_or_else(|err| err.into())
                }) {
                    Err(err) => {
                        // debug!("[message_received] err {:?} {}", err, err as usize);
                        self.schedule_callback(3, err as usize, 0, 0)
                    },
                    Ok(_) => {
                        if new_buffer {
                            self.schedule_callback(3, 0, shared_len as usize, match id {
                                can::Id::Standard(u16) => u16 as usize,
                                can::Id::Extended(u32) => u32 as usize,
                            })
                        }  
                    }
                }
            },
            Err(_) => todo!(),
        };
    }

    fn stopped(&self, buffer: &'static mut [u8]) {
        match self.appid.map_or(Err(ErrorCode::NOMEM), |processid| {
            self.apps
                .enter(*processid, |_, kernel_data| {
                    kernel_data
                        .get_readwrite_processbuffer(0 as usize)
                        .map(|buffer_ref| {
                            buffer_ref.mut_enter(|user_buffer| {
                                // copy buffer in the grant buffer
                                let len = user_buffer.len();
                                debug!("[message_received] received {:?} on buffer {}", &buffer[0..len], 0);
                                user_buffer[0..len].copy_from_slice_or_err(&buffer[0..len])
                            }).unwrap_or_else(|err| err.into())
                        }).unwrap_or_else(|err| err.into())
                }).unwrap_or_else(|err| err.into())
        }) {
            Err(err) => {
                debug!("[message_received] err {:?} {}", err, err as usize);
                self.schedule_callback(4, err as usize, 0, 0)
            },
            Ok(_) => {
                debug!("[message_received] sending ok callback");
                self.schedule_callback(4, 0, 0, 0)
            }
        }
    }
}