use core::cell::Cell;
use kernel::debug;
use kernel::hil::gpio::Pin;
use kernel::hil::spi::{SpiMaster, SpiMasterClient};
use kernel::hil::time::{Alarm, ConvertTicks};
use kernel::hil::wifinina;
use kernel::hil::wifinina::{Network, Scanner, ScannerClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;

const START_CMD: u8 = 0xe0;
const END_CMD: u8 = 0xee;
const ERROR_CMD: u8 = 0xef;

const POS_CMD: usize = 1;
const POS_PARAM_LEN: usize = 2;
const POS_LEN: usize = 2;
const POS_PARAM: usize = 3;

const REPLY_FLAG: u8 = 1 << 7;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum Command {
    GetFwVersion = 0x37,
    StartScanNetworksCmd = 0x36,
    ScanNetworksCmd = 0x27,
    GetConnStatusCmd = 0x20,
}
#[derive(Copy, Clone, PartialEq, Debug)]
enum InitStatus {
    Starting,
    Initialized,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Status {
    Idle,
    Init(InitStatus),
    Send(Command),
    Receive(Command, usize, usize),
    StartScanNetworks,
    ScanNetworks,
}

pub struct NinaW102<'a, S: SpiMaster, P: Pin, A: Alarm<'a>> {
    spi: &'a S,
    write_buffer: TakeCell<'static, [u8]>,
    read_buffer: TakeCell<'static, [u8]>,
    one_byte_read_buffer: TakeCell<'static, [u8]>,
    cs: &'a P,
    ready: &'a P,
    reset: &'a P,
    gpio0: &'a P,
    alarm: &'a A,
    status: Cell<Status>,
    networks: TakeCell<'static, [Network]>,
    wifi_client: OptionalCell<&'a dyn wifinina::ScannerClient>,
}

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'a>> NinaW102<'a, S, P, A> {
    pub fn new(
        spi: &'a S,
        write_buffer: &'static mut [u8],
        read_buffer: &'static mut [u8],
        one_byte_read_buffer: &'static mut [u8],
        cs: &'a P,
        ready: &'a P,
        reset: &'a P,
        gpio0: &'a P,
        alarm: &'a A,
        networks: &'static mut [Network],
    ) -> Self {
        cs.make_output();
        ready.make_input();
        reset.make_output();
        gpio0.make_output();

        NinaW102 {
            spi,
            write_buffer: TakeCell::new(write_buffer),
            read_buffer: TakeCell::new(read_buffer),
            one_byte_read_buffer: TakeCell::new(one_byte_read_buffer),
            cs,
            ready,
            reset,
            gpio0,
            alarm: alarm,
            status: Cell::new(Status::Idle),
            networks: TakeCell::new(networks),
            wifi_client: OptionalCell::empty(),
        }
    }

    pub fn init(&self) -> Result<(), ErrorCode> {
        self.cs.set();
        self.reset.clear();
        self.gpio0.set();
        self.status.set(Status::Init(InitStatus::Starting));

        self.alarm
            .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(10));
        Ok(())
    }

    pub fn get_firmware_version(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetFwVersion, 0)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn scan_networks(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle || self.status.get() == Status::ScanNetworks {
            self.send_command(Command::ScanNetworksCmd, 0)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn start_scan_networks(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle || self.status.get() == Status::StartScanNetworks {
            self.send_command(Command::StartScanNetworksCmd, 0)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_connection_status(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetConnStatusCmd, 0)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn wait_for_chip_ready(&self) -> Result<(), ErrorCode> {
        for _i in 0..100000000 {
            if !self.ready.read() {
                return Ok(());
            }
        }
        Err(ErrorCode::BUSY)
    }

    fn wait_for_chip_select(&self) -> Result<(), ErrorCode> {
        self.cs.clear();
        for _i in 0..100000 {
            if self.ready.read() {
                return Ok(());
            }
        }
        self.cs.set();
        Err(ErrorCode::NOACK)
    }

    fn send_command(&self, command: Command, num_params: u8) -> Result<(), ErrorCode> {
        self.wait_for_chip_ready()?;

        self.wait_for_chip_select()?;

        self.write_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |buffer| {
                buffer[0] = START_CMD;
                buffer[POS_CMD] = (command as u8) & !REPLY_FLAG;
                buffer[POS_PARAM_LEN] = num_params;
                buffer[3] = END_CMD;

                self.spi.release_low();

                self.spi
                    .read_write_bytes(buffer, None, 4)
                    .map_err(|(err, write_buffer, _)| {
                        self.write_buffer.replace(write_buffer);
                        // read_buffer.map(|buffer| self.read_buffer.replace(buffer));
                        err
                    })?;

                self.status.set(Status::Send(command));

                Ok(())
            })
            .map_err(|err| {
                self.cs.set();
                err
            })
    }

    fn receive_byte(
        &self,
        command: Command,
        position: usize,
        timeout: usize,
    ) -> Result<(), ErrorCode> {
        self.write_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |buffer| {
                buffer[0] = 0xff;

                self.one_byte_read_buffer.take().map_or(
                    Err(ErrorCode::NOMEM),
                    move |read_buffer| {
                        self.status.set(Status::Receive(command, position, timeout));
                        self.spi.hold_low();
                        self.spi
                            .read_write_bytes(buffer, Some(read_buffer), 1)
                            .map_err(|(err, write_buffer, read_buffer)| {
                                self.write_buffer.replace(write_buffer);
                                read_buffer.map(|buffer| self.one_byte_read_buffer.replace(buffer));
                                err
                            })
                    },
                )

            })
            .map_err(|err| {
                self.cs.set();
                err
            })
    }

    fn receive_command(&self, command: Command) -> Result<(), ErrorCode> {
        self.wait_for_chip_ready()?;

        self.wait_for_chip_select()?;

        self.receive_byte(command, 0, 1000)
    }

    fn process_buffer(&self, command: Command) -> Result<(), ErrorCode> {
        self.read_buffer
            .map_or(Err(ErrorCode::NOMEM), |read_buffer| {
                if read_buffer[0] == START_CMD {
                    if read_buffer[POS_CMD] == (command as u8) | REPLY_FLAG {
                        let param_len = read_buffer[POS_LEN];
                        let mut current_position = 0;
                        for _parameter_index in 0..param_len {
                            let pos = POS_PARAM + current_position;

                            if pos < read_buffer.len() {
                                current_position =
                                    (current_position + read_buffer[pos] as usize) as usize;
                            } else {
                                break;
                            }
                            current_position = current_position + 1;
                        }

                        let end_pos = POS_PARAM + current_position;

                        if end_pos < read_buffer.len() && read_buffer[end_pos] == END_CMD {
                            match command {
                                Command::GetFwVersion => {
                                    debug!("{:?}", core::str::from_utf8(&read_buffer[4..10]));
                                    Ok(())
                                }
                                Command::GetConnStatusCmd => {
                                    Ok(())
                                }
                                Command::StartScanNetworksCmd => {
                                    self.status.set(Status::ScanNetworks);
                                    self.alarm.set_alarm(
                                        self.alarm.now(),
                                        self.alarm.ticks_from_ms(2000),
                                    );

                                    Ok(())
                                }
                                Command::ScanNetworksCmd => {
                                    self.networks.map(|networks| {
                                        let mut current_position = 0;
                                        for parameter_index in 0..param_len {
                                            let pos = POS_PARAM + current_position;

                                            if pos < read_buffer.len() {
                                                for (d, s) in networks[parameter_index as usize]
                                                    .ssid
                                                    .value
                                                    .iter_mut()
                                                    .zip(
                                                        read_buffer[pos + 1
                                                            ..pos
                                                                + (read_buffer[pos] as usize)
                                                                + 1]
                                                            .iter(),
                                                    )
                                                {
                                                    *d = *s
                                                }
                                                networks[parameter_index as usize].ssid.len =
                                                    read_buffer[pos];
                                                networks[parameter_index as usize].security = None;
                                                networks[parameter_index as usize].rssi = 0;

                                                current_position = (current_position
                                                    + read_buffer[pos] as usize)
                                                    as usize;
                                            } else {
                                                break;
                                            }
                                            current_position = current_position + 1;
                                        }
                                        self.wifi_client.map(|client| {
                                            client.scan_done(Ok(&networks[0..param_len as usize]))
                                        });
                                    });

                                    Ok(())
                                }
                            }
                        } else {
                            Err(ErrorCode::INVAL)
                        }
                    } else if read_buffer[POS_CMD] == ERROR_CMD {
                        Err(ErrorCode::FAIL)
                    } else {
                        Ok(())
                    }
                } else {
                    Err(ErrorCode::INVAL)
                }
            })
    }

    fn schedule_callback_error(&self, command: Command, error: ErrorCode) {
        match command {
            Command::StartScanNetworksCmd | Command::ScanNetworksCmd => {
                self.wifi_client.map(|client| client.scan_done(Err(error)));
            }
            _ => {}
        }
    }
}

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'a>> SpiMasterClient for NinaW102<'a, S, P, A> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        _len: usize,
        status: Result<(), ErrorCode>,
    ) {
        if let Err(err) = status {
            match self.status.get() {
                Status::Send(command) | Status::Receive(command, _, _) => self.schedule_callback_error(command, err),
                _ => {}
            }
            self.write_buffer.replace(write_buffer);

            read_buffer.map(|buffer| self.one_byte_read_buffer.replace(buffer));

            self.status.set(Status::Idle);
        } else {
            match self.status.get() {
                Status::Send(command) => {
                    self.write_buffer.replace(write_buffer);
                    if let Err(error) = self.receive_command(command) {
                        self.schedule_callback_error(command, error);
                        self.status.set(Status::Idle);
                    }
                }
                Status::Receive(command, position, timeout) => {
                    self.status.set(Status::Idle);
                    self.write_buffer.replace(write_buffer);
                    read_buffer
                        .map_or(Err(ErrorCode::NOMEM), |buffer| {
                            let byte = buffer[0];

                            self.one_byte_read_buffer.replace(buffer);
                            if position == 0 {
                                if byte == START_CMD || byte == ERROR_CMD {
                                    self.read_buffer.map(|buffer| {
                                        buffer[0] = byte;
                                    });
                                    if byte == START_CMD {
                                        self.receive_byte(command, 1, 1000)
                                    } else {
                                        Ok(())
                                    }
                                } else if timeout > 0 {
                                    self.receive_byte(command, 0, timeout - 1)
                                } else {
                                    self.cs.set();
                                    Err(ErrorCode::NOACK)
                                }
                            } else {
                                self.read_buffer.map(|buffer| {
                                    buffer[position] = byte;
                                });
                                if byte == END_CMD {
                                    self.cs.set();
                                    self.spi.release_low();

                                    self.process_buffer(command)
                                } else if timeout > 0 {
                                    self.receive_byte(command, position + 1, timeout - 1)
                                } else {
                                    self.cs.set();
                                    Err(ErrorCode::SIZE)
                                }
                            }
                        })
                        .map_err(|error| {
                            self.schedule_callback_error(command, error);
                            self.status.set(Status::Idle);
                        }).ok();
                }
                Status::Idle => {
                    self.write_buffer.replace(write_buffer);
                    read_buffer.map(|read_buffer| self.one_byte_read_buffer.replace(read_buffer));
                }

                Status::ScanNetworks => {}

                _ => {}
            }
        }
    }
}
use kernel::hil::time::AlarmClient;

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'a>> AlarmClient for NinaW102<'a, S, P, A> {
    fn alarm(&self) {
        match self.status.get() {
            Status::Init(init_status) => match init_status {
                InitStatus::Starting => {
                    self.reset.set();
                    self.alarm
                        .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(750));

                    self.status.set(Status::Init(InitStatus::Initialized));
                }

                InitStatus::Initialized => {
                    self.gpio0.clear();
                    self.gpio0.make_input();
                    self.status.set(Status::Idle)
                }
            },

            Status::StartScanNetworks => {
                let _ = self.start_scan_networks();
            }
            Status::ScanNetworks => {
                let _ = self.scan_networks();
            }

            _ => {}
        }
    }
}

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'static>> Scanner<'static> for NinaW102<'static, S, P, A> {
    fn scan(&self) -> Result<(), ErrorCode> {
        self.start_scan_networks()
    }

    fn set_client(&self, client: &'static dyn ScannerClient) {
        self.wifi_client.set(client);
    }
}
