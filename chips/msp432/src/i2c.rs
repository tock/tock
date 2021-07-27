use crate::usci::{self, UsciBRegisters};
use core::cell::Cell;
use kernel::hil::i2c::{self, Error};
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::StaticRef;

#[derive(Copy, Clone, PartialEq)]
pub enum Speed {
    K100, // 100kHz
    K375, // 375kHz
}

#[derive(Copy, Clone, PartialEq)]
enum OperatingMode {
    Unconfigured,
    Disabled,
    Idle,
    Write,
    Read,
    WriteReadWrite,
    WriteReadRead,
}

pub struct I2c<'a> {
    registers: StaticRef<UsciBRegisters>,
    mode: Cell<OperatingMode>,
    read_len: Cell<u8>,
    write_len: Cell<u8>,
    buf_idx: Cell<u8>,
    buffer: TakeCell<'static, [u8]>,
    master_client: OptionalCell<&'a dyn i2c::I2CHwMasterClient>,
}

impl<'a> I2c<'a> {
    pub const fn new(registers: StaticRef<UsciBRegisters>) -> Self {
        Self {
            registers: registers,
            mode: Cell::new(OperatingMode::Unconfigured),
            read_len: Cell::new(0),
            write_len: Cell::new(0),
            buf_idx: Cell::new(0),
            buffer: TakeCell::empty(),
            master_client: OptionalCell::empty(),
        }
    }

    fn set_module_to_reset(&self) {
        // Set USCI module to reset in order to make a certain configurations possible
        self.registers.ctlw0.modify(usci::UCBxCTLW0::UCSWRST::SET);
    }

    fn clear_module_reset(&self) {
        // Set USCI module to reset in order to make a certain configurations possible
        self.registers.ctlw0.modify(usci::UCBxCTLW0::UCSWRST::CLEAR);

        // Setting the module to reset clears the enabled interrupts -> enable them again
        self.enable_interrupts();
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

    fn enable_interrupts(&self) {
        // Enable interrupts
        self.registers.ie.modify(
            // Enable NACK interrupt
            usci::UCBxIE::UCNACKIE::SET
            // Enable TX interrupt
            // + usci::UCBxIE::UCTXIE0::SET
            // Enable RX interrupt
            + usci::UCBxIE::UCRXIE0::SET
            // Enable Stop condition interrupt
            + usci::UCBxIE::UCSTPIE::SET
            // Enable Start condition interrupt
            + usci::UCBxIE::UCSTTIE::SET
            // Enable 'arbitration lost' interrupt
            + usci::UCBxIE::UCALIE::SET,
        );
    }

    fn enable_transmit_mode(&self) {
        self.registers
            .ctlw0
            .modify(usci::UCBxCTLW0::UCTR::Transmitter);
    }

    fn enable_receive_mode(&self) {
        self.registers.ctlw0.modify(usci::UCBxCTLW0::UCTR::Receiver);
    }

    fn enable_transmit_interrupt(&self) {
        self.registers.ie.modify(usci::UCBxIE::UCTXIE0::SET);
    }

    fn disable_transmit_interrupt(&self) {
        self.registers.ie.modify(usci::UCBxIE::UCTXIE0::CLEAR);
    }

    fn set_byte_counter(&self, val: u8) {
        self.registers.tbcnt.set(val as u16);
    }

    fn invoke_callback(&self, status: Result<(), Error>) {
        // Reset buffer index and set mode to Idle in order to start a new transfer properly
        self.buf_idx.set(0);
        self.mode.set(OperatingMode::Idle);

        self.buffer.take().map(|buf| {
            self.master_client
                .map(move |cl| cl.command_complete(buf, status))
        });
    }

    fn setup(&self) {
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
            // Enable Synchronous mode
            + usci::UCBxCTLW0::UCSYNC::SynchronousMode
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

        // Don't clear the module reset here since we set the state to Disabled
        self.mode.set(OperatingMode::Disabled);
    }

    pub fn set_speed(&self, speed: Speed) {
        self.set_module_to_reset();

        // SMCLK is running at 1.5MHz
        // In order to achieve a speed of 100kHz or 375kHz, it's necessary to divide the clock
        // by either 15 (100kHz) or 4 (375kHz)
        if speed == Speed::K100 {
            self.registers.brw.set(15);
        } else if speed == Speed::K375 {
            self.registers.brw.set(4);
        }

        self.clear_module_reset();
    }

    pub fn handle_interrupt(&self) {
        let ifg = self.registers.ifg.get();
        let mode = self.mode.get();
        let idx = self.buf_idx.get();

        // clear all interrupts
        self.registers.ifg.set(0);

        if (ifg & (1 << usci::UCBxIFG::UCTXIFG0.shift)) > 0 {
            // TX interrupt
            if idx < self.write_len.get() {
                // Transmit another byte
                self.buffer
                    .map(|buf| self.registers.txbuf.set(buf[idx as usize] as u16));
                self.buf_idx.set(idx + 1);
            } else {
                self.disable_transmit_interrupt();
                if mode == OperatingMode::WriteReadWrite {
                    // Finished write part -> switch to reading
                    self.mode.set(OperatingMode::WriteReadRead);
                    self.buf_idx.set(0);

                    // Switch to receiving and send a restart condition
                    self.enable_receive_mode();
                    self.generate_start_condition();
                    if self.read_len.get() == 1 {
                        // In this mode the stop condition is set automatically and has to be
                        // requested 1 byte before the last byte was received. If only one byte will
                        // be received request the stop condition immediately after the start.
                        self.generate_stop_condition();
                    }
                }
            }
        } else if (ifg & (1 << usci::UCBxIFG::UCRXIFG0.shift)) > 0 {
            // RX interrupt
            if idx < self.read_len.get() {
                if idx == (self.read_len.get() - 1) && mode == OperatingMode::WriteReadRead {
                    // In this mode we don't use the byte counter to generate an automatic stop
                    // condition, further, the stop condition has to be set before the last byte was
                    // received
                    self.generate_stop_condition();
                }
                // Store received byte in buffer
                self.buffer
                    .map(|buf| buf[idx as usize] = self.registers.rxbuf.get() as u8);
                self.buf_idx.set(idx + 1);
            } else if mode == OperatingMode::WriteReadRead {
                // For some reason generating a stop condition manually in receive mode doesn't
                // trigger a stop condition interrupt -> invoke the callback here when all bytes
                // were received
                self.invoke_callback(Ok(()));
            }
        } else if (ifg & (1 << usci::UCBxIFG::UCSTTIFG.shift)) > 0 {
            // Start condition interrupt
            if mode == OperatingMode::Write || mode == OperatingMode::WriteReadWrite {
                self.buffer
                    .map(|buf| self.registers.txbuf.set(buf[idx as usize] as u16));
                self.buf_idx.set(idx + 1);
            }
        } else if (ifg & (1 << usci::UCBxIFG::UCSTPIFG.shift)) > 0 {
            // Stop condition interrupt

            // This interrupt is the default indicator that a transaction finished, thus raise the
            // callback here and prepare for another transfer
            self.invoke_callback(Ok(()));
        } else if (ifg & (1 << usci::UCBxIFG::UCNACKIFG.shift)) > 0 {
            // NACK interrupt
            // TODO: use byte counter to detect address NAK

            // Cancel i2c transfer
            self.generate_stop_condition();
            self.invoke_callback(Err(Error::DataNak));
        } else if (ifg & (1 << usci::UCBxIFG::UCALIFG.shift)) > 0 {
            // Arbitration lost  interrupt

            // Cancel i2c transfer
            self.generate_stop_condition();
            self.invoke_callback(Err(Error::Busy));
        } else {
            panic!("I2C: unhandled interrupt, ifg: {}", ifg);
        }
    }
}

impl<'a> i2c::I2CMaster for I2c<'a> {
    fn set_master_client(&self, master_client: &'static dyn i2c::I2CHwMasterClient) {
        self.master_client.replace(master_client);
    }

    fn enable(&self) {
        if self.mode.get() == OperatingMode::Unconfigured {
            self.setup();
        }

        self.clear_module_reset();
        self.mode.set(OperatingMode::Idle);
    }

    fn disable(&self) {
        self.set_module_to_reset();
        self.mode.set(OperatingMode::Disabled);
    }

    fn write(
        &self,
        addr: u8,
        data: &'static mut [u8],
        len: u8,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.mode.get() != OperatingMode::Idle {
            // Module is busy or not activated
            return Err((Error::Busy, data));
        }

        self.buffer.replace(data);
        self.write_len.set(len);

        // Set module to reset since some of the registers cannot be modified in running state
        self.set_module_to_reset();

        // Setup the byte counter in order to automatically generate a stop condition after the
        // desired number of bytes were transmitted
        self.set_byte_counter(len);

        // Create stop condition automatically after the number of bytes in the byte counter
        // register were transmitted
        self.set_stop_condition_automatically(true);
        self.clear_module_reset();

        self.set_slave_address(addr);
        self.enable_transmit_mode();
        self.enable_transmit_interrupt();

        self.mode.set(OperatingMode::Write);

        // Start transfer
        self.generate_start_condition();

        Ok(())
    }

    fn read(
        &self,
        addr: u8,
        buffer: &'static mut [u8],
        len: u8,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.mode.get() != OperatingMode::Idle {
            // Module is busy or not activated
            return Err((Error::Busy, buffer));
        }

        self.buffer.replace(buffer);
        self.read_len.set(len);

        // Set module to reset since some of the registers cannot be modified in running state
        self.set_module_to_reset();

        // Setup the byte counter in order to automatically generate a stop condition after the
        // desired number of bytes were transmitted
        self.set_byte_counter(len);

        // Generate a stop condition automatically after the number of bytes in the byte counter
        // register were transmitted
        self.set_stop_condition_automatically(true);
        self.clear_module_reset();

        self.set_slave_address(addr);
        self.enable_receive_mode();
        self.mode.set(OperatingMode::Read);

        // Start transfer
        self.generate_start_condition();
        Ok(())
    }

    fn write_read(
        &self,
        addr: u8,
        data: &'static mut [u8],
        write_len: u8,
        read_len: u8,
    ) -> Result<(), (Error, &'static mut [u8])> {
        if self.mode.get() != OperatingMode::Idle {
            // Module is busy or not activated
            return Err((Error::Busy, data));
        }

        self.buffer.replace(data);
        self.write_len.set(write_len);
        self.read_len.set(read_len);

        // Set module to reset since some of the registers cannot be modified in running state
        self.set_module_to_reset();

        // Disable generating a stop condition automatically since after the write, a repeated
        // start condition will be generated in order to continue reading from the slave
        self.set_stop_condition_automatically(false);
        self.clear_module_reset();

        self.set_slave_address(addr);
        self.enable_transmit_mode();
        self.enable_transmit_interrupt();
        self.mode.set(OperatingMode::WriteReadWrite);

        // Start transfer
        self.generate_start_condition();

        Ok(())
    }
}
