use core::cell::Cell;
use kernel::hil::gpio::Pin;
use kernel::hil::spi;
//use virtual_spi::VirtualSpiMasterDevice;
use kernel::returncode::ReturnCode;
use rf233_const::*;
use core::mem;

macro_rules! pinc_toggle {
    ($x:expr) => {
        unsafe {
            let toggle_reg: &mut u32 = mem::transmute(0x400E1000 + (2 * 0x200) + 0x5c);
            *toggle_reg = 1 << $x;
        }
    }
}

#[allow(unused_variables, dead_code,non_camel_case_types)]
#[derive(Copy, Clone, PartialEq)]
enum InternalState {
    START,
    START_PART_READ,
    START_STATUS_READ,
    START_TURNING_OFF,
    START_CTRL1_SET,
    START_CCA_SET,
    START_PWR_SET,
    START_CTRL2_SET,
    START_IRQMASK_SET,
    START_XAH1_SET,
    START_XAH0_SET,
    START_PANID0_SET,
    START_PANID1_SET,
    START_IEEE0_SET,
    START_IEEE1_SET,
    START_IEEE2_SET,
    START_IEEE3_SET,
    START_IEEE4_SET,
    START_IEEE5_SET,
    START_IEEE6_SET,
    START_IEEE7_SET,
    START_SHORT0_SET,
    START_SHORT1_SET,
    START_RPC_SET,

    ON_STATUS_READ,
    ON_PLL_SET,

    READY,

    UNKNOWN,
}


pub struct RF233 <'a, S: spi::SpiMasterDevice + 'a> {
    spi: &'a S,
    radio_on: Cell<bool>,
    transmitting: Cell<bool>,
    spi_busy: Cell<bool>,
    reset_pin: &'a Pin,
    sleep_pin: &'a Pin,
    state: Cell<InternalState>,
}

static mut read_buf: [u8; 2] =  [0x0; 2];
static mut write_buf: [u8; 2] = [0x0; 2];

impl <'a, S: spi::SpiMasterDevice + 'a> spi::SpiMasterClient for RF233 <'a, S> {

    fn read_write_done(&self,
                       _write: &'static mut [u8],
                       _read: Option<&'static mut [u8]>,
                       _len: usize) {
        self.spi_busy.set(false);
        match self.state.get() {
            InternalState::START => {
                self.state_transition_read(RF233Register::IRQ_STATUS,
                                            InternalState::START_PART_READ);
            }
            InternalState::START_PART_READ => {
                self.state_transition_read(RF233Register::TRX_STATUS,
                                           InternalState::START_STATUS_READ);
            }
            InternalState::START_STATUS_READ => {
                unsafe {
                    let val = read_buf[0];
                    if val == ExternalState::ON as u8{
                        self.state_transition_write(RF233Register::TRX_STATE,
                                                    RF233TrxCmd::OFF as u8,
                                                    InternalState::START_TURNING_OFF);
                    } else {
                        // enable IRQ input
                        // clear IRQ
                        // enable IRQ interrrupt
                        self.state_transition_write(RF233Register::TRX_CTRL_1,
                                                    TRX_CTRL_1,
                                                    InternalState::START_CTRL1_SET);
                    }
                }
            }
            InternalState::START_TURNING_OFF => {
                // enable IRQ input
                // clear IRQ
                // enable IRQ interrrupt
                self.state_transition_write(RF233Register::TRX_CTRL_1,
                                            TRX_CTRL_1,
                                            InternalState::START_CTRL1_SET);
            }
            InternalState::START_CTRL1_SET => {
                self.state_transition_write(RF233Register::PHY_CC_CCA,
                                            PHY_CC_CCA,
                                            InternalState::START_CCA_SET);
            }
            InternalState::START_CCA_SET => {
                self.state_transition_write(RF233Register::PHY_TX_PWR,
                                            PHY_TX_PWR,
                                            InternalState::START_PWR_SET);
            }
            InternalState::START_PWR_SET => {
                self.state_transition_write(RF233Register::TRX_CTRL_2,
                                            TRX_CTRL_2,
                                            InternalState::START_CTRL2_SET)
            }
            InternalState::START_CTRL2_SET => {
                self.state_transition_write(RF233Register::IRQ_MASK,
                                            IRQ_MASK,
                                            InternalState::START_IRQMASK_SET);
            }

            InternalState::START_IRQMASK_SET => {
                self.state_transition_write(RF233Register::XAH_CTRL_1,
                                            XAH_CTRL_1,
                                            InternalState::START_XAH1_SET);
            }

            InternalState::START_XAH1_SET => {
                // This encapsulates the frame retry and CSMA retry
                // settings in the RF233 C code
                self.state_transition_write(RF233Register::XAH_CTRL_0,
                                            XAH_CTRL_0,
                                            InternalState::START_XAH0_SET);
            }
            InternalState::START_XAH0_SET => {
                self.state_transition_write(RF233Register::PAN_ID_0,
                                            PAN_ID_0,
                                            InternalState::START_PANID0_SET);
            }
            InternalState::START_PANID0_SET => {
                self.state_transition_write(RF233Register::PAN_ID_1,
                                            PAN_ID_1,
                                            InternalState::START_PANID1_SET);
            }
            InternalState::START_PANID1_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_0,
                                            IEEE_ADDR_0,
                                            InternalState::START_IEEE0_SET);
            }
            InternalState::START_IEEE0_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_1,
                                            IEEE_ADDR_1,
                                            InternalState::START_IEEE1_SET);
            }
            InternalState::START_IEEE1_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_2,
                                            IEEE_ADDR_2,
                                            InternalState::START_IEEE2_SET);
            }
            InternalState::START_IEEE2_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_3,
                                            IEEE_ADDR_3,
                                            InternalState::START_IEEE3_SET);
            }
            InternalState::START_IEEE3_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_4,
                                            IEEE_ADDR_4,
                                            InternalState::START_IEEE4_SET);
            }
            InternalState::START_IEEE4_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_5,
                                            IEEE_ADDR_5,
                                            InternalState::START_IEEE5_SET);
            }
            InternalState::START_IEEE5_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_6,
                                            IEEE_ADDR_6,
                                            InternalState::START_IEEE6_SET);
            }
            InternalState::START_IEEE6_SET => {
                self.state_transition_write(RF233Register::IEEE_ADDR_7,
                                            IEEE_ADDR_7,
                                            InternalState::START_IEEE7_SET);
            }
            InternalState::START_IEEE7_SET => {
                self.state_transition_write(RF233Register::SHORT_ADDR_0,
                                            SHORT_ADDR_0,
                                            InternalState::START_SHORT0_SET);
            }
            InternalState::START_SHORT0_SET => {
                self.state_transition_write(RF233Register::SHORT_ADDR_1,
                                            SHORT_ADDR_1,
                                            InternalState::START_SHORT1_SET);
            }
            InternalState::START_SHORT1_SET => {
                self.state_transition_write(RF233Register::TRX_RPC,
                                            TRX_RPC,
                                            InternalState::START_RPC_SET);
            }
            InternalState::START_RPC_SET => {
                // If asleep, turn on
                self.state_transition_read(RF233Register::TRX_STATUS,
                                           InternalState::ON_STATUS_READ);
            }
            InternalState::ON_STATUS_READ => {
                unsafe {
                    let val = read_buf[1];
                    self.state_transition_write(RF233Register::TRX_STATE,
                                                TRX_PLL_ON,
                                                InternalState::ON_PLL_SET);
                }
            }
            InternalState::ON_PLL_SET => {
                // We handle an interrupt to denote the PLL is good,
                // so do nothing here
            }
            InternalState::READY => {}
            InternalState::UNKNOWN => {}
        }
    }
}

impl<'a, S: spi::SpiMasterDevice + 'a> RF233 <'a, S> {
    pub fn new(spi: &'a S,
               reset: &'a Pin,
               sleep: &'a Pin) -> RF233<'a, S> {
        RF233 {
            spi: spi,
            reset_pin: reset,
            sleep_pin: sleep,
            radio_on: Cell::new(false),
            transmitting: Cell::new(false),
            spi_busy: Cell::new(false),
            state: Cell::new(InternalState::START),
        }
    }

    pub fn initialize(&self) -> ReturnCode {
        //self.spi.spi.set_client(&self.spi);
        self.spi.configure(spi::ClockPolarity::IdleLow,
                           spi::ClockPhase::SampleLeading,
                           100000);
        self.reset()
    }

    pub fn reset(&self) -> ReturnCode {
        self.reset_pin.make_output();
        self.sleep_pin.make_output();
        self.reset_pin.clear();
        // delay 1 ms
        self.reset_pin.set();
        self.sleep_pin.clear();
        self.transmitting.set(false);
        self.radio_on.set(true);
        ReturnCode::SUCCESS
    }
#[allow(dead_code)]
    pub fn start(&self) -> ReturnCode {
        if self.state.get() != InternalState::START {
            return ReturnCode::FAIL;
        }
        self.register_read(RF233Register::PART_NUM);
        ReturnCode::SUCCESS
    }

#[allow(dead_code)]
    fn register_write(&self,
                      reg: RF233Register,
                      val: u8) -> ReturnCode {

        if self.spi_busy.get() {return ReturnCode::EBUSY;}
        unsafe {
            write_buf[0] = (reg as u8) | RF233BusCommand::REGISTER_WRITE as u8;
            write_buf[1] = val;
            self.spi.read_write_bytes(&mut write_buf, Some(& mut read_buf), 2);
            self.spi_busy.set(true);
        }
        ReturnCode::SUCCESS
    }

    fn register_read(&self,
                     reg: RF233Register) -> ReturnCode {

        if self.spi_busy.get() {return ReturnCode::EBUSY;}
        unsafe {
            write_buf[0] = (reg as u8) | RF233BusCommand::REGISTER_READ as u8;
            write_buf[1] = 0;
            self.spi.read_write_bytes(&mut write_buf, Some(&mut read_buf), 2);
            self.spi_busy.set(true);
        }
        ReturnCode::SUCCESS
    }

    fn state_transition_write(&self,
                              reg: RF233Register,
                              val: u8,
                              state: InternalState) {
        self.state.set(state);
        self.register_write(reg, val);
    }

    fn state_transition_read(&self,
                             reg: RF233Register,
                             state: InternalState) {
        self.state.set(state);
        self.register_read(reg);
    }



}
