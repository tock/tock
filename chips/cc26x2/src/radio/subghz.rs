#![allow(unused_imports)]
use chip::SleepMode;
use core::cell::Cell;
use enum_primitive::cast::FromPrimitive;
use fixedvec::FixedVec;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::radio_client;
use kernel::ReturnCode;
use osc;
use radio::commands as cmd;
use radio::patch_mce_genfsk as mce;
use radio::patch_rfe_genfsk as rfe;
use radio::rfc;
use radio::subghz::prop_commands as prop;
use rtc;

const TEST_PAYLOAD: [u32; 30] = [0; 30];

static mut RFPARAMS: [u32; 25] = [
    // override_use_patch_prop_genfsk.xml
    // PHY: Use MCE RAM patch, RFE RAM patch
    // MCE_RFE_OVERRIDE(1,0,0,1,0,0),
    0x00000847,
    // override_synth_prop_863_930_div5.xml
    // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x02400403, // Synth: Set minimum RTRIM to 6
    0x00068793, // Synth: Configure extra PLL filtering
    0x001C8473, // Synth: Configure extra PLL filtering
    0x00088433, // Synth: Set Fref to 4 MHz
    0x000684A3,
    // Synth: Configure faster calibration
    // HW32_ARRAY_OVERRIDE(0x4004,1),
    0x40014005,
    // Synth: Configure faster calibration
    0x180C0618, // Synth: Configure faster calibration
    0xC00401A1, // Synth: Configure faster calibration
    0x00010101, // Synth: Configure faster calibration
    0xC0040141, // Synth: Configure faster calibration
    0x00214AD3,
    // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x02980243, // Synth: Set loop bandwidth after lock to 20 kHz
    0x0A480583, // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603, // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623,
    // override_phy_tx_pa_ramp_genfsk.xml
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1)
    // ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    0x50880002,
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1)
    // ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    0x51110002,
    // override_phy_rx_frontend_genfsk.xml
    // Rx: Set AGC reference level to 0x1A (default: 0x2E)
    // HW_REG_OVERRIDE(0x609C,0x001A),
    0x001a609c, // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x00018883,
    // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    0x000288A3,
    // override_phy_rx_aaf_bw_0xd.xml
    // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD)
    // ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    0x7ddf0002,
    // TX power override
    // DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0xFFFC08C3,
    // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8)
    // ADI_REG_OVERRIDE(0,12,0xF8),
    0x0cf80002, 0xFFFFFFFF,
];

pub struct Radio {
    rfc: &'static rfc::RFCore,
    tx_radio_client: OptionalCell<&'static radio_client::TxClient>,
    rx_radio_client: OptionalCell<&'static radio_client::RxClient>,
    config_radio_client: OptionalCell<&'static radio_client::ConfigClient>,
    schedule_powerdown: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    cmdr_ready: Cell<bool>,
    radio_ready: Cell<bool>,
}

impl Radio {
    pub const fn new(rfc: &'static rfc::RFCore) -> Radio {
        Radio {
            rfc,
            tx_radio_client: OptionalCell::empty(),
            rx_radio_client: OptionalCell::empty(),
            config_radio_client: OptionalCell::empty(),
            schedule_powerdown: Cell::new(false),
            tx_buf: TakeCell::empty(),
            cmdr_ready: Cell::new(true),
            radio_ready: Cell::new(true),
        }
    }

    pub fn run_tests(&self) {
        self.test_power_up();

        mce::MCE_PATCH.apply_mce_genfsk_patch();
        rfe::RFE_PATCH.apply_rfe_genfsk_patch();

        self.test_configure_radio();

        self.test_radio_fs();

        self.test_radio_tx();
    }

    fn test_power_up(&self) {
        // osc::OSC.switch_to_rc_osc();

        self.rfc.set_mode(rfc::RfcMode::Common);

        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();

        self.rfc.start_rat_test();

        osc::OSC.switch_to_hf_xosc();
    }

    pub fn power_up(&self) -> ReturnCode {
        self.rfc.set_mode(rfc::RfcMode::Common);

        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();

        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        unsafe {
            let reg_overrides: u32 = RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides, 0xFFFE) // No idea what power setting this is
        }

        if self.rfc.check_enabled() {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        }
    }

    pub fn power_down(&self) {
        self.rfc.disable();
    }

    pub fn configure_radio(&self) -> ReturnCode {
        let setup_cmd = prop::CommandRadioDivSetup {
            command_no: 0x3807,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = cmd::RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            modulation: {
                let mut mdl = prop::RfcModulation(0);
                mdl.set_mod_type(0x01);
                mdl.set_deviation(0x64);
                mdl.set_deviation_step(0x0);
                mdl
            },
            symbol_rate: {
                let mut sr = prop::RfcSymbolRate(0);
                sr.set_prescale(0xF);
                sr.set_rate_word(0x8000);
                sr
            },
            rx_bandwidth: 0x52,
            preamble_conf: {
                let mut preamble = prop::RfcPreambleConf(0);
                preamble.set_num_preamble_bytes(0x4);
                preamble.set_pream_mode(0x0);
                preamble
            },
            format_conf: {
                let mut format = prop::RfcFormatConf(0);
                format.set_num_syncword_bits(0x20);
                format.set_bit_reversal(false);
                format.set_msb_first(true);
                format.set_fec_mode(0x0);
                format.set_whiten_mode(0x0);
                format
            },
            config: {
                let mut cfg = cmd::RfcSetupConfig(0);
                cfg.set_frontend_mode(0);
                cfg.set_bias_mode(true);
                cfg.set_analog_config_mode(0x0);
                cfg.set_no_fs_powerup(false);
                cfg
            },
            tx_power: 0x9F3F,
            reg_overrides: 0,
            center_freq: 0x0364,
            int_freq: 0x8000,
            lo_divider: 0x05,
        };

        if self
            .rfc
            .send_test(&setup_cmd)
            .and_then(|_| self.rfc.wait_test(&setup_cmd))
            .is_ok()
        {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        }
    }

    fn test_configure_radio(&self) {
        unsafe {
            let p_overrides: u32 = RFPARAMS.as_mut_ptr() as u32;

            let setup_cmd = prop::CommandRadioDivSetup {
                command_no: 0x3807,
                status: 0,
                p_nextop: 0,
                start_time: 0,
                start_trigger: 0,
                condition: {
                    let mut cond = cmd::RfcCondition(0);
                    cond.set_rule(0x01);
                    cond
                },
                modulation: {
                    let mut mdl = prop::RfcModulation(0);
                    mdl.set_mod_type(0x01);
                    mdl.set_deviation(0x64);
                    mdl.set_deviation_step(0x0);
                    mdl
                },
                symbol_rate: {
                    let mut sr = prop::RfcSymbolRate(0);
                    sr.set_prescale(0xF);
                    sr.set_rate_word(0x8000);
                    sr
                },
                rx_bandwidth: 0x52,
                preamble_conf: {
                    let mut preamble = prop::RfcPreambleConf(0);
                    preamble.set_num_preamble_bytes(0x4);
                    preamble.set_pream_mode(0x0);
                    preamble
                },
                format_conf: {
                    let mut format = prop::RfcFormatConf(0);
                    format.set_num_syncword_bits(0x20);
                    format.set_bit_reversal(false);
                    format.set_msb_first(true);
                    format.set_fec_mode(0x0);
                    format.set_whiten_mode(0x0);
                    format
                },
                config: {
                    let mut cfg = cmd::RfcSetupConfig(0);
                    cfg.set_frontend_mode(0);
                    cfg.set_bias_mode(true);
                    cfg.set_analog_config_mode(0x0);
                    cfg.set_no_fs_powerup(false);
                    cfg
                },
                tx_power: 0x9F3F,
                reg_overrides: p_overrides,
                center_freq: 0x0364,
                int_freq: 0x8000,
                lo_divider: 0x05,
            };

            self.rfc
                .send_test(&setup_cmd)
                .and_then(|_| self.rfc.wait_test(&setup_cmd))
                .ok();
        }
    }

    fn test_radio_tx(&self) {
        let mut packet = TEST_PAYLOAD;
        let mut seq: u32 = 0;
        for p in packet.iter_mut() {
            *p = seq;
            seq += 1;
        }
        let p_packet = packet.as_mut_ptr() as u32;

        let cmd_tx = prop::CommandTx {
            command_no: 0x3801,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = cmd::RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            packet_conf: {
                let mut packet = prop::RfcPacketConf(0);
                packet.set_fs_off(false);
                packet.set_use_crc(true);
                packet.set_var_len(true);
                packet
            },
            packet_len: 0x1E,
            sync_word: 0x930B51DE,
            packet_pointer: p_packet,
        };

        self.rfc
            .send_test(&cmd_tx)
            .and_then(|_| self.rfc.wait_test(&cmd_tx))
            .ok();
    }

    fn test_radio_fs(&self) {
        let cmd_fs = prop::CommandFS {
            command_no: 0x0803,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = cmd::RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            frequency: 0x0364,
            fract_freq: 0x0000,
            synth_conf: {
                let mut synth = prop::RfcSynthConf(0);
                synth.set_tx_mode(false);
                synth.set_ref_freq(0x00);
                synth
            },
        };

        self.rfc
            .send_test(&cmd_fs)
            .and_then(|_| self.rfc.wait_test(&cmd_fs))
            .ok();
    }
}

impl rfc::RFCoreClient for Radio {
    fn command_done(&self) {
        unsafe { rtc::RTC.sync() };
        let status = self.rfc.status.get();
        match status & 0xFFF {
            0x000 => {
                // IDLE
                self.radio_ready.set(true);
                self.cmdr_ready.set(true);
            }
            0x001 => {
                // PENDING
                self.radio_ready.set(false);
            }
            0x002 => {
                // ACTIVE
                self.radio_ready.set(false);
            }
            0x400 => {
                // DONE OK
                self.radio_ready.set(true);
                self.cmdr_ready.set(true);
                self.config_radio_client
                    .take()
                    .map(|client| client.config_done(ReturnCode::SUCCESS));
            }
            _ => (),
        }
        // osc::OSC.switch_to_hf_rcosc;
    }

    fn tx_done(&self) {
        if self.schedule_powerdown.get() {
            self.power_down();
            // osc::OSC.switch_to_hf_rcosc();
        }

        let buf = self.tx_buf.take();
        self.tx_radio_client
            .take()
            .map(|client| client.transmit_event(buf.unwrap(), ReturnCode::SUCCESS));
    }

    fn rx_ok(&self) {}
}

impl radio_client::Radio for Radio {}

impl radio_client::RadioDriver for Radio {
    fn set_transmit_client(&self, tx_client: &'static radio_client::TxClient) {
        self.tx_radio_client.set(tx_client);
    }

    fn set_receive_client(
        &self,
        rx_client: &'static radio_client::RxClient,
        _rx_buf: &'static mut [u8],
    ) {
        self.rx_radio_client.set(rx_client);
    }

    fn set_receive_buffer(&self, _rx_buf: &'static mut [u8]) {
        // maybe make a rx buf only when needed?
    }

    fn set_config_client(&self, config_client: &'static radio_client::ConfigClient) {
        self.config_radio_client.set(config_client);
    }

    fn transmit(
        &self,
        tx_buf: &'static mut [u8],
        _frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::SUCCESS, Some(tx_buf))
    }
}

impl radio_client::RadioConfig for Radio {
    fn initialize(&self) -> ReturnCode {
        self.power_up()
    }

    fn reset(&self) -> ReturnCode {
        self.power_down();
        self.power_up()
    }

    fn stop(&self) -> ReturnCode {
        let cmd_stop = cmd::DirectCommand::new(0x0402, 0);
        let stopped = self.rfc.send_direct(&cmd_stop).is_ok();
        if stopped {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        }
    }

    fn is_on(&self) -> bool {
        self.rfc.check_enabled()
    }

    fn busy(&self) -> bool {
        // Might be an obsolete command here in favor of get_command_status and some logic on the
        // user size to determine if the radio is busy. Not sure what is best to have here but
        // arguing best might be bikeshedding
        let status = self.rfc.status.get();
        match status {
            0x0001 => true,
            0x0002 => true,
            _ => false,
        }
    }

    fn config_commit(&self) {
        // TODO confirm set new config here
    }

    fn get_tx_power(&self) -> u32 {
        // TODO get tx power radio command
        0x00000000
    }

    fn get_radio_status(&self) -> u32 {
        // TODO get power status of radio
        0x00000000
    }

    fn get_command_status(&self) -> (ReturnCode, Option<u32>) {
        // TODO get command status specifics
        let status = self.rfc.status.get();
        match status & 0x0F00 {
            0 => (ReturnCode::SUCCESS, Some(status)),
            4 => (ReturnCode::SUCCESS, Some(status)),
            8 => (ReturnCode::FAIL, Some(status)),
            _ => (ReturnCode::EINVAL, Some(status)),
        }
    }

    fn set_tx_power(&self, power: u16) -> ReturnCode {
        // Send direct command for TX power change
        let command = cmd::DirectCommand::new(0x0010, power);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn send_stop_command(&self) -> ReturnCode {
        // Send "Gracefull" stop radio operation direct command
        let command = cmd::DirectCommand::new(0x0402, 0);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn send_kill_command(&self) -> ReturnCode {
        // Send immidiate command kill all radio operation commands
        let command = cmd::DirectCommand::new(0x0401, 0);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }
}

enum_from_primitive!{
#[derive(Clone, Copy)]
pub enum CMDSTA {
    Pending = 0x00,
    Done = 0x01,
    IllegalPointer = 0x81,
    UnknownCommand = 0x82,
    UnknownDirCommand = 0x83,
    ContextError = 0x85,
    SchedulingError = 0x86,
    ParError = 0x87,
    QueueError = 0x88,
    QueueBusy = 0x89,
}
}

enum_from_primitive!{
#[derive(Clone, Copy)]
pub enum RadioOpStatus {
    Idle = 0x0000,
    Pending = 0x0001,
    Active = 0x0002,
    Skipped = 0x0003,
    DoneOk = 0x0400,
    DoneCountdown = 0x0401,
    DoneRxErr = 0x0402,
    DoneTimeout = 0x0403,
    DoneStopped = 0x0404,
    DoneAbort = 0x0405,
    ErrorPastStart = 0x0800,
    ErrorStartTrig = 0x0801,
    ErrorCondition = 0x0802,
    ErrorPar = 0x0803,
    ErrorPointer = 0x0804,
    ErrorCmdId = 0x0805,
    ErrorNoSetup = 0x0807,
    ErrorNoFs = 0x0808,
    ErrorSynthProg = 0x0809,
    ErrorTxUnf = 0x080A,
    ErrorRxOvf = 0x080B,
    ErrorNoRx = 0x080C,
}
}

enum_from_primitive!{
#[derive(Clone, Copy)]
pub enum RadioPropStatus {
    Idle = 0x0000,
    Pending = 0x0001,
    Active = 0x0002,
    DoneOk = 0x3400,
    DoneRxTimeout = 0x3401,
    DoneBreak = 0x3402,
    DoneEnded = 0x3403,
    DoneStopped = 0x3404,
    DoneAbort = 0x3405,
    DoneRxErr = 0x3406,
    DoneIdle = 0x3407,
    DoneBusy = 0x3408,
    DoneIdleTimeout = 0x3409,
    DoneBusyTimeout = 0x340A,
    ErrorPar = 0x0800,
    ErrorRxBuf = 0x0801,
    ErrorRxFull = 0x0802,
    ErrorNoSetup = 0x0803,
    ErrorNoFs = 0x0804,
    ErrorRxOvf = 0x0805,
    ErrorTxUnf = 0x0806,
}
}

pub mod prop_commands {
    #![allow(unused)]
    use kernel::common::registers::ReadOnly;
    use radio::commands::{RfcCondition, RfcSetupConfig, RfcTrigger};

    #[repr(C)]
    pub struct CommandCommon {
        pub command_no: ReadOnly<u16>,
        pub status: ReadOnly<u16>,
        pub p_nextop: ReadOnly<u32>,
        pub ratmr: ReadOnly<u32>,
        pub start_trigger: ReadOnly<u8>,
        pub condition: RfcCondition,
    }

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
    pub struct CommandRadioDivSetup {
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
        pub center_freq: u16,
        pub int_freq: u16,
        pub lo_divider: u8,
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

    // Custom FS, unimplemented
    #[repr(C)]
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

}
