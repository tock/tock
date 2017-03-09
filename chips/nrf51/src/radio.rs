use chip;
use core::cell::Cell;
use core::mem;

use gpio;
use kernel::common::VolatileCell;
use kernel::common::take_cell::TakeCell;
use kernel::hil::gpio::Pin;
use kernel::hil::radio_nrf51dk::{RadioDummy, Client};
use kernel::returncode::ReturnCode;
use nvic;
use peripheral_interrupts::NvicIdx;
extern crate capsules;
use self::capsules::led::LED;
use peripheral_registers::{RADIO_REGS, RADIO_BASE};

#[deny(no_mangle_const_items)]

pub const PACKET0_S1_SIZE: u32 = 0;
pub const PACKET0_S0_SIZE: u32 = 0;

pub const RADIO_PCNF0_S0LEN_Pos: u32 = 8;
pub const RADIO_PCNF0_S1LEN_Pos: u32 = 16;
pub const RADIO_PCNF0_LFLEN_Pos: u32 = 0;

pub const RADIO_PCNF1_WHITEEN_Disabled: u32 = 0;
pub const RADIO_PCNF1_WHITEEN_Enabled: u32 = 1;
pub const RADIO_PCNF1_WHITEEN_Pos: u32 = 25;

pub const RADIO_PCNF1_BALEN_Pos: u32 = 16;
pub const RADIO_PCNF1_STATLEN_Pos: u32 = 8;
pub const RADIO_PCNF1_MAXLEN_Pos: u32 = 0;

pub const RADIO_PCNF1_ENDIAN_Pos: u32 = 24;
pub const RADIO_PCNF1_ENDIAN_Big: u32 = 1;

pub const PACKET_LENGTH_FIELD_SIZE: u32 = 0;
pub const PACKET_PAYLOAD_MAXSIZE: u32 = 64;
pub const PACKET_BASE_ADDRESS_LENGTH: u32 = 4;
pub const PACKET_STATIC_LENGTH: u32 = 64;



static mut tx_buf: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
static mut rx_buf: [u8; 12] = [0x00; 12];

// FROM LEFT
// ADVTYPE      ;;      4 bits
// RFU          ;;      2 bits
// TxAdd        ;;      1 bit
// RxAdd        ;;      1 bit
// Length      ;;      6 bits
// RFU          ;;      2 bits
// AdvD         ;;      6 bytes
// AdvData      ;;      4 bytes
// static mut payload: [u8; 12] = [0x02, 0x28, 0x41,0x41,0x41, 0x41, 0x41, 0x41, 1, 2, 3, 4];
// static mut payload: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0x00];

//static mut payload: [u8; 12] = [0x02, 0x28, 0x41,0x41,0x41, 0x41, 0x41, 0x41, 1, 2, 3, 4];
static mut payload: [u8; 22] = [ 0x02, 0x13, 0x00, // ADV_IND, public addr
                    0x90, 0xD8, 0x7A, 0xBD, 0xA3, 0xED, // Address
                    0x0C, 0x09, 0x41, 0x77, 0x65, 0x73, 0x6f, 0x6d, 0x65, 0x52, 0x75, 0x73, 0x74]; //Payload
#[no_mangle]
pub struct Radio {
    regs: *mut RADIO_REGS,
    client: Cell<Option<&'static Client>>, 
    // tx_buffer: TakeCell<'static, [u8]>,
    // rx_buffer: TakeCell<'static, [u8]>,
}

pub static mut RADIO: Radio = Radio::new();


impl Radio {
    #[inline(never)]
    #[no_mangle]
    pub const fn new() -> Radio {
        Radio {
            regs: RADIO_BASE as *mut RADIO_REGS,
            client: Cell::new(None), 
            // tx_buffer: TakeCell::empty(),
            // rx_buffer : TakeCell::empty(),
        }
    }
    pub fn set_client<C: Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }


    pub fn turnOnLeds(&self) {

        unsafe {
            let led0 = &gpio::PORT[21];
            led0.make_output();
            led0.toggle();
        }
    }


    pub fn config(&self) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };

        self.radio_on();

        // TX Power 0 dB
        self.set_txpower(0);

        // BLE MODE
        self.set_channel_rate(0x03);

        self.set_channel_freq(37);
        self.set_data_white_iv(37);

        // Set PREFIX | BASE Address
        regs.PREFIX0.set(0x0000008e);
        regs.BASE0.set(0x89bed600);

        self.set_tx_address(0x00);
        self.set_rx_address(0x01);
        // regs.RXMATCH.set(0x00);

        // Set Packet Config
        self.set_packet_config(0x00);

        // CRC Config
        self.set_crc_config();

        // Buffer configuration
        self.set_tx_buffer();

        self.enable_interrupts();
        self.enable_nvic();
    }

    pub fn set_crc_config(&self) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        // CRC Config
        regs.CRCCNF.set(0x103); // 3 bytes CRC and don't include Address field in the CRC
        regs.CRCINIT.set(0x555555); // INIT CRC Value
        // CRC Polynomial  x24 + x10 + x9 + x6 + x4 + x3 + x + 1
        regs.CRCPOLY.set(0x00065B);
    }


    // Packet configuration
    pub fn set_packet_config(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };

        // This initlization have to do with the header in the PDU it is 2 bytes
        // ADVTYPE      ;;      4 bits
        // RFU          ;;      2 bits
        // TxAdd        ;;      1 bit
        // RxAdd        ;;      1 bit
        // Legngth      ;;      6 bits
        // RFU          ;;      2 bits
        regs.PCNF0.set(// set S0 to 1 byte
                       (1 << RADIO_PCNF0_S0LEN_Pos) |
            // set S1 to 2 bits
            (2 << RADIO_PCNF0_S1LEN_Pos) |
            // set length to 6 bits
            (6 << RADIO_PCNF0_LFLEN_Pos));


        regs.PCNF1.set(//
                       (RADIO_PCNF1_WHITEEN_Enabled << RADIO_PCNF1_WHITEEN_Pos) |
            // set little-endian
            (0 << RADIO_PCNF1_ENDIAN_Pos)  |
            // Set BASE + PREFIX address to 4 bytes
            (3 << RADIO_PCNF1_BALEN_Pos)   |
            // don't extend packet length
            (0 << RADIO_PCNF1_STATLEN_Pos) |
            // max payload size 37
            (37 << RADIO_PCNF1_MAXLEN_Pos));
    }

    pub fn set_rx_address(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        regs.RXADDRESSES.set(0x01);
    }

    pub fn set_tx_address(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        regs.TXADDRESS.set(0x00);
    }

    pub fn set_channel_rate(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        // set channel rate,  3 - BLE 1MBIT/s
        regs.MODE.set(3);
    }
    pub fn set_data_white_iv(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        // DATAIV
        regs.DATAWHITEIV.set(val);
    }

    pub fn set_channel_freq(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        //37, 38 and 39 for adv.
        match val {
            37 => regs.FREQEUNCY.set(2),
            38 => regs.FREQEUNCY.set(20),
            39 => regs.FREQEUNCY.set(80),
            _ => regs.FREQEUNCY.set(7),
        }
    }

    pub fn radio_on(&self) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        // reset and enable power
        regs.POWER.set(0);
        regs.POWER.set(1);
    }

    pub fn set_txpower(&self, val: u32) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        regs.TXPOWER.set(val);
    }

    pub fn set_tx_buffer(&self) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        unsafe {
            regs.PACKETPTR.set((&payload as *const u8) as u32);
        }
    }

    pub fn set_rx_buffer(&self) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        unsafe {
            regs.PACKETPTR.set((&rx_buf as *const u8) as u32);
        }
    }

    #[inline(never)]
    #[no_mangle]
    pub fn tx(&self, dest: u16, tx_data: &'static mut [u8], tx_len: u8) {

        for (i, c) in tx_data.as_ref()[0..16].iter().enumerate() {
            unsafe {
                tx_buf[i] = *c;
            }
        }
        self.set_tx_buffer();
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        regs.READY.set(0);
        regs.TXEN.set(1);
    }

    #[inline(never)]
    #[no_mangle]
    pub fn rx(&self) {

        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        regs.READY.set(0);
        regs.RXEN.set(1);
    }

    // #[inline(never)]
    // #[no_mangle]
    pub fn handle_interrupt(&self) {
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };

        if regs.READY.get() == 1 {
            if regs.STATE.get() <= 4 {
                self.set_rx_buffer();
            } else {
                self.set_tx_buffer();
            }
            regs.READY.set(0);
            regs.END.set(0);
            regs.START.set(1);
        }

        if regs.PAYLOAD.get() == 1 {
            regs.PAYLOAD.set(0);
        }

        if regs.ADDRESS.get() == 1 {
            regs.ADDRESS.set(0);
            // panic!("ADDRESS");
        }

        if regs.END.get() == 1 {
            self.turnOnLeds();
            regs.END.set(0);
            regs.DISABLE.set(1);
            if regs.STATE.get() <= 4 {
                if (regs.CRCSTATUS.get() == 0) {

                    panic!("crc status {:?}\n", regs.CRCSTATUS.get());
                }
                unsafe {
                    self.client.get().map(|client| client.receive_done(&mut rx_buf, 0));
                }
            } else {
                // TODO: Implement something.
                unsafe {
                    self.client.get().map(|client| client.transmit_done(&mut tx_buf, 0));
                }
            }


        }
        nvic::clear_pending(NvicIdx::RADIO);
    }


    pub fn enable_interrupts(&self) {
        // INTENSET REG
        let regs: &mut RADIO_REGS = unsafe { mem::transmute(self.regs) };
        // 15 i.e set 4 LSB
        regs.INTENSET.set(1 | 1 << 1 | 1 << 2 | 1 << 3);
    }

    pub fn disable_interrupts(&self) {
        panic!("NOT IMPLEMENTED YET");
    }

    pub fn enable_nvic(&self) {
        nvic::enable(NvicIdx::RADIO);
    }

    pub fn disable_nvic(&self) {
        nvic::disable(NvicIdx::RADIO);
    }
}
// Methods of RadioDummy Trait/Interface and are shared between Capsules and Chips
impl RadioDummy for Radio {
    // This Function is called once Tock is booted
    fn init(&self) {
        self.config()
    }

    // This Function is called once a radio packet is to be sent
    fn send(&self) {
        unsafe {
            self.tx(0, &mut tx_buf, 0);
        }
        // panic!("NOT USED ATM");
    }

    // This Function is called once a radio packet is to be sent
    fn receive(&self) {
        self.rx();
    }

    #[inline(never)]
    #[no_mangle]
    fn transmit(&self, dest: u16, tx_data: &'static mut [u8], tx_len: u8) -> ReturnCode {

        self.tx(dest, tx_data, tx_len);
        ReturnCode::SUCCESS
    }


    fn dummy(&self) -> isize {
        1337
    }

    fn set_channel(&self, ch: usize) {
        self.set_channel_freq(ch as u32);
        self.set_data_white_iv(ch as u32);
    }
}


#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn RADIO_Handler() {
    use kernel::common::Queue;
    nvic::disable(NvicIdx::RADIO);
    chip::INTERRUPT_QUEUE.as_mut().unwrap().enqueue(NvicIdx::RADIO);
}
