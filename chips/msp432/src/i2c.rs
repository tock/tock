use crate::dma;
use crate::usci::{self, UsciBRegisters};
use core::cell::Cell;
use kernel::common::cells::OptionalCell;
use kernel::common::peripherals::{PeripheralManagement, PeripheralManager};
use kernel::common::registers::{ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::i2c;
use kernel::NoClockControl;
use kernel::ReturnCode;

#[derive(Copy, Clone, PartialEq)]
pub enum Speed {
    K100, // 100kHz
    K375, // 375kHz
}

#[derive(Copy, Clone, PartialEq)]
enum OperatingMode {
    Idle,
    Write,
    Read,
    WriteRead,
}

pub struct I2c<'a> {
    registers: StaticRef<UsciBRegisters>,
    mode: Cell<OperatingMode>,
    read_len: Cell<u8>,
    master_client: OptionalCell<&'a dyn i2c::I2CHwMasterClient>,

    tx_dma: OptionalCell<&'a dma::DmaChannel<'a>>,
    pub(crate) tx_dma_chan: usize,
    tx_dma_src: u8,

    rx_dma: OptionalCell<&'a dma::DmaChannel<'a>>,
    pub(crate) rx_dma_chan: usize,
    rx_dma_src: u8,
}

type I2cRegisterManager<'a> = PeripheralManager<'a, I2c<'a>, NoClockControl>;

impl<'a> PeripheralManagement<NoClockControl> for I2c<'a> {
    type RegisterType = UsciBRegisters;

    fn get_registers(&self) -> &Self::RegisterType {
        &self.registers
    }

    fn get_clock(&self) -> &NoClockControl {
        unsafe { &kernel::NO_CLOCK_CONTROL }
    }

    fn before_peripheral_access(&self, _c: &NoClockControl, r: &Self::RegisterType) {
        // Set USCI module to reset in order to make a proper configuration possible
        r.ctlw0.modify(usci::UCBxCTLW0::UCSWRST::SET);
    }

    fn after_peripheral_access(&self, _c: &NoClockControl, r: &Self::RegisterType) {
        // Set USCI module to reset in order to make a proper configuration possible
        r.ctlw0.modify(usci::UCBxCTLW0::UCSWRST::CLEAR);
    }
}

impl<'a> I2c<'a> {
    pub const fn new(
        registers: StaticRef<UsciBRegisters>,
        tx_dma_chan: usize,
        rx_dma_chan: usize,
        tx_dma_src: u8,
        rx_dma_src: u8,
    ) -> Self {
        Self {
            registers: registers,
            mode: Cell::new(OperatingMode::Idle),
            read_len: Cell::new(0),
            master_client: OptionalCell::empty(),
            tx_dma: OptionalCell::empty(),
            tx_dma_chan: tx_dma_chan,
            tx_dma_src: tx_dma_src,
            rx_dma: OptionalCell::empty(),
            rx_dma_chan: rx_dma_chan,
            rx_dma_src: rx_dma_src,
        }
    }

    pub fn set_dma(&self, tx_dma: &'a dma::DmaChannel<'a>, rx_dma: &'a dma::DmaChannel<'a>) {
        self.tx_dma.replace(tx_dma);
        self.rx_dma.replace(rx_dma);
    }

    pub fn set_speed(&self, speed: Speed) {
        // let i2c = I2cRegisterManager::new(self);
        // SMCLK is running at 1.5MHz
        // In order to achieve a speed of 100kHz or 375kHz, it's necessary to divide the clock
        // by either 15 (100kHz) or 4 (375kHz)
        if speed == Speed::K100 {
            self.registers.brw.set(15);
        } else if speed == Speed::K375 {
            self.registers.brw.set(4);
        }
    }

    fn setup(&self) {
        // let i2c = I2cRegisterManager::new(&self);
        self.set_module_to_reset();

        self.registers.ctlw0.modify(
            // Use 7 bit addresses
            usci::UCBxCTLW0::UCSLA10::AddressSlaveWith7BitAddress
            // Setup to master mode
            + usci::UCBxCTLW0::UCMST::MasterMode
            // Setup to single master environment
            + usci::UCBxCTLW0::UCMM::SingleMasterEnvironment
            // Configure USCI module to I2C mode
            + usci::UCBxCTLW0::UCMODE::I2CMode
            // Set clock source to SMCLK (1.5MHz)
            + usci::UCBxCTLW0::UCSSEL::SMCLK,
        );

        self.registers.ctlw1.modify(
            // Disable clock low timeout
            usci::UCBxCTLW1::UCCLTO::CLEAR
            // Send a NACK before a stop condition
            + usci::UCBxCTLW1::UCSTPNACK::NackBeforeStop
            // Generate the ACK bit by hardware
            + usci::UCBxCTLW1::UCSWACK::HardwareTriggered
            // Set glitch filtering to 50ns (according to I2C standard)
            + usci::UCBxCTLW1::UCGLIT::_50ns,
        );

        // Enable interrupts
        self.registers.ie.modify(
            // Enable NACK interrupt
            usci::UCBxIE::UCNACKIE::SET
            // Enable 'arbitration lost' interrupt
            + usci::UCBxIE::UCALIE::SET,
        );

        self.clear_module_reset();
    }

    fn set_module_to_reset(&self) {
        // Set USCI module to reset in order to make a proper configuration possible
        self.registers.ctlw0.modify(usci::UCBxCTLW0::UCSWRST::SET);
    }

    fn clear_module_reset(&self) {
        // Set USCI module to reset in order to make a proper configuration possible
        self.registers.ctlw0.modify(usci::UCBxCTLW0::UCSWRST::CLEAR);
    }

    fn set_slave_address(&self, addr: u8) {
        self.registers.i2csa.set(addr as u16);
    }

    fn generate_start_condition(&self) {
        self.registers
            .ctlw0
            .modify(usci::UCBxCTLW0::UCTXSTT::GenerateSTARTCondition);
    }

    fn generate_stop_condition(&self) {
        self.registers
            .ctlw0
            .modify(usci::UCBxCTLW0::UCTXSTP::GenerateSTOP);
    }

    fn set_stop_condition_automatically(&self, val: bool) {
        if val {
            self.registers
                .ctlw1
                .modify(usci::UCBxCTLW1::UCASTP::ByteCounterStopCondition)
        } else {
            self.registers.ctlw1.modify(usci::UCBxCTLW1::UCASTP::Manual);
        }
    }

    fn enable_transmit_mode(&self) {
        self.registers
            .ctlw0
            .modify(usci::UCBxCTLW0::UCTR::Transmitter);
    }

    fn enable_receive_mode(&self) {
        self.registers.ctlw0.modify(usci::UCBxCTLW0::UCTR::Receiver);
    }

    fn set_byte_counter(&self, val: u8) {
        self.registers.tbcnt.set(val as u16);
    }
}

impl<'a> dma::DmaClient for I2c<'a> {
    fn transfer_done(
        &self,
        tx_buf: Option<&'static mut [u8]>,
        rx_buf: Option<&'static mut [u8]>,
        _transmitted_bytes: usize,
    ) {
        // If this function is entered, an I2C transaction finished without any error.
        // If an error occurs, the interrupt-handler of the I2C module will handle it and invoke the
        // callback with the appropriate error

        match self.mode.get() {
            OperatingMode::Write => {
                self.master_client.map(move |cl| {
                    tx_buf.map(|buf| cl.command_complete(buf, i2c::Error::CommandComplete));
                });
                self.mode.replace(OperatingMode::Idle);
            }
            OperatingMode::Read => {
                self.master_client.map(move |cl| {
                    rx_buf.map(|buf| cl.command_complete(buf, i2c::Error::CommandComplete));
                });
                self.mode.replace(OperatingMode::Idle);
            }
            OperatingMode::WriteRead => {
                if tx_buf.is_some() {
                    // Write part finished

                    // Configure module to receive mode
                    self.enable_receive_mode();

                    // Setup DMA transfer
                    let rx_reg = &self.registers.rxbuf as *const ReadOnly<u16> as *const ();
                    self.rx_dma.map(move |dma| {
                        dma.transfer_periph_to_mem(
                            rx_reg,
                            tx_buf.unwrap(),
                            self.read_len.get() as usize,
                        )
                    });

                    // Generate repeated start condition
                    self.generate_start_condition();
                } else if rx_buf.is_some() {
                    // Read part finished

                    // Generate stop condition to finish the I2c transaction
                    self.generate_stop_condition();

                    // Invoke client callback
                    self.master_client.map(|cl| {
                        cl.command_complete(rx_buf.unwrap(), i2c::Error::CommandComplete)
                    });
                }
            }
            _ => {}
        }
    }
}

impl<'a> i2c::I2CMaster for I2c<'a> {
    fn set_master_client(&self, master_client: &'static dyn i2c::I2CHwMasterClient) {
        self.master_client.replace(master_client);
    }

    fn enable(&self) {
        self.setup();
    }

    fn disable(&self) {
        self.set_module_to_reset();
        self.mode.replace(OperatingMode::Idle);
    }

    fn write(&self, addr: u8, data: &'static mut [u8], len: u8) {
        if self.mode.get() != OperatingMode::Idle {
            // Module is busy
            return;
        }

        // Set module to reset since some of the registers cannot be modified in running state
        self.set_module_to_reset();

        // Setup the slave address
        self.set_slave_address(addr);

        // Setup the I2C module to transmit mode
        self.enable_transmit_mode();

        // Setup the byte counter in order to automatically generate a stop condition after the
        // desired number of bytes were transmitted
        self.set_byte_counter(len);

        // Create stop condition automatically after the number of bytes in the byte counter
        // register were transmitted
        self.set_stop_condition_automatically(true);

        self.clear_module_reset();
        self.mode.replace(OperatingMode::Write);

        // Setup a DMA transfer
        let tx_reg = &self.registers.txbuf as *const ReadWrite<u16> as *const ();
        self.tx_dma
            .map(move |dma| dma.transfer_mem_to_periph(tx_reg, data, len as usize));

        // Start transfer
        self.generate_start_condition();
    }

    fn read(&self, addr: u8, buffer: &'static mut [u8], len: u8) {
        if self.mode.get() != OperatingMode::Idle {
            // Module is busy
            return;
        }

        // Set module to reset since some of the registers cannot be modified in running state
        self.set_module_to_reset();

        // Setup the slave address
        self.set_slave_address(addr);

        // Setup the I2C module to receive mode
        self.enable_receive_mode();

        // Setup the byte counter in order to automatically generate a stop condition after the
        // desired number of bytes were transmitted
        self.set_byte_counter(len);

        // Generate a stop condition automatically after the number of bytes in the byte counter
        // register were transmitted
        self.set_stop_condition_automatically(true);

        self.clear_module_reset();
        self.mode.replace(OperatingMode::Read);

        // Setup a DMA transfer
        let rx_reg = &self.registers.rxbuf as *const ReadOnly<u16> as *const ();
        self.rx_dma
            .map(move |dma| dma.transfer_periph_to_mem(rx_reg, buffer, len as usize));

        // Start transfer
        self.generate_start_condition();
    }

    fn write_read(&self, addr: u8, data: &'static mut [u8], write_len: u8, read_len: u8) {
        if self.mode.get() != OperatingMode::Idle {
            // Module is busy
            return;
        }

        // Set module to reset since some of the registers cannot be modified in running state
        self.set_module_to_reset();

        // Setup the slave address
        self.set_slave_address(addr);

        // Setup the I2C module to transmit mode
        self.enable_transmit_mode();

        // Disable generating a stop condition automatically since after the write, a repeated
        // start condition will be generated in order to continue reading from the slave
        self.set_stop_condition_automatically(false);

        // Store read_len since it will be used in the DMA callback to setup the read transfer
        self.read_len.replace(read_len);

        self.clear_module_reset();
        self.mode.replace(OperatingMode::WriteRead);

        // Setup a DMA transfer
        let tx_reg = &self.registers.txbuf as *const ReadWrite<u16> as *const ();
        self.tx_dma
            .map(move |dma| dma.transfer_mem_to_periph(tx_reg, data, write_len as usize));

        // Start transfer
        self.generate_start_condition();
    }
}
