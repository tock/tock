#![allow(unused_imports)]
use radio::commands as cmd;
use osc; 
use peripheral_manager;
use chip::SleepMode;
use radio::rfc;
use rom_fns::oscfh;
use core::cell::Cell;
use core::ptr;
use fixedvec::FixedVec;
use kernel::common::cells::{TakeCell, OptionalCell};
use kernel::hil::radio_client::{self, RadioAttrs};
use kernel::{AppId, AppSlice, Callback, Driver, ReturnCode, Grant, Shared};
use radio::rfcore_const::{
    RFCommandStatus, RadioCommands, RfcDriverCommands, RfcOperationStatus, State, CMD_STACK };

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
    0x00214AD3, // Synth: Configure faster calibration
    0x02980243, // Synth: Decrease synth programming time-out by 90 us from default (0x0298 RAT ticks = 166 us)
    0x0A480583, // Synth: Set loop bandwidth after lock to 20 kHz
    0x7AB80603, // Synth: Set loop bandwidth after lock to 20 kHz
    0x00000623, // Synth: Set loop bandwidth after lock to 20 kHz
    // Tx: Configure PA ramping, set wait time before turning off (0x1F ticks of 16/24 us = 20.7 us).
    // HW_REG_OVERRIDE(0x6028,0x001F),
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[3]=1)
    // ADI_HALFREG_OVERRIDE(0,16,0x8,0x8),
    // Tx: Configure PA ramp time, PACTL2.RC=0x3 (in ADI0, set PACTL2[4]=1)
    // ADI_HALFREG_OVERRIDE(0,17,0x1,0x1),
    // Rx: Set AGC reference level to 0x1A (default: 0x2E)
    // HW_REG_OVERRIDE(0x609C,0x001A),
    0x00018883, // Rx: Set LNA bias current offset to adjust +1 (default: 0)
    0x000288A3, // Rx: Set RSSI offset to adjust reported RSSI by -2 dB (default: 0)
    // Rx: Set anti-aliasing filter bandwidth to 0xD (in ADI0, set IFAMPCTL3[7:4]=0xD)
    // ADI_HALFREG_OVERRIDE(0,61,0xF,0xD),
    // TX power override
    // DC/DC regulator: In Tx with 14 dBm PA setting, use DCDCCTL5[3:0]=0xF (DITHER_EN=1 and IPEAK=7). In Rx, use DCDCCTL5[3:0]=0xC (DITHER_EN=1 and IPEAK=4).
    0xFFFC08C3,
    // Tx: Set PA trim to max to maximize its output power (in ADI0, set PACTL0=0xF8)
    // ADI_REG_OVERRIDE(0,12,0xF8),
    0xFFFFFFFF,
];

pub struct Radio {
    rfc: &'static rfc::RFCore,
    callback: Cell<Option<Callback>>,
    tx_radio_client: OptionalCell<&'static radio_client::TxClient>,
    rx_radio_client: OptionalCell<&'static radio_client::RxClient>,
    schedule_powerdown: Cell<bool>,
    tx_buf: TakeCell<'static, [u8]>,
}

impl Radio {
    pub const fn new(rfc: &'static rfc::RFCore) -> Radio {
        Radio {
            rfc,
            callback: Cell::new(None),
            tx_radio_client: OptionalCell::empty(),
            rx_radio_client: OptionalCell::empty(),
            schedule_powerdown: Cell::new(false),
            tx_buf: TakeCell::empty(),
        }
    }

    pub fn power_up(&self) {
        self.rfc.set_mode(rfc::RfcMode::IEEE);

        // unsafe { oscfh::OSCHF_TurnOnXosc() };
        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();
        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();
        // unsafe { oscfh::OSCHF_AttemptToSwitchToXosc() };
        
        unsafe {
            let reg_overrides: u32 = RFPARAMS.as_mut_ptr() as u32;
            self.rfc.setup(reg_overrides, 0xFFF)
        }
    }

    pub fn power_down(&self) {
        self.rfc.disable();
    }
}

impl rfc::RFCoreClient for Radio {
    fn command_done(&self) {
        // Map standard callback to a command client.
    }

    fn tx_done(&self) {
        if self.schedule_powerdown.get() {
            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
        }

        let buf = self.tx_buf.take();
        self.tx_radio_client
            .take()
            .map(|client| client.transmit_event(buf.unwrap(), ReturnCode::SUCCESS));
    }

    fn rx_ok(&self) {}
}

impl peripheral_manager::PowerClient for Radio {
    fn before_sleep(&self, _sleep_mode: u32) {
    }

    fn after_wakeup(&self, _sleep_mode: u32) {
    }

    fn lowest_sleep_mode(&self) -> u32 {
        /*
        if self.safe_to_deep_sleep.get() {
            SleepMode::DeepSleep as u32
        } else {
            SleepMode::Sleep as u32
        }
        */
        SleepMode::Sleep as u32
    }
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
        let command: RfcDriverCommands = minor_num.into();
        match command {
            // Handle callback for command status after command is finished
            RfcDriverCommands::Direct => {
                /*
                match self.rfc.wait(&r2) {
                    Ok(()) => {
                        ReturnCode::SUCCESS
                    }
                    Err(e) => {
                        ReturnCode::FAIL
                    }
                }
                */
                ReturnCode::SUCCESS
            }
            RfcDriverCommands::NotSupported => panic!("Invalid command status"),
            _ => panic!("Unimplemented!"),
        }
    }
}

impl RadioAttrs for Radio {
    fn set_tx_client(&self, tx_client: &'static radio_client::TxClient) {
        self.tx_radio_client.set(tx_client);
    }

    fn set_rx_client(
        &self,
        rx_client: &'static radio_client::RxClient,
        _rx_buf: &'static mut [u8],
    ) {
        self.rx_radio_client.set(rx_client);
    }

    fn set_receive_buffer(&self, _rx_buf: &'static mut [u8]) {
        // maybe make a rx buf only when needed?
    }

    fn transmit(&self, _tx_buf: &'static mut [u8], _frame_len: usize) -> (ReturnCode, Option<&'static mut [u8]>) {
        (ReturnCode::SUCCESS, None)
    }
}
