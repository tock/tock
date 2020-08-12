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
//!
//!   Advertising is currently hardcoded to connectable advertisements, with no
//!   control for what happens when something does connect.

use core::{cell::RefCell, convert::TryInto, marker::PhantomData};
use kernel::debug;
use kernel::hil::{
    rubble::{
        Duration, NextUpdate, RubbleBleRadio, RubbleCmd, RubbleImplementation, RubbleLinkLayer,
        RubblePacketQueue, RubbleResponder,
    },
    time::Alarm,
};
use kernel::{AppId, AppSlice, ReturnCode, Shared};

use crate::driver;

// Syscall driver number.

pub const DRIVER_NUM: usize = driver::NUM::RubbleBle as usize;

// Command Consants
pub const CMD_START_ADVERTISING: usize = 0;
pub const CMD_STOP_ADVERTISING: usize = 1;

pub const CMD_ARG_UNUSED: usize = 0;

pub const ALLOW_OUTGOING_AD_BUFFER: usize = 0;

/// Process specific memory
pub struct App {
    outgoing_advertisement_data: Option<kernel::AppSlice<kernel::Shared, u8>>,
    advertisement_interval: Duration,
}

impl Default for App {
    fn default() -> App {
        App {
            outgoing_advertisement_data: None,
            advertisement_interval: Duration::from_millis(200),
        }
    }
}

struct MutableBleData<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleImplementation<'a, A>,
{
    radio: R::BleRadio,
    ll: R::LinkLayer,
    responder: R::Responder,
    _phantom_timer: PhantomData<&'a A>,
}

impl<'a, A, R> MutableBleData<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleImplementation<'a, A>,
{
    pub fn handle_cmd(&mut self, cmd: R::Cmd) -> NextUpdate {
        let queued_work = cmd.queued_work();
        let next_update = cmd.next_update();
        self.radio.accept_cmd(cmd.into_radio_cmd());
        if queued_work {
            // TODO: do this some time more appropriate? It should be in an
            // idle loop, when other things don't need to be done.
            while self.responder.has_work() {
                // unwrap: we've just checked we have work, so we can't reach Eof.
                self.responder.process_one().unwrap();
            }
        }
        next_update
    }
}

pub struct BLE<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleImplementation<'a, A>,
{
    mutable_data: RefCell<MutableBleData<'a, A, R>>,
    app: kernel::Grant<App>,
    alarm: &'a A,
}

impl<'a, A, R> BLE<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleImplementation<'a, A>,
{
    pub fn new(container: kernel::Grant<App>, radio: R::BleRadio, alarm: &'a A) -> Self {
        // Determine device address
        let device_address = R::get_device_address();

        let (tx, _tx_cons) = <R as RubbleImplementation<'a, A>>::tx_packet_queue().split();
        let (_rx_prod, rx) = <R as RubbleImplementation<'a, A>>::rx_packet_queue().split();

        let ll = R::LinkLayer::new(device_address, alarm);
        let responder = R::Responder::new(tx, rx);
        BLE {
            mutable_data: RefCell::new(MutableBleData {
                radio,
                ll,
                responder,
                _phantom_timer: PhantomData,
            }),
            app: container,
            alarm,
        }
    }

    pub fn start_advertising(&self, app: &mut App) -> Result<(), ReturnCode> {
        let data = &mut *self.mutable_data.borrow_mut();
        let (_tx, tx_cons) = <R as RubbleImplementation<'a, A>>::tx_packet_queue().split();
        let (rx_prod, _rx) = <R as RubbleImplementation<'a, A>>::rx_packet_queue().split();

        // this errors if we provide too much ad data.

        let ad_bytes = app
            .outgoing_advertisement_data
            .as_ref()
            .ok_or(ReturnCode::FAIL)?
            .as_ref();
        let next_update = data
            .ll
            .start_advertise(
                app.advertisement_interval,
                &ad_bytes,
                &mut data.radio,
                tx_cons,
                rx_prod,
            )
            .map_err(|e| {
                debug!("Error advertising with app ad data: {}", e);
                ReturnCode::EINVAL
            })?;

        self.set_alarm_for(next_update);
        Ok(())
    }

    pub fn set_alarm_for(&self, update: NextUpdate) {
        match update {
            NextUpdate::Keep => {}
            NextUpdate::Disable => self.alarm.disable(),
            NextUpdate::At(time) => self.alarm.set_alarm(time.to_alarm_time(self.alarm)),
        }
    }
}

// Timer alarm
impl<'a, A, R> kernel::hil::time::AlarmClient for BLE<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleImplementation<'a, A>,
{
    fn fired(&self) {
        let data = &mut *self.mutable_data.borrow_mut();

        let cmd = data.ll.update_timer(&mut data.radio);
        let next_update = data.handle_cmd(cmd);
        self.set_alarm_for(next_update);
    }
}

// System Call implementation
impl<'a, A, R> kernel::Driver for BLE<'a, A, R>
where
    A: Alarm<'a>,
    R: RubbleImplementation<'a, A>,
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
            _ => ReturnCode::ENOSUPPORT,
        }
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
