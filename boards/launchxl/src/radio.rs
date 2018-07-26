#![allow(unused_imports)]

use cc26x2::{ rfc, rtc, osc };
use cc26x2::commands as cmd;
use kernel::common::cells::TakeCell;
use fixedvec::FixedVec;
use core::cell::Cell;
use kernel::{ Callback, ReturnCode, Driver, AppId };

static mut RFPARAMS: [u32; 18] = [
    // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x02400403,
    // Synth: Set minimum RTRIM to 6
    0x00068793,
    // Synth: Configure extra PLL filtering
    0x001C8473,
    // Synth: Configure extra PLL filtering
    0x00088433,
    // Synth: Set Fref to 4 MHz
    0x000684A3,
    // Synth: Configure faster calibration
    // HW32_ARRAY_OVERRIDE(0x4004,1),
    // Synth: Configure faster calibration
    0x180C0618,
    // Synth: Configure faster calibration
    0xC00401A1,
    // Synth: Configure faster calibration
    0x00010101,
    // Synth: Configure faster calibration
    0xC0040141,
    // Synth: Configure faster calibration
    0x00214AD3,
    // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x02980243,
    // Synth: Set loop bandwidth after lock to 20 kHz
    0x0A480583,
    // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603,
    // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623,
    // Tx: Configure PA ramping, set wait time before turning off (0x1F ticks of 16/24 us = 20.7 us).
    // HW_REG_OVERRIDE(0x6028,0x001F),
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1)
    // ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1)
    // ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    // Rx: Set AGC reference level to 0x1A (default: 0x2E)
    // HW_REG_OVERRIDE(0x609C,0x001A),
    // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x00018883,
    // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    0x000288A3,
    // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD)
    // ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    // TX power override
    // DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0xFFFC08C3,
    // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8)
    // ADI_REG_OVERRIDE(0,12,0xF8),
    0xFFFFFFFF,
];

// static mut PAYLOAD: [u8; 256] = [0; 256];
#[derive(Debug, Clone, Copy)]
pub enum State {
    Start,
    Pending,
    CommandStatus(RfcOperationStatus),
    Command(RadioCommands),
    Done,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum RfcOperationStatus {
    Idle,
    Pending,
    Active,
    Skipped,
    SendDone,
    CommandDone,
    LastCommandDone,
    RxOk,
    TxDone,
    Setup,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum RadioCommands {
    Direct(cmd::DirectCommand),
    RadioSetup(cmd::CmdRadioSetup),
    NoOp(cmd::CmdNop),
    FSPowerup(cmd::CmdFSPowerup),
    FSPowerdown(cmd::CmdFSPowerdown),
    StartRat(cmd::CmdSyncStartRat),
    StopRat(cmd::CmdSyncStopRat),
    NotSupported,
}

pub static mut CMD_STACK: [RadioCommands; 6] = [
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
];

impl Default for RadioCommands {
    fn default() -> RadioCommands {
        RadioCommands::NoOp(cmd::CmdNop::new())
    }
}
pub static mut RFC_STACK: [State; 6] = [State::Start; 6];

pub struct Radio {
    rfc: &'static rfc::RFCore,
    cmd_stack: TakeCell<'static, FixedVec<'static, RadioCommands>>,
    state_stack: TakeCell<'static, FixedVec<'static, State>>,
    callback: Cell<Option<Callback>>,
}

impl Radio {
    pub fn new(rfc: &'static rfc::RFCore) -> Radio {
        let rfc_stack = unsafe { static_init!(
            FixedVec<'static, State>,
            FixedVec::new( &mut RFC_STACK )
        )};

        let cmd_stack = unsafe { static_init!(
            FixedVec<'static, RadioCommands>,
            FixedVec::new( &mut CMD_STACK )
        )};
        debug_assert_eq!(rfc_stack.len(), 0);
        rfc_stack
            .push(State::Start)
            .expect("Rfc stack should be empty");
        debug_assert_eq!(cmd_stack.len(), 0);

        Radio {
            rfc,
            cmd_stack: TakeCell::new(cmd_stack),
            state_stack: TakeCell::new(rfc_stack),
            callback: Cell::new(None),
        }
    }

    pub fn power_up(&self) {
        
        self.rfc.set_mode(rfc::RfcMode::PROPRF);
        
        osc::OSC.hfosc_config(osc::SCLKHFSRC::XOSC_HF);

        self.rfc.enable();
        
        self.rfc.start_rat();

        osc::OSC.hfosc_config(osc::SCLKHFSRC::XOSC_HF);

        unsafe {
            let reg_overrides: u32 = RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides, 0x9F3F)
        }
    }

    pub fn power_down(&self) {
        self.rfc.disable();
    }
    
    // Functions for pushing and popping the radio state from the state stack
    fn push_state(&self, state: State) {
        let state_stack = self
            .state_stack
            .take()
            .expect("self.state_stack must be some here");
        state_stack.push(state).expect("self.state_stack is full");
        self.state_stack.replace(state_stack);
    }

    fn pop_state(&self) -> State {
        let state_stack = self
            .state_stack
            .take()
            .expect("self.state_stack must be some here");
        let state = state_stack.pop().expect("self.state_stack is empty");
        self.state_stack.replace(state_stack);
        state
    }

    // Functions for pushing and popping radio commands from the command stack
    fn push_cmd(&self, cmd: RadioCommands) {
        let cmd_stack = self
            .cmd_stack
            .take()
            .expect("self.cmd_stack must be some here");
        cmd_stack.push(cmd).expect("self.cmd_stack is full");
        self.cmd_stack.replace(cmd_stack);
    }

    fn pop_cmd(&self) -> RadioCommands {
        let cmd_stack = self
            .cmd_stack
            .take()
            .expect("self.cmd_stack must be some here");
        let cmd = cmd_stack.pop().expect("self.cmd_stack is empty");
        self.cmd_stack.replace(cmd_stack);
        cmd
    }
}

impl rfc::RFCoreClient for Radio {
    fn send_command_done(&self) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(cmd::RfcOperationStatus::SendDone as usize, 0, 0));
    }
/*
    fn last_command_done(&self) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(cmd::RfcOperationStatus::LastCommandDone as usize, 0, 0));
    }
*/
    fn wait_command_done(&self) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(cmd::RfcOperationStatus::CommandDone as usize, 0, 0));
    }
/*
    fn tx_done(&self) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(cmd::RfcOperationStatus::TxDone as usize, 0, 0));
    }

    fn rx_ok(&self) {
        self.callback
            .get()
            .map(|mut cb| cb.schedule(cmd::RfcOperationStatus::RxOk as usize, 0, 0));
    }
    */
}

impl Driver for Radio {
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
                        self.push_cmd(current_command);
                        ReturnCode::SUCCESS
                    }
                    ReturnCode::EBUSY => {
                        self.push_cmd(current_command);
                        ReturnCode::EBUSY
                    }
                    ReturnCode::EINVAL => {
                        self.pop_state();
                        ReturnCode::EINVAL
                    }
                    _ => {
                        self.pop_state();
                        self.pop_cmd();
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
                        self.push_cmd(current_command);
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
