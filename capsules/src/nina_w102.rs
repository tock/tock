use core::cell::Cell;
use core::iter::Take;
use kernel::hil::gpio::Pin;
use kernel::hil::spi::{SpiMaster, SpiMasterClient};
use kernel::hil::time::{Alarm, ConvertTicks};
use kernel::hil::wifinina::{self, Psk, Ssid, Station, StationClient, StationStatus};
use kernel::hil::wifinina::{Network, Scanner, ScannerClient};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::ErrorCode;
use kernel::{debug, debug_flush_queue};

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
enum SockMode {
    TcpMode,
    UdpMode,
    TlsMode,
    UdpMulticastMode,
    TlsBearsslMode,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum TcpState {
    TCPStateClosed,
    TCPStateListen,
    TCPStateSynSent,
    TCPStateSynRcvd,
    TCPStateEstablished,
    TCPStateFinWait1,
    TCPStateFinWait2,
    TCPStateCloseWait,
    TCPStateClosing,
    TCPStateLastACK,
    TCPStateTimeWait,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum Command {
    SetNetCmd = 0x10,
    SetPassPhraseCmd = 0x11,
    GetConnStatusCmd = 0x20,
    GetIpAddressCmd = 0x21,
    GetMacAddressCmd = 0x22,
    ScanNetworksCmd = 0x27,
    StartTcpServer = 0x28,
    StartTcpClient = 0x2D,
    StopTcpClient = 0x2E,
    GetClientStateTCP = 0x2F,
    GetIdxRSSICmd = 0x32,
    StartScanNetworksCmd = 0x36,
    GetFwVersion = 0x37,
    SendUdpPacket = 0x39,
    SendPing = 0x3E,
    GetSocket = 0x3F,
    InsertDataBuf = 0x46,
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
    GetConnStatus,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ConnectionStatus {
    Idle = 0,
    NoSSIDAvail = 1,
    ScanCompleted = 2,
    Connected = 3,
    ConnectFailed = 4,
    ConnectionLost = 5,
    Disconnected = 6,
    NoShield = 255,
    NoConnection,
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
    station_status: Cell<StationStatus>,
    networks: TakeCell<'static, [Network]>,
    scanner_client: OptionalCell<&'a dyn wifinina::ScannerClient>,
    station_client: OptionalCell<&'a dyn wifinina::StationClient>,
    second_time: Cell<u8>,
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
            station_status: Cell::new(StationStatus::Disconnected),
            networks: TakeCell::new(networks),
            scanner_client: OptionalCell::empty(),
            station_client: OptionalCell::empty(),
            second_time: Cell::new(0),
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

    pub fn number_to_connection_status(&self, status: usize) -> ConnectionStatus {
        match status {
            0 => ConnectionStatus::Idle,
            1 => ConnectionStatus::NoSSIDAvail,
            2 => ConnectionStatus::ScanCompleted,
            3 => ConnectionStatus::Connected,
            4 => ConnectionStatus::ConnectFailed,
            5 => ConnectionStatus::ConnectionLost,
            6 => ConnectionStatus::Disconnected,
            255 => ConnectionStatus::NoShield,
            _ => ConnectionStatus::NoConnection,
        }
    }

    pub fn get_firmware_version(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetFwVersion, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn scan_networks(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle || self.status.get() == Status::ScanNetworks {
            self.send_command(Command::ScanNetworksCmd, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn start_scan_networks(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle || self.status.get() == Status::StartScanNetworks {
            self.send_command(Command::StartScanNetworksCmd, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn send_ping(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::SendPing, &[&[172, 20, 10, 7], &[128]])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn start_tcp_server(&self, socket: u8) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(
                Command::StartTcpServer,
                &[&[0x9, 0x56], &[socket], &[SockMode::UdpMode as u8]],
            )
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn start_tcp_client(&self, socket: u8) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            // open socket connection to 172.20.10.7:3000;
            // 192.168.100.175
            // &[175, 100, 168, 192],
            self.send_command(
                Command::StartTcpClient,
                &[
                    // &[191, 101, 164, 144],
                    &[172, 20, 10, 7],
                    &[0xff, 0x9b],
                    &[socket],
                    &[SockMode::UdpMode as u8],
                ],
            )
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn stop_tcp_client(&self, socket: u8) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::StopTcpClient, &[&[socket]])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn insert_data_buf(&self, socket: u8) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            // Inserting buffer "Buna!"
            self.send_command(
                Command::InsertDataBuf,
                &[&[socket], &[0x42, 0x75, 0x6e, 0x61, 0x21]],
            )
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_networks_rssi(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetIdxRSSICmd, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_connection_status(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetConnStatusCmd, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_ip_address(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetIpAddressCmd, &[&[0xff]])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_mac_address(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetMacAddressCmd, &[&[0xff]])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_socket(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetSocket, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn get_tcp_client_state(&self, socket: u8) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::GetClientStateTCP, &[&[socket]])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn set_network(&self) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::SetNetCmd, &[])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn set_passphrase(&self, ssid: &[u8], psk: &[u8]) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::SetPassPhraseCmd, &[ssid, psk])
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    pub fn send_udp_packet(&self, socket: u8) -> Result<(), ErrorCode> {
        if self.status.get() == Status::Idle {
            self.send_command(Command::SendUdpPacket, &[&[socket]])
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

    fn send_command<'b>(&self, command: Command, params: &'b [&'b [u8]]) -> Result<(), ErrorCode> {
        debug!("Send command");
        self.wait_for_chip_ready()?;

        self.wait_for_chip_select()?;

        self.write_buffer
            .take()
            .map_or(Err(ErrorCode::NOMEM), |buffer| {
                buffer[0] = START_CMD;
                buffer[POS_CMD] = (command as u8) & !REPLY_FLAG;
                buffer[POS_PARAM_LEN] = params.len() as u8;
                let mut position = 3;
                for param in params {
                    //let mut param_bytes_pos = 0;
                    if command == Command::InsertDataBuf {
                        buffer[position] = 0;
                        position = position + 1;
                        buffer[position] = param.len() as u8;
                    } else {
                        buffer[position] = param.len() as u8;
                    }
                    position = position + 1;
                    for byte in *param {
                        buffer[position] = *byte;
                        position = position + 1;
                    }

                    //position = position + 1;
                }
                buffer[position] = END_CMD;

                // Vedem daca e util sau nu ?
                // if params.len() != 0 {
                //     for i in ((4 - ((position + 1) % 4)) & 3)..0 {
                //         position = position + 1;
                //         buffer[position] = 0xff;
                //     }
                // }

                if command == Command::StartTcpServer {
                    position = position + 1;
                    buffer[position + 1];
                    for i in 0..position+1 {
                        debug!("{:x} ", buffer[i]);
                    }
                }
                if command == Command::StartTcpClient {
                    debug!("StartTcpClient: chars to be written {}", position + 1);
                    debug!(
                        "{:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
                        buffer[0],
                        buffer[1],
                        buffer[2],
                        buffer[3],
                        buffer[4],
                        buffer[5],
                        buffer[6],
                        buffer[7],
                        buffer[8],
                        buffer[9],
                        buffer[10],
                        buffer[11],
                        buffer[12],
                        buffer[13],
                        buffer[14],
                        buffer[15],
                    );
                }
                if command == Command::InsertDataBuf {
                    // for i in 0..position + 1 {
                    //     debug!("Byte {}: {:x}", i, buffer[i]);
                    // }
                    // debug!("InsertDataBuf: {}", position + 1);
                    while (position + 1) % 4 != 0 {
                        debug!("Aici");
                        buffer[position + 1] = 0xff;
                        position = position + 1;
                    }
                    debug!("InsertDataBuf: chars to be written {}", position + 1);
                    // debug!("{:?}", buffer);
                    debug!(
                        "{:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x} {:x}",
                        buffer[0],
                        buffer[1],
                        buffer[2],
                        buffer[3],
                        buffer[4],
                        buffer[5],
                        buffer[6],
                        buffer[7],
                        buffer[8],
                        buffer[9],
                        buffer[10],
                        buffer[11],
                        buffer[12],
                        buffer[13],
                    );
                }
                if command == Command::SendUdpPacket {
                    buffer[position + 1] = 0xff;
                    position = position + 1;
                    buffer[position + 1] = 0xff;
                    position = position + 1;
                    debug!("InsertDataBuf: chars to be written {}", position + 1);
                    // debug!("{:?}", buffer);
                    debug!(
                        "{:x} {:x} {:x} {:x} {:x} {:x}",
                        buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5]
                    );
                }

                self.spi.release_low();

                self.spi
                    .read_write_bytes(buffer, self.read_buffer.take(), (position + 1) as usize)
                    .map_err(|(err, write_buffer, read_buffer)| {
                        self.write_buffer.replace(write_buffer);
                        read_buffer.map(|buffer| self.read_buffer.replace(buffer));
                        err
                    })?;

                self.status.set(Status::Send(command));
                debug!("command sent: {:?}", self.status.get());

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

                self.one_byte_read_buffer
                    .take()
                    .map_or(Err(ErrorCode::NOMEM), move |read_buffer| {
                        self.status.set(Status::Receive(command, position, timeout));
                        self.spi.hold_low();
                        self.spi
                            .read_write_bytes(buffer, Some(read_buffer), 1)
                            .map_err(|(err, write_buffer, read_buffer)| {
                                self.write_buffer.replace(write_buffer);
                                read_buffer.map(|buffer| self.one_byte_read_buffer.replace(buffer));
                                err
                            })
                    })
            })
            .map_err(|err| {
                self.cs.set();
                err
            })
    }

    fn receive_command(&self, command: Command) -> Result<(), ErrorCode> {
        debug!("received command {:?}", command);
        self.wait_for_chip_ready()?;

        self.wait_for_chip_select()?;

        self.receive_byte(command, 0, 1000)
    }

    fn process_buffer(&self, command: Command) -> Result<(), ErrorCode> {
        debug!("Process buffer for command **** {:?} *****", command);
        // if command == Command::GetSocket {
        //     debug!("We get here in process buffer for Command::GetSocket!");
        // }
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
                                    let status =
                                        self.number_to_connection_status(read_buffer[4] as usize);
                                    if status == ConnectionStatus::Connected {
                                        if let StationStatus::Connecting(net) =
                                            self.station_status.get()
                                        {
                                            debug!("Getting here!");
                                            self.station_status.set(StationStatus::Connected(net));
                                            self.station_client.map(|client| {
                                                client
                                                    .command_complete(Ok(self.station_status.get()))
                                            });
                                            // self.get_ip_address();
                                            // self.start_tcp_client(0);
                                            // self.get_socket();
                                            self.status.set(Status::Send(Command::SendPing));
                                            self.alarm.set_alarm(
                                                self.alarm.now(),
                                                self.alarm.ticks_from_ms(10),
                                            );
                                            // self.send_ping();
                                        }
                                    } else if status == ConnectionStatus::ConnectFailed {
                                        if let StationStatus::Connecting(net) =
                                            self.station_status.get()
                                        {
                                            debug!("Getting here!");
                                            self.station_status.set(StationStatus::Disconnected);
                                            self.station_client.map(|client| {
                                                client
                                                    .command_complete(Ok(self.station_status.get()))
                                            });
                                        }
                                    } else {
                                        debug!("Getting here! {:?}", status);
                                        self.status.set(Status::GetConnStatus);
                                        self.alarm.set_alarm(
                                            self.alarm.now(),
                                            self.alarm.ticks_from_ms(2000),
                                        );
                                    }
                                    Ok(())
                                }
                                Command::StartScanNetworksCmd => {
                                    debug!("Starts scanning");
                                    self.status.set(Status::ScanNetworks);
                                    self.alarm.set_alarm(
                                        self.alarm.now(),
                                        self.alarm.ticks_from_ms(2000),
                                    );

                                    Ok(())
                                }
                                Command::ScanNetworksCmd => {
                                    debug!("Scan networks command");
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
                                        self.scanner_client.map(|client| {
                                            client.scan_done(Ok(&networks[0..param_len as usize]))
                                        });
                                    });
                                    Ok(())
                                }

                                Command::GetIdxRSSICmd => Ok(()),

                                Command::SetNetCmd => Ok(()),

                                Command::SetPassPhraseCmd => {
                                    // debug!("{}", end_pos);
                                    // debug!(
                                    //     "{:x} {:x} {:x} {:x} {:x} {:x}",
                                    //     read_buffer[0],
                                    //     read_buffer[1],
                                    //     read_buffer[2],
                                    //     read_buffer[3],
                                    //     read_buffer[4],
                                    //     read_buffer[5]
                                    // );

                                    if read_buffer[4] == 1 {
                                        debug!("SetPassPhraseCmd worked!");
                                    } else {
                                        debug!("SetPassPhraseCmd: error!");
                                    }

                                    self.status.set(Status::Send(Command::GetConnStatusCmd));
                                    self.alarm.set_alarm(
                                        self.alarm.now(),
                                        self.alarm.ticks_from_ms(2000),
                                    );
                                    // self.send_ping();
                                    Ok(())
                                }

                                Command::GetIpAddressCmd => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                            }
                                            debug!("Array nr {}: {:?}", count, buf);
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
                                    self.get_mac_address();
                                    // debug!(
                                    //     "Asta: IP Address {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                                    //     read_buffer[0],
                                    //     read_buffer[1],
                                    //     read_buffer[2],
                                    //     read_buffer[3],
                                    //     read_buffer[4],
                                    //     read_buffer[5],
                                    // );
                                    Ok(())
                                }

                                Command::GetMacAddressCmd => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                            }
                                            debug!(
                                                "MAC Address {:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
                                                buf[5], buf[4], buf[3], buf[2], buf[1], buf[0],
                                            );
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
                                    self.get_ip_address();
                                    Ok(())
                                }
                                Command::SendPing => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!("Num elements: {:x}", read_buffer[pos]);
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                            }
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
                                    self.status.set(Status::Send(Command::GetSocket));
                                    self.alarm
                                        .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(100));
                                    self.get_socket();
                                    Ok(())
                                }
                                Command::GetSocket => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "GetSocket: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            debug!("Socket num: {:x}", read_buffer[pos + 1]);
                                            // for i in 0..read_buffer[pos] {
                                            //     buf[i as usize] = read_buffer[pos + i as usize + 1];
                                            //     debug!("Element {}: {:x}", i, buf[i as usize]);
                                            // }
                                            self.status.set(Status::Send(Command::StartTcpServer));
                                            self.alarm.set_alarm(
                                                self.alarm.now(),
                                                self.alarm.ticks_from_ms(1000),
                                            );
                                            // self.start_tcp_client(read_buffer[pos + 1]);
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
                                    // self.get_socket();
                                    Ok(())
                                }
                                Command::StartTcpClient => {
                                    debug!(
                                        "Process buffer for Command::StartTcpClient {:?}",
                                        param_len
                                    );
                                    // if param_len == 0 {
                                    //     self.get_tcp_client_state(0);
                                    // }
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        // debug!("Dar Intru aici?");
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "StartTcpClient: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                            }
                                            // self.start_tcp_client(read_buffer[pos + 1]);
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                            debug!(
                                                "StartTcpClient end, status: {:?}",
                                                self.status.get()
                                            );
                                            // self.get_tcp_client_state(0);
                                        }
                                    }
                                    self.status.set(Status::Send(Command::SendUdpPacket));
                                    self.alarm
                                        .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(10));
                                    Ok(())
                                }
                                Command::StartTcpServer => {
                                    debug!(
                                        "Process buffer for Command::StartTcpServer {:?}",
                                        param_len
                                    );
                                    // if param_len == 0 {
                                    //     self.get_tcp_client_state(0);
                                    // }
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        // debug!("Dar Intru aici?");
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "StartTcpServer: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                            }
                                            // self.start_tcp_client(read_buffer[pos + 1]);
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                            debug!(
                                                "StartTcpServer end, status: {:?}",
                                                self.status.get()
                                            );
                                            // self.get_tcp_client_state(0);
                                        }
                                    }
                                    self.second_time.set(1);
                                    self.status.set(Status::Send(Command::StartTcpClient));
                                    self.alarm
                                        .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(100));
                                    Ok(())
                                }
                                Command::GetClientStateTCP => {
                                    debug!(
                                        "Process buffer for Command::GetClientStateTCP {:?}",
                                        param_len
                                    );
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        // debug!("Dar Intru aici?");
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "GetClientStateTCP: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                                match buf[i as usize] {
                                                    0 => {
                                                        debug!("{:?}", TcpState::TCPStateClosed);
                                                        self.status.set(Status::Send(
                                                            Command::GetClientStateTCP,
                                                        ));
                                                        self.alarm.set_alarm(
                                                            self.alarm.now(),
                                                            self.alarm.ticks_from_ms(1000),
                                                        )
                                                    }
                                                    1 => {
                                                        debug!("{:?}", TcpState::TCPStateListen)
                                                    }
                                                    2 => {
                                                        debug!("{:?}", TcpState::TCPStateSynSent)
                                                    }
                                                    3 => {
                                                        debug!("{:?}", TcpState::TCPStateSynRcvd)
                                                    }
                                                    4 => {
                                                        debug!(
                                                            "{:?}",
                                                            TcpState::TCPStateEstablished
                                                        );
                                                        self.insert_data_buf(0);
                                                        ()
                                                        // self.status.set(Status::Send(
                                                        //     Command::InsertDataBuf,
                                                        // ));
                                                        // self.alarm.set_alarm(
                                                        //     self.alarm.now(),
                                                        //     self.alarm.ticks_from_ms(2000),
                                                        // )
                                                    }
                                                    5 => {
                                                        debug!("{:?}", TcpState::TCPStateFinWait1)
                                                    }
                                                    6 => {
                                                        debug!("{:?}", TcpState::TCPStateFinWait2)
                                                    }
                                                    7 => {
                                                        debug!("{:?}", TcpState::TCPStateCloseWait)
                                                    }
                                                    8 => {
                                                        debug!("{:?}", TcpState::TCPStateClosing)
                                                    }
                                                    9 => {
                                                        debug!("{:?}", TcpState::TCPStateLastACK)
                                                    }
                                                    10 => {
                                                        debug!("{:?}", TcpState::TCPStateTimeWait)
                                                    }
                                                    _ => {
                                                        debug!("Other value here!");
                                                    }
                                                }
                                            }
                                            // self.start_tcp_client(read_buffer[pos + 1]);
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                            // self.status.set(Status::Send(Command::InsertDataBuf));
                                            // self.alarm.set_alarm(
                                            //     self.alarm.now(),
                                            //     self.alarm.ticks_from_ms(10),
                                            // );
                                        }
                                    }
                                    Ok(())
                                }
                                Command::InsertDataBuf => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "InsertDataBuf: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                            }
                                            self.status.set(Status::Send(Command::SendUdpPacket));
                                            self.alarm.set_alarm(
                                                self.alarm.now(),
                                                self.alarm.ticks_from_ms(10),
                                            );
                                            // self.send_udp_packet(0);
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
                                    Ok(())
                                }
                                Command::SendUdpPacket => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "SendUdpPacket: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                                // if buf[i as usize] == 0 {
                                                self.status
                                                    .set(Status::Send(Command::StartTcpClient));
                                                self.alarm.set_alarm(
                                                    self.alarm.now(),
                                                    self.alarm.ticks_from_ms(2000),
                                                )
                                                // }
                                            }
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
                                    Ok(())
                                }
                                Command::StopTcpClient => {
                                    let mut current_position = 0;
                                    let mut count = 0;
                                    for parameter_index in 0..param_len {
                                        count = count + 1;
                                        let pos = POS_PARAM + current_position;
                                        let mut buf: [u8; 20] = [0; 20];
                                        // debug!("buffer: {:?}", read_buffer);
                                        if pos < read_buffer.len() {
                                            debug!(
                                                "SendUdpPacket: Num elements: {:x}",
                                                read_buffer[pos]
                                            );
                                            for i in 0..read_buffer[pos] {
                                                buf[i as usize] = read_buffer[pos + i as usize + 1];
                                                debug!("Element {}: {:x}", i, buf[i as usize]);
                                                // self.status.set(Status::Send(Command::GetSocket));
                                                // self.alarm.set_alarm(
                                                //     self.alarm.now(),
                                                //     self.alarm.ticks_from_ms(100),
                                                // )
                                            }
                                            current_position = (current_position
                                                + read_buffer[pos] as usize)
                                                as usize;
                                            current_position = current_position + 1;
                                        }
                                    }
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
                    Err(ErrorCode::INVAL)
                }
            })
    }

    fn schedule_callback_error(&self, command: Command, error: ErrorCode) {
        match command {
            Command::StartScanNetworksCmd | Command::ScanNetworksCmd => {
                self.scanner_client
                    .map(|client| client.scan_done(Err(error)));
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
                Status::Send(command) | Status::Receive(command, _, _) => {
                    self.schedule_callback_error(command, err)
                }
                _ => {}
            }
            self.write_buffer.replace(write_buffer);

            // TO BE CHANGED??
            read_buffer.map(|buffer| self.one_byte_read_buffer.replace(buffer));

            self.status.set(Status::Idle);
        } else {
            match self.status.get() {
                Status::Send(command) => {
                    // if command == Command::GetConnStatusCmd {
                    //     debug!("In read_write_done in Status::send");
                    // }
                    self.write_buffer.replace(write_buffer);
                    read_buffer.map(|buffer| self.read_buffer.replace(buffer));
                    if let Err(error) = self.receive_command(command) {
                        self.schedule_callback_error(command, error);
                        self.status.set(Status::Idle);
                    }
                }
                Status::Receive(command, position, timeout) => {
                    // if command == Command::GetSocket {
                    //     debug!("In read_write_done in Status::send");
                    // }
                    self.status.set(Status::Idle);
                    self.write_buffer.replace(write_buffer);
                    // debug!("In status::Receive");
                    read_buffer
                        .map_or(Err(ErrorCode::NOMEM), |buffer| {
                            let byte = buffer[0];
                            // if command == Command::SendPing && byte != 0xff {
                            //     debug!(
                            //         "Aiciii: command: {:?}, byte {:x}, position {:x}",
                            //         command, byte, position
                            //     );
                            // }

                            self.one_byte_read_buffer.replace(buffer);
                            if position == 0 {
                                if byte == START_CMD || byte == ERROR_CMD {
                                    self.read_buffer.map(|buffer| {
                                        buffer[0] = byte;
                                    });
                                    if byte == START_CMD {
                                        if command == Command::GetSocket
                                            || command == Command::InsertDataBuf
                                        {
                                            // debug!("Intru aici la START_CMD");
                                        }
                                        self.receive_byte(command, 1, 1000)
                                    } else {
                                        if command == Command::GetSocket
                                            || command == Command::InsertDataBuf
                                        {
                                            debug!("Intru aici la err");
                                            // self.start_tcp_client();
                                        }
                                        Ok(())
                                    }
                                } else if timeout > 0 {
                                    // if (command == Command::GetClientStateTCP) {
                                    //     debug!("read byte {} {}", position, timeout);
                                    // }
                                    self.receive_byte(command, 0, timeout - 1)
                                } else {
                                    if command == Command::GetSocket
                                        || command == Command::InsertDataBuf
                                    {
                                        debug!("Timeout...");
                                        if command == Command::GetSocket {
                                            debug!("At InsertDataBuf");
                                            self.status.set(Status::Send(Command::GetSocket));
                                            self.alarm.set_alarm(
                                                self.alarm.now(),
                                                self.alarm.ticks_from_ms(100),
                                            );
                                        }
                                        // }
                                        // self.start_tcp_client();
                                        Ok(())
                                    } else {
                                        self.cs.set();
                                        Err(ErrorCode::NOACK)
                                    }
                                }
                            } else {
                                self.read_buffer.map(|buffer| {
                                    buffer[position] = byte;
                                    // if command == Command::GetIpAddressCmd {
                                    //     debug!(
                                    //         "command: {:?}, byte {:x}, position {:x}",
                                    //         command, buffer[position], position
                                    //     );
                                    // }
                                });
                                if byte == END_CMD {
                                    self.cs.set();
                                    self.spi.release_low();
                                    if command == Command::GetSocket
                                        || command == Command::InsertDataBuf
                                    {
                                        // debug!("LA end!!");
                                    }

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
                        })
                        .ok();
                }
                Status::Idle => {
                    self.write_buffer.replace(write_buffer);
                    read_buffer.map(|read_buffer| self.read_buffer.replace(read_buffer));
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
                    self.status.set(Status::Idle);
                }
            },

            // Status::Idle => {
            //     let _ = self.start_tcp_client(0);
            // }
            Status::StartScanNetworks => {
                let _ = self.start_scan_networks();
            }
            Status::ScanNetworks => {
                // debug!("ScanNetworks status from alarm");
                let _ = self.scan_networks();
            }

            Status::GetConnStatus => {
                // debug!("Status get conn");
                self.status.set(Status::Idle);
                let _ = self.get_connection_status();
            }

            Status::Send(command) => match command {
                Command::GetConnStatusCmd => {
                    self.status.set(Status::Idle);
                    self.get_connection_status();
                }
                Command::GetSocket => {
                    self.status.set(Status::Idle);
                    self.get_socket();
                }
                Command::StartTcpServer => {
                    self.status.set(Status::Idle);
                    self.start_tcp_server(0);
                }
                Command::StartTcpClient => {
                    self.status.set(Status::Idle);
                    self.start_tcp_client(0);
                }
                Command::StopTcpClient => {
                    self.status.set(Status::Idle);
                    self.stop_tcp_client(0);
                }
                Command::GetClientStateTCP => {
                    self.status.set(Status::Idle);
                    self.get_tcp_client_state(0);
                }
                Command::InsertDataBuf => {
                    self.status.set(Status::Idle);
                    self.insert_data_buf(0);
                }
                Command::SendUdpPacket => {
                    self.status.set(Status::Idle);
                    self.send_udp_packet(0);
                }
                Command::SendPing => {
                    self.status.set(Status::Idle);
                    self.send_ping();
                }
                _ => {
                    self.status.set(Status::Idle);
                }
            },

            _ => {}
        }
    }
}

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'static>> Scanner<'static> for NinaW102<'static, S, P, A> {
    fn scan(&self) -> Result<(), ErrorCode> {
        debug!("Nina starts scanning");
        self.start_scan_networks()
    }

    fn set_client(&self, client: &'static dyn ScannerClient) {
        self.scanner_client.set(client);
    }
}

impl<'a, S: SpiMaster, P: Pin, A: Alarm<'static>> Station<'static> for NinaW102<'static, S, P, A> {
    // try to initiatie a connection to the `Network`
    fn connect(&self, ssid: Ssid, psk: Option<Psk>) -> Result<(), ErrorCode> {
        //if let Some(psk) = psk.unwrap() {}
        self.station_status.set(StationStatus::Connecting(Network {
            ssid: ssid,
            rssi: 0,
            security: None,
        }));
        self.set_passphrase(
            &ssid.value[0..ssid.value.len()],
            &psk.unwrap().value[0..psk.unwrap().value.len()],
        )
    }
    // try to disconnect from the network that it is currently connected to
    fn disconnect(&self) -> Result<(), ErrorCode> {
        Err(ErrorCode::NOSUPPORT)
    }

    // return the status
    fn get_status(&self) -> Result<(), ErrorCode> {
        self.get_connection_status()
    }
    fn set_client(&self, client: &'static dyn StationClient) {
        self.station_client.set(client);
    }
}
