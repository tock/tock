use radio::commands as cmd;
use radio::commands::prop_commands as prop;
const TEST_PAYLOAD: [u32; 30] = [0; 30];

// This setup command sets the radio in DSSS, FEC (4x spreading factor) with the following
// parameters:
// Frequency: 915.00000 MHz
// Data Format: Serial mode disable
// Deviation: 2.500 kHz
// pktLen: 30
// 802.15.4g Mode: 0
// Select bit order to transmit PSDU octets:: 1
// Packet Length Config: Variable
// Max Packet Length: 128
// Packet Length: 20
// Packet Data: 255
// RX Filter BW: 34.1 kHz
// Symbol Rate: 10.00061 kBaud
// Sync Word Length: 32 Bits
// TX Power: 14 dBm (requires define CCFG_FORCE_VDDR_HH = 1 in ccfg.c, see CC13xx/CC26xx Technical Reference Manual)
// Whitening: No whitening
#[repr(C)]
pub struct CommandRadioDivSetupLongRange {
    pub command_no: u16, // 0x3807
    pub status: u16,
    pub p_nextop: u32,
    pub start_time: u32,
    pub start_trigger: u8,
    pub condition: cmd::RfcCondition,
    pub modulation: prop::RfcModulation,
    pub symbol_rate: prop::RfcSymbolRate,
    pub rx_bandwidth: u8,
    pub preamble_conf: prop::RfcPreambleConf,
    pub format_conf: prop::RfcFormatConf,
    pub config: cmd::RfcSetupConfig,
    pub tx_power: u16,
    pub reg_overrides: u32,
    pub center_freq: u16,
    pub int_freq: u16,
    pub lo_divider: u8,
}

impl CommandRadioDivSetupLongRange {
    pub fn new() -> CommandRadioDivSetupLongRange {
        unsafe {
            let p_overrides: u32 = LONGRANGE_RFPARAMS.as_mut_ptr() as u32;

            let setup_cmd = CommandRadioDivSetupLongRange {
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
                    mdl.set_deviation(0xA);
                    mdl.set_deviation_step(0x0);
                    mdl
                },
                symbol_rate: {
                    let mut sr = prop::RfcSymbolRate(0);
                    sr.set_prescale(0xF);
                    sr.set_rate_word(0x199A);
                    sr
                },
                rx_bandwidth: 0x4C,
                preamble_conf: {
                    let mut preamble = prop::RfcPreambleConf(0);
                    preamble.set_num_preamble_bytes(0x2);
                    preamble.set_pream_mode(0x0);
                    preamble
                },
                format_conf: {
                    let mut format = prop::RfcFormatConf(0);
                    format.set_num_syncword_bits(0x20);
                    format.set_bit_reversal(false);
                    format.set_msb_first(false);
                    format.set_fec_mode(0x8);
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
                center_freq: 0x0393,
                int_freq: 0x8000,
                lo_divider: 0x05,
            };
            setup_cmd
        }
    }
}

#[repr(C)]
pub struct CommandTxLongRangeTest {
    pub command_no: u16, // 0x3801
    pub status: u16,
    pub p_nextop: u32,
    pub start_time: u32,
    pub start_trigger: u8,
    pub condition: cmd::RfcCondition,
    pub packet_conf: prop::RfcPacketConf,
    pub packet_len: u8,
    pub sync_word: u32,
    pub packet_pointer: u32,
}

impl CommandTxLongRangeTest {
    pub fn new() -> CommandTxLongRangeTest {
        let mut packet = TEST_PAYLOAD;
        let mut seq: u32 = 0;
        for p in packet.iter_mut() {
            *p = seq;
            seq += 1;
        }
        let p_packet = packet.as_mut_ptr() as u32;

        let cmd_tx = CommandTxLongRangeTest {
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
            packet_len: 0x14,
            sync_word: 0x00000000,
            packet_pointer: p_packet,
        };
        cmd_tx
    }
}

static mut LONGRANGE_RFPARAMS: [u32; 28] = [
    // override_use_patch_prop_genfsk.xml
    0x00000847, // PHY: Use MCE RAM patch, RFE RAM patch MCE_RFE_OVERRIDE(1,0,0,1,0,0),
    0x006E88E3, // PHY: Use MCE RAM patch only for Rx (0xE), use MCE ROM bank 6 for Tx (0x6)
    // override_synth_prop_863_930_div5.xml
    0x02400403, // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x00068793, // Synth: Set minimum RTRIM to 6
    0x001C8473, // Synth: Configure extra PLL filtering
    0x00088433, // Synth: Configure extra PLL filtering
    0x000684A3, // Synth: Set Fref to 4 MHz
    0x40014005, // Synth: Configure faster calibration HW32_ARRAY_OVERRIDE(0x4004,1),
    0x180C0618, // Synth: Configure faster calibration
    0xC00401A1, // Synth: Configure faster calibration
    0x00010101, // Synth: Configure faster calibration
    0xC0040141, // Synth: Configure faster calibration
    0x00214AD3, // Synth: Configure faster calibration
    // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x02980243, // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x0A480583, // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603, // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623, // Synth: Set loop bandwidth after lock to 20 kHz
    // override_phy_simplelink_long_range_dsss2.xml
    0x030c5068, // PHY: Configure DSSS SF=2 for payload data HW_REG_OVERRIDE(0x5068,0x0100),
    0x146f5128, // PHY: Set SimpleLink Long Range bit-inverted sync word pattern (uncoded, before spreading to fixed-size 64-bit pattern): 0x146F HW_REG_OVERRIDE(0x5128,0x146F),
    0xeb90512c, // PHY: Set SimpleLink Long Range sync word pattern (uncoded, before spreading to fixed-size 64-bit pattern): 0xEB90 HW_REG_OVERRIDE(0x512C,0xEB90),
    0x362e5124, // PHY: Reduce demodulator correlator threshold for improved Rx sensitivity HW_REG_OVERRIDE(0x5124,0x362E),
    0x004c5118, // PHY: Reduce demodulator correlator threshold for improved Rx sensitivity HW_REG_OVERRIDE(0x5118,0x004C),
    0x3e055140, // PHY: Configure limit on frequency offset compensation tracker HW_REG_OVERRIDE(0x5140,0x3E05),
    // override_phy_rx_frontend_simplelink_long_range.xml
    0x000288A3, // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    // override_phy_rx_aaf_bw_0xd.xml
    0x7ddf0002, // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD) ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    0xFFFC08C3, // TX power override DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0x0cf80002, // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8) ADI_REG_OVERRIDE(0,12,0xF8),
    0xFFFFFFFF,
];

// This setup command configuration test the radio setup in 2GFSK mode with the following
// parameters:
// Frequency: 915.00000 MHz
// Data Format: Serial mode disable
// Deviation: 25.000 kHz
// Packet Length: 30
// 802.15.4g Mode: 0
// Select bit order to transmit PSDU octets:: 1
// Packet Length Config: Variable
// Max Packet Length: 128
// Packet Length: 20
// Packet Data: 255
// RX Filter BW: 98.0 kHz
// Symbol Rate: 50.00000 kBaud
// Sync Word Length: 32 Bits
// TX Power: 14 dBm (requires define CCFG_FORCE_VDDR_HH = 1 in ccfg.c, see CC13xx/CC26xx Technical Reference Manual)
// Whitening: No whitening
#[repr(C)]
pub struct CommandRadioDivSetupGfskTest {
    pub command_no: u16, // 0x3806
    pub status: u16,
    pub p_nextop: u32,
    pub start_time: u32,
    pub start_trigger: u8,
    pub condition: cmd::RfcCondition,
    pub modulation: prop::RfcModulation,
    pub symbol_rate: prop::RfcSymbolRate,
    pub rx_bandwidth: u8,
    pub preamble_conf: prop::RfcPreambleConf,
    pub format_conf: prop::RfcFormatConf,
    pub config: cmd::RfcSetupConfig,
    pub tx_power: u16,
    pub reg_overrides: u32,
    pub center_freq: u16,
    pub int_freq: u16,
    pub lo_divider: u8,
}

impl CommandRadioDivSetupGfskTest {
    pub fn new() -> CommandRadioDivSetupGfskTest {
        unsafe {
            let p_overrides: u32 = GFSK_RFPARAMS.as_mut_ptr() as u32;

            let setup_cmd = CommandRadioDivSetupGfskTest {
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
                center_freq: 0x0393,
                int_freq: 0x8000,
                lo_divider: 0x05,
            };
            setup_cmd
        }
    }
}

#[repr(C)]
pub struct CommandTxGfskTest {
    pub command_no: u16, // 0x3801
    pub status: u16,
    pub p_nextop: u32,
    pub start_time: u32,
    pub start_trigger: u8,
    pub condition: cmd::RfcCondition,
    pub packet_conf: prop::RfcPacketConf,
    pub packet_len: u8,
    pub sync_word: u32,
    pub packet_pointer: u32,
}

impl CommandTxGfskTest {
    pub fn new() -> CommandTxGfskTest {
        let mut packet = TEST_PAYLOAD;
        let mut seq: u32 = 0;
        for p in packet.iter_mut() {
            *p = seq;
            seq += 1;
        }
        let p_packet = packet.as_mut_ptr() as u32;

        let cmd_tx = CommandTxGfskTest {
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
            packet_len: 0x14,
            sync_word: 0x930B51DE,
            packet_pointer: p_packet,
        };
        cmd_tx
    }
}

#[repr(C)]
pub struct CommandFSTest_USISM {
    pub command_no: u16, // 0x0803
    pub status: u16,
    pub p_nextop: u32,
    pub start_time: u32,
    pub start_trigger: u8,
    pub condition: cmd::RfcCondition,
    pub frequency: u16,
    pub fract_freq: u16,
    pub synth_conf: prop::RfcSynthConf,
}

impl CommandFSTest_USISM {
    pub fn new() -> CommandFSTest_USISM {
        let cmd_fs = CommandFSTest_USISM {
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
            frequency: 0x0393,
            fract_freq: 0x0000,
            synth_conf: {
                let mut synth = prop::RfcSynthConf(0);
                synth.set_tx_mode(false);
                synth.set_ref_freq(0x00);
                synth
            },
        };
        cmd_fs
    }
}

static mut GFSK_RFPARAMS: [u32; 25] = [
    // override_use_patch_prop_genfsk.xml
    0x00000847, // PHY: Use MCE RAM patch, RFE RAM patch MCE_RFE_OVERRIDE(1,0,0,1,0,0),
    // override_synth_prop_863_930_div5.xml
    0x02400403, // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x00068793, // Synth: Set minimum RTRIM to 6
    0x001C8473, // Synth: Configure extra PLL filtering
    0x00088433, // Synth: Configure extra PLL filtering
    0x000684A3, // Synth: Set Fref to 4 MHz
    0x40014005, // Synth: Configure faster calibration HW32_ARRAY_OVERRIDE(0x4004,1),
    0x180C0618, // Synth: Configure faster calibration
    0xC00401A1, // Synth: Configure faster calibration
    0x00010101, // Synth: Configure faster calibration
    0xC0040141, // Synth: Configure faster calibration
    0x00214AD3, // Synth: Configure faster calibration
    0x02980243, // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us) Synth: Set loop bandwidth after lock to 20 kHz
    0x0A480583, // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603, // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623,
    // override_phy_tx_pa_ramp_genfsk.xml
    0x50880002, // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1) ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    0x51110002, // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1) ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    // override_phy_rx_frontend_genfsk.xml
    0x001a609c, // Rx: Set AGC reference level to 0x1A (default: 0x2E) HW_REG_OVERRIDE(0x609C,0x001A),
    0x00018883, // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x000288A3, // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    // override_phy_rx_aaf_bw_0xd.xml
    0x7ddf0002, // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD) ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    0xFFFC08C3, // TX power override DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0x0cf80002, // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8) ADI_REG_OVERRIDE(0,12,0xF8),
    0xFFFFFFFF, // Stop word
];
