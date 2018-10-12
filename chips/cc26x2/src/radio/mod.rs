// pub mod ble;
pub mod patch_cpe_prop;
pub mod patch_mce_genfsk;
pub mod patch_mce_longrange;
pub mod patch_rfe_genfsk;

pub mod rfc;
pub mod subghz;
use cortexm4::nvic;
use peripheral_interrupts;

pub mod commands;

const RF_ACK_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CMD_ACK as u32) };
const RF_CPE0_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CORE_CPE0 as u32) };
const RF_CPE1_NVIC: nvic::Nvic =
    unsafe { nvic::Nvic::new(peripheral_interrupts::NVIC_IRQ::RF_CORE_CPE1 as u32) };

pub static mut RFC: rfc::RFCore = rfc::RFCore::new(&RF_ACK_NVIC, &RF_CPE0_NVIC, &RF_CPE1_NVIC);
pub static mut RADIO: subghz::Radio = unsafe { subghz::Radio::new(&RFC) };
// pub static mut BLE: ble::Ble = unsafe { ble::Ble::new(&RFC) };
//
pub static mut GFSK_RFPARAMS: [u32; 25] = [
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

pub static mut LONGRANGE_RFPARAMS: [u32; 28] = [
    // override_use_patch_prop_genfsk.xml
    // PHY: Use MCE RAM patch, RFE RAM patch
    // MCE_RFE_OVERRIDE(1,0,0,1,0,0),
    0x00000847,
    // PHY: Use MCE RAM patch only for Rx (0xE), use MCE ROM bank 6 for Tx (0x6)
    0x006E88E3,
    // override_synth_prop_863_930_div5.xml
    // Synth: Use 48 MHz crystal as synth clock, enable extra PLL filtering
    0x02400403, // Synth: Set minimum RTRIM to 6
    0x00068793, // Synth: Configure extra PLL filtering
    0x001C8473, // Synth: Configure extra PLL filtering
    0x00088433, // Synth: Set Fref to 4 MHz
    0x000684A3,
    // Synth: Configure faster calibration
    //HW32_ARRAY_OVERRIDE(0x4004,1),
    0x40014005, // Synth: Configure faster calibration
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
    // override_phy_simplelink_long_range_dsss2.xml
    // PHY: Configure DSSS SF=2 for payload data
    // HW_REG_OVERRIDE(0x5068,0x0100),
    0x01005068,
    // PHY: Set SimpleLink Long Range bit-inverted sync word pattern (uncoded, before spreading to fixed-size 64-bit pattern): 0x146F
    //HW_REG_OVERRIDE(0x5128,0x146F),
    0x146f5128,
    // PHY: Set SimpleLink Long Range sync word pattern (uncoded, before spreading to fixed-size 64-bit pattern): 0xEB90
    // HW_REG_OVERRIDE(0x512C,0xEB90),
    0xeb90512c,
    // PHY: Reduce demodulator correlator threshold for improved Rx sensitivity
    // HW_REG_OVERRIDE(0x5124,0x362E),
    0x362e5124,
    // PHY: Reduce demodulator correlator threshold for improved Rx sensitivity
    // HW_REG_OVERRIDE(0x5118,0x004C),
    0x004c5118,
    // PHY: Configure limit on frequency offset compensation tracker
    // HW_REG_OVERRIDE(0x5140,0x3E05),
    0x3e055140,
    // override_phy_rx_frontend_simplelink_long_range.xml
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
