use kernel::common::registers::ReadOnly;

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
    pub _analog_cfg_mode, set_analog_config_mode: 9, 4;
    pub _no_fs_powerup, set_no_fs_powerup: 10;
}

// Radio Commands

// RFC Immediate commands
pub const RFC_CMD0: u16 = 0x607; // found in driverlib SDK
pub const RFC_PING: u16 = 0x406;
pub const RFC_BUS_REQUEST: u16 = 0x40E;
pub const RFC_START_RAT_TIMER: u16 = 0x080A;
pub const RFC_STOP_RAT_TIMER: u16 = 0x0809;
pub const RFC_SETUP: u16 = 0x0802;
pub const RFC_STOP: u16 = 0x0402;
pub const RFC_FS_POWERDOWN: u16 = 0x080D;

#[repr(C)]
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

#[repr(C)]
pub struct CommandCommon {
    pub command_no: ReadOnly<u16>,
    pub status: ReadOnly<u16>,
    pub p_nextop: ReadOnly<u32>,
    pub ratmr: ReadOnly<u32>,
    pub start_trigger: ReadOnly<u8>,
    pub condition: RfcCondition,
}

// Command and parameters for radio setup

pub unsafe trait RadioCommand {
    fn guard(&mut self);
}

pub mod prop_commands {
    #![allow(unused)]
    use kernel::common::registers::ReadOnly;
    use radio::commands::{RadioCommand, RfcCondition, RfcSetupConfig, RfcTrigger};

    // Radio and data commands bitfields
    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcModulation(u16);
        impl Debug;
        pub _mod_type, set_mod_type                : 2, 0;
        pub _deviation, set_deviation              : 13, 3;
        pub _deviation_step, set_deviation_step    : 15, 14;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcSymbolRate(u32);
        impl Debug;
        pub _prescale, set_prescale    : 7, 0;
        pub _rate_word, set_rate_word  : 28, 8;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcPreambleConf(u8);
        impl Debug;
        pub _num_preamble_bytes, set_num_preamble_bytes    : 5, 0;
        pub _pream_mode, set_pream_mode                    : 6, 7;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcFormatConf(u16);
        impl Debug;
        pub _num_syncword_bits, set_num_syncword_bits  : 5, 0;
        pub _bit_reversal, set_bit_reversal            : 6;
        pub _msb_first, set_msb_first                  : 7;
        pub _fec_mode, set_fec_mode                    : 11, 8;
        pub _whiten_mode, set_whiten_mode              : 15, 13;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcPacketConf(u8);
        impl Debug;
        pub _fs_off, set_fs_off         : 0;
        pub _reserved, _set_reserved    : 2, 1;
        pub _use_crc, set_use_crc       : 3;
        pub _var_len, set_var_len       : 4;
        pub _reserved2, _set_reserved2  : 7, 5;
    }

    bitfield! {
        #[derive(Copy, Clone)]
        pub struct RfcSynthConf(u8);
        impl Debug;
        pub _tx_mode, set_tx_mode       : 0;
        pub _ref_freq, set_ref_freq     : 6, 1;
        pub _reserved, _set_reserved    : 7;
    }

    // Radio Operation Commands
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandRadioDivSetup {
        pub command_no: u16, // 0x3807
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub modulation: RfcModulation,
        pub symbol_rate: RfcSymbolRate,
        pub rx_bandwidth: u8,
        pub preamble_conf: RfcPreambleConf,
        pub format_conf: RfcFormatConf,
        pub config: RfcSetupConfig,
        pub tx_power: u16,
        pub reg_overrides: u32,
        pub center_freq: u16,
        pub int_freq: u16,
        pub lo_divider: u8,
    }

    unsafe impl RadioCommand for CommandRadioDivSetup {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    pub struct CommandRadioSetup {
        pub command_no: u16, // 0x3806
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub modulation: RfcModulation,
        pub symbol_rate: RfcSymbolRate,
        pub rx_bandwidth: u8,
        pub preamble_conf: RfcPreambleConf,
        pub format_conf: RfcFormatConf,
        pub config: RfcSetupConfig,
        pub tx_power: u16,
        pub reg_overrides: u32,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandSyncRat {
        pub command_no: u16,
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub _reserved: u16,
        pub rat0: u32,
    }

    unsafe impl RadioCommand for CommandSyncRat {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandTx {
        pub command_no: u16, // 0x3801
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub packet_conf: RfcPacketConf,
        pub packet_len: u8,
        pub sync_word: u32,
        pub packet_pointer: u32,
    }

    unsafe impl RadioCommand for CommandTx {
        fn guard(&mut self) {}
    }

    // Custom FS
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct CommandFS {
        pub command_no: u16, // 0x0803
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
        pub frequency: u16,
        pub fract_freq: u16,
        pub synth_conf: RfcSynthConf,
    }

    unsafe impl RadioCommand for CommandFS {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    pub struct CommandFSPowerdown {
        pub command_no: u16, // 0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

    unsafe impl RadioCommand for CommandFSPowerdown {
        fn guard(&mut self) {}
    }

    #[repr(C)]
    pub struct CommandRx {
        pub command_no: u16, // 0x080D
        pub status: u16,
        pub p_nextop: u32,
        pub start_time: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,
    }

}
