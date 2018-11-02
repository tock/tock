use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::hil::rfcore;
use kernel::ReturnCode;
use osc;
use radio::commands::{prop_commands as prop, DirectCommand, RadioCommand, RfcCondition};
use radio::patch_cpe_prop as cpe;
use radio::patch_mce_genfsk as mce;
use radio::patch_mce_longrange as mce_lr;
use radio::patch_rfe_genfsk as rfe;
use radio::rfc;
use rtc;

const TEST_PAYLOAD: [u32; 30] = [0; 30];

static mut GFSK_RFPARAMS: [u32; 26] = [
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
    0x00000623, 0x0cf80002, //
    // override_phy_tx_pa_ramp_genfsk.xml
    0x50880002, // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1) ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    0x51110002, // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1) ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    // override_phy_rx_frontend_genfsk.xml
    0x001a609c, // Rx: Set AGC reference level to 0x1A (default: 0x2E) HW_REG_OVERRIDE(0x609C,0x001A),
    0x00018883, // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x000288A3, // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    // override_phy_rx_aaf_bw_0xd.xml
    0x7ddf0002, // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD) ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    0xFCFC08C3, // TX power override DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0x82A86C2B,
    /*
    0xFFFC08C3, // TX power override DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0x0cf80002, // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8) ADI_REG_OVERRIDE(0,12,0xF8),
    */
    0xFFFFFFFF, // Stop word
];

#[allow(unused)]
#[derive(Copy, Clone)]
pub enum CpePatch {
    GenFsk { patch: cpe::Patches },
}

#[allow(unused)]
#[derive(Copy, Clone)]
pub enum RfePatch {
    #[derive(Copy, Clone)]
    GenFsk { patch: rfe::Patches },
}

#[allow(unused)]
#[derive(Copy, Clone)]
pub enum McePatch {
    GenFsk { patch: mce::Patches },
    LongRange { patch: mce_lr::Patches },
}

#[allow(unused)]
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
            cpe_patch: CpePatch::GenFsk {
                patch: cpe::CPE_PATCH,
            },
            rfe_patch: RfePatch::GenFsk {
                patch: rfe::RFE_PATCH,
            },
            mce_patch: McePatch::GenFsk {
                patch: mce::MCE_PATCH,
            },
        }
    }
}

static mut COMMAND_BUF: [u8; 256] = [0; 256];
static mut TX_BUF: [u8; 240] = [0; 240];

#[allow(unused)]
// TODO Implement update config for changing radio modes and tie in the WIP power client to manage
// power state.
pub struct Radio {
    rfc: &'static rfc::RFCore,
    mode: OptionalCell<RadioMode>,
    tx_client: OptionalCell<&'static rfcore::TxClient>,
    rx_client: OptionalCell<&'static rfcore::RxClient>,
    cfg_client: OptionalCell<&'static rfcore::ConfigClient>,
    power_client: OptionalCell<&'static rfcore::PowerClient>,
    update_config: Cell<bool>,
    schedule_powerdown: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
    rx_buf: TakeCell<'static, [u8]>,
    tx_power: Cell<u16>,
}

impl Radio {
    pub const fn new(rfc: &'static rfc::RFCore) -> Radio {
        Radio {
            rfc,
            mode: OptionalCell::empty(),
            tx_client: OptionalCell::empty(),
            rx_client: OptionalCell::empty(),
            cfg_client: OptionalCell::empty(),
            power_client: OptionalCell::empty(),
            update_config: Cell::new(false),
            schedule_powerdown: Cell::new(false),
            tx_buf: TakeCell::empty(),
            rx_buf: TakeCell::empty(),
            tx_power: Cell::new(0x9F3F),
        }
    }

    pub fn power_up(&self) {
        // TODO Need so have some mode setting done in initialize callback perhaps to pass into
        // power_up() here, the RadioMode enum is defined above which will set a mode in this
        // multimode context along with applying the patches which are attached. Maybe it would be
        // best for the client to just pass an int for the mode and do it all here? not sure yet.

        // self.mode.set(m);

        self.rfc.set_mode(rfc::RfcMode::BLE);

        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();

        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        // Need to match on patches here but for now, just default to genfsk patches
        mce::MCE_PATCH.apply_patch();
        rfe::RFE_PATCH.apply_patch();

        unsafe {
            let reg_overrides: u32 = GFSK_RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides, 0x9F3F);
        }

        self.power_client
            .map(|client| client.power_mode_changed(true));
    }

    pub fn power_down(&self) {
        self.rfc.disable();

        self.power_client
            .map(|client| client.power_mode_changed(false));
    }

    unsafe fn replace_and_send_tx_buffer(&self, buf: &'static mut [u8], len: usize) {
        for i in 0..COMMAND_BUF.len() {
            COMMAND_BUF[i] = 0;
        }

        for i in 0..TX_BUF.len() {
            TX_BUF[i] = 0;
        }

        for (i, c) in buf.as_ref()[0..len].iter().enumerate() {
            TX_BUF[i] = *c;
        }

        self.tx_buf.put(Some(buf));

        self.tx_buf.map(|buf| {
            let p_packet = buf.as_mut_ptr() as u32;

            let cmd: &mut prop::CommandTx =
                &mut *(COMMAND_BUF.as_mut_ptr() as *mut prop::CommandTx);
            cmd.command_no = 0x3801;
            cmd.status = 0;
            cmd.p_nextop = 0;
            cmd.start_time = 0;
            cmd.start_trigger = 0;
            cmd.condition = {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            };
            cmd.packet_conf = {
                let mut packet = prop::RfcPacketConf(0);
                packet.set_fs_off(false);
                packet.set_use_crc(true);
                packet.set_var_len(false);
                packet
            };
            cmd.packet_len = len as u8;
            cmd.sync_word = 0x930B51DE;
            cmd.packet_pointer = p_packet;

            RadioCommand::guard(cmd);
            self.rfc
                .send_sync(cmd)
                .and_then(|_| self.rfc.wait(cmd))
                .ok();
        });
    }

    pub fn run_tests(&self) {
        self.rfc.set_mode(rfc::RfcMode::BLE);

        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();

        cpe::CPE_PATCH.apply_patch();
        mce::MCE_PATCH.apply_patch();
        rfe::RFE_PATCH.apply_patch();

        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        unsafe {
            let reg_overrides: u32 = GFSK_RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides, 0xFFFF);
        }

        self.test_radio_fs();

        self.test_radio_tx();
    }

    fn test_radio_tx(&self) {
        let mut packet = TEST_PAYLOAD;
        let mut seq: u32 = 0;
        for p in packet.iter_mut() {
            *p = seq;
            seq += 1;
        }
        let p_packet = packet.as_mut_ptr() as u32;

        unsafe {
            let cmd: &mut prop::CommandTx =
                &mut *(COMMAND_BUF.as_mut_ptr() as *mut prop::CommandTx);
            cmd.command_no = 0x3801;
            cmd.status = 0;
            cmd.p_nextop = 0;
            cmd.start_time = 0;
            cmd.start_trigger = 0;
            cmd.condition = {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            };
            cmd.packet_conf = {
                let mut packet = prop::RfcPacketConf(0);
                packet.set_fs_off(false);
                packet.set_use_crc(true);
                packet.set_var_len(false);
                packet
            };
            cmd.packet_len = 0x1E;
            cmd.sync_word = 0x930B51DE;
            cmd.packet_pointer = p_packet;

            RadioCommand::guard(cmd);

            self.rfc
                .send_sync(cmd)
                .and_then(|_| self.rfc.wait(cmd))
                .ok();
        }
    }

    fn test_radio_fs(&self) {
        let mut cmd_fs = prop::CommandFS {
            command_no: 0x0803,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
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

        RadioCommand::guard(&mut cmd_fs);
        self.rfc
            .send_sync(&cmd_fs)
            .and_then(|_| self.rfc.wait(&cmd_fs))
            .ok();
    }
}

impl rfc::RFCoreClient for Radio {
    fn command_done(&self) {
        unsafe { rtc::RTC.sync() };

        if self.schedule_powerdown.get() {
            // TODO Need to handle powerdown failure here or we will not be able to enter low power
            // modes
            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
            // do sleep mode here later
        }

        self.tx_buf.take().map_or(ReturnCode::ERESERVE, |tbuf| {
            self.tx_client
                .map(move |client| client.transmit_event(tbuf, ReturnCode::SUCCESS));
            ReturnCode::SUCCESS
        });
    }

    fn tx_done(&self) {
        unsafe { rtc::RTC.sync() };

        if self.schedule_powerdown.get() {
            // TODO Need to handle powerdown failure here or we will not be able to enter low power
            // modes
            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
            // do sleep mode here later
        }
        self.tx_buf.take().map_or(ReturnCode::ERESERVE, |tbuf| {
            self.tx_client
                .map(move |client| client.transmit_event(tbuf, ReturnCode::SUCCESS));
            ReturnCode::SUCCESS
        });
    }

    fn rx_ok(&self) {
        unsafe { rtc::RTC.sync() };

        self.rx_buf.take().map_or(ReturnCode::ERESERVE, |rbuf| {
            let frame_len = rbuf.len();
            let crc_valid = true;
            self.rx_client.map(move |client| {
                client.receive_event(rbuf, frame_len, crc_valid, ReturnCode::SUCCESS)
            });
            ReturnCode::SUCCESS
        });
    }
}

impl rfcore::Radio for Radio {}

impl rfcore::RadioDriver for Radio {
    fn set_transmit_client(&self, tx_client: &'static rfcore::TxClient) {
        self.tx_client.set(tx_client);
    }

    fn set_receive_client(&self, rx_client: &'static rfcore::RxClient, _rx_buf: &'static mut [u8]) {
        self.rx_client.set(rx_client);
    }

    fn set_receive_buffer(&self, _rx_buf: &'static mut [u8]) {
        // maybe make a rx buf only when needed?
    }

    fn set_config_client(&self, config_client: &'static rfcore::ConfigClient) {
        self.cfg_client.set(config_client);
    }

    fn set_power_client(&self, power_client: &'static rfcore::PowerClient) {
        self.power_client.set(power_client);
    }

    fn transmit(
        &self,
        buf: &'static mut [u8],
        frame_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        if frame_len > 240 {
            return (ReturnCode::ENOSUPPORT, Some(buf));
        }

        if self.tx_buf.is_none() {
            unsafe { self.replace_and_send_tx_buffer(buf, frame_len) };
            (ReturnCode::SUCCESS, None)
        } else {
            (ReturnCode::EBUSY, Some(buf))
        }
    }
}

impl rfcore::RadioConfig for Radio {
    fn initialize(&self) {
        self.power_up();
    }

    fn reset(&self) {
        self.power_down();
        self.power_up();
    }

    fn stop(&self) -> ReturnCode {
        let cmd_stop = DirectCommand::new(0x0402, 0);
        let stopped = self.rfc.send_direct(&cmd_stop).is_ok();
        if stopped {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        }
    }

    fn is_on(&self) -> bool {
        // TODO IMPL RADIO OPERATION COMMAND PING HERE
        true
    }

    fn busy(&self) -> bool {
        // TODO Might be an obsolete command here in favor of get_command_status and some logic on the
        // user size to determine if the radio is busy. Not sure what is best to have here but
        // arguing best might be bikeshedding
        let status = self.rfc.status.get();
        match status {
            0x0001 => true,
            0x0002 => true,
            _ => false,
        }
    }

    fn config_commit(&self) -> ReturnCode {
        // TODO confirm set new config here
        ReturnCode::SUCCESS
    }

    fn get_tx_power(&self) -> u16 {
        // TODO get tx power radio command
        self.tx_power.get()
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
        // TODO put some guards around the possible range for TX power
        self.tx_power.set(power);
        let command = DirectCommand::new(0x0010, power);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn send_stop_command(&self) -> ReturnCode {
        // Send "Gracefull" stop radio operation direct command
        let command = DirectCommand::new(0x0402, 0);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn send_kill_command(&self) -> ReturnCode {
        // Send immidiate command kill all radio operation commands
        let command = DirectCommand::new(0x0401, 0);
        if self.rfc.send_direct(&command).is_ok() {
            return ReturnCode::SUCCESS;
        } else {
            return ReturnCode::FAIL;
        }
    }

    fn set_frequency(&self, frequency: u16) -> ReturnCode {
        let mut cmd_fs = prop::CommandFS {
            command_no: 0x0803,
            status: 0,
            p_nextop: 0,
            start_time: 0,
            start_trigger: 0,
            condition: {
                let mut cond = RfcCondition(0);
                cond.set_rule(0x01);
                cond
            },
            frequency: frequency,
            fract_freq: 0x0000,
            synth_conf: {
                let mut synth = prop::RfcSynthConf(0);
                synth.set_tx_mode(false);
                synth.set_ref_freq(0x00);
                synth
            },
        };

        RadioCommand::guard(&mut cmd_fs);

        if self
            .rfc
            .send_sync(&cmd_fs)
            .and_then(|_| self.rfc.wait(&cmd_fs))
            .is_ok()
        {
            ReturnCode::SUCCESS
        } else {
            ReturnCode::FAIL
        }
    }
}
