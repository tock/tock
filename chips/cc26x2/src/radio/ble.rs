//! BLE Controller
//!     Manages bluetooth.
//!

use self::ble_commands::*;
use core::cell::Cell;
use osc;
use radio::{commands, rfc};
use rtc;

use kernel;
use radio::ble::ble_commands::BleAdvertise;

use chip::SleepMode;
use kernel::hil::ble_advertising::{self, RadioChannel};
use peripheral_manager;

static mut BLE_OVERRIDES: [u32; 7] = [
    0x00364038, /* Synth: Set RTRIM (POTAILRESTRIM) to 6 */
    0x000784A3, /* Synth: Set FREF = 3.43 MHz (24 MHz / 7) */
    0xA47E0583, /* Synth: Set loop bandwidth after lock to 80 kHz (K2) */
    0xEAE00603, /* Synth: Set loop bandwidth after lock to 80 kHz (K3, LSB) */
    0x00010623, /* Synth: Set loop bandwidth after lock to 80 kHz (K3, MSB) */
    0x00456088, /* Adjust AGC reference level */
    0xFFFFFFFF, /* End of override list */
];

/*
    We need to use static buffers in order to make them
    constantly accessible by the radio MCU (we need to assure that they
    won't be deallocated).
*/
static mut BLE_PARAMS_BUF: [u32; 32] = [0; 32];
static mut BLE_ADV_PAYLOAD: [u8; 64] = [0; 64];
static mut BLE_ADV_PAYLOAD_LEN: u8 = 0;
static mut PACKET_BUF: [u8; 128] = [0; 128];
static mut DEVICE_ADDRESS: [u8; 6] = [0; 6];

pub struct Ble {
    rfc: &'static rfc::RFCore,
    rx_client: Cell<Option<&'static ble_advertising::RxClient>>,
    tx_client: Cell<Option<&'static ble_advertising::TxClient>>,
    schedule_powerdown: Cell<bool>,
    safe_to_deep_sleep: Cell<bool>,
}

#[allow(unused)]
#[repr(u16)]
enum BleAdvertiseCommands {
    ConnectUndirected = 0x1803,
    ConnectDirected = 0x1804,
    NonConnectUndirected = 0x1805,

    // TODO(cpluss): implement scan
    ScanRequest = 0x1808,
    ScanUndirected = 0x1806,
    // TODO(cpluss): correct and add these
    // ScanResponse = 0x04,
    // ConnectRequest = 0x05,
}

impl Ble {
    pub const fn new(rfc: &'static rfc::RFCore) -> Ble {
        Ble {
            rfc,
            rx_client: Cell::new(None),
            tx_client: Cell::new(None),
            schedule_powerdown: Cell::new(false),
            safe_to_deep_sleep: Cell::new(true),
        }
    }

    pub fn power_up(&self) {
        self.safe_to_deep_sleep.set(false);

        self.rfc.set_mode(rfc::RfcMode::BLE);

        /*
            The BLE communication is synchronous, so we need to be synchronized to the same
            clock frequency. The best accuracy is achieved when using the XTAL Oscillator.
            However, it takes a while for it to pulse correctly, so we enable it
            before switching to it.
        */
        osc::OSC.request_switch_to_hf_xosc();

        self.rfc.enable();
        self.rfc.start_rat();

        osc::OSC.switch_to_hf_xosc();

        unsafe {
            let reg_overrides: u32 = BLE_OVERRIDES.as_mut_ptr() as u32; //(&BLE_OVERRIDES[0] as *const u32) as u32;
            self.rfc.setup(reg_overrides, 0xFFF);
        }
    }

    pub fn power_down(&self) {
        self.rfc.disable();
    }

    /*
        The payload is assembled be the Cortex-M0 radio MCU. We need to extract
        parts of the payload to correctly propagate them.
    */
    unsafe fn replace_adv_payload_buffer(
        &self,
        buf: &'static mut [u8],
        len: usize,
    ) -> &'static mut [u8] {
        const PACKET_ADDR_START: usize = 2;
        const PACKET_ADDR_END: usize = 8;
        const PACKET_PAYLOAD_START: usize = 8;
        const PACKET_HDR_PDU: usize = 0;

        // Extract the device address
        for (i, a) in buf.as_ref()[PACKET_ADDR_START..PACKET_ADDR_END]
            .iter()
            .enumerate()
        {
            DEVICE_ADDRESS[i] = *a;
        }

        // Copy the rest of the payload
        for (i, c) in buf.as_ref()[PACKET_PAYLOAD_START..len].iter().enumerate() {
            BLE_ADV_PAYLOAD[i] = *c;
        }

        BLE_ADV_PAYLOAD_LEN = (len - PACKET_PAYLOAD_START) as u8;

        // Reset the packet buffers
        for i in 0..BLE_PARAMS_BUF.len() {
            BLE_PARAMS_BUF[i] = 0;
        }
        for i in 0..PACKET_BUF.len() {
            PACKET_BUF[i] = 0;
        }

        let params: &mut BleAdvertiseParams =
            &mut *(BLE_PARAMS_BUF.as_mut_ptr() as *mut BleAdvertiseParams);
        params.device_address = &mut DEVICE_ADDRESS[0] as *mut u8;
        params.adv_len = BLE_ADV_PAYLOAD_LEN;
        params.adv_data = BLE_ADV_PAYLOAD.as_ptr() as u32;
        params.end_time = 0;
        params.end_trigger = 1;

        let pdu: u8 = buf[PACKET_HDR_PDU] & 0xF;
        let rfc_command_num: u16 = match pdu {
            0x00 => BleAdvertiseCommands::ConnectUndirected,
            0x01 => BleAdvertiseCommands::ConnectDirected,
            0x02 => BleAdvertiseCommands::NonConnectUndirected,
            _ => panic!("{} ble PDU not implemented yet.", pdu),
        } as u16;

        let cmd: &mut BleAdvertise = &mut *(PACKET_BUF.as_mut_ptr() as *mut BleAdvertise);
        cmd.command_no = rfc_command_num;
        cmd.condition = {
            let mut cnd = commands::RfcCondition(0);
            cnd.set_rule(1); // COND_NEVER
            cnd
        };
        cmd.whitening = {
            let mut wht = BleWhitening(0);
            wht.set_override(true);
            wht.set_init(0x51);
            wht
        };
        cmd.params = BLE_PARAMS_BUF.as_ptr() as u32;

        buf
    }

    pub fn advertise(&self, radio_channel: RadioChannel) {
        if self.rfc.current_mode() != Some(rfc::RfcMode::BLE) {
            self.power_up();
        }

        let _channel = match radio_channel {
            RadioChannel::AdvertisingChannel37 => 37,
            RadioChannel::AdvertisingChannel38 => 38,
            RadioChannel::AdvertisingChannel39 => 39,
            _ => panic!("Tried to advertise on a communication channel.\r"),
        };
        /*
        unsafe {
            let cmd: &mut BleAdvertise = &mut *(PACKET_BUF.as_mut_ptr() as *mut BleAdvertise);
            cmd.status = 0;
            cmd.channel = channel;
            match self.rfc.send(cmd) {
                Err(status) => panic!("Could not send advertisement, status=0x{:x}", status),
                Ok(()) => (),
            }
        }
        */
    }
}

impl rfc::RFCoreClient for Ble {
    fn command_done(&self) {
        for _i in 0..0xFFF {
            unsafe { rtc::RTC.sync() };
        }

        if self.schedule_powerdown.get() {
            self.power_down();
            osc::OSC.switch_to_hf_rcosc();

            self.schedule_powerdown.set(false);
            self.safe_to_deep_sleep.set(true);
        }

        self.tx_client
            .get()
            .map(|client| client.transmit_event(kernel::ReturnCode::SUCCESS));
    }

    fn tx_done(&self) {}

    fn rx_ok(&self) {}
}

impl ble_advertising::BleAdvertisementDriver for Ble {
    fn transmit_advertisement(
        &self,
        buf: &'static mut [u8],
        len: usize,
        channel: RadioChannel,
    ) -> &'static mut [u8] {
        if channel == RadioChannel::AdvertisingChannel37 {
            self.schedule_powerdown.set(false);
            self.power_up();
        }

        let res = unsafe { self.replace_adv_payload_buffer(buf, len) };
        self.advertise(channel);

        if channel == RadioChannel::AdvertisingChannel39 {
            self.schedule_powerdown.set(true);
        }

        res
    }

    fn receive_advertisement(&self, _channel: RadioChannel) {}

    fn set_receive_client(&self, client: &'static ble_advertising::RxClient) {
        self.rx_client.set(Some(client));
    }

    fn set_transmit_client(&self, client: &'static ble_advertising::TxClient) {
        self.tx_client.set(Some(client));
    }
}

impl ble_advertising::BleConfig for Ble {
    fn set_tx_power(&self, _tx_power: u8) -> kernel::ReturnCode {
        kernel::ReturnCode::SUCCESS
    }
}

impl peripheral_manager::PowerClient for Ble {
    fn before_sleep(&self, _sleep_mode: u32) {}

    fn after_wakeup(&self, _sleep_mode: u32) {}

    fn lowest_sleep_mode(&self) -> u32 {
        if self.safe_to_deep_sleep.get() {
            SleepMode::DeepSleep as u32
        } else {
            SleepMode::Sleep as u32
        }
    }
}

pub mod ble_commands {
    use radio::commands::RfcCondition;

    #[repr(C)]
    pub struct BleAdvertise {
        pub command_no: u16,
        pub status: u16,
        pub p_nextop: u32,
        pub ratmr: u32,
        pub start_trigger: u8,
        pub condition: RfcCondition,

        pub channel: u8,
        pub whitening: BleWhitening,

        pub params: u32,
        pub output: u32,
    }

    #[repr(C)]
    pub struct BleAdvertiseParams {
        pub rx_queue: u32, // pointer to receive queue
        pub rx_config: u8,
        pub adv_config: u8,

        pub adv_len: u8,
        pub scan_rsp_len: u8,

        pub adv_data: u32,
        pub scan_rsp_data: u32,
        pub device_address: *const u8,

        pub white_list: u32,

        pub __dummy0: u16,
        pub __dummy1: u8,

        pub end_trigger: u8,
        pub end_time: u32,
    }

    bitfield!{
        #[derive(Copy, Clone)]
        pub struct BleWhitening(u8);
        impl Debug;
        pub _init, set_init: 6, 0;
        pub _override, set_override: 1;
    }
}
