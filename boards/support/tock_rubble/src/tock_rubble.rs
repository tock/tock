//! Interface crate between Tock and the third party library Rubble.
//!
//! This implements the [`kernel::hil::rubble`] interface using the Rubble
//! library.
//!
//! The naming can be confusing in this adapter because the name "rubble" is
//! used on both sides (in the actual Rubble library and in the kernel
//! interfaces). To try to keep the Rust types separated, `rubble::` corresponds
//! to the Rubble library, and `hil::rubble` corresponds to the interfaces in
//! Tock.

use core::marker::PhantomData;

// Include traits so we can call the included functions.
use rubble::bytes::FromBytes;
use rubble::bytes::ToBytes;
use rubble::link::queue::PacketQueue;

use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::hil::ble_advertising;
use kernel::hil::time;
use kernel::ReturnCode;

use crate::refcell_packet_queue;
use crate::timer_wrapper;

/// A packet buffer that can hold header and payload of any advertising or data
/// channel packet supported by rubble.
pub type PacketBuffer = [u8; rubble::link::MIN_PDU_BUF];

static RX_PACKET_QUEUE: refcell_packet_queue::RefCellQueue =
    refcell_packet_queue::RefCellQueue::new();
static TX_PACKET_QUEUE: refcell_packet_queue::RefCellQueue =
    refcell_packet_queue::RefCellQueue::new();

/// The main struct that provides the glue between Tock and Rubble.
#[derive(Default)]
pub struct TockRubble<'a, A, R>(PhantomData<(&'a A, R)>)
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a;

impl<'a, A, R> hil::rubble::RubbleStack<'a, A> for TockRubble<'a, A, R>
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    type BleRadio = BleRadioWrapper<'a, R>;
    type LinkLayer = LinkLayerWrapper<'a, A, R>;
    type Responder = ResponderWrapper<'a, A, R>;
    type Cmd = CmdWrapper;
    type PacketQueue = refcell_packet_queue::RefCellQueue;

    fn get_device_address() -> hil::rubble::types::DeviceAddress {
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
        _rx_end: hil::rubble::types::Instant,
        buf: &'static mut [u8],
        _result: ReturnCode,
    ) {
        assert_eq!(buf.len(), rubble::link::MIN_PDU_BUF);
        radio.tx_buf.replace(buf);
    }

    fn receive_event(
        radio: &mut Self::BleRadio,
        ll: &mut Self::LinkLayer,
        rx_end: hil::rubble::types::Instant,
        buf: &'static mut [u8],
        _len: u8,
        _result: ReturnCode,
    ) -> CmdWrapper {
        let rx_end = rubble::time::Instant::from_raw_micros(rx_end.raw_micros());
        let cmd = match radio.currently_receiving {
            CurrentlyReceiving::Advertisement => {
                let header = rubble::link::advertising::Header::parse(&buf[..2]);
                // TODO: do we actually check the CRC in the radio
                // driver? If we do, this should be documented. If we don't,
                // then this code is incorrect.
                ll.0.process_adv_packet(rx_end, radio, header, &buf[2..], true)
            }
            CurrentlyReceiving::Data => {
                let header = rubble::link::data::Header::parse(&buf[..2]);
                ll.0.process_data_packet(rx_end, radio, header, &buf[2..], true)
            }
        };
        CmdWrapper::new(cmd)
    }
}

impl kernel::hil::rubble::RubblePacketQueue for refcell_packet_queue::RefCellQueue {
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
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    radio: PhantomData<R>,
    alarm: PhantomData<&'a A>,
}

impl<'a, A, R> rubble::config::Config for RubbleConfig<'a, A, R>
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    type Timer = timer_wrapper::TimerWrapper<'a, A>;
    type Transmitter = BleRadioWrapper<'a, R>;
    type ChannelMapper =
        rubble::l2cap::BleChannelMap<rubble::att::NoAttributes, rubble::security::NoSecurity>;
    type PacketQueue = &'static refcell_packet_queue::RefCellQueue;
}

#[derive(Debug)]
pub struct CmdWrapper(rubble::link::Cmd);

impl CmdWrapper {
    fn new(cmd: rubble::link::Cmd) -> Self {
        CmdWrapper(cmd)
    }
}

impl kernel::hil::rubble::RubbleCmd for CmdWrapper {
    type RadioCmd = rubble::link::RadioCmd;
    fn next_update(&self) -> hil::rubble::types::NextUpdate {
        match self.0.next_update {
            rubble::link::NextUpdate::At(time) => hil::rubble::types::NextUpdate::At(
                hil::rubble::types::Instant::from_raw_micros(time.raw_micros()),
            ),
            rubble::link::NextUpdate::Disable => hil::rubble::types::NextUpdate::Disable,
            rubble::link::NextUpdate::Keep => hil::rubble::types::NextUpdate::Keep,
        }
    }

    fn queued_work(&self) -> bool {
        self.0.queued_work
    }

    fn into_radio_cmd(self) -> rubble::link::RadioCmd {
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
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    radio: &'a R,
    currently_receiving: CurrentlyReceiving,
    tx_buf: TakeCell<'static, [u8]>,
    _lifetime_phantom: PhantomData<&'a ()>,
}

impl<'a, R> BleRadioWrapper<'a, R>
where
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
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

impl<'a, A, R> kernel::hil::rubble::RubbleBleRadio<'a, A, TockRubble<'a, A, R>>
    for BleRadioWrapper<'a, R>
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    fn accept_cmd(&mut self, _cmd: rubble::link::RadioCmd) {
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

impl<'a, R> rubble::link::Transmitter for BleRadioWrapper<'a, R>
where
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
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
            .to_bytes(&mut rubble::bytes::ByteWriter::new(&mut tx_buf[..2]))
            .unwrap();
        let len = usize::from(header.payload_length()) + 2;
        // panic safety: AdvertisingChannel's allowed ints is a subset of
        // allowed ints
        let channel = kernel::hil::ble_advertising::RadioChannel::from_channel_index(
            channel.channel().into(),
        )
        .unwrap();
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
            .to_bytes(&mut rubble::bytes::ByteWriter::new(&mut tx_buf[..2]))
            .unwrap();
        // panic safety: DataChannel's allowed ints is a subset of
        // allowed ints
        let channel =
            kernel::hil::ble_advertising::RadioChannel::from_channel_index(channel.index().into())
                .unwrap();
        self.radio
            .transmit_data(tx_buf, access_address, crc_iv, channel)
    }
}

pub struct ResponderWrapper<'a, A, R>(rubble::link::Responder<RubbleConfig<'a, A, R>>)
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a;

impl<'a, A, R> kernel::hil::rubble::RubbleResponder<'a, A, TockRubble<'a, A, R>>
    for ResponderWrapper<'a, A, R>
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    type Error = rubble::Error;
    fn new(
        tx: refcell_packet_queue::RefCellProducer<'static>,
        rx: refcell_packet_queue::RefCellConsumer<'static>,
    ) -> Self {
        ResponderWrapper(rubble::link::Responder::new(
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

pub struct LinkLayerWrapper<'a, A, R>(rubble::link::LinkLayer<RubbleConfig<'a, A, R>>)
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a;

impl<'a, A, R> kernel::hil::rubble::RubbleLinkLayer<'a, A, TockRubble<'a, A, R>>
    for LinkLayerWrapper<'a, A, R>
where
    A: time::Alarm<'a>,
    R: hil::rubble::radio::RubbleData<'a>
        + ble_advertising::BleAdvertisementDriver<'a>
        + ble_advertising::BleConfig
        + 'a,
{
    type Error = rubble::Error;

    fn new(device_address: hil::rubble::types::DeviceAddress, alarm: &'a A) -> Self {
        LinkLayerWrapper(rubble::link::LinkLayer::new(
            rubble::link::DeviceAddress::new(
                device_address.bytes,
                match device_address.kind {
                    hil::rubble::types::AddressKind::Public => rubble::link::AddressKind::Public,
                    hil::rubble::types::AddressKind::Random => rubble::link::AddressKind::Random,
                },
            ),
            timer_wrapper::TimerWrapper::new(alarm),
        ))
    }

    fn start_advertise(
        &mut self,
        interval: hil::rubble::types::Duration,
        data: &[u8],
        transmitter: &mut BleRadioWrapper<'a, R>,
        tx: refcell_packet_queue::RefCellConsumer<'static>,
        rx: refcell_packet_queue::RefCellProducer<'static>,
    ) -> Result<hil::rubble::types::NextUpdate, Self::Error> {
        // This is inefficient, as we're parsing then directly turning back into
        // bytes. However, this is a minimal-effort code to comply with Rubble's
        // interface.

        // A more efficient method might be to write a more minimal parser, and
        // just directly create `AdStructure::Unknown` instances.
        let mut ad_byte_reader = rubble::bytes::ByteReader::new(data);

        // Note: advertising data is at most 31 octets, and each ad must be at
        // least 2 octets, so we have at most 16 adstructures.
        // TODO: allocating this on the stack might not be the most wise.
        let mut ad_structures =
            [rubble::link::ad_structure::AdStructure::Unknown { ty: 0, data: &[] }; 16];
        let mut num_ads = 0;
        while ad_byte_reader.bytes_left() > 0 {
            let ad = rubble::link::ad_structure::AdStructure::from_bytes(&mut ad_byte_reader)?;
            ad_structures[num_ads] = ad;
            num_ads += 1;
            if num_ads > 16 {
                return Err(rubble::Error::InvalidLength);
            }
        }
        let ad_structures = &ad_structures[..num_ads];

        let res = self.0.start_advertise(
            rubble::time::Duration::from_micros(interval.as_micros()),
            ad_structures,
            transmitter,
            tx,
            rx,
        );
        res.map(|next_update| match next_update {
            rubble::link::NextUpdate::Keep => hil::rubble::types::NextUpdate::Keep,
            rubble::link::NextUpdate::Disable => hil::rubble::types::NextUpdate::Disable,
            rubble::link::NextUpdate::At(time) => hil::rubble::types::NextUpdate::At(
                hil::rubble::types::Instant::from_raw_micros(time.raw_micros()),
            ),
        })
    }
    fn is_advertising(&self) -> bool {
        self.0.is_advertising()
    }
    fn update_timer(&mut self, tx: &mut BleRadioWrapper<'a, R>) -> CmdWrapper {
        CmdWrapper::new(self.0.update_timer(tx))
    }
}
