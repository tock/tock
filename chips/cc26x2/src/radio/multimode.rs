use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::rfcore;
use kernel::ReturnCode;
use osc;
use radio::rfc;
use radio::commands::{prop_commands as prop};
use radio::patch_mce_genfsk as mce;
use radio::patch_mce_longrange as mce_lr;
use radio::patch_rfe_genfsk as rfe;
use radio::patch_cpe_prop as cpe;

const TEST_PAYLOAD: [u32; 30] = [0; 30];

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

#[derive(Copy, Clone)]
pub enum CpePatch {
    GenFsk { patch: cpe::Patches },
}

#[derive(Copy,Clone)]
pub enum RfePatch {
    #[derive(Copy, Clone)]
    GenFsk { patch: rfe::Patches },
}

#[derive(Copy, Clone)]
pub enum McePatch {
    GenFsk { patch: mce::Patches },
    LongRange { patch: mce_lr::Patches },
}

#[derive(Copy, Clone)]
pub struct RadioMode {
    mode: rfc::RfcMode,
    cpe_patch: CpePatch,
    rfe_patch: RfePatch,
    mce_patch: McePatch,
}

impl Default for RadioMode {
    fn default() -> RadioMode {
        RadioMode {
            mode: rfc::RfcMode::Unchanged,
            cpe_patch: CpePatch::GenFsk{ patch: cpe::CPE_PATCH },
            rfe_patch: RfePatch::GenFsk{ patch: rfe::RFE_PATCH },
            mce_patch: McePatch::GenFsk{ patch: mce::MCE_PATCH },
        }
    }
}

#[derive(Copy, Clone)]
pub enum RadioSetupCommand {
    Ble,
    PropGfsk { cmd: prop::CommandRadioDivSetup },
}

pub struct Radio {
    rfc: &'static rfc::RFCore,
    mode: OptionalCell<RadioMode>,
    setup: OptionalCell<RadioSetupCommand>,
    tx_client: OptionalCell<&'static rfcore::TxClient>,
    rx_client: OptionalCell<&'static rfcore::RxClient>,
    cfg_client: OptionalCell<&'static rfcore::ConfigClient>,
    update_config: Cell<bool>,
    schedule_powerdown: Cell<bool>,
    yeilded: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
}

impl Radio {
    pub const fn new(rfc: &'static rfc::RFCore) -> Radio {
        Radio {
            rfc,
            mode: OptionalCell::empty(),
            setup: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            cfg_client: OptionalCell::empty(),
            update_config: Cell::new(false),
            schedule_powerdown: Cell::new(false),
            yeilded: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
        }
    }

    pub fn power_up(&self, m: RadioMode) -> ReturnCode {
        self.mode.set(m); // maybe do in cfg?

        self.rfc.set_mode(m.mode);

        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();

        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        // Need to match on patches here but for now, just default to genfsk patches
        mce::MCE_PATCH.apply_patch();
        rfe::RFE_PATCH.apply_patch();

        unsafe {
            let reg_overrides: u32 = GFSK_RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides, 0x9F3F)
        }
    }

    pub fn power_down(&self) -> ReturnCode {
        self.rfc.disable()
    }
}
