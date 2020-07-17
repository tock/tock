//! Bluetooth Low Energy Driver
//!
//! System call driver for Bluetooth Low Energy advertising and connections.
//! This driver supports a single application controlling the bluetooth stack,
//! and having full control over connections and advertisements made.
//!
//! ### Allow system call
//!
//! The allow system calls are used to hand over buffers allocated by userland.
//!
//! - 0: give buffer to be read for outgoing advertisement data
//!
//!   These are the raw bytes to send out as the advertisement. Can be
//!   constructed nicely using rubble::link::ad_structure::AdStructure.
//!
//!   TODO: maybe a way to do this nicer? could use C structures to let us do
//!   rubble computations?
//! * 1: give buffer to be written to for incoming scanning data
//!
//! ### Subscribe system call
//!
//! * 0: subscribe to advertisement scanning data
//!
//! ### Command system call
//!
//! The `command` system call supports two arguments, `command number` and
//! `subcommand number`.
//!
//! We use `command number` to specify one of the following operations:
//!
//! - 0: start or restart advertising
//!
//!   First argument is the advertisement interval in milliseconds. Second argument should be 0.
//!
//!   This will initiate advertising using the current contents of the "outgoing
//!   advertisement data" buffer set using ALLOW 0. If the contents change, this
//!   must be run again to restart advertising the new data.
//!
//!   Will return EFAIL if no advertising buffer has been given via ALLOW 0, or
//!   EINVAL if the advertising data currently present in said buffer is invalid.
//! - 1: stop advertising
//!
//!   Both arguments should be 0.
//! - 2: start or restart scanning
//!
//!   First argument is scanning interval in milliseconds. Second argument should be 0.
//! - 3: stop scanning
//!
//!   Both arguments should be 0.
//! - TODO: scanning
//! - TODO: connections??
mod timer;

use core::{cell::RefCell, convert::TryInto, marker::PhantomData};
use kernel::debug;
use kernel::hil::rubble::BleRadio;
use kernel::{AppId, AppSlice, Callback, ReturnCode, Shared};

use rubble::{
    bytes::{ByteReader, FromBytes},
    config::Config,
    link::{
        ad_structure::AdStructure,
        queue::{PacketQueue, SimpleQueue},
        LinkLayer, NextUpdate, Responder,
    },
    time::Duration,
};

use crate::driver;

use self::timer::RubbleTimer;

// Syscall driver number.

pub const DRIVER_NUM: usize = driver::NUM::RubbleBle as usize;

// Command Consants
pub const CMD_START_ADVERTISING: usize = 0;
pub const CMD_STOP_ADVERTISING: usize = 1;
pub const CMD_START_SCANNING: usize = 2;
pub const CMD_STOP_SCANNING: usize = 3;

pub const CMD_ARG_UNUSED: usize = 0;

pub const ALLOW_OUTGOING_AD_BUFFER: usize = 0;
pub const ALLOW_INCOMING_SCANNING_DATA: usize = 1;

/// Process specific memory
pub struct App {
    outgoing_advertisement_data: Option<kernel::AppSlice<kernel::Shared, u8>>,
    incoming_scanning_data: Option<kernel::AppSlice<kernel::Shared, u8>>,
    advertisement_interval: Duration,
    scan_interval_ms: Duration,
}

impl Default for App {
    fn default() -> App {
        App {
            outgoing_advertisement_data: None,
            incoming_scanning_data: None,
            advertisement_interval: Duration::from_millis(200),
            scan_interval_ms: Duration::from_millis(200),
        }
    }
}

#[derive(Default)]
struct RubbleConfig<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    radio: PhantomData<R>,
    alarm: PhantomData<&'a A>,
}

impl<'a, R, A> Config for RubbleConfig<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    type Timer = self::timer::RubbleTimer<'a, A>;
    type Transmitter = R::Transmitter;
    type ChannelMapper =
        rubble::l2cap::BleChannelMap<rubble::att::NoAttributes, rubble::security::NoSecurity>;
    type PacketQueue = &'static mut SimpleQueue;
}

static mut TX_QUEUE: SimpleQueue = SimpleQueue::new();
static mut RX_QUEUE: SimpleQueue = SimpleQueue::new();

struct MutableBleData<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    radio: R::Transmitter,
    ll: LinkLayer<RubbleConfig<'a, R, A>>,
    responder: Responder<RubbleConfig<'a, R, A>>,
}

pub struct BLE<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    mutable_data: RefCell<MutableBleData<'a, R, A>>,
    app: kernel::Grant<App>,
    alarm: &'a A,
}

impl<'a, R, A> BLE<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    pub fn new(container: kernel::Grant<App>, radio: R::Transmitter, alarm: &'a A) -> Self {
        // Determine device address
        let device_address = R::get_device_address();
        debug!("Hello! I'm {:?}", device_address);

        // TODO: this is emulating a rtic pattern, and I don't think it's
        // currently sound as we're doing it.
        let (tx, _tx_cons) = unsafe { &mut TX_QUEUE }.split();
        let (_rx_prod, rx) = unsafe { &mut RX_QUEUE }.split();

        let ll = LinkLayer::new(device_address, RubbleTimer::new(alarm));
        let responder = Responder::new(
            tx,
            rx,
            rubble::l2cap::L2CAPState::new(rubble::l2cap::BleChannelMap::with_attributes(
                rubble::att::NoAttributes,
            )),
        );
        BLE {
            mutable_data: RefCell::new(MutableBleData {
                radio,
                ll,
                responder,
            }),
            app: container,
            alarm,
        }
    }

    pub fn start_advertising(&self, app: &mut App) -> Result<(), ReturnCode> {
        let data = &mut *self.mutable_data.borrow_mut();
        debug!("Starting advertising with app.");
        // TODO: this is unsound.
        let (_tx, tx_cons) = unsafe { &mut TX_QUEUE }.split();
        let (rx_prod, _rx) = unsafe { &mut RX_QUEUE }.split();
        // errors if we provide too much ad data.

        let ad_bytes = app
            .outgoing_advertisement_data
            .as_ref()
            .ok_or(ReturnCode::FAIL)?
            .as_ref();
        let ad = AdStructure::from_bytes(&mut ByteReader::new(ad_bytes)).map_err(|e| {
            debug!("Error converting app adv bytes to AdStructure: {}", e);
            ReturnCode::EINVAL
        })?;

        let next_update = data
            .ll
            .start_advertise(
                app.advertisement_interval,
                &[ad],
                &mut data.radio,
                tx_cons,
                rx_prod,
            )
            .unwrap();
        debug!("Done. Going to set alarm to {:?}", next_update);

        self.set_alarm_for(next_update);
        Ok(())
    }

    pub fn stop_advertising(&self) -> Result<(), ReturnCode> {
        let data = &mut *self.mutable_data.borrow_mut();
        if data.ll.is_advertising() {
            data.ll.enter_standby();
            Ok(())
        } else {
            Err(ReturnCode::EALREADY)
        }
    }

    pub fn set_alarm_for(&self, update: NextUpdate) {
        match update {
            NextUpdate::Keep => {}
            NextUpdate::Disable => {
                debug!("Disabling alarm.");
                self.alarm.disable()
            }
            NextUpdate::At(time) => {
                let tock_time = self::timer::rubble_instant_to_alarm_time::<A>(&self.alarm, time);
                debug!(
                    "Setting alarm for at {} (we're now at {})",
                    tock_time,
                    self.alarm.now()
                );
                self.alarm.set_alarm(tock_time);
            }
        }
    }
}

// Timer alarm
impl<'a, R, A> kernel::hil::time::AlarmClient for BLE<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    fn fired(&self) {
        debug!("Alarm fired");
        let data = &mut *self.mutable_data.borrow_mut();

        let cmd = data.ll.update_timer(&mut data.radio);
        debug!("Got cmd: {:?}", cmd);
        R::radio_accept_cmd(&mut data.radio, cmd.radio);
        debug!("Radio accepted cmd");
        if cmd.queued_work {
            // TODO: do this some time more appropriate? It should be in an
            // idle loop, when other things don't need to be done.
            while data.responder.has_work() {
                debug!("Working on queued work");
                // unwrap: we've just checked we have work, so we can't reach Eof.
                data.responder.process_one().unwrap();
                debug!("Did one queued work");
            }
        }
        self.set_alarm_for(cmd.next_update);
    }
}

// System Call implementation
impl<'a, R, A> kernel::Driver for BLE<'a, R, A>
where
    R: BleRadio,
    A: kernel::hil::time::Alarm<'a>,
{
    fn command(&self, command_num: usize, r2: usize, r3: usize, app_id: AppId) -> ReturnCode {
        match command_num {
            CMD_START_ADVERTISING => {
                let advertisement_interval_ms = r2;
                assert_eq!(r3, CMD_ARG_UNUSED);

                self.app
                    .enter(app_id, |app, _alloc| {
                        app.advertisement_interval = Duration::from_millis(
                            advertisement_interval_ms
                                .try_into()
                                .map_err(|_| ReturnCode::EINVAL)?,
                        );
                        self.start_advertising(app)?;
                        Ok(ReturnCode::SUCCESS)
                    })
                    .unwrap_or_else(|err| Err(err.into()))
                    .unwrap_or_else(|e| e)
            }
            CMD_STOP_ADVERTISING => {
                assert_eq!(r2, CMD_ARG_UNUSED);
                assert_eq!(r3, CMD_ARG_UNUSED);

                match self.stop_advertising() {
                    Ok(()) => ReturnCode::SUCCESS,
                    Err(e) => e,
                }
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, minor_num: usize, callback: Option<Callback>, app_id: AppId) -> ReturnCode {
        ReturnCode::ENOSUPPORT
    }

    fn allow(
        &self,
        app_id: AppId,
        minor_num: usize,
        slice: Option<AppSlice<Shared, u8>>,
    ) -> ReturnCode {
        match minor_num {
            ALLOW_OUTGOING_AD_BUFFER => self
                .app
                .enter(app_id, |app, _alloc| {
                    app.outgoing_advertisement_data = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
