#![no_std]
//! Interface crate between Tock and the third party library Rubble.
//!
//! This implements the [`kernel::hil::rubble`] interface using the Rubble library.
use kernel::{
    common::cells::TakeCell,
    hil::{
        ble_advertising::{BleAdvertisementDriver, BleConfig, RadioChannel},
        rubble::{self as rubble_hil, RubbleCmd, RubbleDataDriver, RubbleImplementation},
        time::Alarm,
    },
    ReturnCode,
};

use core::marker::PhantomData;
use rubble::{
    bytes::{ByteReader, ByteWriter, FromBytes, ToBytes},
    config::Config,
    link::{
        ad_structure::AdStructure, advertising, data, queue::PacketQueue, AddressKind, Cmd,
        DeviceAddress, LinkLayer, NextUpdate, RadioCmd, Responder, Transmitter,
    },
    time::{Duration, Instant},
    Error,
};
use rubble_hil::{RubbleBleRadio, RubbleLinkLayer, RubblePacketQueue, RubbleResponder};
use timer::TimerWrapper;

/// A packet buffer that can hold header and payload of any advertising or data
/// channel packet supported by rubble.
pub type PacketBuffer = [u8; MIN_PDU_BUF];

pub use rubble::link::MIN_PDU_BUF;

mod refcell_packet_queue;
mod timer;

static RX_PACKET_QUEUE: refcell_packet_queue::RefCellQueue =
    refcell_packet_queue::RefCellQueue::new();
static TX_PACKET_QUEUE: refcell_packet_queue::RefCellQueue =
    refcell_packet_queue::RefCellQueue::new();

#[derive(Default)]
pub struct Implementation<'a, A, R>(PhantomData<(&'a A, R)>)
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a;

impl<'a, A, R> RubbleImplementation<'a, A> for Implementation<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    type BleRadio = BleRadioWrapper<'a, R>;
    type LinkLayer = LinkLayerWrapper<'a, A, R>;
    type Responder = ResponderWrapper<'a, A, R>;
    type Cmd = CmdWrapper;
    type PacketQueue = refcell_packet_queue::RefCellQueue;
    fn get_device_address() -> rubble_hil::DeviceAddress {
        R::get_device_address()
    }
    fn rx_packet_queue() -> &'static Self::PacketQueue {
        &RX_PACKET_QUEUE
    }
    fn tx_packet_queue() -> &'static Self::PacketQueue {
        &TX_PACKET_QUEUE
    }
    fn transmit_event(
        radio: &mut Self::BleRadio,
        _ll: &mut Self::LinkLayer,
        _rx_end: rubble_hil::Instant,
        buf: &'static mut [u8],
        _result: ReturnCode,
    ) {
        assert_eq!(buf.len(), MIN_PDU_BUF);
        radio.tx_buf.replace(buf);
    }
    fn receive_event(
        radio: &mut Self::BleRadio,
        ll: &mut Self::LinkLayer,
        rx_end: rubble_hil::Instant,
        buf: &'static mut [u8],
        _len: u8,
        _result: ReturnCode,
    ) -> CmdWrapper {
        let rx_end = Instant::from_raw_micros(rx_end.raw_micros());
        let cmd = match radio.currently_receiving {
            CurrentlyReceiving::Advertisement => {
                let header = advertising::Header::parse(&buf[..2]);
                // TODO: do we actually check the CRC in the radio
                // driver? If we do, this should be documented. If we don't,
                // then this code is incorrect.
                ll.0.process_adv_packet(rx_end, radio, header, &buf[2..], true)
            }
            CurrentlyReceiving::Data => {
                let header = data::Header::parse(&buf[..2]);
                ll.0.process_data_packet(rx_end, radio, header, &buf[2..], true)
            }
        };
        CmdWrapper::new(cmd)
    }
}

impl RubblePacketQueue for refcell_packet_queue::RefCellQueue {
    type Producer = refcell_packet_queue::RefCellProducer<'static>;
    type Consumer = refcell_packet_queue::RefCellConsumer<'static>;

    fn split(
        &'static self,
    ) -> (
        refcell_packet_queue::RefCellProducer<'static>,
        refcell_packet_queue::RefCellConsumer<'static>,
    ) {
        // forward to the concrete non-trait method
        <&'static refcell_packet_queue::RefCellQueue>::split(self)
    }
}

#[derive(Default)]
struct RubbleConfig<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    radio: PhantomData<R>,
    alarm: PhantomData<&'a A>,
}

impl<'a, A, R> Config for RubbleConfig<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    type Timer = self::timer::TimerWrapper<'a, A>;
    type Transmitter = BleRadioWrapper<'a, R>;
    type ChannelMapper =
        rubble::l2cap::BleChannelMap<rubble::att::NoAttributes, rubble::security::NoSecurity>;
    type PacketQueue = &'static refcell_packet_queue::RefCellQueue;
}

#[derive(Debug)]
pub struct CmdWrapper(Cmd);

impl CmdWrapper {
    fn new(cmd: Cmd) -> Self {
        CmdWrapper(cmd)
    }
}

impl RubbleCmd for CmdWrapper {
    type RadioCmd = RadioCmd;
    fn next_update(&self) -> rubble_hil::NextUpdate {
        match self.0.next_update {
            NextUpdate::At(time) => {
                rubble_hil::NextUpdate::At(rubble_hil::Instant::from_raw_micros(time.raw_micros()))
            }
            NextUpdate::Disable => rubble_hil::NextUpdate::Disable,
            NextUpdate::Keep => rubble_hil::NextUpdate::Keep,
        }
    }
    fn queued_work(&self) -> bool {
        self.0.queued_work
    }
    fn into_radio_cmd(self) -> RadioCmd {
        self.0.radio
    }
}

enum CurrentlyReceiving {
    Advertisement,
    #[allow(unused)]
    Data,
}

pub struct BleRadioWrapper<'a, R>
where
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    radio: &'a R,
    currently_receiving: CurrentlyReceiving,
    tx_buf: TakeCell<'static, [u8]>,
    _lifetime_phantom: PhantomData<&'a ()>,
}

impl<'a, R> BleRadioWrapper<'a, R>
where
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    pub fn new(radio: &'a R, tx_buf: &'static mut PacketBuffer) -> Self {
        BleRadioWrapper {
            radio,
            currently_receiving: CurrentlyReceiving::Advertisement,
            tx_buf: TakeCell::new(tx_buf),
            _lifetime_phantom: PhantomData,
        }
    }
}

impl<'a, A, R> RubbleBleRadio<'a, A, Implementation<'a, A, R>> for BleRadioWrapper<'a, R>
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    fn accept_cmd(&mut self, _cmd: RadioCmd) {
        // TODO: The current NRF52840 ble_radio driver doesn't support
        // simultaneous sending & receiving data in the same way that
        // rubble-nrf5x does. So, until we fix either that or rubble's
        // expectations, just disable listening.

        // match cmd {
        //     RadioCmd::Off => {
        //         // This is currently unsupported.
        //     }
        //     RadioCmd::ListenAdvertising { channel } => {
        //         // panic safety: AdvertisingChannel's allowed ints is a subset of
        //         // allowed ints
        //         let channel = RadioChannel::from_channel_index(channel.channel().into()).unwrap();
        //         self.currently_receiving = CurrentlyReceiving::Advertisement;
        //         self.radio.receive_advertisement(channel);
        //     }
        //     RadioCmd::ListenData {
        //         channel,
        //         access_address,
        //         crc_init,
        //         timeout: _,
        //     } => {
        //         // panic safety: DataChannel's allowed ints is a subset of
        //         // allowed ints
        //         let channel = RadioChannel::from_channel_index(channel.index().into()).unwrap();
        //         self.currently_receiving = CurrentlyReceiving::Data;
        //         self.radio.receive_data(channel, access_address, crc_init);
        //     }
        // }
    }
}

impl<'a, R> Transmitter for BleRadioWrapper<'a, R>
where
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    fn tx_payload_buf(&mut self) -> &mut [u8] {
        // TODO: To fully comply with what the rubble stack expects, we would
        // need to block here until the transmission is done. This should be
        // fixed on rubble's side.
        let full_buf = self
            .tx_buf
            .get_mut()
            .expect("tx_payload_buf called while transmission ongoing");
        &mut full_buf[2..]
    }

    fn transmit_advertising(
        &mut self,
        header: rubble::link::advertising::Header,
        channel: rubble::phy::AdvertisingChannel,
    ) {
        let tx_buf = self
            .tx_buf
            .take()
            .expect("transmit_advertising called while transmission ongoing");
        header
            .to_bytes(&mut ByteWriter::new(&mut tx_buf[..2]))
            .unwrap();
        let len = usize::from(header.payload_length()) + 2;
        // panic safety: AdvertisingChannel's allowed ints is a subset of
        // allowed ints
        let channel = RadioChannel::from_channel_index(channel.channel().into()).unwrap();
        self.radio.set_tx_power(0);
        self.radio.transmit_advertisement(tx_buf, len, channel);
    }
    fn transmit_data(
        &mut self,
        access_address: u32,
        crc_iv: u32,
        header: rubble::link::data::Header,
        channel: rubble::phy::DataChannel,
    ) {
        let tx_buf = self
            .tx_buf
            .take()
            .expect("transmit_data called while transmission ongoing");
        header
            .to_bytes(&mut ByteWriter::new(&mut tx_buf[..2]))
            .unwrap();
        // panic safety: DataChannel's allowed ints is a subset of
        // allowed ints
        let channel = RadioChannel::from_channel_index(channel.index().into()).unwrap();
        self.radio
            .transmit_data(tx_buf, access_address, crc_iv, channel)
    }
}

pub struct ResponderWrapper<'a, A, R>(Responder<RubbleConfig<'a, A, R>>)
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a;

impl<'a, A, R> RubbleResponder<'a, A, Implementation<'a, A, R>> for ResponderWrapper<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    type Error = rubble::Error;
    fn new(
        tx: refcell_packet_queue::RefCellProducer<'static>,
        rx: refcell_packet_queue::RefCellConsumer<'static>,
    ) -> Self {
        ResponderWrapper(Responder::new(
            tx,
            rx,
            rubble::l2cap::L2CAPState::new(rubble::l2cap::BleChannelMap::with_attributes(
                rubble::att::NoAttributes,
            )),
        ))
    }

    fn has_work(&mut self) -> bool {
        self.0.has_work()
    }
    fn process_one(&mut self) -> Result<(), Self::Error> {
        self.0.process_one()
    }
}

pub struct LinkLayerWrapper<'a, A, R>(LinkLayer<RubbleConfig<'a, A, R>>)
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a;

impl<'a, A, R> RubbleLinkLayer<'a, A, Implementation<'a, A, R>> for LinkLayerWrapper<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleDataDriver<'a> + BleAdvertisementDriver<'a> + BleConfig + 'a,
{
    type Error = rubble::Error;

    fn new(device_address: rubble_hil::DeviceAddress, alarm: &'a A) -> Self {
        LinkLayerWrapper(LinkLayer::new(
            DeviceAddress::new(
                device_address.bytes,
                match device_address.kind {
                    rubble_hil::AddressKind::Public => AddressKind::Public,
                    rubble_hil::AddressKind::Random => AddressKind::Random,
                },
            ),
            TimerWrapper::new(alarm),
        ))
    }

    fn start_advertise(
        &mut self,
        interval: rubble_hil::Duration,
        data: &[u8],
        transmitter: &mut BleRadioWrapper<'a, R>,
        tx: refcell_packet_queue::RefCellConsumer<'static>,
        rx: refcell_packet_queue::RefCellProducer<'static>,
    ) -> Result<rubble_hil::NextUpdate, Self::Error> {
        // This is inefficient, as we're parsing then directly turning back into
        // bytes. However, this is a minimal-effort code to comply with Rubble's
        // interface.

        // A more efficient method might be to write a more minimal parser, and
        // just directly create `AdStructure::Unknown` instances.
        let mut ad_byte_reader = ByteReader::new(data);

        // Note: advertising data is at most 31 octets, and each ad must be at
        // least 2 octets, so we have at most 16 adstructures.
        // TODO: allocating this on the stack might not be the most wise.
        let mut ad_structures = [AdStructure::Unknown { ty: 0, data: &[] }; 16];
        let mut num_ads = 0;
        while ad_byte_reader.bytes_left() > 0 {
            let ad = AdStructure::from_bytes(&mut ad_byte_reader)?;
            ad_structures[num_ads] = ad;
            num_ads += 1;
            if num_ads > 16 {
                return Err(Error::InvalidLength);
            }
        }
        let ad_structures = &ad_structures[..num_ads];

        let res = self.0.start_advertise(
            Duration::from_micros(interval.as_micros()),
            ad_structures,
            transmitter,
            tx,
            rx,
        );
        res.map(|next_update| match next_update {
            NextUpdate::Keep => rubble_hil::NextUpdate::Keep,
            NextUpdate::Disable => rubble_hil::NextUpdate::Disable,
            NextUpdate::At(time) => {
                rubble_hil::NextUpdate::At(rubble_hil::Instant::from_raw_micros(time.raw_micros()))
            }
        })
    }
    fn is_advertising(&self) -> bool {
        self.0.is_advertising()
    }
    fn update_timer(&mut self, tx: &mut BleRadioWrapper<'a, R>) -> CmdWrapper {
        CmdWrapper::new(self.0.update_timer(tx))
    }
}
