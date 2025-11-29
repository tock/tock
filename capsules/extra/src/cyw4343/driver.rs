// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive 2025.

//! Driver for CYW4343x
//!
//! The driver handles encoding/decoding SDPCM protocol packets:
//! - CDC = control packets
//! - BDC = data/Ethernet packets
//!
//! Control packets are chained to implement WiFi functionalities (see `tasks`) and data packets
//! are used to implement the Ethernet interface.

use super::macros::reset_and_restore_bufs;
use super::{bus, constants, sdpcm};
use crate::wifi;
use core::cell::Cell;
use core::iter::{Enumerate, Peekable};
use core::slice::Chunks;
use enum_primitive::cast::FromPrimitive;
use kernel::hil::time::ConvertTicks;
use kernel::utilities::cells::{MapCell, OptionalCell};
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{hil, ErrorCode};

/// Current state of the CYW43x device driver
#[derive(Clone, Copy, Debug, Default)]
enum State {
    #[default]
    Idle,
    NotInit,
    PoweredDown,
    PoweredUp,
    BusInit,
    Command(Command),
    Ethernet,
}

#[derive(Clone, Copy, Debug)]
enum Command {
    Init,
    Join,
    Leave,
    Scan,
    StopScan,
    Ap,
    Sta,
}

/// Possible pending IOCTLs
#[derive(Clone, Copy)]
enum Pending {
    Scan,
    MacAddr,
}

/// CYW4343x device driver
pub struct CYW4343x<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>> {
    /// Alarm used for setting delays to wait some
    /// operations to take effect
    alarm: &'a A,
    /// Data bus (gSPI/SDIO)
    bus: &'a B,
    /// Power pin
    pwr: &'a P,
    /// Current driver state
    state: Cell<State>,
    /// Inner buffer for constructing WLAN packets
    buffer: OptionalCell<SubSliceMut<'static, u8>>,
    /// Wifi device client
    client: OptionalCell<&'a dyn wifi::Client>,
    /// Ethernet driver client
    eth_client: OptionalCell<&'a dyn hil::ethernet::EthernetAdapterDatapathClient>,
    /// Sequence number, part of the SDPCM header
    sdpcm_seq: Cell<u8>,
    /// Unique id for rx/tx pair, part of the CDC header
    id: Cell<u16>,
    /// Ethernet receive enable
    receive: Cell<bool>,
    /// Ethernet transmission identifier and buffer
    eth_tx_data: OptionalCell<(usize, &'static mut [u8])>,
    /// Reference to the current IOCTL task list and index
    ioctl_tasks: ioctl::Tasks,
    /// CLM chunk index
    clm: MapCell<Peekable<Enumerate<Chunks<'static, u8>>>>,
    /// SSID
    ssid: OptionalCell<wifi::Ssid>,
    /// Security passphrase
    security: OptionalCell<wifi::Passphrase>,
    /// Wifi channel
    channel: OptionalCell<u8>,
    /// Current async operation (the chip may send a SDPCM response packet anytime)
    pending: OptionalCell<Pending>,
    /// MAC address
    mac: OptionalCell<[u8; 6]>,
}

impl<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>>
    CYW4343x<'a, P, A, B>
{
    pub fn new(
        alarm: &'a A,
        bus: &'a B,
        pwr: &'a P,
        clm: &'static [u8],
        buffer: &'static mut [u8; 1600],
    ) -> Self {
        Self {
            alarm,
            bus,
            pwr,
            state: Cell::new(State::NotInit),
            client: OptionalCell::empty(),
            eth_client: OptionalCell::empty(),
            sdpcm_seq: Cell::new(0),
            id: Cell::new(0),
            receive: Cell::new(false),
            ssid: OptionalCell::empty(),
            security: OptionalCell::empty(),
            channel: OptionalCell::empty(),
            ioctl_tasks: ioctl::Tasks::new(),
            eth_tx_data: OptionalCell::empty(),
            pending: OptionalCell::empty(),
            clm: MapCell::new(clm.chunks(constants::CLM_CHUNK_SIZE).enumerate().peekable()),
            buffer: OptionalCell::new(SubSliceMut::new(buffer)),
            mac: OptionalCell::empty(),
        }
    }

    /// Send BDC (Bulk Data) packet
    fn send_bdc(&self, data: &[u8]) -> Result<(), ErrorCode> {
        let Some(mut buffer) = self.buffer.take() else {
            return Err(ErrorCode::NOMEM);
        };

        let total_len = sdpcm::SdpcmHeader::SIZE
            + constants::BDC_PADDING_SIZE
            + sdpcm::BdcHeader::SIZE
            + data.len();

        let seq = self.sdpcm_seq.get();
        self.sdpcm_seq.set(seq.wrapping_add(1));

        let sdpcm_header = sdpcm::SdpcmHeader {
            len: total_len as u16,
            len_inv: !total_len as u16,
            seq,
            flags: sdpcm::ChannelType::Data as _,
            next_len: 0,
            data_offset: (sdpcm::SdpcmHeader::SIZE + constants::BDC_PADDING_SIZE) as _,
            flow_ctrl: 0,
            data_credit: 0,
            reserved: 0,
        }
        .into_bytes();

        let bdc_header = sdpcm::BdcHeader {
            flags: constants::BDC_VERSION << constants::BDC_VERSION_SHIFT,
            priority: 0,
            flags2: 0,
            data_offset: 0,
        }
        .into_bytes();

        buffer.slice(0..total_len);
        let slice = buffer.as_mut_slice();
        slice[0..sdpcm::SdpcmHeader::SIZE].copy_from_slice(&sdpcm_header);
        slice[sdpcm::SdpcmHeader::SIZE + constants::BDC_PADDING_SIZE..][..sdpcm::BdcHeader::SIZE]
            .copy_from_slice(&bdc_header);
        slice[sdpcm::SdpcmHeader::SIZE + constants::BDC_PADDING_SIZE + sdpcm::BdcHeader::SIZE..]
            [..data.len()]
            .copy_from_slice(data);

        self.bus.write_bytes(buffer).map_err(|(err, mut buffer)| {
            reset_and_restore_bufs!(self, buffer);
            err
        })
    }

    fn send_cdc(
        &self,
        ioctl: sdpcm::IoctlType,
        cmd: sdpcm::IoctlCommand,
        data: &[u8],
    ) -> Result<(), ErrorCode> {
        let Some(mut buffer) = self.buffer.take() else {
            return Err(ErrorCode::NOMEM);
        };

        let total_len = sdpcm::SdpcmHeader::SIZE + sdpcm::CdcHeader::SIZE + data.len();

        let seq = self.sdpcm_seq.get();
        self.sdpcm_seq.set(seq.wrapping_add(1));
        self.id.set(self.id.get().wrapping_add(1));

        let sdpcm_header = sdpcm::SdpcmHeader {
            len: total_len as u16,
            len_inv: !total_len as u16,
            seq,
            flags: sdpcm::ChannelType::Control as u8,
            next_len: 0,
            data_offset: sdpcm::SdpcmHeader::SIZE as _,
            flow_ctrl: 0,
            data_credit: 0,
            reserved: 0,
        }
        .into_bytes();

        let cdc_header = sdpcm::CdcHeader {
            cmd: cmd as u32,
            len: data.len() as _,
            flags: ioctl as u32 | (self.id.get() as u32) << 16,
            status: 0,
        }
        .into_bytes();

        buffer.slice(0..total_len);
        let slice = buffer.as_mut_slice();
        slice[0..sdpcm::SdpcmHeader::SIZE].copy_from_slice(&sdpcm_header);
        slice[sdpcm::SdpcmHeader::SIZE..][..sdpcm::CdcHeader::SIZE].copy_from_slice(&cdc_header);
        slice[sdpcm::SdpcmHeader::SIZE + sdpcm::CdcHeader::SIZE..][..data.len()]
            .copy_from_slice(data);

        self.bus.write_bytes(buffer).map_err(|(err, mut buffer)| {
            reset_and_restore_bufs!(self, buffer);
            err
        })
    }
}

impl<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>>
    CYW4343x<'a, P, A, B>
{
    /// Initialize the task buffer and executes the first task
    fn init_tasks(&self, tasks: &'static [ioctl::Op]) -> Result<(), ErrorCode> {
        if !self.ioctl_tasks.is_empty() || self.pending.is_some() {
            Err(ErrorCode::BUSY)
        } else {
            self.ioctl_tasks.init(tasks);
            let advance = self.do_task(tasks[0])?;
            if advance {
                self.ioctl_tasks.advance();
            }
            Ok(())
        }
    }

    /// Do a task (start a CDC transfer or set an alarm)
    ///
    /// This returns whether this should advance or not
    fn do_task(&self, task: ioctl::Op) -> Result<bool, ErrorCode> {
        let mut advance = true;
        match task {
            ioctl::Op::Ioctl(ioctl) => self.ioctl(ioctl),
            ioctl::Op::LoadCLM => self
                .clm
                .map(|chunks| {
                    let Some(curr) = chunks.next() else {
                        return Err(ErrorCode::FAIL);
                    };
                    let next = chunks.peek();
                    let mut flag = constants::CLM_DOWNLOAD_FLAG_HANDLER_VER;

                    if next.is_none() {
                        flag |= constants::CLM_DOWNLOAD_FLAG_END;
                    } else {
                        advance = false;
                    }

                    if curr.0 == 0 {
                        flag |= constants::CLM_DOWNLOAD_FLAG_BEGIN;
                    }
                    let header = sdpcm::WlDloadData {
                        flag,
                        dload_type: constants::CLM_DOWNLOAD_TYPE,
                        len: curr.1.len() as _,
                        crc: 0,
                    }
                    .into_bytes();
                    const IOVAR_SIZE: usize = sdpcm::Iovar::ClmLoad.len();
                    let mut data =
                        [0; IOVAR_SIZE + sdpcm::WlDloadData::SIZE + constants::CLM_CHUNK_SIZE];
                    data[0..IOVAR_SIZE].copy_from_slice(sdpcm::Iovar::ClmLoad.into());
                    data[IOVAR_SIZE..][..sdpcm::WlDloadData::SIZE].copy_from_slice(&header);

                    data[IOVAR_SIZE + sdpcm::WlDloadData::SIZE..][..curr.1.len()]
                        .copy_from_slice(curr.1);

                    let total_len = IOVAR_SIZE + sdpcm::WlDloadData::SIZE + curr.1.len();
                    self.send_cdc(
                        sdpcm::IoctlType::Set,
                        sdpcm::IoctlCommand::SetVar,
                        &data[..total_len],
                    )
                })
                .unwrap_or(Err(ErrorCode::FAIL)),
            ioctl::Op::WaitMs(ms) => {
                self.alarm
                    .set_alarm(self.alarm.now(), self.alarm.ticks_from_ms(ms));
                Ok(())
            }
            ioctl::Op::MacAddr => {
                self.pending.set(Pending::MacAddr);
                self.send_cdc(
                    sdpcm::IoctlType::Get,
                    sdpcm::IoctlCommand::GetVar,
                    sdpcm::Iovar::CurEthAddr.into(),
                )
                .map(|()| self.pending.set(Pending::MacAddr))
            }
        }
        .map(|()| advance)
    }

    /// Do a IOCTL operation
    fn ioctl(&self, ioctl: ioctl::Ioctl) -> Result<(), ErrorCode> {
        let data = match ioctl.data {
            ioctl::IoctlData::Empty => &[],
            ioctl::IoctlData::Word(ref bytes) => &bytes[..],
            ioctl::IoctlData::DWord(ref bytes) => &bytes[..],
            ioctl::IoctlData::BssSsid => &self
                .ssid
                .take()
                .map(|wifi::Ssid { len, buf }| {
                    sdpcm::SsidInfoWithIndex {
                        idx: 0,
                        len: len.get() as _,
                        buf,
                    }
                    .into_bytes()
                })
                .ok_or(ErrorCode::FAIL)?,
            ioctl::IoctlData::Ssid => &self
                .ssid
                .take()
                .map(|wifi::Ssid { len, buf }| {
                    sdpcm::SsidInfo {
                        len: len.get() as _,
                        buf,
                    }
                    .into_bytes()
                })
                .ok_or(ErrorCode::FAIL)?,
            ioctl::IoctlData::Wpa1Passphrase => &self
                .security
                .take()
                .map(<[u8; sdpcm::PassphraseInfo::SIZE]>::from)
                .ok_or(ErrorCode::FAIL)?,
            ioctl::IoctlData::Wpa3Passphrase => &self
                .security
                .take()
                .map(<[u8; sdpcm::SaePassphraseInfo::SIZE]>::from)
                .ok_or(ErrorCode::FAIL)?,
            ioctl::IoctlData::Channel => &self
                .channel
                .take()
                .map(|channel| [channel])
                .ok_or(ErrorCode::FAIL)?,
            ioctl::IoctlData::ScanParameters => &ioctl::start_scan::SCAN_PARAMS,
            ioctl::IoctlData::AbortScanParameters => &ioctl::stop_scan::SCAN_PARAMS,
            ioctl::IoctlData::CountryInfo => &ioctl::init::COUNTRY_INFO,
            ioctl::IoctlData::EventMask => &ioctl::init::EVENTS,
        };

        if let Some(name) = ioctl.name {
            const MAX_LEN: usize = sdpcm::SaePassphraseInfo::SIZE + sdpcm::MAX_IOVAR_LEN;
            let len = name.len() + data.len();

            let mut iovar: [u8; MAX_LEN] = [0; MAX_LEN];
            iovar[..name.len()].copy_from_slice(name.into());
            iovar[name.len()..][..data.len()].copy_from_slice(data);

            self.send_cdc(sdpcm::IoctlType::Set, ioctl.cmd, &iovar[..len])
        } else {
            self.send_cdc(sdpcm::IoctlType::Set, ioctl.cmd, data)
        }
    }

    /// Parse a slice as a WLAN packet
    fn parse(&self, buffer: &[u8]) {
        if buffer.len() < sdpcm::SdpcmHeader::SIZE {
            return;
        }

        let mut header = &buffer[..sdpcm::SdpcmHeader::SIZE];
        let sdpcm_header = sdpcm::SdpcmHeader::from_bytes(header);

        let Some(channel) = sdpcm::ChannelType::from_u8(sdpcm_header.flags & 0xf) else {
            return;
        };
        let mut data = &buffer[sdpcm_header.data_offset as usize..];

        match channel {
            // IOCTL response
            sdpcm::ChannelType::Control => {
                if data.len() < sdpcm::CdcHeader::SIZE {
                    return;
                }

                (header, data) = data.split_at(sdpcm::CdcHeader::SIZE);
                let cdc_header = sdpcm::CdcHeader::from_bytes(header);
                if cdc_header.status != 0 {
                    return;
                }

                if (cdc_header.flags >> 16) as u16 == self.id.get()
                    && cdc_header.cmd == sdpcm::IoctlCommand::GetVar as u32
                {
                    if let Some(Pending::MacAddr) = self.pending.get() {
                        let mut mac = [0u8; 6];
                        mac[..].copy_from_slice(&data[..6]);
                        self.mac.set(mac);
                        self.pending.clear();
                    }
                }
            }
            // Asynchronous events
            sdpcm::ChannelType::Event => {
                // Events have BDC headers
                if data.len()
                    < sdpcm::BdcHeader::SIZE
                        + sdpcm::EthernetHeader::SIZE
                        + sdpcm::EventHeader::SIZE
                        + sdpcm::EventMessage::SIZE
                {
                    return;
                }

                (header, data) = data.split_at(sdpcm::BdcHeader::SIZE);
                let bdc_hdr = sdpcm::BdcHeader::from_bytes(header);
                let offset = 4 * bdc_hdr.data_offset as usize;
                if offset > data.len() {
                    return;
                }
                data = &data[offset..];
                (header, data) = data.split_at(sdpcm::EthernetHeader::SIZE);
                let eth_hdr = sdpcm::EthernetHeader::from_bytes(header);
                (header, data) = data.split_at(sdpcm::EventHeader::SIZE);
                let event_hdr = sdpcm::EventHeader::from_bytes(header);

                if eth_hdr.ethertype.to_be() != constants::ETHER_TYPE_BRCM
                    || event_hdr.oui != constants::BRCM_OUI
                    || event_hdr.subtype.to_be() != constants::EVT_SUBTYPE
                {
                    return;
                }

                (header, data) = data.split_at(sdpcm::EventMessage::SIZE);
                let evt_msg = sdpcm::EventMessage::from_bytes(header);
                let Some(evt_type) = sdpcm::Event::from_u8(evt_msg.event_type.to_be() as _) else {
                    return;
                };

                const ESCAN_PARTIAL: u32 = 8;
                match evt_type {
                    sdpcm::Event::EscanResult if evt_msg.status.to_be() == ESCAN_PARTIAL => {
                        let Some(Pending::Scan) = self.pending.get() else {
                            return;
                        };
                        if data.len() < sdpcm::ScanResults::SIZE + sdpcm::BssInfo::SIZE {
                            return;
                        }
                        data = &data[sdpcm::ScanResults::SIZE..];
                        let bss_info = sdpcm::BssInfo::from_bytes(&data[..sdpcm::BssInfo::SIZE]);
                        if let Ok(mut ssid) = wifi::Ssid::try_new(bss_info.ssid_len) {
                            ssid.buf = bss_info.ssid;
                            self.client.map(|client| client.scanned_network(ssid));
                        }
                    }
                    sdpcm::Event::EscanResult => {
                        let Some(Pending::Scan) = self.pending.get() else {
                            return;
                        };
                        // TODO: Notify client that scanning is done
                        self.client.map(|client| client.scan_done());
                        self.pending.clear();
                    }
                    sdpcm::Event::SetSsid => {
                        let State::Command(Command::Join) = self.state.get() else {
                            return;
                        };
                        self.client.map(|client| {
                            client.command_done(if evt_msg.status == 0 {
                                Ok(())
                            } else {
                                Err(ErrorCode::FAIL)
                            });
                            self.tasks_done();
                        });
                    }
                    _ => (),
                }
            }
            // Data packets
            sdpcm::ChannelType::Data if self.receive.get() => {
                if data.len() < sdpcm::BdcHeader::SIZE {
                    return;
                }
                (_, data) = data.split_at(sdpcm::BdcHeader::SIZE);
                self.eth_client
                    .map(|client| client.received_frame(data, None));
            }
            _ => (),
        }
    }

    fn waiting_or_busy(&self) -> bool {
        if self.pending.is_some() {
            true
        } else {
            match self.bus.state().unwrap() {
                bus::State::Incoming => true,
                bus::State::Available(len) => {
                    self.bus
                        .read_bytes(self.buffer.take().unwrap(), len)
                        .unwrap();
                    true
                }
                bus::State::Idle => false,
            }
        }
    }

    fn tasks_done(&self) {
        let _ = self.state.take();
        self.ioctl_tasks.reset();
    }

    /// Do the current task and advance the task list
    fn update_task(&self) {
        let Some(task) = self.ioctl_tasks.get() else {
            if let State::Command(command) = self.state.get() {
                if let Command::Join = command {
                    // Here we wait for the `SET_SSID` event.
                    return;
                } else if let Command::Scan = command {
                    self.pending.set(Pending::Scan);
                }
                self.client.map(|client| client.command_done(Ok(())));
                self.tasks_done();
            }
            return;
        };

        match self.do_task(*task) {
            Ok(advance) => {
                if advance {
                    self.ioctl_tasks.advance();
                }
            }
            Err(err) => {
                if let State::Command(_) = self.state.take() {
                    self.client.map(|client| client.command_done(Err(err)));
                }
                self.tasks_done();
            }
        }
    }
}

impl<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>>
    hil::ethernet::EthernetAdapterDatapath<'a> for CYW4343x<'a, P, A, B>
{
    fn set_client(&self, client: &'a dyn hil::ethernet::EthernetAdapterDatapathClient) {
        self.eth_client.set(client)
    }

    fn enable_receive(&self) {
        self.receive.set(true);
    }

    fn disable_receive(&self) {
        self.receive.set(false);
    }

    fn transmit_frame(
        &self,
        frame_buffer: &'static mut [u8],
        len: u16,
        transmission_identifier: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u8])> {
        let State::Idle = self.state.get() else {
            return Err((ErrorCode::BUSY, frame_buffer));
        };

        if let Err(err) = self.send_bdc(&frame_buffer[..len as _]) {
            Err((err, frame_buffer))
        } else {
            self.eth_tx_data
                .set((transmission_identifier, frame_buffer));
            self.state.set(State::Ethernet);
            Ok(())
        }
    }
}

impl<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>> wifi::Device<'a>
    for CYW4343x<'a, P, A, B>
{
    fn set_client(&self, client: &'a dyn wifi::Client) {
        self.client.set(client);
    }

    fn init(&self) -> Result<(), kernel::ErrorCode> {
        let State::NotInit = self.state.get() else {
            return Err(ErrorCode::ALREADY);
        };

        // Start the process by powering down the chip
        self.pwr.clear();
        let now = self.alarm.now();
        self.alarm.set_alarm(now, self.alarm.ticks_from_ms(20));
        self.state.set(State::PoweredDown);

        Ok(())
    }

    fn mac(&self) -> Result<[u8; 6], ErrorCode> {
        self.mac.get().ok_or(ErrorCode::BUSY)
    }

    fn join(
        &self,
        ssid: wifi::Ssid,
        security: Option<(wifi::Security, wifi::Passphrase)>,
    ) -> Result<(), kernel::ErrorCode> {
        if let State::NotInit = self.state.get() {
            return Err(ErrorCode::FAIL);
        }

        if let Some((security, passphrase)) = security {
            match security {
                wifi::Security::Wpa => self.init_tasks(&ioctl::join_wpa::WPA1)?,
                wifi::Security::Wpa2 => self.init_tasks(&ioctl::join_wpa::WPA2)?,
                wifi::Security::Wpa2Wpa3 => {
                    self.init_tasks(&ioctl::join_wpa::WPA2_WPA3)?;
                }
                wifi::Security::Wpa3 => self.init_tasks(&ioctl::join_wpa::WPA3)?,
            }
            self.security.set(passphrase);
        } else {
            self.init_tasks(&ioctl::join_open::OPS)?;
        }
        self.ssid.set(ssid);
        self.state.set(State::Command(Command::Join));

        Ok(())
    }

    fn leave(&self) -> Result<(), kernel::ErrorCode> {
        if let State::NotInit = self.state.get() {
            return Err(ErrorCode::FAIL);
        }

        self.init_tasks(&[ioctl::leave::OP])?;
        self.state.set(State::Command(Command::Leave));
        Ok(())
    }

    fn scan(&self) -> Result<(), kernel::ErrorCode> {
        if let State::NotInit = self.state.get() {
            return Err(ErrorCode::FAIL);
        }

        self.init_tasks(&[ioctl::start_scan::OP])?;
        self.state.set(State::Command(Command::Scan));
        Ok(())
    }

    fn stop_scan(&self) -> Result<(), kernel::ErrorCode> {
        if let State::NotInit = self.state.get() {
            return Err(ErrorCode::FAIL);
        }

        self.init_tasks(&[ioctl::stop_scan::OP])?;
        self.state.set(State::Command(Command::StopScan));
        Ok(())
    }

    fn access_point(
        &self,
        ssid: wifi::Ssid,
        security: Option<(wifi::Security, wifi::Passphrase)>,
        channel: u8,
    ) -> Result<(), kernel::ErrorCode> {
        let (None | Some((wifi::Security::Wpa2, _))) = security else {
            return Err(ErrorCode::NOSUPPORT);
        };

        if let State::NotInit = self.state.get() {
            return Err(ErrorCode::FAIL);
        }

        if let Some((_, passphrase)) = security {
            self.init_tasks(&ioctl::start_ap_wpa::OPS)?;
            self.security.set(passphrase);
        } else {
            self.init_tasks(&ioctl::start_ap::OPS)?;
        }

        self.ssid.set(ssid);
        self.channel.set(channel);
        self.state.set(State::Command(Command::Ap));
        Ok(())
    }

    fn station(&self) -> Result<(), kernel::ErrorCode> {
        if let State::NotInit = self.state.get() {
            return Err(ErrorCode::FAIL);
        }

        self.init_tasks(&ioctl::stop_ap::OPS)?;
        self.state.set(State::Command(Command::Sta));
        Ok(())
    }
}

impl<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>> hil::time::AlarmClient
    for CYW4343x<'a, P, A, B>
{
    fn alarm(&self) {
        match self.state.get() {
            State::PoweredDown => {
                self.pwr.set();

                let now = self.alarm.now();
                self.alarm.set_alarm(now, self.alarm.ticks_from_ms(250));
                self.state.set(State::PoweredUp);
            }
            State::PoweredUp => {
                // Now we can start initialising the bus
                if let Err(err) = self.bus.init().map(|()| self.state.set(State::BusInit)) {
                    self.client.map(|client| client.command_done(Err(err)));
                }
            }
            State::Command(_) if !self.waiting_or_busy() => self.update_task(),
            // The driver alarm shouldn't fire in any other state
            _ => (),
        }
    }
}

impl<'a, P: hil::gpio::Pin, A: hil::time::Alarm<'a>, B: bus::CYW4343xBus<'a>> bus::CYW4343xBusClient
    for CYW4343x<'a, P, A, B>
{
    fn init_done(&self, rval: Result<(), ErrorCode>) {
        if let Err(err) = rval.and_then(|()| self.init_tasks(ioctl::init::OPS)) {
            self.client.map(|client| client.command_done(Err(err)));
            self.state.set(State::Idle);
        } else {
            self.state.set(State::Command(Command::Init));
        }
    }

    fn packet_available(&self, len: usize) {
        if len == 0 && !self.waiting_or_busy() {
            self.update_task();
        } else {
            self.bus
                .read_bytes(self.buffer.take().unwrap(), len)
                .unwrap();
        }
    }

    fn write_bytes_done(
        &self,
        mut buffer: SubSliceMut<'static, u8>,
        rval: Result<(), kernel::ErrorCode>,
    ) {
        reset_and_restore_bufs!(self, buffer);

        match (self.state.get(), rval) {
            (State::Command(_), Err(err)) => {
                self.client.map(|client| client.command_done(Err(err)));
                if let State::Command(Command::Init) = self.state.get() {
                    self.state.set(State::NotInit);
                } else {
                    self.state.set(State::Idle);
                }
            }
            (State::Command(_), Ok(())) if !self.waiting_or_busy() => {
                self.update_task();
            }
            (State::Ethernet, _) => todo!(),
            _ => {}
        }
    }

    fn read_bytes_done(
        &self,
        mut buffer: SubSliceMut<'static, u8>,
        rval: Result<(), kernel::ErrorCode>,
    ) {
        if rval.is_ok() {
            self.parse(buffer.as_mut_slice());
        }

        reset_and_restore_bufs!(self, buffer);
        if let State::Command(_) = self.state.get() {
            if !self.waiting_or_busy() {
                self.update_task()
            }
        }
    }
}

/// Configuring a WiFi functionality requires a set of IOCTL commands to be sent to the chip.
/// This module defines lists of IOCTL operations needed
/// for each functionality required by the WiFi device interface.
mod ioctl {
    use crate::cyw4343::sdpcm::{self, IoctlCommand};
    use core::cell::Cell;
    use kernel::utilities::cells::OptionalCell;

    #[derive(Default)]
    pub struct Tasks {
        list: OptionalCell<&'static [Op]>,
        idx: Cell<u8>,
    }

    impl Tasks {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn init(&self, list: &'static [Op]) {
            self.list.set(list);
            self.idx.set(0);
        }

        pub fn reset(&self) {
            self.list.clear();
            self.idx.set(0);
        }

        pub fn get(&self) -> Option<&Op> {
            self.list
                .map(|list| list.get(self.idx.get() as usize))
                .flatten()
        }

        pub fn advance(&self) {
            self.idx.set(self.idx.get() + 1)
        }

        pub fn is_empty(&self) -> bool {
            self.list.is_none()
        }
    }

    /// Driver task
    ///
    /// Commands sent to the WiFi chip (join a network, start AP mode, etc.)
    /// can be broken down as a list of IOCTL operations. In a few cases,
    /// a delay between operations is necessary. In order to keep the tasks in lists,
    /// the types of operations are aggregated in the [`Op`] enum.
    #[derive(Debug, Clone, Copy)]
    pub enum Op {
        /// Delay between IOCTLs
        WaitMs(u32),
        /// IOCTL operation
        Ioctl(Ioctl),
        /// Load CLM
        LoadCLM,
        /// Get the MAC address
        MacAddr,
    }

    impl Default for Op {
        fn default() -> Self {
            Self::WaitMs(0)
        }
    }

    impl Op {
        pub(super) const fn ioctl(cmd: IoctlCommand, data: IoctlData) -> Self {
            Self::Ioctl(Ioctl {
                cmd,
                data,
                name: None,
            })
        }

        pub(super) const fn iovar(name: sdpcm::Iovar, data: IoctlData) -> Self {
            Self::Ioctl(Ioctl {
                cmd: IoctlCommand::SetVar,
                data,
                name: Some(name),
            })
        }

        pub(super) const fn wait_ms(ms: u32) -> Self {
            Self::WaitMs(ms)
        }
    }

    /// IOCTL operation
    #[derive(Debug, Clone, Copy)]
    pub struct Ioctl {
        pub cmd: IoctlCommand,
        pub data: IoctlData,
        pub name: Option<sdpcm::Iovar>,
    }

    /// IOCTL data sources
    #[derive(Debug, Clone, Copy)]
    pub enum IoctlData {
        // Pre-configured
        Word([u8; 4]),
        DWord([u8; 8]),
        Empty,
        // These should be retrieved from the driver
        Ssid,
        BssSsid,
        Wpa1Passphrase,
        Wpa3Passphrase,
        Channel,
        // These are constants
        ScanParameters,
        AbortScanParameters,
        CountryInfo,
        EventMask,
    }

    impl IoctlData {
        pub const fn from_2xu32(val0: u32, val1: u32) -> Self {
            let mut bytes = [0u8; 8];
            let val0_bytes = val0.to_le_bytes();
            [bytes[0], bytes[1], bytes[2], bytes[3]] =
                [val0_bytes[0], val0_bytes[1], val0_bytes[2], val0_bytes[3]];
            let val1_bytes = val1.to_le_bytes();
            [bytes[4], bytes[5], bytes[6], bytes[7]] =
                [val1_bytes[0], val1_bytes[1], val1_bytes[2], val1_bytes[3]];

            Self::DWord(bytes)
        }

        pub const fn from_u32(val: u32) -> Self {
            Self::Word(val.to_le_bytes())
        }

        pub const fn empty() -> Self {
            Self::Empty
        }
    }

    //////////// Task lists

    pub mod init {
        use super::sdpcm::{CountryInfo, Event, EventMask, Iovar};
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};

        pub const COUNTRY_INFO: [u8; CountryInfo::SIZE] = CountryInfo {
            country_abbrev: [88, 88, 0, 0],
            country_code: [88, 88, 0, 0],
            rev: -1,
        }
        .into_bytes();

        pub const EVENTS: [u8; EventMask::SIZE] = EventMask::with_masked_evts(&[
            Event::Radio,
            Event::If,
            Event::ProbreqMsg,
            Event::ProbreqMsgRx,
            Event::Roam,
            Event::ProbreqMsg,
        ])
        .into_bytes();

        pub static OPS: &[Op] = &[
            Op::WaitMs(2000),
            Op::LoadCLM,
            Op::iovar(Iovar::BusTxGlom, Data::from_u32(0)),
            Op::iovar(Iovar::Apsta, Data::from_u32(1)),
            Op::MacAddr,
            Op::iovar(Iovar::Country, Data::CountryInfo),
            Op::WaitMs(100),
            Op::ioctl(Cmd::SetAntdiv, Data::from_u32(0)),
            Op::iovar(Iovar::BusTxGlom, Data::from_u32(0)),
            Op::WaitMs(100),
            Op::iovar(Iovar::AmpduBaWsize, Data::from_u32(8)),
            Op::WaitMs(100),
            Op::iovar(Iovar::AmpduMpdu, Data::from_u32(4)),
            Op::WaitMs(100),
            Op::iovar(Iovar::BssCfgEventMsgs, Data::EventMask),
            Op::WaitMs(100),
            Op::ioctl(Cmd::Up, Data::Empty),
            Op::WaitMs(100),
            Op::ioctl(Cmd::SetGmode, Data::from_u32(1)),
            Op::ioctl(Cmd::SetBand, Data::from_u32(0)),
            Op::WaitMs(100),
        ];
    }

    /// Access point (open) start tasks list
    pub mod start_ap {
        use super::sdpcm::Iovar;
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};

        pub static OPS: [Op; 9] = [
            Op::ioctl(Cmd::Down, Data::empty()),
            Op::iovar(Iovar::Apsta, Data::from_u32(0)),
            Op::ioctl(Cmd::Up, Data::empty()),
            Op::ioctl(Cmd::SetAp, Data::from_u32(1)),
            Op::iovar(Iovar::BssCfgSsid, Data::BssSsid),
            Op::ioctl(Cmd::SetChannel, Data::Channel),
            Op::iovar(Iovar::BssCfgWsec, Data::from_2xu32(0, 0)),
            Op::iovar(Iovar::G2Mrate, Data::from_u32(11000000 / 500000)),
            Op::iovar(Iovar::Bss, Data::from_2xu32(0, 1)),
        ];
    }

    /// Access point (WPA) start tasks list
    pub mod start_ap_wpa {
        use super::sdpcm::Iovar;
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};

        pub static OPS: [Op; 12] = [
            Op::ioctl(Cmd::Down, Data::empty()),
            Op::iovar(Iovar::Apsta, Data::from_u32(0)),
            Op::ioctl(Cmd::Up, Data::empty()),
            Op::ioctl(Cmd::SetAp, Data::from_u32(1)),
            Op::iovar(Iovar::BssCfgSsid, Data::BssSsid),
            Op::ioctl(Cmd::SetChannel, Data::Channel),
            Op::iovar(Iovar::BssCfgWsec, Data::from_2xu32(0, 0x4)),
            Op::iovar(Iovar::BssCfgWpaAuth, Data::from_2xu32(0, 0x084)),
            Op::wait_ms(100),
            Op::ioctl(Cmd::SetWsecPmk, Data::Wpa1Passphrase),
            Op::iovar(Iovar::G2Mrate, Data::from_u32(11000000 / 500000)),
            Op::iovar(Iovar::Bss, Data::from_2xu32(0, 1)),
        ];
    }

    /// Access point stop tasks list
    pub mod stop_ap {
        use super::sdpcm::Iovar;
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};

        pub static OPS: [Op; 5] = [
            Op::iovar(Iovar::Bss, Data::from_2xu32(0, 0)),
            Op::ioctl(Cmd::SetAp, Data::from_u32(0)),
            Op::ioctl(Cmd::Down, Data::empty()),
            Op::iovar(Iovar::Apsta, Data::from_u32(1)),
            Op::ioctl(Cmd::Up, Data::empty()),
        ];
    }

    /// Join open network tasks list
    pub mod join_open {
        use super::sdpcm::Iovar;
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};

        pub static OPS: [Op; 7] = [
            Op::iovar(Iovar::AmpduBaWsize, Data::from_u32(8)),
            Op::ioctl(Cmd::SetWsec, Data::from_u32(0)),
            Op::iovar(Iovar::BssCfgSupWpa, Data::from_2xu32(0, 0)),
            Op::ioctl(Cmd::SetInfra, Data::from_u32(1)),
            Op::ioctl(Cmd::SetAuth, Data::from_u32(0)),
            Op::ioctl(Cmd::SetWpaAuth, Data::from_u32(0)),
            Op::ioctl(Cmd::SetSsid, Data::Ssid),
        ];
    }

    /// Join secured network tasks list
    pub mod join_wpa {
        use super::sdpcm::Iovar;
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};
        use crate::cyw4343::constants;

        mod wpa1 {
            pub(super) const MFP: u32 = 0;
            pub(super) const AUTH: u32 = 0;
            pub(super) const WPA_AUTH: u32 = 0x4;
        }

        mod wpa2 {
            pub(super) const MFP: u32 = 1;
            pub(super) const AUTH: u32 = 0;
            pub(super) const WPA_AUTH: u32 = 0x80;
        }

        mod wpa3 {
            pub(super) const MFP: u32 = 2;
            pub(super) const AUTH: u32 = 3;
            pub(super) const WPA_AUTH: u32 = 0x40000;
        }

        const WPA1_SET: Op = Op::ioctl(Cmd::SetWsecPmk, Data::Wpa1Passphrase);
        const WPA3_SET: Op = Op::iovar(Iovar::SaePassword, Data::Wpa3Passphrase);
        const fn ops(wpa_set: Op, mfp: u32, auth: u32, wpa_auth: u32) -> [Op; 12] {
            [
                Op::iovar(Iovar::AmpduBaWsize, Data::from_u32(8)),
                Op::ioctl(Cmd::SetWsec, Data::from_u32(constants::WSEC_AES)),
                Op::iovar(Iovar::BssCfgSupWpa, Data::from_2xu32(0, 1)),
                Op::iovar(Iovar::BssCfgSupWpa2Eapver, Data::from_2xu32(0, 0xFFFF_FFFF)),
                Op::iovar(Iovar::BssCfgSupWpaTmo, Data::from_2xu32(0, 2500)),
                Op::wait_ms(110),
                wpa_set,
                Op::ioctl(Cmd::SetInfra, Data::from_u32(1)),
                Op::ioctl(Cmd::SetAuth, Data::from_u32(auth)),
                Op::iovar(Iovar::Mfp, Data::from_u32(mfp)),
                Op::ioctl(Cmd::SetWpaAuth, Data::from_u32(wpa_auth)),
                Op::ioctl(Cmd::SetSsid, Data::Ssid),
            ]
        }

        pub static WPA1: [Op; 12] = ops(WPA1_SET, wpa1::MFP, wpa1::AUTH, wpa1::WPA_AUTH);
        pub static WPA2: [Op; 12] = ops(WPA1_SET, wpa2::MFP, wpa2::AUTH, wpa2::WPA_AUTH);
        pub static WPA3: [Op; 12] = ops(WPA3_SET, wpa3::MFP, wpa3::AUTH, wpa3::WPA_AUTH);
        pub static WPA2_WPA3: [Op; 13] = [
            Op::iovar(Iovar::AmpduBaWsize, Data::from_u32(8)),
            Op::ioctl(Cmd::SetWsec, Data::from_u32(constants::WSEC_AES)),
            Op::iovar(Iovar::BssCfgSupWpa, Data::from_2xu32(0, 1)),
            Op::iovar(Iovar::BssCfgSupWpa2Eapver, Data::from_2xu32(0, 0xFFFF_FFFF)),
            Op::iovar(Iovar::BssCfgSupWpaTmo, Data::from_2xu32(0, 2500)),
            Op::wait_ms(110),
            WPA1_SET,
            WPA3_SET,
            Op::ioctl(Cmd::SetInfra, Data::from_u32(1)),
            Op::ioctl(Cmd::SetAuth, Data::from_u32(wpa3::AUTH)),
            Op::iovar(Iovar::Mfp, Data::from_u32(wpa2::MFP)),
            Op::ioctl(Cmd::SetWpaAuth, Data::from_u32(wpa3::WPA_AUTH)),
            Op::ioctl(Cmd::SetSsid, Data::Ssid),
        ];
    }

    /// Leave network task
    pub mod leave {
        use super::{IoctlCommand as Cmd, IoctlData as Data, Op};

        pub const OP: Op = Op::ioctl(Cmd::Disassoc, Data::empty());
    }

    /// Start scan task
    pub mod start_scan {
        use super::sdpcm::Iovar;
        use super::{IoctlData as Data, Op};
        use crate::cyw4343::{constants, sdpcm};

        pub const SCAN_PARAMS: [u8; sdpcm::ScanParams::SIZE] = sdpcm::ScanParams {
            version: 1,
            action: constants::WL_SCAN_ACTION_START,
            sync_id: 1,
            ssid_len: 0,
            ssid: [0; 32],
            bssid: [0xff; 6],
            bss_type: 2,
            scan_type: constants::SCANTYPE_PASSIVE,
            nprobes: !0,
            active_time: !0,
            passive_time: !0,
            home_time: !0,
            channel_num: 0,
            channel_list: 0,
        }
        .into_bytes();

        pub const OP: Op = Op::iovar(Iovar::Escan, Data::ScanParameters);
    }

    /// Stop scan task
    pub mod stop_scan {
        use super::sdpcm::Iovar;
        use super::{IoctlData as Data, Op};
        use crate::cyw4343::{constants, sdpcm};

        pub const SCAN_PARAMS: [u8; sdpcm::ScanParams::SIZE] = sdpcm::ScanParams {
            version: 1,
            action: constants::WL_SCAN_ACTION_ABORT,
            sync_id: 0,
            ssid_len: 0,
            ssid: [0; 32],
            bssid: [0x00; 6],
            bss_type: 0,
            scan_type: 0,
            nprobes: 0,
            active_time: 0,
            passive_time: 0,
            home_time: 0,
            channel_num: 0,
            channel_list: 0,
        }
        .into_bytes();
        pub const OP: Op = Op::iovar(Iovar::Escan, Data::AbortScanParameters);
    }
}
