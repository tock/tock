#![no_std]
//! Interface crate between Tock and the third party library Rubble.
//!
//! This implements the [`kernel::hil::rubble`] interface using the Rubble library.
use kernel::hil::{
    rubble::{self as rubble_hil, BleHardware, RubbleCmd, RubbleImplementation},
    time::Alarm,
};

use core::marker::PhantomData;
use rubble::{
    bytes::{ByteReader, FromBytes},
    config::Config,
    link::{
        ad_structure::AdStructure, queue::PacketQueue, AddressKind, Cmd, DeviceAddress, LinkLayer,
        NextUpdate, RadioCmd, Responder, Transmitter,
    },
    time::Duration,
};
use rubble_hil::{RubbleBleRadio, RubbleLinkLayer, RubblePacketQueue, RubbleResponder};
use timer::TimerWrapper;

mod refcell_packet_queue;
mod timer;

static RX_PACKET_QUEUE: refcell_packet_queue::RefCellQueue =
    refcell_packet_queue::RefCellQueue::new();
static TX_PACKET_QUEUE: refcell_packet_queue::RefCellQueue =
    refcell_packet_queue::RefCellQueue::new();

#[derive(Default)]
pub struct Implementation<'a, A, H>(PhantomData<(&'a A, H)>)
where
    A: Alarm<'a>,
    H: BleHardware;

impl<'a, A, H> RubbleImplementation<'a, A> for Implementation<'a, A, H>
where
    A: Alarm<'a>,
    H: BleHardware,
{
    type BleRadio = BleRadioWrapper<H>;
    type LinkLayer = LinkLayerWrapper<'a, A, H>;
    type Responder = ResponderWrapper<'a, A, H>;
    type Cmd = CmdWrapper;
    type PacketQueue = refcell_packet_queue::RefCellQueue;
    fn get_device_address() -> rubble_hil::DeviceAddress {
        H::get_device_address()
    }
    fn rx_packet_queue() -> &'static Self::PacketQueue {
        &RX_PACKET_QUEUE
    }
    fn tx_packet_queue() -> &'static Self::PacketQueue {
        &TX_PACKET_QUEUE
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
struct RubbleConfig<'a, A, H>
where
    H: BleHardware,
    A: Alarm<'a>,
{
    radio: PhantomData<H>,
    alarm: PhantomData<&'a A>,
}

impl<'a, A, H> Config for RubbleConfig<'a, A, H>
where
    A: Alarm<'a>,
    H: BleHardware,
{
    type Timer = self::timer::TimerWrapper<'a, A>;
    type Transmitter = HilToRubbleTransmitterWrapper<H::Transmitter>;
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
    fn into_radio_cmd(self) -> rubble_hil::RadioCmd {
        match self.0.radio {
            RadioCmd::Off => rubble_hil::RadioCmd::Off,
            RadioCmd::ListenAdvertising { channel } => rubble_hil::RadioCmd::ListenAdvertising {
                channel: rubble_hil::AdvertisingChannel::new(channel.channel()).unwrap(),
            },
            RadioCmd::ListenData {
                channel,
                access_address,
                crc_init,
                timeout,
            } => rubble_hil::RadioCmd::ListenData {
                channel: rubble_hil::DataChannel::new(channel.index()).unwrap(),
                access_address,
                crc_init,
                timeout,
            },
        }
    }
}

pub struct BleRadioWrapper<H: BleHardware>(HilToRubbleTransmitterWrapper<H::Transmitter>);

impl<H> BleRadioWrapper<H>
where
    H: BleHardware,
{
    pub fn new(transmitter: H::Transmitter) -> Self {
        BleRadioWrapper(HilToRubbleTransmitterWrapper(transmitter))
    }
}

impl<'a, A, H> RubbleBleRadio<'a, A, Implementation<'a, A, H>> for BleRadioWrapper<H>
where
    A: Alarm<'a>,
    H: BleHardware,
{
    fn accept_cmd(&mut self, cmd: rubble_hil::RadioCmd) {
        H::radio_accept_cmd(&mut (self.0).0, cmd)
    }
}

struct HilToRubbleTransmitterWrapper<T>(T);

impl<T> Transmitter for HilToRubbleTransmitterWrapper<T>
where
    T: rubble_hil::Transmitter,
{
    fn tx_payload_buf(&mut self) -> &mut [u8] {
        self.0.tx_payload_buf()
    }
    fn transmit_advertising(
        &mut self,
        header: rubble::link::advertising::Header,
        channel: rubble::phy::AdvertisingChannel,
    ) {
        self.0.transmit_advertising(
            rubble_hil::AdvertisingHeader::from_bytes(header.to_u16().to_le_bytes()),
            // never panics: rubble_hil::AdvertisingChannel requires same list
            // channels as rubble::phy::AdvertisingChannel
            rubble_hil::AdvertisingChannel::new(channel.channel()).unwrap(),
        )
    }
    fn transmit_data(
        &mut self,
        access_address: u32,
        crc_iv: u32,
        header: rubble::link::data::Header,
        channel: rubble::phy::DataChannel,
    ) {
        self.0.transmit_data(
            access_address,
            crc_iv,
            rubble_hil::DataHeader::from_bytes(header.to_u16().to_le_bytes()),
            // never panics: rubble_hil::DataChannel requires same list
            // channels as rubble::phy::DataChannel
            rubble_hil::DataChannel::new(channel.index()).unwrap(),
        )
    }
}

pub struct ResponderWrapper<'a, A, H>(Responder<RubbleConfig<'a, A, H>>)
where
    A: Alarm<'a>,
    H: BleHardware;

impl<'a, A, H> RubbleResponder<'a, A, Implementation<'a, A, H>> for ResponderWrapper<'a, A, H>
where
    A: Alarm<'a>,
    H: BleHardware,
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

pub struct LinkLayerWrapper<'a, A, H>(LinkLayer<RubbleConfig<'a, A, H>>)
where
    A: Alarm<'a>,
    H: BleHardware;

impl<'a, A, H> RubbleLinkLayer<'a, A, Implementation<'a, A, H>> for LinkLayerWrapper<'a, A, H>
where
    A: Alarm<'a>,
    // this line shouldn't be required, but is
    // Implementation<'a, A, H>: RubbleImplementation<'a, A, RadioCmd = RadioCmd, Cmd = CmdWrapper>,
    H: BleHardware,
{
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
    type Error = rubble::Error;
    fn start_advertise(
        &mut self,
        interval: rubble_hil::Duration,
        data: &[u8],
        transmitter: &mut BleRadioWrapper<H>,
        tx: refcell_packet_queue::RefCellConsumer<'static>,
        rx: refcell_packet_queue::RefCellProducer<'static>,
    ) -> Result<rubble_hil::NextUpdate, Self::Error> {
        // This is inefficient, as we're parsing then directly turning back into
        // bytes. However, this is a minimal-effort code to comply with Rubble's
        // interface.

        // A more efficient method might be to write a more minimal parser, and
        // just directly create `AdStructure::Unknown` instances.
        let mut ad_byte_reader = ByteReader::new(data);

        // TODO: this should work, but doesn't. Debug it (rather than only
        // allowing one ad structure).
        // // Note: advertising data is at most 31 octets, and each ad must be at
        // // least 2 octets, so we have at most 16 adstructures.
        // let mut ad_structures = [AdStructure::Unknown { ty: 0, data: &[] }; 16];
        // let mut num_ads = 0;
        // while ad_byte_reader.bytes_left() > 0 {
        //     let ad = AdStructure::from_bytes(&mut ad_byte_reader)?;
        //     ad_structures[num_ads] = ad;
        //     num_ads += 1;
        //     if num_ads > 16 {
        //         return Err(Error::InvalidLength);
        //     }
        // }
        // let ad_structures = &ad_structures[..num_ads];
        let ad_structures = &[AdStructure::from_bytes(&mut ad_byte_reader)?];

        let res = self.0.start_advertise(
            Duration::from_micros(interval.as_micros()),
            ad_structures,
            &mut transmitter.0,
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
    fn update_timer(&mut self, tx: &mut BleRadioWrapper<H>) -> CmdWrapper {
        CmdWrapper::new(self.0.update_timer(&mut tx.0))
    }
}
