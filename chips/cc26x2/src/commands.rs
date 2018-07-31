#![allow(dead_code)]

// Radio and data commands bitfields
bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcTrigger(u8);
    impl Debug;
    pub _trigger_type, _set_trigger_type : 3, 0;
    pub _enable_cmd, _set_enable_cmd      : 4;
    pub _trigger_no, _set_trigger_no      : 6, 5;
    pub _past_trigger, _set_past_trigger  : 7;
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcCondition(u8);
    impl Debug;
    pub _rule, set_rule : 3, 0;
    pub _skip, _set_skip : 7, 4;
}

bitfield! {
    #[derive(Copy, Clone)]
    pub struct RfcSetupConfig(u16);
    impl Debug;
    pub _frontend_mode, set_frontend_mode: 2, 0;
    pub _bias_mode, set_bias_mode: 3;
    pub _analog_cfg_mode, _set_analog_config_mode: 9, 4;
    pub _no_fs_powerup, _set_no_fs_powerup: 10;
}

// Radio Command Operation Status

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

// Radio Commands

// RFC Immediate commands
pub const RFC_CMD0: u16 = 0x801;
pub const RFC_PING: u16 = 0x406;
pub const RFC_BUS_REQUEST: u16 = 0x40E;
pub const RFC_START_RAT_TIMER: u16 = 0x0405;
pub const RFC_STOP_RAT_TIMER: u16 = 0x0809;
pub const RFC_SETUP: u16 = 0x0802;
pub const RFC_STOP: u16 = 0x0402;
pub const RFC_FS_POWERDOWN: u16 = 0x080D;

#[derive(Debug, Clone, Copy)]
pub struct DirectCommand {
    pub command_no: u16,
    pub params: u16,
}

impl DirectCommand {
    pub const fn new(command_no: u16, params: u16) -> DirectCommand {
        DirectCommand { command_no, params }
    }
}

// Common command header for all radio commands
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdCommon {
    command_no: u16,
    pub status: u16,
    p_next_op: u32,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
}

impl CmdCommon {
    pub fn new(
        command_no: u16,
        status: u16,
        p_next_op: u32,
        start_time: u32,
        start_trigger: u8,
        condition: RfcCondition,
    ) -> CmdCommon {
        CmdCommon {
            command_no,
            status,
            p_next_op,
            start_time,
            start_trigger,
            condition,
        }
    }
}
// Command and parameters for radio setup

pub unsafe trait RadioCommand {
    fn pack(&self, common: CmdCommon) -> Self;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RadioSetup {
    common: CmdCommon,
    mode: u8,
    io_divider: u8,
    config: RfcSetupConfig,
    tx_power: u16,
    reg_override: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdRadioSetup {
    common: CmdCommon,
    mode: u8,
    io_divider: u8,
    config: RfcSetupConfig,
    tx_power: u16,
    reg_override: u32,
}

impl CmdRadioSetup {
    pub fn new(
        c: CmdCommon,
        io_divider: u8,
        reg_override: u32,
        mode: u8,
        tx_power: u16,
    ) -> CmdRadioSetup {
        CmdRadioSetup {
            common: CmdCommon {
                command_no: c.command_no,
                status: c.status,
                p_next_op: c.p_next_op,
                start_time: c.start_time,
                start_trigger: c.start_trigger,
                condition: c.condition,
            },
            mode,
            io_divider,
            config: {
                let mut cfg = RfcSetupConfig(0);
                cfg.set_frontend_mode(0);
                cfg.set_bias_mode(false);
                cfg
            },
            tx_power,
            reg_override,
        }
    }
}

unsafe impl RadioCommand for CmdRadioSetup {
    fn pack(&self, common: CmdCommon) -> CmdRadioSetup {
        CmdRadioSetup {
            common: CmdCommon {
                command_no: common.command_no,
                status: common.status,
                p_next_op: common.p_next_op,
                start_time: common.start_time,
                start_trigger: common.start_trigger,
                condition: common.condition,
            },
            mode: self.mode,
            io_divider: self.io_divider,
            config: {
                let mut cfg = RfcSetupConfig(0);
                cfg.set_frontend_mode(0);
                cfg.set_bias_mode(false);
                cfg
            },
            tx_power: self.tx_power,
            reg_override: self.reg_override,
        }
    }
}

// Command for pinging radio, no operation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdNop {
    command_no: u16, //0x0801
    pub status: u16,
    p_next_op: u32,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
}

impl CmdNop {
    pub fn new() -> CmdNop {
        CmdNop {
            command_no: 0x0801,
            status: 0,
            p_next_op: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
        }
    }
}

// Power up frequency synthesizer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdFSPowerup {
    command_no: u16, //0x080C
    pub status: u16,
    p_next_op: u32,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
    reserved: u16,
    reg_override: u32,
}

// Power down frequency synthesizer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdFSPowerdown {
    common: CmdCommon,
}

impl CmdFSPowerdown {
    pub fn new(c: CmdCommon) -> CmdFSPowerdown {
        CmdFSPowerdown {
            common: CmdCommon {
                command_no: 0x080D,
                status: c.status,
                p_next_op: c.p_next_op,
                start_time: c.start_time,
                start_trigger: c.start_trigger,
                condition: c.condition,
            },
        }
    }
}

unsafe impl RadioCommand for CmdFSPowerdown {
    fn pack(&self, common: CmdCommon) -> Self {
        CmdFSPowerdown {
            common: CmdCommon {
                command_no: common.command_no,
                status: common.status,
                p_next_op: common.p_next_op,
                start_time: common.start_time,
                start_trigger: common.start_trigger,
                condition: common.condition,
            },
        }
    }
}
// Custom FS, unimplemented
#[repr(C)]
pub struct CmdFS {
    command_no: u16, // 0x0803
    pub status: u16,
    p_next_op: u32,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
    fract_freq: u16,
    synth_conf: u8,
    _reserved: [u8; 5],
}

// Disable FS, unimplemented
#[repr(C)]
pub struct CmdFSOff {
    command_no: u16, // 0x0804
    pub status: u16,
    p_next_op: u32,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
}

// Continuous RX test, unimplemented
#[repr(C)]
pub struct CmdRxTest {
    command_no: u16, // 0x0807
    pub status: u16,
    p_next_op: u32,
    start_time: u32,
    start_trigger: u8,
    condition: RfcCondition,
    config: u8,
    end_trigger: u8,
    sync_word: u32,
    end_time: u32,
}

// Continuous TX test, unimplemented
#[repr(C)]
pub struct CmdTxTest {
    // command_no 0x0808
    common: CmdCommon,
    config: u8,
    _reserved_a: u8,
    tx_word: u16,
    _reserved_b: u8,
    end_trigger: RfcTrigger,
    sync_word: u32,
    end_time: u32,
}

impl CmdTxTest {
    pub fn new(c: CmdCommon, trigger: RfcTrigger, time: u32) -> CmdTxTest {
        CmdTxTest {
            common: CmdCommon {
                command_no: 0x0808,
                status: c.status,
                p_next_op: c.p_next_op,
                start_time: c.start_time,
                start_trigger: c.start_trigger,
                condition: c.condition,
            },
            config: 0,
            _reserved_a: 0,
            tx_word: 0x8888,
            _reserved_b: 0,
            end_trigger: trigger,
            sync_word: 0xDED13370,
            end_time: time,
        }
    }
}

unsafe impl RadioCommand for CmdTxTest {
    fn pack(&self, common: CmdCommon) -> CmdTxTest {
        CmdTxTest {
            common,
            config: self.config,
            _reserved_a: self._reserved_a,
            tx_word: self.tx_word,
            _reserved_b: self._reserved_b,
            end_trigger: self.end_trigger,
            sync_word: self.sync_word,
            end_time: self.end_time,
        }
    }
}

// Stop radio RAT timer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdSyncStopRat {
    // command_no: u16, // 0x0809
    common: CmdCommon,
    _reserved: u16,
    rat0: u32,
}

impl CmdSyncStopRat {
    pub fn new(c: CmdCommon, rat: u32) -> CmdSyncStopRat {
        CmdSyncStopRat {
            common: CmdCommon {
                command_no: 0x0809,
                status: c.status,
                p_next_op: c.p_next_op,
                start_time: c.start_time,
                start_trigger: c.start_trigger,
                condition: c.condition,
            },
            _reserved: 0x0000,
            rat0: rat,
        }
    }
}

unsafe impl RadioCommand for CmdSyncStopRat {
    fn pack(&self, common: CmdCommon) -> CmdSyncStopRat {
        CmdSyncStopRat {
            common: CmdCommon {
                command_no: common.command_no,
                status: common.status,
                p_next_op: common.p_next_op,
                start_time: common.start_time,
                start_trigger: common.start_trigger,
                condition: common.condition,
            },
            _reserved: 0x0000,
            rat0: self.rat0,
        }
    }
}

// Start radio RAT timer
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CmdSyncStartRat {
    common: CmdCommon,
    _reserved: u16,
    rat0: u32,
}

impl CmdSyncStartRat {
    pub fn new(c: CmdCommon, rat: u32) -> CmdSyncStartRat {
        CmdSyncStartRat {
            common: CmdCommon {
                command_no: 0x080A,
                status: c.status,
                p_next_op: c.p_next_op,
                start_time: c.start_time,
                start_trigger: c.start_trigger,
                condition: c.condition,
            },
            _reserved: 0x0000,
            rat0: rat,
        }
    }
}

unsafe impl RadioCommand for CmdSyncStartRat {
    fn pack(&self, common: CmdCommon) -> CmdSyncStartRat {
        CmdSyncStartRat {
            common: CmdCommon {
                command_no: common.command_no,
                status: common.status,
                p_next_op: common.p_next_op,
                start_time: common.start_time,
                start_trigger: common.start_trigger,
                condition: common.condition,
            },
            _reserved: 0x0000,
            rat0: self.rat0,
        }
    }
}
