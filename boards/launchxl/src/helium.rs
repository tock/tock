#![allow(unused_imports)]

use cc26x2::commands as cmd;
use cc26x2::{osc, rfc, rtc};
use core::cell::Cell;
use fixedvec::FixedVec;
use kernel::common::cells::TakeCell;
use kernel::{AppId, Callback, Driver, ReturnCode};
use kernel::hil::radio_client;
// static mut PAYLOAD: [u8; 256] = [0; 256];
#[derive(Debug, Clone, Copy)]
pub enum HeliumState {
    NotInitialized,
    Idle(radio_client::PowerMode),
    Pending(radio_client::RadioOperation),
    Done,
    Invalid,
}

pub enum PowerMode {
    Active,
    Sleep,
    DeepSleep,
}

pub static mut RFC_STACK: [State; 6] = [State::Start; 6];

#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
struct AlarmData {
    t0: u32,
    expiration: Expiration,
}

impl AlarmData {
    fn new() -> AlarmData {
        AlarmData {
            t0: 0,
            expiration: Expiration::Disabled,
        }
    }
}

#[derive(Default)]
pub struct App {
    process_status: Option<State>,
    alarm_data: AlarmData,
    callback: Option<Callback>,
}

impl Default for App {
    fn default() -> App {
        App {
            process_status: Some(State::NotInitialized),
            alarm_data: AlarmData::new(),
            callback: None,
        }
    }
}

impl App {
    fn initialize_radio<'a, R, A>(&self, radio: &Helium<'a, R, A>) -> ReturnCode  
    where
        R: radio_client::Radio,
        A: kernel::hil::time::Alarm,
    {
        
    }
    
    // Set the next alarm for this app using the period and provided start time.
    fn set_next_alarm<F: Frequency>(&mut self, now: u32) {
        self.alarm_data.t0 = now;
        let period_ms = (self.advertisement_interval_ms + nonce) * F::frequency() / 1000;
        self.alarm_data.expiration = Expiration::Abs(now.wrapping_add(period_ms));
    }
}

pub struct Helium<'a, R, A> 
where
    R: radio_client::Radio,
    A: kernel::hil::time::Alarm,
{
    radio: &'a R,
    alarm: &'a A,
    app: kernel::Grant<App>,
    state_stack: TakeCell<'static, FixedVec<'static, State>>,
    callback: Cell<Option<Callback>>,
}

impl<R, A> Helium <'a, R, A> {
where 
    R: radio_client::Radio,
    A: kernel::hil::time::Alarm,
    pub fn new(radio: &'a R, alarm: &'a A, container: kernel::Grant<App>) -> Helium<'a, R, A> {
        let rfc_stack =
            unsafe { static_init!(FixedVec<'static, State>, FixedVec::new(&mut RFC_STACK)) };
        debug_assert_eq!(rfc_stack.len(), 0);
        rfc_stack
            .push(State::NotInitialized)
            .expect("Rfc stack should be empty");

        Helium {
            radio: radio,
            alarm: alarm,
            app: container,
            state_stack: TakeCell::new(rfc_stack),
            callback: Cell::new(None), 
        }
    }
    
}

// Timer alarm
impl<B, A> kernel::hil::time::Client for Helium<'a, B, A>
where
    B: radio_client::Radio,
    A: kernel::hil::time::Alarm,
{
    // When an alarm is fired, we find which apps have expired timers. Expired
    // timers indicate a desire to perform some operation (e.g. start an
    // advertising or scanning event). We know which operation based on the
    // current app's state.
    //
    // In case of collision---if there is already an event happening---we'll
    // just delay the operation for next time and hope for the best. Since some
    // randomness is added for each period in an app's timer, collisions should
    // be rare in practice.
    //
    // TODO: perhaps break ties more fairly by prioritizing apps that have least
    // recently performed an operation.
    fn fired(&self) {
        let now = self.alarm.now();

        self.app.each(|app| {
            if let Expiration::Abs(exp) = app.alarm_data.expiration {
                let expired =
                    now.wrapping_sub(app.alarm_data.t0) >= exp.wrapping_sub(app.alarm_data.t0);
                if expired {
                    if self.busy.get() {
                        // The radio is currently busy, so we won't be able to start the
                        // operation at the appropriate time. Instead, reschedule the
                        // operation for later. This is _kind_ of simulating actual
                        // on-air interference
                        debug!("BLE: operation delayed for app {:?}", app.appid());
                        app.set_next_alarm::<A::Frequency>(self.alarm.now());
                        return;
                    }

                    app.alarm_data.expiration = Expiration::Disabled;

                    match app.process_status {
                        Some(HeliumState::Start) => {
                            self.busy.set(true);
                            app.process_status = Some(HeliumState::Idle);
                            self.sending_app.set(app.appid());
                            self.radio.set_power_mode(app.power_mode);
                            app.radio_setup(&self, app.radio_config);
                        }
                        Some(HeliumState::Idle) => {
                            self.busy.set(true);
                            app.process_status =
                                Some(HeliumState::;
                            self.receiving_app.set(app.appid());
                            self.radio.set_tx_power(app.tx_power);
                            self.radio
                                .receive_advertisement(RadioChannel::AdvertisingChannel37);
                        }
                        _ => debug!(
                            "app: {:?} \t invalid state {:?}",
                            app.appid(),
                            app.process_status
                        ),
                    }
                }
            }
        });
        self.reset_active_alarm();
    }
}

impl Driver for Helium {
    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<Callback>,
        _appid: AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Callback for RFC Interrupt ready
            0 => {
                self.callback.set(callback);
                return ReturnCode::SUCCESS;
            }
            // Default
            _ => return ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, minor_num: usize, _r2: usize, _r3: usize, _caller_id: AppId) -> ReturnCode {
        let command_status: RfcOperationStatus = minor_num.into();

        match command_status {
            // Handle callback for CMDSTA after write to CMDR
            RfcOperationStatus::SendDone => {
                let current_command = self.pop_cmd();
                self.push_state(State::CommandStatus(command_status));
                match self.rfc.cmdsta() {
                    ReturnCode::SUCCESS => {
                        ReturnCode::SUCCESS
                    }
                    ReturnCode::EBUSY => {
                        ReturnCode::EBUSY
                    }
                    ReturnCode::EINVAL => {
                        self.pop_state();
                        ReturnCode::EINVAL
                    }
                    _ => {
                        self.pop_state();
                        ReturnCode::ENOSUPPORT
                    }
                }
            }
            // Handle callback for command status after command is finished
            RfcOperationStatus::CommandDone => {
                // let current_command = self.rfc.command.as_ptr() as u32;
                let current_command = self.pop_cmd();
                self.push_state(State::CommandStatus(command_status));
                match self.rfc.wait(&current_command) {
                    // match self.rfc.wait_cmdr(current_command) {
                    ReturnCode::SUCCESS => {
                        self.pop_state();
                        ReturnCode::SUCCESS
                    }
                    ReturnCode::EBUSY => {
                        ReturnCode::EBUSY
                    }
                    ReturnCode::ECANCEL => {
                        self.pop_state();
                        ReturnCode::ECANCEL
                    }
                    ReturnCode::FAIL => {
                        self.pop_state();
                        ReturnCode::FAIL
                    }
                    _ => {
                        self.pop_state();
                        ReturnCode::ENOSUPPORT
                    }
                }
            }
            RfcOperationStatus::Invalid => panic!("Invalid command status"),
            _ => panic!("Unimplemented!"),
        }
    }
}

impl From<usize> for RfcOperationStatus {
    fn from(val: usize) -> RfcOperationStatus {
        match val {
            0 => RfcOperationStatus::Idle,
            1 => RfcOperationStatus::Pending,
            2 => RfcOperationStatus::Active,
            3 => RfcOperationStatus::Skipped,
            4 => RfcOperationStatus::SendDone,
            5 => RfcOperationStatus::TxDone,
            6 => RfcOperationStatus::CommandDone,
            7 => RfcOperationStatus::LastCommandDone,
            8 => RfcOperationStatus::RxOk,
            9 => RfcOperationStatus::TxDone,
            val => {
                debug_assert!(false, "{} does not represent a valid command.", val);
                RfcOperationStatus::Invalid
            }
        }
    }
}
