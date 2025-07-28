// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! CYW43439 PIO SPI support

use crate::gpio::{RPGpio, RPGpioPin};
use crate::pio::{Pio, PioSmClient, SMNumber, StateMachineConfiguration};
use core::cell::Cell;
use core::ffi::CStr;
use enum_primitive::cast::FromPrimitive;
use ioctl::{Ioctl, IoctlCommand};
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::ethernet::{EthernetAdapterDatapath, EthernetAdapterDatapathClient};
use kernel::hil::time::AlarmClient;
use kernel::hil::wifi_cyw43::{
    AccessPoint, AccessPointClient, Passphrase, Scanner, ScannerClient, Security, Ssid, Station,
    StationClient, WifiCtrl, WifiCtrlClient, PS_SIZE,
};
use kernel::hil::{gpio::Output as _, time::ConvertTicks};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::{hil::time::Alarm, ErrorCode};
use packets::{
    BdcHeader, BssInfo, CdcHeader, CountryInfo, DownloadHeader, Event, EventPacket, ScanResults,
    SdpcmHeader,
};
use utils::{as_bytes, slice8_mut, ChannelType};
use utils::{Access, Command, Function};
use utils::{Cyw43Cmd, NVRAM};

pub mod ioctl;
pub mod packets;
pub mod utils;

/// The CYW4343 SPI PIO program
const PROG: [u16; 11] = [
    0x6020, //  0: out    x, 32           side 0
    0x6040, //  1: out    y, 32           side 0
    0xe081, //  2: set    pindirs, 1      side 0
    //     .wrap_target
    0x6001, //  3: out    pins, 1         side 0
    0x1043, //  4: jmp    x--, 3          side 1
    0xe080, //  5: set    pindirs, 0      side 0
    0xa042, //  6: nop                    side 0
    0x5001, //  7: in     pins, 1         side 1
    0x0087, //  8: jmp    y--, 7          side 0
    0x20a0, //  9: wait   1 pin, 0        side 0
    0xc000, // 10: irq    nowait 0        side 0
            //     .wrap
];

/// gSPI word length. The reset value is 16.
enum WordLength {
    _16Bit = 0,
    _32Bit = 1,
}

/// Device cores that can be controlled through gSPI
#[derive(Clone, Copy, Debug)]
enum Core {
    WlanArm,
    SocRam,
}

impl Core {
    const fn base_addr(&self) -> u32 {
        match self {
            Core::WlanArm => 0x18103000,
            Core::SocRam => 0x18104000,
        }
    }
}

/// The CYW43439 PIO peripheral driver
pub struct PioCyw43439<'a, A: Alarm<'a>> {
    // PIO configuration fields
    pio: &'a Pio,
    clock_pin: u32,
    dio_pin: u32,
    pwr_pin: u32,
    cs_pin: RPGpioPin<'a>,
    sm_number: SMNumber,

    /// Alarm is needed because some configuring steps need
    /// a small delay between them.
    alarm: &'a A,

    /// Internal buffer for building packets sent to the chip
    buffer: TakeCell<'a, [u32; 513]>,

    // CYW43439 fields:
    /// The backplane window address for the current rx/tx operation
    backplane_window: Cell<u32>,
    /// Rx/tx sequence number, part of the SDPCM header
    sdpcm_seq: Cell<u8>,
    /// Unique id for rx/tx pair, part of the CDC header
    id: Cell<u16>,
    /// Last status received from the chip
    status: Cell<u32>,

    // Ethernet TAP HIL fields:
    /// Transmission identifier, length and buffer
    tx: OptionalCell<(usize, &'static mut [u8], u16)>,
    /// Ethernet TAP client
    ethernet_client: OptionalCell<&'a dyn EthernetAdapterDatapathClient>,
    /// Ethernet receive enable
    receive_en: Cell<bool>,

    // WiFi HIL clients
    scan_client: OptionalCell<&'a dyn ScannerClient>,
    sta_client: OptionalCell<&'a dyn StationClient>,
    ap_client: OptionalCell<&'a dyn AccessPointClient>,
    ctrl_client: OptionalCell<&'a dyn WifiCtrlClient>,

    /// Current state
    state: Cell<State>,
    scanning: Cell<bool>,

    /// Current pending IOCTL operation. Needed for when we perform
    /// a `get` ioctl and we retrieve the response in the interrupt
    /// handler
    pending_ioctl: OptionalCell<&'static CStr>,

    deferred_call: DeferredCall,

    // Infineon blobs
    clm: &'a [u8],
    fw: &'a [u8],
}

impl<'a, A: Alarm<'a>> PioCyw43439<'a, A> {
    /// Create a new `PioCyw43Spi` instance
    pub fn new(
        pio: &'a Pio,
        alarm: &'a A,
        clock_pin: u32,
        dio_pin: u32,
        pwr_pin: u32,
        cs_pin: RPGpioPin<'a>,
        sm_number: SMNumber,
        clm: &'a [u8],
        fw: &'a [u8],
    ) -> Self {
        Self {
            pio,
            alarm,
            clock_pin,
            dio_pin,
            pwr_pin,
            cs_pin,
            sm_number,
            backplane_window: Cell::new(0xAAAA_AAAA),
            sdpcm_seq: Cell::new(0),
            id: Cell::new(0),
            buffer: TakeCell::empty(),
            pending_ioctl: OptionalCell::empty(),
            state: Cell::new(State::Init(InitState::PoweredDown)),
            scanning: Cell::new(false),
            status: Cell::new(0),
            tx: OptionalCell::empty(),
            receive_en: Cell::new(false),
            ethernet_client: OptionalCell::empty(),
            scan_client: OptionalCell::empty(),
            sta_client: OptionalCell::empty(),
            ap_client: OptionalCell::empty(),
            ctrl_client: OptionalCell::empty(),
            deferred_call: DeferredCall::new(),
            clm,
            fw,
        }
    }

    /// Return SM number
    pub fn sm_number(&self) -> SMNumber {
        self.sm_number
    }

    pub fn set_buffer(&self, buffer: &'a mut [u32; 513]) {
        self.buffer.put(Some(buffer));
    }

    /// Configure the PIO peripheral as an SPI device to communicate with the CYW43439 chip
    pub fn init(&self) {
        self.pio.init();
        self.cs_pin.clear();

        // load program
        self.pio.add_program16(None::<usize>, &PROG).unwrap();

        let config = StateMachineConfiguration {
            out_pins_count: 1,
            out_pins_base: self.dio_pin,
            set_pins_count: 1,
            set_pins_base: self.dio_pin,
            in_pins_base: self.dio_pin,
            side_set_base: self.clock_pin,
            side_set_bit_count: 1,
            in_push_threshold: 0,
            out_pull_threshold: 0,
            div_int: 2u32,
            div_frac: 0u32,
            wrap: 10,
            wrap_to: 3,
            in_autopush: true,
            out_autopull: true,
            in_shift_direction_right: false,
            out_shift_direction_right: false,
            ..Default::default()
        };

        self.pio
            .cyw43_spi_program_init(self.sm_number, self.clock_pin, self.dio_pin, &config);

        self.pio
            .set_irq_source(0, crate::pio::InterruptSources::Interrupt0, true);
    }

    fn pwr_on(&self) {
        let pwr_gpio = RPGpioPin::new(RPGpio::from_u32(self.pwr_pin).unwrap());
        pwr_gpio.set();
    }

    fn pwr_down(&self) {
        let pwr_gpio = RPGpioPin::new(RPGpio::from_u32(self.pwr_pin).unwrap());
        pwr_gpio.clear();
    }
}

#[derive(Clone, Copy, Debug)]
enum State {
    Init(InitState),
    Station(StationState),
    AccessPoint(APState),
}

#[derive(Clone, Copy, Debug)]
enum StationState {
    Config {
        sec_type: Security,
        ssid: Ssid,
        passphrase: Passphrase,
    },
    Joining,
    Joined,
    NotConnected,
}

#[derive(Clone, Copy, Debug)]
enum APState {
    StartingWpa(Passphrase),
    StartingOpen,
    Started,
}

#[derive(Clone, Copy, Debug)]
enum InitState {
    PoweredDown,
    PoweredUp,

    DisableDevice(Core),
    ResetDevice(Core),
    PostResetDevice(Core),

    BusGlommOff,
    AmpduWindowSize,
    AmpduMpdus,
    EventMask,
    WifiUp,
    SetBand,
    GetMacAddress,
}

impl<'a, A: Alarm<'a>> Scanner<'a> for PioCyw43439<'a, A> {
    fn set_client(&self, client: &'a dyn kernel::hil::wifi_cyw43::ScannerClient) {
        self.scan_client.set(client);
    }

    fn start_scan(&self) -> Result<(), ErrorCode> {
        const SCANTYPE_PASSIVE: u8 = 1;
        if let State::Init(_) = self.state.get() {
            return Err(ErrorCode::BUSY);
        }

        if self.scanning.get() {
            return Err(ErrorCode::ALREADY);
        }

        let scan_params = packets::ScanParams {
            version: 1,
            action: utils::WL_SCAN_ACTION_START,
            sync_id: 1,
            ssid_len: 0,
            ssid: [0; 32],
            bssid: [0xff; 6],
            bss_type: 2,
            scan_type: SCANTYPE_PASSIVE,
            nprobes: !0,
            active_time: !0,
            passive_time: !0,
            home_time: !0,
            channel_num: 0,
            channel_list: [0; 1],
        };

        self.set_iovar(c"escan", as_bytes(&scan_params));
        self.scanning.set(true);

        Ok(())
    }

    fn stop_scan(&self) -> Result<(), ErrorCode> {
        if let State::Init(_) = self.state.get() {
            return Err(ErrorCode::BUSY);
        }

        if !self.scanning.get() {
            return Err(ErrorCode::ALREADY);
        }

        let scan_params = packets::ScanParams {
            version: 1,
            action: utils::WL_SCAN_ACTION_ABORT,
            ..Default::default()
        };

        self.set_iovar(c"escan", as_bytes(&scan_params));
        self.scanning.set(false);

        Ok(())
    }
}

impl<'a, A: Alarm<'a>> Station<'a> for PioCyw43439<'a, A> {
    fn set_client(&self, client: &'a dyn kernel::hil::wifi_cyw43::StationClient) {
        self.sta_client.set(client);
    }

    fn leave(&self) -> Result<(), ErrorCode> {
        if let State::Station(StationState::Joined) = self.state.get() {
            self.ioctl(Ioctl::Set, IoctlCommand::Disassoc, 0, &[]);
            self.state.set(State::Station(StationState::NotConnected));
            Ok(())
        } else {
            Err(ErrorCode::FAIL)
        }
    }

    fn join(&self, ssid: Ssid, security: Option<(Security, Passphrase)>) -> Result<(), ErrorCode> {
        if let State::Station(StationState::NotConnected) = self.state.get() {
            self.set_iovar_u32(c"ampdu_ba_wsize", 8, None);

            match security {
                None => {
                    self.ioctl(Ioctl::Set, IoctlCommand::SetWsec, 0, &0u32.to_le_bytes());
                    self.set_iovar_u32(c"bsscfg:sup_wpa", 0, Some(0));
                    self.ioctl(Ioctl::Set, IoctlCommand::SetInfra, 0, &1u32.to_le_bytes());
                    self.ioctl(Ioctl::Set, IoctlCommand::SetAuth, 0, &0u32.to_le_bytes());
                    self.ioctl(Ioctl::Set, IoctlCommand::SetWpaAuth, 0, &0u32.to_le_bytes());

                    self.set_ssid(ssid);
                    self.state.set(State::Station(StationState::Joining));

                    Ok(())
                }
                Some((sec_type, passphrase)) => {
                    const WSEC_AES: u32 = 4;
                    // enable AES encryption
                    self.ioctl(
                        Ioctl::Set,
                        IoctlCommand::SetWsec,
                        0,
                        &WSEC_AES.to_le_bytes(),
                    );

                    self.set_iovar_u32(c"bsscfg:sup_wpa", 0, Some(1));
                    self.set_iovar_u32(c"bsscfg:sup_wpa2_eapver", 0, Some(0xFFFF_FFFF));
                    self.set_iovar_u32(c"bsscfg:sup_wpa_tmo", 0, Some(2500));

                    // Set the current state and fire the alarm
                    self.state.set(State::Station(StationState::Config {
                        sec_type,
                        ssid,
                        passphrase,
                    }));
                    let now = self.alarm.now();
                    self.alarm.set_alarm(now, self.alarm.ticks_from_ms(110));

                    Ok(())
                }
            }
        } else {
            Err(ErrorCode::FAIL)
        }
    }
}

impl<'a, A: Alarm<'a>> AccessPoint<'a> for PioCyw43439<'a, A> {
    fn set_client(&self, client: &'a dyn kernel::hil::wifi_cyw43::AccessPointClient) {
        self.ap_client.set(client);
    }

    fn start_ap(
        &self,
        ssid: Ssid,
        security: Option<(Security, Passphrase)>,
        channel: u8,
    ) -> Result<(), ErrorCode> {
        if let State::Station(StationState::NotConnected) = self.state.get() {
            if let Some((ref security, ref passphrase)) = security {
                if passphrase.len < 8 || passphrase.len > 64 {
                    return Err(ErrorCode::SIZE);
                }
                if let Security::Wpa | Security::Wpa3 | Security::Wpa2Wpa3 = security {
                    return Err(ErrorCode::NOSUPPORT);
                }
            }

            self.ioctl(Ioctl::Set, IoctlCommand::Down, 0, &[]);
            self.set_iovar_u32(c"apsta", 0, None);
            self.ioctl(Ioctl::Set, IoctlCommand::Up, 0, &[]);

            self.ioctl(Ioctl::Set, IoctlCommand::SetAp, 0, &1u32.to_le_bytes());

            // set ssid
            let info = packets::SsidInfoWithIndex {
                index: 0,
                ssid_info: packets::SsidInfo {
                    len: ssid.len as _,
                    ssid: ssid.buf,
                },
            };

            self.set_iovar(c"bsscfg:ssid", as_bytes(&info));

            // channel
            self.ioctl(
                Ioctl::Set,
                IoctlCommand::SetChannel,
                0,
                &channel.to_le_bytes(),
            );

            // security
            let sec = if security.is_none() { 0 } else { 0x04 };
            self.set_iovar_u32(c"bsscfg:wsec", 0, Some(sec));

            if let Some((_, passphrase)) = security {
                self.set_iovar_u32(c"bsscfg:wpa_auth", 0, Some(0x0084));
                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state
                    .set(State::AccessPoint(APState::StartingWpa(passphrase)));
                Ok(())
            } else {
                // Change mutlicast rate from 1 Mbps to 11 Mbps
                self.set_iovar_u32(c"2g_mrate", 11000000 / 500000, None);

                // Start AP
                self.set_iovar_u32(c"bss", 0, Some(1));
                self.state.set(State::AccessPoint(APState::StartingOpen));
                self.deferred_call.set();
                Ok(())
            }
        } else {
            Err(ErrorCode::FAIL)
        }
    }

    fn stop_ap(&self) -> Result<(), ErrorCode> {
        self.set_iovar_u32(c"bss", 0, Some(0));

        self.ioctl(Ioctl::Set, IoctlCommand::SetAp, 0, &0u32.to_le_bytes());

        self.ioctl(Ioctl::Set, IoctlCommand::Down, 0, &[]);
        self.set_iovar_u32(c"apsta", 1, None);
        self.ioctl(Ioctl::Set, IoctlCommand::Up, 0, &[]);

        self.state.set(State::Station(StationState::NotConnected));

        Ok(())
    }
}

impl<'a, A: Alarm<'a>> WifiCtrl<'a> for PioCyw43439<'a, A> {
    fn init(&self) -> Result<(), ErrorCode> {
        if let State::Init(InitState::PoweredDown) = self.state.get() {
            // We first set pin off, wait 20 milliseconds, and then set on
            self.pwr_down();

            let now = self.alarm.now();
            self.alarm.set_alarm(now, self.alarm.ticks_from_ms(20));

            Ok(())
        } else {
            Err(ErrorCode::FAIL)
        }
    }
}

impl<'a, A: Alarm<'a>> DeferredCallClient for PioCyw43439<'a, A> {
    fn handle_deferred_call(&self) {
        // If we transmitted an ethernet packet, notify the client it's done.
        if let Some((transmission_identifier, frame_buffer, len)) = self.tx.take() {
            self.ethernet_client.map(|client| {
                client.transmit_frame_done(Ok(()), frame_buffer, len, transmission_identifier, None)
            });
        }

        if let State::AccessPoint(APState::StartingOpen) = self.state.get() {
            self.ap_client.map(|client| client.started_ap(Ok(())));
            self.state.set(State::AccessPoint(APState::Started));
        }
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}

impl<'a, A: Alarm<'a>> EthernetAdapterDatapath<'a> for PioCyw43439<'a, A> {
    fn set_client(&self, client: &'a dyn EthernetAdapterDatapathClient) {
        self.ethernet_client.set(client);
    }

    fn enable_receive(&self) {
        self.receive_en.set(true);
    }

    fn disable_receive(&self) {
        self.receive_en.set(false);
    }

    fn transmit_frame(
        &self,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        if let State::Init(_) = self.state.get() {
            return Err((ErrorCode::BUSY, frame_buffer));
        }

        let buffer = self.buffer.take().unwrap();
        let buf8 = slice8_mut(buffer);

        const PADDING_SIZE: usize = 2;
        let total_len = SdpcmHeader::SIZE + PADDING_SIZE + BdcHeader::SIZE + len as usize;

        let seq = self.sdpcm_seq.get();
        self.sdpcm_seq.set(seq.wrapping_add(1));

        let sdpcm_header = SdpcmHeader {
            len: total_len as u16,
            len_inv: !total_len as u16,
            sequence: seq,
            channel_and_flags: ChannelType::Data as _,
            next_length: 0,
            header_length: (SdpcmHeader::SIZE + PADDING_SIZE) as _,
            wireless_flow_control: 0,
            bus_data_credit: 0,
            reserved: [0, 0],
        };

        const BDC_VERSION: u8 = 2;
        const BDC_VERSION_SHIFT: u8 = 4;
        let bdc_header = BdcHeader {
            flags: BDC_VERSION << BDC_VERSION_SHIFT,
            priority: 0,
            flags2: 0,
            data_offset: 0,
        };

        buf8[0..SdpcmHeader::SIZE].copy_from_slice(as_bytes(&sdpcm_header));

        buf8[SdpcmHeader::SIZE + PADDING_SIZE..][..BdcHeader::SIZE]
            .copy_from_slice(as_bytes(&bdc_header));

        buf8[SdpcmHeader::SIZE + PADDING_SIZE + BdcHeader::SIZE..][..len as usize]
            .copy_from_slice(&frame_buffer[..len as usize]);

        let total_len = (total_len + 3) & !3;

        let cmd = Cyw43Cmd::new(
            Command::Write,
            Access::IncAddr,
            Function::Wlan,
            0,
            total_len as u32,
        )
        .get();

        let len_32 = total_len / 4;

        buffer.copy_within(..len_32, 1);
        buffer[0] = cmd;

        self.write_buf(&buffer[..len_32 + 1]);
        self.buffer.put(Some(buffer));

        self.tx.set((transmission_identifier, frame_buffer, len));
        self.deferred_call.set();

        Ok(())
    }
}

impl<'a, A: Alarm<'a>> PioSmClient for PioCyw43439<'a, A> {
    fn on_irq(&self) {
        // Clear interrupt
        self.pio.interrupt_clear(0);

        // Read the interrupt cause from the REG_BUS_INTERRUPT register.
        let irq = self.read_n(
            WordLength::_32Bit,
            Function::Spi,
            utils::REG_BUS_INTERRUPT,
            2,
        ) as u16;

        // Read and process the packets if available
        if irq & utils::IRQ_F2_PACKET_AVAILABLE != 0 {
            self.recv();
        }

        if irq & utils::IRQ_DATA_UNAVAILABLE != 0 {
            self.write_n(
                WordLength::_32Bit,
                Function::Spi,
                utils::REG_BUS_INTERRUPT,
                1,
                2,
            );
        }
    }
}

impl<'a, A: Alarm<'a>> PioCyw43439<'a, A> {
    fn recv(&self) {
        let mut status = self.status.get();

        let Some(buffer) = self.buffer.take() else {
            // If we don't have a buffer to put the packets into,
            // we return.
            return;
        };

        // While there are packets available, continue reading and parsing
        while status & utils::STATUS_F2_PKT_AVAILABLE != 0 {
            let len_in_u8 =
                (status & utils::STATUS_F2_PKT_LEN_MASK) >> utils::STATUS_F2_PKT_LEN_SHIFT;
            let len_in_u32 = len_in_u8.div_ceil(4) as usize;

            let cmd = Cyw43Cmd::new(Command::Read, Access::IncAddr, Function::Wlan, 0, len_in_u8);

            // Read the packet and parse it
            self.read_buf(cmd.get(), &mut buffer[..len_in_u32]);
            self.parse_recv(&mut slice8_mut(buffer)[..len_in_u8 as usize]);
            status = self.status.get();
        }

        self.buffer.put(Some(buffer));
    }

    fn parse_recv(&self, packet: &mut [u8]) {
        let Some((sdpcm_header, payload)) = SdpcmHeader::parse(packet) else {
            return;
        };

        let Some(channel) = ChannelType::from_u8(sdpcm_header.channel_and_flags & 0x0f) else {
            return;
        };

        match channel {
            // IOCTL response
            ChannelType::Control => {
                let Some((cdc_header, response)) = CdcHeader::parse(payload) else {
                    return;
                };

                // we're only interested in getvar responses
                if cdc_header.id == self.id.get()
                    && cdc_header.cmd == IoctlCommand::GetVar as u32
                    && cdc_header.status == 0
                {
                    if let Some(name) = self.pending_ioctl.take() {
                        // MAC address get response
                        if name == ioctl::CUR_ETHERADDR_IOCTL {
                            let mut mac_addr = [0u8; 6];
                            mac_addr[..].copy_from_slice(&response[..6]);

                            // We're in the waiting phase after the initialisation, set the flag
                            // to done
                            if let State::Init(InitState::GetMacAddress) = self.state.get() {
                                self.ctrl_client
                                    .map(|client| client.init_done(Ok(mac_addr)));

                                self.state.set(State::Station(StationState::NotConnected));
                            }
                        }
                    }
                    // Else, we ignore
                }
            }
            // Event
            ChannelType::Event => {
                let Some((_header, bdc_packet)) = BdcHeader::parse(payload) else {
                    return;
                };
                let Some((event_packet, evt_data)) = EventPacket::parse(bdc_packet) else {
                    return;
                };

                if event_packet.eth.ether_type != 0x886c
                    || event_packet.hdr.oui != [0x00, 0x10, 0x18]
                    || event_packet.hdr.subtype != 32769
                    || event_packet.hdr.user_subtype != 1
                {
                    return;
                }

                let evt_type = Event::from_u8(event_packet.msg.event_type as u8).unwrap();
                let status = event_packet.msg.status;
                const ESCAN_PARTIAL: u32 = 8;
                match evt_type {
                    // Network scan result
                    Event::ESCAN_RESULT if status == ESCAN_PARTIAL => {
                        let Some((_, bss_info)) = ScanResults::parse(evt_data) else {
                            return;
                        };
                        let Some(bss_info) = BssInfo::parse(bss_info) else {
                            return;
                        };
                        // If the string is not null, notify client.
                        if self.scanning.get() && bss_info.ssid_len > 0 {
                            let ssid = Ssid {
                                buf: bss_info.ssid,
                                len: bss_info.ssid_len as _,
                            };

                            self.scan_client.map(|client| client.scanned_network(ssid));
                        }
                    }
                    // ESCAN operation done
                    Event::ESCAN_RESULT => {
                        self.scan_client.map(|client| {
                            let res = if status == 0 {
                                Ok(())
                            } else {
                                Err(ErrorCode::FAIL)
                            };

                            client.scan_done(res)
                        });
                    }
                    // Network join events
                    Event::SET_SSID => {
                        // Join ends with ssid
                        if let State::Station(StationState::Joining) = self.state.get() {
                            let err = if status == 0 {
                                self.state.set(State::Station(StationState::Joined));
                                Ok(())
                            } else {
                                self.state.set(State::Station(StationState::NotConnected));
                                Err(ErrorCode::FAIL)
                            };
                            // Notify client join
                            self.sta_client.map(|client| client.join_done(err));
                        }
                    }
                    _ => {}
                }
            }

            // Data
            ChannelType::Data => {
                if self.receive_en.get() {
                    // If receive is enabled and the packet was parsed ok,
                    // notify the client
                    let Some((_, packet)) = BdcHeader::parse(payload) else {
                        return;
                    };
                    self.ethernet_client
                        .map(|client| client.received_frame(packet, None));
                }
            }
        }
    }
}

// Command functions
impl<'a, A: Alarm<'a>> PioCyw43439<'a, A> {
    /// Read at maximum 4 bytes
    fn read_n(&self, mode: WordLength, func: Function, address: u32, length: u32) -> u32 {
        let cmd = Cyw43Cmd::new(Command::Read, Access::IncAddr, func, address, length).get();

        let cmd = match mode {
            WordLength::_16Bit => cmd.rotate_left(16),
            WordLength::_32Bit => cmd,
        };

        let len = if let Function::Backplane = func {
            // If the function is backplane, we need to reserve an extra word
            2
        } else {
            1
        };

        let current_sm = self.pio.sm(self.sm_number);

        self.cs_pin.clear();

        current_sm.set_enabled(false);
        current_sm.push(31).unwrap();
        current_sm.push(len * 32 + 31).unwrap();

        current_sm.push(cmd).unwrap();
        current_sm.exec(0);
        current_sm.set_enabled(true);

        let mut res;

        while current_sm.rx_empty() {}
        res = current_sm.pull().unwrap();

        if let Function::Backplane = func {
            while current_sm.rx_empty() {}
            res = current_sm.pull().unwrap();
        }

        while current_sm.rx_empty() {}
        let status = current_sm.pull().unwrap();
        self.status.set(match mode {
            WordLength::_16Bit => status.rotate_left(16),
            WordLength::_32Bit => status,
        });

        self.cs_pin.set();
        match mode {
            WordLength::_16Bit => res.rotate_left(16),
            WordLength::_32Bit => res,
        }
    }

    /// Write maximum 4 bytes
    fn write_n(&self, mode: WordLength, func: Function, address: u32, data: u32, length: u32) {
        let cmd = Cyw43Cmd::new(Command::Write, Access::IncAddr, func, address, length).get();

        let (cmd, data) = match mode {
            WordLength::_16Bit => (
                cmd.rotate_left(16),
                if length == 4 {
                    data.rotate_left(16)
                } else {
                    data
                },
            ),
            WordLength::_32Bit => (cmd, data),
        };

        let current_sm = self.pio.sm(self.sm_number);

        self.cs_pin.clear();

        current_sm.set_enabled(false);
        current_sm.push(63).unwrap();
        current_sm.push(31).unwrap();

        current_sm.push(cmd).unwrap();
        current_sm.push(data).unwrap();

        current_sm.exec(0);
        current_sm.set_enabled(true);

        while current_sm.rx_empty() {}
        let status = current_sm.pull().unwrap();

        self.cs_pin.set();

        self.status.set(match mode {
            WordLength::_16Bit => status.rotate_left(16),
            WordLength::_32Bit => status,
        });
    }

    /// Write a buffer of bytes
    fn write_buf(&self, data: &[u32]) {
        let write_bits = (data.len() as u32) * 32 - 1;

        let current_sm = self.pio.sm(self.sm_number);

        self.cs_pin.clear();
        current_sm.set_enabled(false);

        current_sm.push(write_bits).unwrap();
        current_sm.push(31).unwrap();

        current_sm.exec(0);
        current_sm.set_enabled(true);

        for idx in 0..data.len() {
            current_sm.push_blocking(data[idx]).unwrap();
        }

        while current_sm.rx_empty() {}
        let status = current_sm.pull().unwrap();
        self.status.set(status);

        self.cs_pin.set();
    }

    /// Read in a buffer
    fn read_buf(&self, cmd: u32, data: &mut [u32]) {
        let read_bits = (data.len() as u32) * 32 + 31;

        let current_sm = self.pio.sm(self.sm_number);

        self.cs_pin.clear();
        current_sm.set_enabled(false);

        current_sm.push(31).unwrap();
        current_sm.push(read_bits).unwrap();

        current_sm.push_blocking(cmd).unwrap();

        current_sm.exec(0);
        current_sm.set_enabled(true);

        for idx in 0..data.len() {
            data[idx] = current_sm.pull_blocking().unwrap();
        }

        // get status
        while current_sm.rx_empty() {}
        let status = current_sm.pull().unwrap();
        self.status.set(status);

        self.cs_pin.set();
    }

    // BACKPLANE

    fn backplane_write_n(&self, address: u32, data: u32, count: u32) {
        self.set_backplane_window(address);

        let mut bus_addr = address & utils::BACKPLANE_ADDRESS_MASK;
        if count == 4 {
            bus_addr |= utils::BACKPLANE_WINDOW_SIZE;
        }

        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            bus_addr,
            data,
            count,
        );
    }

    fn backplane_read_n(&self, address: u32, count: u32) -> u32 {
        self.set_backplane_window(address);

        let mut bus_addr = address & utils::BACKPLANE_ADDRESS_MASK;
        if count == 4 {
            bus_addr |= utils::BACKPLANE_WINDOW_SIZE;
        }

        self.read_n(WordLength::_32Bit, Function::Backplane, bus_addr, count)
    }

    fn set_backplane_window(&self, address: u32) {
        // Set window
        let new_window = address & !utils::BACKPLANE_ADDRESS_MASK;

        if (new_window >> 24) as u8 != (self.backplane_window.get() >> 24) as u8 {
            self.write_n(
                WordLength::_32Bit,
                Function::Backplane,
                utils::REG_BACKPLANE_BACKPLANE_ADDRESS_HIGH,
                new_window >> 24,
                1,
            );
        }
        if (new_window >> 16) as u8 != (self.backplane_window.get() >> 16) as u8 {
            self.write_n(
                WordLength::_32Bit,
                Function::Backplane,
                utils::REG_BACKPLANE_BACKPLANE_ADDRESS_MID,
                new_window >> 16,
                1,
            );
        }
        if (new_window >> 8) as u8 != (self.backplane_window.get() >> 8) as u8 {
            self.write_n(
                WordLength::_32Bit,
                Function::Backplane,
                utils::REG_BACKPLANE_BACKPLANE_ADDRESS_LOW,
                new_window >> 8,
                1,
            );
        }

        self.backplane_window.set(new_window);
    }

    fn backplane_write_buf(&self, mut addr: u32, mut data: &[u8]) {
        assert!(addr % 4 == 0);

        let mut buf = [0u32; utils::BACKPLANE_MAX_TRANSFER_SIZE / 4 + 1];

        while !data.is_empty() {
            let window_offset = addr & utils::BACKPLANE_ADDRESS_MASK;
            let window_remaining = (utils::BACKPLANE_WINDOW_SIZE - window_offset) as usize;

            let len = data
                .len()
                .min(utils::BACKPLANE_MAX_TRANSFER_SIZE)
                .min(window_remaining);

            slice8_mut(&mut buf[1..])[..len].copy_from_slice(&data[..len]);

            self.set_backplane_window(addr);

            let cmd = Cyw43Cmd::new(
                Command::Write,
                Access::IncAddr,
                Function::Backplane,
                window_offset,
                len as u32,
            )
            .get();

            buf[0] = cmd;

            self.write_buf(&buf[..(len.div_ceil(4) + 1)]);

            addr += len as u32;
            data = &data[len..];
        }
    }
}

// IOCTLs
impl<'a, A: Alarm<'a>> PioCyw43439<'a, A> {
    /// Send IOCTL packet of type "get var", expecting a response of `len` bytes
    fn get_iovar(&self, name: &'static CStr, _len: usize) {
        let mut data = [0; 64];
        let bytes = name.to_bytes_with_nul();
        data[..bytes.len()].copy_from_slice(bytes);

        self.ioctl(Ioctl::Get, IoctlCommand::GetVar, 0, &data[..bytes.len()]);

        // Set name of the pending ioctl
        self.pending_ioctl.set(name);

        self.recv();
    }

    /// Send IOCTL packet of type "set var" and a payload of 4 bytes
    fn set_iovar_u32(&self, name: &CStr, val0: u32, val1: Option<u32>) {
        let val0_bytes = val0.to_le_bytes();
        match val1 {
            Some(val1) => {
                let mut data = [0; 8];
                data[0..4].copy_from_slice(&val0_bytes);
                data[4..8].copy_from_slice(&val1.to_le_bytes());

                self.set_iovar(name, &data);
            }
            None => {
                self.set_iovar(name, &val0_bytes);
            }
        }
    }

    /// Send IOCTL packet of type "set var" with `data` payload
    fn set_iovar(&self, name: &CStr, data: &[u8]) {
        // Copy name and data into buffer
        let mut send = [0; 196];
        let name = name.to_bytes_with_nul();

        send[..name.len()].copy_from_slice(name);
        send[name.len()..][..data.len()].copy_from_slice(data);

        let total_len = name.len() + data.len();
        self.ioctl(Ioctl::Set, IoctlCommand::SetVar, 0, &send[..total_len]);

        self.recv();
    }

    // Send IOCTL packet
    fn ioctl(&self, ioctl: Ioctl, cmd: IoctlCommand, iface: u32, data: &[u8]) {
        let total_len = SdpcmHeader::SIZE + CdcHeader::SIZE + data.len();

        let sequence = self.sdpcm_seq.get();
        self.sdpcm_seq.set(sequence.wrapping_add(1));
        self.id.set(self.id.get().wrapping_add(1));

        let sdpcm_header = SdpcmHeader {
            len: total_len as u16,
            len_inv: !total_len as u16,
            sequence,
            channel_and_flags: ChannelType::Control as u8,
            next_length: 0,
            header_length: SdpcmHeader::SIZE as _,
            wireless_flow_control: 0,
            bus_data_credit: 0,
            reserved: [0, 0],
        };

        let cdc_header = CdcHeader {
            cmd: cmd as u32,
            len: data.len() as _,
            flags: ioctl as u16 | (iface as u16) << 12,
            id: self.id.get(),
            status: 0,
        };

        self.buffer.map(|buf| {
            let buf8: &mut [u8] = slice8_mut(buf);
            buf8[0..SdpcmHeader::SIZE].copy_from_slice(as_bytes(&sdpcm_header));
            buf8[SdpcmHeader::SIZE..][..CdcHeader::SIZE].copy_from_slice(as_bytes(&cdc_header));
            buf8[SdpcmHeader::SIZE + CdcHeader::SIZE..][..data.len()].copy_from_slice(data);

            let total_len = ((total_len + 3) & !0b011) / 4;

            buf.copy_within(..total_len, 1);

            buf[0] = Cyw43Cmd::new(
                Command::Write,
                Access::IncAddr,
                Function::Wlan,
                0,
                total_len as u32 * 4,
            )
            .get();

            self.write_buf(&buf[..total_len + 1]);
        });
    }
}

// Initialisation state machine helper functions
impl<'a, A: Alarm<'a>> PioCyw43439<'a, A> {
    fn core_reset(&self, core: Core) {
        let base = core.base_addr();

        self.backplane_write_n(
            base + utils::AI_IOCTRL_OFFSET,
            (utils::AI_IOCTRL_BIT_FGC | utils::AI_IOCTRL_BIT_CLOCK_EN) as u32,
            1,
        );
        let _ = self.backplane_read_n(base + utils::AI_IOCTRL_OFFSET, 1);

        self.backplane_write_n(base + utils::AI_RESETCTRL_OFFSET, 0, 1);
    }

    fn core_clk_en(&self, core: Core) {
        self.backplane_write_n(
            core.base_addr() + utils::AI_IOCTRL_OFFSET,
            utils::AI_IOCTRL_BIT_CLOCK_EN as u32,
            1,
        );

        let _ = self.backplane_read_n(core.base_addr() + utils::AI_IOCTRL_OFFSET, 1);
    }

    fn core_is_up(&self, core: Core) -> bool {
        let base = core.base_addr();

        let io = self.backplane_read_n(base + utils::AI_IOCTRL_OFFSET, 1) as u8;
        if io & (utils::AI_IOCTRL_BIT_FGC | utils::AI_IOCTRL_BIT_CLOCK_EN)
            != utils::AI_IOCTRL_BIT_CLOCK_EN
        {
            return false;
        }

        let r = self.backplane_read_n(base + utils::AI_RESETCTRL_OFFSET, 1) as u8;
        if r & (utils::AI_RESETCTRL_BIT_RESET) != 0 {
            return false;
        }

        true
    }

    fn core_disable_prepare(&self, core: Core) -> bool {
        let base = core.base_addr();

        let _ = self.backplane_read_n(base + utils::AI_RESETCTRL_OFFSET, 1);
        let res = self.backplane_read_n(base + utils::AI_RESETCTRL_OFFSET, 1);

        if (res as u8) & utils::AI_RESETCTRL_BIT_RESET != 0 {
            return true;
        }

        self.backplane_write_n(base + utils::AI_IOCTRL_OFFSET, 0, 1);
        let _ = self.backplane_read_n(base + utils::AI_IOCTRL_OFFSET, 1);

        false
    }

    fn core_disable(&self, core: Core) {
        self.backplane_write_n(
            core.base_addr() + utils::AI_RESETCTRL_OFFSET,
            utils::AI_RESETCTRL_BIT_RESET as u32,
            1,
        );
        let _ = self.backplane_read_n(core.base_addr() + utils::AI_RESETCTRL_OFFSET, 1);
    }

    fn init_clm(&self) {
        const CHUNK_SIZE: usize = 1024;

        const DOWNLOAD_FLAG_HANDLER_VER: u16 = 0x1000;
        const DOWNLOAD_FLAG_BEGIN: u16 = 0x2;
        const DOWNLOAD_FLAG_END: u16 = 0x4;
        const DOWNLOAD_TYPE_CLM: u16 = 0x2;

        let mut offset = 0;
        for chunk in self.clm.chunks(CHUNK_SIZE) {
            let mut flag = DOWNLOAD_FLAG_HANDLER_VER;
            if offset == 0 {
                flag |= DOWNLOAD_FLAG_BEGIN;
            }
            offset += chunk.len();
            if offset == self.clm.len() {
                flag |= DOWNLOAD_FLAG_END;
            }

            let header = DownloadHeader {
                flag,
                dload_type: DOWNLOAD_TYPE_CLM,
                len: chunk.len() as _,
                crc: 0,
            };

            let mut data = [0; 8 + 12 + CHUNK_SIZE];
            data[0..8].copy_from_slice(b"clmload\x00");
            data[8..20].copy_from_slice(as_bytes(&header));
            data[20..][..chunk.len()].copy_from_slice(chunk);

            let total_len = 8 + 12 + chunk.len();

            self.ioctl(Ioctl::Set, IoctlCommand::SetVar, 0, &data[..total_len]);

            self.recv();
        }
    }

    fn power_up_sequence(&self) {
        // Test register
        let mut res = 0;
        while res != 0xFEEDBEAD {
            res = self.read_n(WordLength::_16Bit, Function::Spi, utils::REG_BUS_TEST_RO, 4);
        }
        self.write_n(
            WordLength::_16Bit,
            Function::Spi,
            utils::REG_BUS_TEST_RW,
            0x12345678,
            4,
        );
        res = self.read_n(WordLength::_16Bit, Function::Spi, utils::REG_BUS_TEST_RW, 4);
        assert_eq!(res, 0x12345678, "Write test failed");

        // Configure
        self.write_n(
            WordLength::_16Bit,
            Function::Spi,
            utils::REG_BUS_CTRL,
            utils::CONFIG_DATA,
            4,
        );

        // Second test
        while res != 0xFEEDBEAD {
            res = self.read_n(WordLength::_32Bit, Function::Spi, utils::REG_BUS_TEST_RO, 4);
        }
        self.write_n(
            WordLength::_32Bit,
            Function::Spi,
            utils::REG_BUS_TEST_RW,
            0xCAFEBABE,
            4,
        );
        res = self.read_n(WordLength::_32Bit, Function::Spi, utils::REG_BUS_TEST_RW, 4);
        assert_eq!(res, 0xCAFEBABE, "Write test failed");

        // F1 Response Delay Time is 4 by default, so no need for us to modify it

        // Interrupts
        self.write_n(
            WordLength::_32Bit,
            Function::Spi,
            utils::REG_BUS_INTERRUPT,
            utils::INTR_STATUS_RESET,
            1,
        );

        self.write_n(
            WordLength::_32Bit,
            Function::Spi,
            utils::REG_BUS_INTERRUPT_ENABLE,
            utils::INTR_ENABLE_RESET,
            1,
        );

        // Init clock
        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            utils::BACKPLANE_ALP_AVAIL_REQ as u32,
            1,
        );

        // Set F2 watermark
        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_FUNCTION2_WATERMARK,
            0x10,
            1,
        );

        let watermark = self.read_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_FUNCTION2_WATERMARK,
            1,
        );
        assert_eq!(watermark, 0x10, "Watermark write failed");

        // Wait for clock
        while self.read_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            1,
        ) & (utils::BACKPLANE_ALP_AVAIL as u32)
            == 0
        {}

        // Clear alp request
        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            0,
            1,
        );
    }

    fn load_fw(&self) {
        // Disable remap for SRAM_3
        self.backplane_write_n(Core::SocRam.base_addr() + 0x10, 3, 4);
        self.backplane_write_n(Core::SocRam.base_addr() + 0x44, 0, 4);

        // Now we need to load the firmware
        self.backplane_write_buf(utils::ATCM_RAM_BASE_ADDRESS, self.fw);

        // Load NVRAM
        let nvram_len = (NVRAM.len().div_ceil(4) * 4) as u32;
        self.backplane_write_buf(
            utils::ATCM_RAM_BASE_ADDRESS + utils::RAM_SIZE - 4 - nvram_len,
            NVRAM,
        );

        let nvram_words = nvram_len / 4;
        let nvram_magic = (!nvram_words << 16) | nvram_words;
        // Write magic
        self.backplane_write_n(
            utils::ATCM_RAM_BASE_ADDRESS + utils::RAM_SIZE - 4,
            nvram_magic,
            4,
        );
    }

    fn init_wlan_core(&self) {
        // check if the core is up
        assert!(self.core_is_up(Core::WlanArm), "core is not up :(");

        // wait until HT clock is available
        while self.read_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            1,
        ) as u8
            & 0x80u8
            == 0
        {}

        self.backplane_write_n(
            utils::SDIOD_CORE_BASE_ADDRESS + utils::SDIO_INT_HOST_MASK,
            utils::I_HMB_SW_MASK,
            4,
        );

        self.write_n(
            WordLength::_32Bit,
            Function::Spi,
            utils::REG_BUS_INTERRUPT_ENABLE,
            utils::IRQ_F2_PACKET_AVAILABLE as u32,
            2,
        );

        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_FUNCTION2_WATERMARK,
            utils::SPI_F2_WATERMARK,
            1,
        );

        while self.read_n(WordLength::_32Bit, Function::Spi, utils::REG_BUS_STATUS, 4)
            & utils::STATUS_F2_RX_READY
            == 0
        {}

        // clear pad pulls
        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_PULL_UP,
            0,
            1,
        );

        let _ = self.read_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_PULL_UP,
            1,
        );

        // start HT clock
        self.write_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            0x10,
            1,
        );
        while self.read_n(
            WordLength::_32Bit,
            Function::Backplane,
            utils::REG_BACKPLANE_CHIP_CLOCK_CSR,
            1,
        ) & 0x80
            == 0
        {}
    }
}

impl<'a, A: Alarm<'a>> PioCyw43439<'a, A> {
    fn set_ssid(&self, ssid: Ssid) {
        let i = packets::SsidInfo {
            len: ssid.len as _,
            ssid: ssid.buf,
        };

        self.ioctl(Ioctl::Set, IoctlCommand::SetSsid, 0, as_bytes(&i));
    }

    fn set_wpa12_passphrase(&self, passphrase: &Passphrase) {
        let mut passphrase_info = packets::PassphraseInfo {
            len: passphrase.len as _,
            flags: 1,
            passphrase: [0; 64],
        };
        passphrase_info.passphrase[..PS_SIZE].copy_from_slice(&passphrase.buf[..]);

        self.ioctl(
            Ioctl::Set,
            IoctlCommand::SetWsecPmk,
            0,
            as_bytes(&passphrase_info),
        );
    }

    fn set_wpa3_passphrase(&self, passphrase: &Passphrase) {
        let mut passphrase_info = packets::SaePassphraseInfo {
            len: passphrase.len as _,
            passphrase: [0; 128],
        };

        passphrase_info.passphrase[..PS_SIZE].copy_from_slice(&passphrase.buf[..]);
        self.set_iovar(c"sae_password", as_bytes(&passphrase_info));
    }
}

// Initialisation sequence state machine
impl<'a, A: Alarm<'a>> AlarmClient for PioCyw43439<'a, A> {
    fn alarm(&self) {
        match self.state.get() {
            State::Init(InitState::PoweredDown) => {
                self.pwr_on();

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(250));
                self.state.set(State::Init(InitState::PoweredUp));
            }
            State::Init(InitState::PoweredUp) => {
                self.power_up_sequence();
                // Prepare to disable the WLAN core
                self.core_disable_prepare(Core::WlanArm);

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(1));
                self.state
                    .set(State::Init(InitState::DisableDevice(Core::WlanArm)));
            }
            State::Init(InitState::DisableDevice(core)) => {
                self.core_disable(core);

                match core {
                    Core::WlanArm => {
                        self.core_disable_prepare(Core::SocRam);

                        let now = self.alarm.now();
                        self.alarm.set_alarm(now, self.alarm.ticks_from_ms(1));
                        self.state
                            .set(State::Init(InitState::DisableDevice(Core::SocRam)));
                    }
                    Core::SocRam => {
                        self.core_reset(core);

                        let now = self.alarm.now();
                        self.alarm.set_alarm(now, self.alarm.ticks_from_ms(1));
                        self.state
                            .set(State::Init(InitState::ResetDevice(Core::SocRam)));
                    }
                }
            }
            State::Init(InitState::ResetDevice(core)) => {
                self.core_clk_en(core);

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(1));
                self.state
                    .set(State::Init(InitState::PostResetDevice(core)));
            }
            State::Init(InitState::PostResetDevice(core)) => match core {
                Core::SocRam => {
                    self.load_fw();
                    self.core_reset(Core::WlanArm);

                    let now = self.alarm.now();
                    self.alarm.set_alarm(now, self.alarm.ticks_from_ms(1));
                    self.state
                        .set(State::Init(InitState::ResetDevice(Core::WlanArm)));
                }
                Core::WlanArm => {
                    self.init_wlan_core();
                    self.init_clm();

                    self.set_iovar_u32(c"bus:txglom", 0, None);
                    self.set_iovar_u32(c"apsta", 1, None);

                    let country_info = CountryInfo {
                        country_abbrev: [88, 88, 0, 0],
                        country_code: [88, 88, 0, 0],
                        rev: -1,
                    };
                    self.set_iovar(c"country", as_bytes(&country_info));

                    let now = self.alarm.now();
                    self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                    self.state.set(State::Init(InitState::BusGlommOff));
                }
            },
            State::Init(InitState::BusGlommOff) => {
                // set antenna to chip antenna
                self.ioctl(Ioctl::Set, IoctlCommand::SetAntdiv, 0, &0u32.to_le_bytes());
                self.set_iovar_u32(c"bus:txglom", 0, None);

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state.set(State::Init(InitState::AmpduWindowSize));
            }
            State::Init(InitState::AmpduWindowSize) => {
                self.set_iovar_u32(c"ampdu_ba_wsize", 8, None);
                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state.set(State::Init(InitState::AmpduMpdus));
            }
            State::Init(InitState::AmpduMpdus) => {
                // set number of MPDUs
                self.set_iovar_u32(c"ampdu_mpdu", 4, None);
                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state.set(State::Init(InitState::EventMask));
            }
            State::Init(InitState::EventMask) => {
                // Set ioctl event mask

                let mut evts = packets::EventMask {
                    iface: 0,
                    events: [0xFF; 24],
                };

                // disable part of the events
                evts.unset(Event::RADIO);
                evts.unset(Event::IF);
                evts.unset(Event::PROBREQ_MSG);
                evts.unset(Event::PROBREQ_MSG_RX);
                evts.unset(Event::PROBRESP_MSG);
                evts.unset(Event::PROBRESP_MSG);
                evts.unset(Event::ROAM);

                self.set_iovar(c"bsscfg:event_msgs", as_bytes(&evts));

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state.set(State::Init(InitState::WifiUp));
            }
            State::Init(InitState::WifiUp) => {
                self.ioctl(Ioctl::Set, IoctlCommand::Up, 0, &[]);

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state.set(State::Init(InitState::SetBand));
            }
            State::Init(InitState::SetBand) => {
                self.ioctl(Ioctl::Set, IoctlCommand::SetGmode, 0, &1u32.to_le_bytes());
                self.ioctl(Ioctl::Set, IoctlCommand::SetBand, 0, &0u32.to_le_bytes());

                // finally done
                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(100));
                self.state.set(State::Init(InitState::GetMacAddress));
            }
            // we request the mac address
            State::Init(InitState::GetMacAddress) => {
                self.get_iovar(ioctl::CUR_ETHERADDR_IOCTL, 6);
            }
            State::Station(StationState::Config {
                sec_type,
                ssid,
                passphrase,
            }) => {
                match sec_type {
                    Security::Wpa => {
                        self.set_wpa12_passphrase(&passphrase);

                        self.ioctl(Ioctl::Set, IoctlCommand::SetInfra, 0, &1u32.to_le_bytes());
                        self.ioctl(Ioctl::Set, IoctlCommand::SetAuth, 0, &0u32.to_le_bytes());

                        self.set_iovar_u32(c"mfp", 0, None);
                        self.ioctl(Ioctl::Set, IoctlCommand::SetWpaAuth, 0, &4u32.to_le_bytes());
                    }
                    Security::Wpa2 => {
                        self.set_wpa12_passphrase(&passphrase);

                        self.ioctl(Ioctl::Set, IoctlCommand::SetInfra, 0, &1u32.to_le_bytes());
                        self.ioctl(Ioctl::Set, IoctlCommand::SetAuth, 0, &0u32.to_le_bytes());

                        self.set_iovar_u32(c"mfp", 1, None);
                        self.ioctl(
                            Ioctl::Set,
                            IoctlCommand::SetWpaAuth,
                            0,
                            &0x80u32.to_le_bytes(),
                        );
                    }
                    Security::Wpa3 => {
                        self.set_wpa3_passphrase(&passphrase);

                        self.ioctl(Ioctl::Set, IoctlCommand::SetInfra, 0, &1u32.to_le_bytes());
                        self.ioctl(Ioctl::Set, IoctlCommand::SetAuth, 0, &3u32.to_le_bytes());

                        self.set_iovar_u32(c"mfp", 2, None);
                        self.ioctl(
                            Ioctl::Set,
                            IoctlCommand::SetWpaAuth,
                            0,
                            &0x40000u32.to_le_bytes(),
                        );
                    }
                    Security::Wpa2Wpa3 => {
                        self.set_wpa12_passphrase(&passphrase);
                        self.set_wpa3_passphrase(&passphrase);

                        self.ioctl(Ioctl::Set, IoctlCommand::SetInfra, 0, &1u32.to_le_bytes());
                        self.ioctl(Ioctl::Set, IoctlCommand::SetAuth, 0, &3u32.to_le_bytes());

                        self.set_iovar_u32(c"mfp", 1, None);
                        self.ioctl(
                            Ioctl::Set,
                            IoctlCommand::SetWpaAuth,
                            0,
                            &0x40000u32.to_le_bytes(),
                        );
                    }
                }

                self.set_ssid(ssid);
                self.state.set(State::Station(StationState::Joining));
            }
            State::AccessPoint(APState::StartingWpa(passphrase)) => {
                let mut passphrase_info = packets::PassphraseInfo {
                    len: passphrase.len as _,
                    flags: 1,
                    // in embassy this has 64 bytes although 63 should be the max length.
                    // not modifying this
                    passphrase: [0; 64],
                };

                passphrase_info.passphrase[..PS_SIZE].copy_from_slice(&passphrase.buf);

                self.ioctl(
                    Ioctl::Set,
                    IoctlCommand::SetWsecPmk,
                    0,
                    as_bytes(&passphrase_info),
                );

                self.set_iovar_u32(c"2g_mrate", 11000000 / 500000, None);

                // Start AP
                self.set_iovar_u32(c"bss", 0, Some(1));

                self.state.set(State::AccessPoint(APState::Started));
                self.ap_client.map(|client| client.started_ap(Ok(())));
            }
            // Alarm should not fire while being in other states
            _ => unreachable!(),
        }
    }
}
