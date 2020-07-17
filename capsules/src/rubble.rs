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
//! TODO:
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * SUCCESS: The buffer has successfully been filled
//! * ENOMEM: No sufficient memory available
//! * EINVAL: Invalid address of the buffer or other error
//! * EBUSY: The driver is currently busy with other tasks
//! * ENOSUPPORT: The operation is not supported
//! * ERROR: Operation `map` on Option failed
//!
//! ### Subscribe system call
//!
//! N/A
//!
//! ### Command system call
//!
//! The `command` system call supports two arguments, `command number` and
//! `subcommand number`.
//!
//! We use `command number` to specify one of the following operations:
//!
//! - 0: stop advertising
//! - 181: start advertising
//! - 2: start scanning
//! - TODO: connections??

use core::{cell::RefCell, marker::PhantomData};
use kernel::debug;
use kernel::hil::rubble::BleRadio;
use kernel::ReturnCode;

use rubble::{
    config::Config,
    link::{
        ad_structure::AdStructure,
        queue::{PacketQueue, SimpleQueue},
        LinkLayer, NextUpdate, Responder,
    },
    time::Duration,
};

mod timer;

/// Syscall driver number.
use crate::driver;
use timer::RubbleTimer;
pub const DRIVER_NUM: usize = driver::NUM::RubbleBle as usize;

/// Process specific memory
pub struct App;
// {
//     adv_data: Option<kernel::AppSlice<kernel::Shared, u8>>,
//     advertisement_interval_ms: u32,
// }

impl Default for App {
    fn default() -> App {
        App
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

    pub fn start_advertising(&self) {
        use rubble::time::Timer;

        let data = &mut *self.mutable_data.borrow_mut();
        debug!("Starting advertising.");
        // TODO: this is unsound.
        let (_tx, tx_cons) = unsafe { &mut TX_QUEUE }.split();
        let (rx_prod, _rx) = unsafe { &mut RX_QUEUE }.split();
        // errors if we provide too much ad data.
        debug!("Alarm gives time {}", self.alarm.now());
        debug!(
            "Rubble interpreted timer gives time {}",
            data.ll.timer().now()
        );
        let next_update = data
            .ll
            .start_advertise(
                Duration::from_millis(200),
                &[AdStructure::CompleteLocalName("Tock Full Rubble")],
                &mut data.radio,
                tx_cons,
                rx_prod,
            )
            .unwrap();
        debug!("Done. Going to set alarm to {:?}", next_update);

        self.set_alarm_for(next_update);
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
    fn command(
        &self,
        command_num: usize,
        data: usize,
        interval: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {
            // Boilerplate kept to use shortly.
            0 => self
                .app
                .enter(appid, |app, _| ReturnCode::SUCCESS)
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
