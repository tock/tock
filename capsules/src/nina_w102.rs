use core::cell::Cell;
use core::iter::Take;
use core::ptr::read;

use kernel::debug;
use kernel::hil::gpio::Pin;
use kernel::hil::spi::{SpiMaster, SpiMasterClient};
use kernel::hil::time::{Alarm, ConvertTicks};
use kernel::syscall::SyscallDriver;
use kernel::utilities::cells::{TakeCell, OptionalCell};
use kernel::ErrorCode;
use kernel::hil::wifi::{Network, Scanner, ScannerClient};
use kernel::hil::wifi;


const START_CMD: u8 = 0xe0;
const END_CMD: u8 = 0xee;
const ERROR_CMD: u8 = 0xef;

const POS_CMD: usize = 1;
const POS_PARAM_LEN: usize = 2;
const POS_LEN: usize = 2;
const POS_PARAM: usize = 3;

const CMD_FLAG: u8 = 0;
const REPLY_FLAG: u8 = 1 << 7;
const DATA_FLAG: u8 = 1 << 6;

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
    GetFirmware,
    StartScanNetworks,
    ScanNetworks,
    GetConnStatus,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum ConnectionStatus {
    Idle,
    NoSSIDAvailable,
    ScanCompleted,
    Connected,
    ConnectFailed,
    ConnectionLost,
    Disconnected,
    NoShield,
    Unknown,
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
    wifi_client:OptionalCell<&'a dyn wifi::ScannerClient>,
   
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
        debug!("set alarm");
        self.alarm
            .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(10));
        Ok(())
    }

    pub fn get_firmware_version(&self) -> Result<(), ErrorCode> {
        //panic!("Get Firmware");
        if self.status.get() == Status::Idle {
            // while self.ready.read() {}
            // debug!("Iese din while");
            self.send_command(Command::GetFwVersion, 0);
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn scan_networks(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle || self.status.get() == Status::ScanNetworks {
            // debug!("Iese din while");
            self.send_command(Command::ScanNetworksCmd, 0);
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn start_scan_networks(&self) -> Result<(), ErrorCode> {// while self.ready.read() {}
            // debug!("Iese din while");
            if self.status.get() == Status::Idle || self.status.get() == Status::StartScanNetworks{
            self.send_command(Command::StartScanNetworksCmd, 0)
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_connection_status(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            // while self.ready.read() {}
            // debug!("Iese din while");
            self.send_command(Command::GetConnStatusCmd, 0);
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn wait_for_chip_ready(&self) -> Result<(), ErrorCode> {
        for i in 0..100000000 {
            if !self.ready.read() {
                return Ok(());
            }
        }
        panic!("chip not ready");
        Err(ErrorCode::BUSY)
    }

    fn wait_for_chip_select(&self) -> Result<(), ErrorCode> {
        self.cs.clear();
        for i in 0..100000 {
            if self.ready.read() {
                return Ok(());
            }
        }
        panic!("chip not select");
        self.cs.set();
        Err(ErrorCode::NOACK)
    }

    fn send_command(&self, command: Command, num_params: u8) -> Result<(), ErrorCode> {
        // should be async
        self.wait_for_chip_ready()?;
        //panic!("Chip not ready!");

        self.wait_for_chip_select()?;
        //panic!("Chip not selected!");
        /*
        if let Err(err) = self.wait_for_chip_ready() {
            return Err(err);
        }
         */
        // panic!("is ready");
        
        // let buffer_wrapper = self.write_buffer.take();
        // if let Some(buffer) = buffer_wrapper {
        //     ...
        //     Ok(())
        // }
        // else
        // {
        //     Err(ErrorCode::NOMEM)
        // }
        
        self.write_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |buffer| {
                buffer[0] = START_CMD;
                buffer[POS_CMD] = (command as u8) & !REPLY_FLAG;
                buffer[POS_PARAM_LEN] = num_params;
                // send parameters
                buffer[3] = END_CMD;
                debug!("{:?}", &buffer[0..4]);

                // while !self.ready.read() {}

                self.spi.release_low();

                // let ret = self.spi
                // .read_write_bytes(buffer, self.read_buffer.take(), 4);
                // if let Err((err, write_buffer, read_buffer)) = ret {
                //     err;   
                // }
                // else
                // {
                //     Ok(())
                // }?;

                self.spi
                    .read_write_bytes(buffer, self.read_buffer.take(), 4)
                    .map_err(|(err, write_buffer, read_buffer)| {
                        self.write_buffer.replace(write_buffer);
                        read_buffer.map(|buffer| self.read_buffer.replace(buffer));
                        panic!("{:?}", err);
                        err
                    })?;

                //panic!("Read write bytes");
                //self.status.set(Status::Send(Command::GetFwVersion));
                self.status.set(Status::Send(command));
                //panic!("send command {:?}", command);

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
                // self.cs.clear();
                // while !self.ready.read() {}
                // if self.wait_for_chip_ready() {
                self.one_byte_read_buffer.take().map_or(
                    Err(ErrorCode::NOMEM),
                    move |read_buffer| {
                        self.status.set(Status::Receive(command, position, timeout));
                        self.spi.hold_low();
                        self.spi
                            .read_write_bytes(
                                buffer,
                                Some(read_buffer), //TODO put the read buffer back
                                1,
                            )
                            .map_err(|(err, write_buffer, read_buffer)| {
                                self.write_buffer.replace(write_buffer);
                                read_buffer.map(|buffer| self.one_byte_read_buffer.replace(buffer));
                                panic!("err {:?}", err);
                                err
                            })
                    },
                )?;
                //self.status.set(Status::Receive(Command::GetFwVersion));
                // debug!("Actual status in receive_command{:?}", self.status.get());
                // } else {
                //     self.status.set(Status::Idle)
                // }
                Ok(())
            })
            .map_err(|err| {
                self.cs.set();
                err
            })
    }

    fn receive_command(&self, command: Command) -> Result<(), ErrorCode> {
        // // should be async
        //while self.ready.read() {}
        debug!("receive command");
        // debug!("Intra pe receive_command");
        self.wait_for_chip_ready()?;
        // panic! ("chip ready");
        self.wait_for_chip_select()?;

        self.receive_byte(command, 0, 1000)
    }

    fn process_buffer(&self, command: Command) -> Result<(), ErrorCode> {
        debug!("Intra in process buffer");
        self.read_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |read_buffer| {
                if read_buffer[0] == START_CMD {
                    debug!("E start");
                    debug!("byte {}", read_buffer[POS_CMD]);
                    if read_buffer[POS_CMD] == (command as u8) | REPLY_FLAG {
                        debug!("Pachetul e ok");
                        /*  debug!(
                            "{:?}",
                            core::str::from_utf8(&read_buffer[index + 2..index + 4])
                        );*/
                        let param_len = read_buffer[POS_LEN]; //comanda start scan networks are si param len

                        // debug!("params {}", param_len);

                        let mut current_position = 0;
                        for parameter_index in 0..param_len {
                            let pos = POS_PARAM + current_position;
                            // debug!("params position {}", param_len);
                            // debug!("Schimba currebnt pos");
                            if pos < read_buffer.len() {
                                current_position =
                                    (current_position + read_buffer[pos] as usize) as usize;
                                // debug!("Schimba currebnt pos");
                            } else {
                                break;
                            }
                            current_position = current_position + 1;
                        }

                        // debug!("Iese din for");
                        let end_pos = POS_PARAM + current_position;

                        // debug!("End pos este {:?} ", end_pos);
                        // debug!("read_buffer[end_pos] este {:?}", read_buffer[end_pos]);
                        if end_pos < read_buffer.len() && read_buffer[end_pos] == END_CMD {
                            // ok
                            // debug!("A gasit end cmd");
                            match command {
                                Command::GetFwVersion => {
                                    debug!("{:?}", core::str::from_utf8(&read_buffer[4..10]));
                                    self.read_buffer.replace(read_buffer);
                                    self.get_connection_status()
                                }
                                Command::GetConnStatusCmd => {
                                    // debug!("{:?}", core::str::from_utf8(&read_buffer[4..10]));
                                    self.read_buffer.replace(read_buffer);
                                    self.start_scan_networks()
                                }
                                Command::StartScanNetworksCmd => {
                                    // debug!("{:?}", core::str::from_utf8(&read_buffer[4..10]));
                                    self.read_buffer.replace(read_buffer);
                                    self.status.set(Status::ScanNetworks);
                                    self.alarm.set_alarm(
                                        self.alarm.now(),
                                        self.alarm.ticks_from_ms(2000),
                                    );

                                    Ok(())
                                }
                                Command::ScanNetworksCmd => {
                                    // debug!("{:?}", &read_buffer[0..end_pos+1]);
                                 
                                            self.networks.map(|networks|{
                                                let mut current_position = 0;
                                                for parameter_index in 0..param_len {
                                                    let pos = POS_PARAM + current_position;
                                                    // debug!("params position {}", param_len);
                                                    // debug!("Schimba current pos");
                                                    
                                                    if pos < read_buffer.len() {
                                                        // for
                                                        for (d, s) in networks[parameter_index as usize].ssid.value.iter_mut().zip(read_buffer[pos+1..pos+(read_buffer[pos] as usize)+1].iter()){
                                                        //networks[parameter_index as usize].ssid.value = read_buffer[pos+1..pos+(read_buffer[pos] as usize)+1] as [u8];
                                                                *d = *s
                                                        }
                                                        networks[parameter_index as usize].ssid.len =  read_buffer[pos];
                                                        networks[parameter_index as usize].security = None;
                                                        networks[parameter_index as usize].rssi = 0;

                                                        current_position = (current_position
                                                            + read_buffer[pos] as usize)
                                                            as usize;
                                                        // debug!("Schimba current pos");
                                                    } else {
                                                        break;
                                                    }
                                                    current_position = current_position + 1;
                                                }
                                            });
                                            
                                    self.read_buffer.replace(read_buffer);
                                    // self.get_connection_status()
                                    Ok(())
                                }
                                _ => Ok(()),
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
                    debug!("Nu e start");
                            Err(ErrorCode::INVAL)
                }
            })
    }

    fn schedule_callback_error (&self, command: Command, error: ErrorCode) {
        match command {
            Command::StartScanNetworksCmd | Command::ScanNetworksCmd => {
                self.wifi_client.map(|client| client.scan_done(Err(error)));
            }
            _ => {

            }
        }
    } 
}

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'a>> SpiMasterClient for NinaW102<'a, S, P, A> {
    fn read_write_done(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
        status: Result<(), ErrorCode>,
    ) {
        // if status Ok(()) 
        //panic!("sent command {:?}", self.status.get());
        if let Err(err) = status {
            match self.status.get() {
                Status::Send(command) => {
                    self.schedule_callback_error(command, err)
                }
                Status::Receive(command, position, timeout) => {
                    self.schedule_callback_error(command, err)
                }
                _ => {
                    //panic!("error for {:?}", command);
                }
            }
            self.write_buffer.replace(write_buffer);

            if let Some(buffer) = read_buffer {
            self.read_buffer.replace(buffer);
            }

            self.status.set (Status::Idle);
            // put buffers back
        }
        else
        {
            match self.status.get() {
                Status::Send(command) => {
                    debug!("sent command {:?}", command);

                    self.write_buffer.replace(write_buffer);
                    read_buffer.map(|buffer| self.read_buffer.replace(buffer));
                    if let Err(error) = self.receive_command(command) {
                        self.schedule_callback_error(command, error);
                        self.status.set (Status::Idle);
                    }
                }
                Status::Receive(command, position, timeout) => {
                    // debug!("received command {:?} {}", command, position);
                    self.status.set(Status::Idle);
                    self.write_buffer.replace(write_buffer);
                    read_buffer.map_or(Err(ErrorCode::NOMEM), |buffer| {
                        let byte = buffer[0];
                        // debug!("position {} byte {} timeout {}", position, byte, timeout);
                        self.one_byte_read_buffer.replace(buffer);
                        if position == 0 {
                            if byte == START_CMD || byte == ERROR_CMD {
                                debug!("byte {} timeout {}", byte, timeout);
                                self.read_buffer.map(|buffer| {
                                    buffer[0] = byte;
                                });
                                if byte == START_CMD {
                                    // TODO replace 100 with self.read_buffer.len() (use map)
                                    self.receive_byte(command, 1, 1000)
                                } else {
                                    debug!("process error buffer");
                                    Ok(())
                                }
                            } else if timeout > 0 {
                                self.receive_byte(command, 0, timeout - 1)
                            } else {
                                // stop spi
                                self.cs.set();
                                Err(ErrorCode::NOACK)
                            }
                        } else {
                            self.read_buffer.map(|buffer| {
                                buffer[position] = byte;
                            });
                            if byte == END_CMD {
                                // stop spi
                                self.cs.set();
                                self.spi.release_low();
                                debug!("process the buffer");
                                self.process_buffer(command)
                                //self.cs.set()
                            } else if timeout > 0 {
                                self.receive_byte(command, position + 1, timeout - 1)
                            } else {
                                // stop spi
                                self.cs.set();
                                Err(ErrorCode::SIZE)
                                //TODO ERR
                                //self.cs.set()
                            }
                        }
                    }).map_err(|error| {
                        self.schedule_callback_error(command, error);
                    });

                
                }
                Status::Idle => {
                    self.write_buffer.replace(write_buffer);
                    read_buffer.map(|read_buffer| self.read_buffer.replace(read_buffer));
                }

                Status::GetFirmware => {
                    debug!("status get firmware");
                }

                Status::ScanNetworks => {
                    debug!("status scan networks");
                }

                _ => {}
            }
        
            // else
            // {
            //     // pui buffere inapoi
            //     match self.status {
            //         Status::StartScanNetworks | Status::ScanNetworks => self.wifi_client.map(|client| client.scan_done(None, status)),
            //         _ => {
            //             panic! ("should not get here");
            //         }
            //     }
                
            //}
        }
    }
}
use kernel::hil::time::AlarmClient;


impl<'a, S: SpiMaster, P: Pin, A: Alarm<'a>> AlarmClient for NinaW102<'a, S, P, A> {
    fn alarm(&self) {
        debug!("alarm {:?}", self.status.get());
        match self.status.get() {
            Status::Init(init_status) => match init_status {
                InitStatus::Starting => {
                    debug!("alarm status starting");
                    self.reset.set();
                    self.alarm
                        .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(750));

                    self.status.set(Status::Init(InitStatus::Initialized));
                    debug!("{:?}", self.status.get());
                }

                InitStatus::Initialized => {
                    //panic!("Alarm status initialized");
                    self.gpio0.clear();
                    self.gpio0.make_input();
                    self.status.set(Status::Idle)
                    //panic!("{:?}", self.status.get());
                    //self.get_firmware_version();
                    //    self.get_connection_status();
                    //  self.start_scan_networks();
                }
            }
          Status::GetFirmware => {
                // debug!("status get firmware");
            }

            Status::StartScanNetworks => {
                self.start_scan_networks();  
            }     
                Status::ScanNetworks => {
                // debug!("status scan networks");
                self.scan_networks();
            }

            Status::GetConnStatus => {
                // debug!("status get conn status");
                // self.get_connection_status();
            }

            _ => {
                panic!("Alarm not starting");
            }
        }
    }
}





impl <'a,S:SpiMaster, P: Pin, A:Alarm<'static>> Scanner <'static> for NinaW102<'static, S, P, A>  {
    fn scan(&self) -> Result<(), ErrorCode>{
        //self.networks.replace(self.networks.take());
        self.start_scan_networks()
       //Ok((0))
    }
    

    fn set_client(&self, client: &'static dyn ScannerClient) {
        self.wifi_client.replace(client);
    }
}



