//! Serial Peripheral Interface (SPI)
//!
//! Implementation of DMA-based SPI master communication for the MSP432.
//! Both eUSCI_A and eUSCI_B can be used for SPI communication, but the
//! the registers have a different meaning than in UART/I2C mode.
//!

use crate::dma;
use crate::usci::{self, UCSPIxCTLW0, UCSPIxIE, UCSPIxIFG, UCSPIxSTATW};
use core::cell::Cell;
use kernel::common::{
    cells::{OptionalCell, TakeCell},
    registers::{ReadOnly, ReadWrite},
    StaticRef,
};
use kernel::hil::gpio::Pin;
use kernel::hil::spi;
use kernel::ErrorCode;

/// The SPI related registers are identical between the USCI_A and the USCI_B module, but the usage
/// of the certain bits is different to UART and I2C mode. Thus we simply cast the concerned
/// registers to a UCSPIx-registers instead of UCAx or UCBx registers. With this trick we can pass
/// references of USCI_A and USCI_B modules to the Spi-constructor.
pub trait UsciSpiRef {
    fn ctlw0(&self) -> &ReadWrite<u16, UCSPIxCTLW0::Register>;
    fn brw(&self) -> &ReadWrite<u16>;
    fn statw(&self) -> &ReadWrite<u16, UCSPIxSTATW::Register>;
    fn rxbuf(&self) -> &ReadOnly<u16>;
    fn txbuf(&self) -> &ReadWrite<u16>;
    fn ie(&self) -> &ReadWrite<u16, UCSPIxIE::Register>;
    fn ifg(&self) -> &ReadWrite<u16, UCSPIxIFG::Register>;
}

impl UsciSpiRef for StaticRef<usci::UsciBRegisters> {
    fn ctlw0(&self) -> &ReadWrite<u16, usci::UCSPIxCTLW0::Register> {
        unsafe {
            &*(&self.ctlw0 as *const ReadWrite<u16, usci::UCBxCTLW0::Register>
                as *const ReadWrite<u16, usci::UCSPIxCTLW0::Register>)
        }
    }

    fn brw(&self) -> &ReadWrite<u16> {
        &self.brw
    }

    fn statw(&self) -> &ReadWrite<u16, usci::UCSPIxSTATW::Register> {
        unsafe {
            &*(&self.statw as *const ReadWrite<u16, usci::UCBxSTATW::Register>
                as *const ReadWrite<u16, usci::UCSPIxSTATW::Register>)
        }
    }

    fn rxbuf(&self) -> &ReadOnly<u16> {
        &self.rxbuf
    }

    fn txbuf(&self) -> &ReadWrite<u16> {
        &self.txbuf
    }

    fn ie(&self) -> &ReadWrite<u16, usci::UCSPIxIE::Register> {
        unsafe {
            &*(&self.ie as *const ReadWrite<u16, usci::UCBxIE::Register>
                as *const ReadWrite<u16, usci::UCSPIxIE::Register>)
        }
    }

    fn ifg(&self) -> &ReadWrite<u16, usci::UCSPIxIFG::Register> {
        unsafe {
            &*(&self.ifg as *const ReadWrite<u16, usci::UCBxIFG::Register>
                as *const ReadWrite<u16, usci::UCSPIxIFG::Register>)
        }
    }
}

impl UsciSpiRef for StaticRef<usci::UsciARegisters> {
    fn ctlw0(&self) -> &ReadWrite<u16, usci::UCSPIxCTLW0::Register> {
        unsafe {
            &*(&self.ctlw0 as *const ReadWrite<u16, usci::UCAxCTLW0::Register>
                as *const ReadWrite<u16, usci::UCSPIxCTLW0::Register>)
        }
    }

    fn brw(&self) -> &ReadWrite<u16> {
        &self.brw
    }

    fn statw(&self) -> &ReadWrite<u16, usci::UCSPIxSTATW::Register> {
        unsafe {
            &*(&self.statw as *const ReadWrite<u16, usci::UCAxSTATW::Register>
                as *const ReadWrite<u16, usci::UCSPIxSTATW::Register>)
        }
    }

    fn rxbuf(&self) -> &ReadOnly<u16> {
        &self.rxbuf
    }

    fn txbuf(&self) -> &ReadWrite<u16> {
        &self.txbuf
    }

    fn ie(&self) -> &ReadWrite<u16, usci::UCSPIxIE::Register> {
        unsafe {
            &*(&self.ie as *const ReadWrite<u16, usci::UCAxIE::Register>
                as *const ReadWrite<u16, usci::UCSPIxIE::Register>)
        }
    }

    fn ifg(&self) -> &ReadWrite<u16, usci::UCSPIxIFG::Register> {
        unsafe {
            &*(&self.ifg as *const ReadWrite<u16, usci::UCAxIFG::Register>
                as *const ReadWrite<u16, usci::UCSPIxIFG::Register>)
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum OperatingMode {
    Unconfigured,
    Idle,
    Write,
    WriteRead,
}

#[derive(Clone, Copy)]
#[repr(u16)]
enum SpiClock {
    K1500 = 1, // 1.5MHz
    K750 = 2,  // 750kHz
    K500 = 3,  // 500kHz
    K375 = 4,  // 375kHz
    K300 = 5,  // 300kHz
    K250 = 6,  // 250kHz
    K150 = 10, // 150kHz
    K100 = 15, // 100kHz
}

impl From<u32> for SpiClock {
    fn from(rate: u32) -> Self {
        match rate {
            0..=124_999 => SpiClock::K100,
            125_000..=199_999 => SpiClock::K150,
            200_000..=274_999 => SpiClock::K250,
            275_000..=337_499 => SpiClock::K300,
            337_500..=437_499 => SpiClock::K375,
            437_500..=624_999 => SpiClock::K500,
            625_000..=1124_999 => SpiClock::K750,
            _ => SpiClock::K1500,
        }
    }
}

impl From<SpiClock> for u32 {
    fn from(clk: SpiClock) -> Self {
        match clk {
            SpiClock::K100 => 100_000,
            SpiClock::K150 => 150_000,
            SpiClock::K250 => 250_000,
            SpiClock::K300 => 300_000,
            SpiClock::K375 => 375_000,
            SpiClock::K500 => 500_000,
            SpiClock::K750 => 750_000,
            SpiClock::K1500 => 1_500_000,
        }
    }
}

pub struct Spi<'a> {
    registers: &'static dyn UsciSpiRef,
    cs: OptionalCell<&'a dyn Pin>,
    hold_cs: Cell<bool>,
    operating_mode: Cell<OperatingMode>,
    clock: Cell<SpiClock>,
    tx_buf: TakeCell<'static, [u8]>,
    master_client: OptionalCell<&'a dyn spi::SpiMasterClient>,

    tx_dma: OptionalCell<&'a dma::DmaChannel<'a>>,
    pub(crate) tx_dma_chan: usize,
    tx_dma_src: u8,

    rx_dma: OptionalCell<&'a dma::DmaChannel<'a>>,
    pub(crate) rx_dma_chan: usize,
    rx_dma_src: u8,
}

impl<'a> Spi<'a> {
    pub fn new(
        registers: &'static dyn UsciSpiRef,
        tx_dma_chan: usize,
        rx_dma_chan: usize,
        tx_dma_src: u8,
        rx_dma_src: u8,
    ) -> Self {
        Self {
            registers: registers,
            cs: OptionalCell::empty(),
            hold_cs: Cell::new(false),
            operating_mode: Cell::new(OperatingMode::Unconfigured),
            clock: Cell::new(SpiClock::K100),
            tx_buf: TakeCell::empty(),
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

    fn set_module_to_reset(&self) {
        // Set module to reset in order to enable the configuration
        self.registers.ctlw0().modify(UCSPIxCTLW0::UCSWRST::Enabled);
    }

    fn clear_module_reset(&self) {
        self.registers
            .ctlw0()
            .modify(UCSPIxCTLW0::UCSWRST::Disabled);
    }

    fn finish_transfer(&self) {
        self.operating_mode.set(OperatingMode::Idle);

        if !self.hold_cs.get() {
            self.cs.map(|pin| pin.set());
        }
    }
}

impl<'a> dma::DmaClient for Spi<'a> {
    fn transfer_done(
        &self,
        tx_buf: Option<&'static mut [u8]>,
        rx_buf: Option<&'static mut [u8]>,
        transmitted_bytes: usize,
    ) {
        if let Some(buf) = tx_buf {
            // Transmitting finished
            if self.operating_mode.get() == OperatingMode::Write {
                // Only a write operation was done -> invoke callback

                self.finish_transfer();
                self.master_client
                    .map(move |cl| cl.read_write_done(buf, None, transmitted_bytes));
            } else {
                // Also a read operation was done -> wait for RX callback
                self.tx_buf.replace(buf);
            }
        }

        if let Some(buf) = rx_buf {
            // Receiving finished

            self.finish_transfer();
            self.master_client.map(move |cl| {
                cl.read_write_done(
                    self.tx_buf
                        .take()
                        .unwrap_or_else(|| panic!("SPI: no TX buffer was returned from DMA")),
                    Some(buf),
                    transmitted_bytes,
                )
            });
        }
    }
}

impl<'a> spi::SpiMaster for Spi<'a> {
    type ChipSelect = &'a dyn Pin;

    fn set_client(&self, client: &'static dyn spi::SpiMasterClient) {
        self.master_client.set(client);
    }

    fn init(&self) {
        self.set_module_to_reset();

        self.registers.ctlw0().modify(
            // Transmit LSB first
            UCSPIxCTLW0::UCMSB::MSBFirst
            // Enable 8bit modus
            + UCSPIxCTLW0::UC7BIT::_8Bit
            // Configure to Master mode
            + UCSPIxCTLW0::UCMST::Master
            // Use 3-pin SPI mode since CS is controlled by hand
            + UCSPIxCTLW0::UCMODE::_3PinSPI
            // Enable synchronous mode
            + UCSPIxCTLW0::UCSYNC::SynchronousMode
            // Use SMCLK as clock
            + UCSPIxCTLW0::UCSSEL::SMCLK
            //enable clk high
            + UCSPIxCTLW0::UCCKPL::InactiveLow,
        );

        // Configure the DMA
        let tx_conf = dma::DmaConfig {
            src_chan: self.tx_dma_src,
            mode: dma::DmaMode::Basic,
            width: dma::DmaDataWidth::Width8Bit,
            src_incr: dma::DmaPtrIncrement::Incr8Bit,
            dst_incr: dma::DmaPtrIncrement::NoIncr,
        };

        let rx_conf = dma::DmaConfig {
            src_chan: self.rx_dma_src,
            mode: dma::DmaMode::Basic,
            width: dma::DmaDataWidth::Width8Bit,
            src_incr: dma::DmaPtrIncrement::NoIncr,
            dst_incr: dma::DmaPtrIncrement::Incr8Bit,
        };

        self.tx_dma.map(|dma| dma.initialize(&tx_conf));
        self.rx_dma.map(|dma| dma.initialize(&rx_conf));

        self.operating_mode.set(OperatingMode::Idle);
        self.clear_module_reset();
    }

    fn is_busy(&self) -> bool {
        self.operating_mode.get() != OperatingMode::Idle
    }

    fn read_write_bytes(
        &self,
        write_buffer: &'static mut [u8],
        read_buffer: Option<&'static mut [u8]>,
        len: usize,
    ) -> Result<(), ErrorCode> {
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }

        let mut cnt = len;
        cnt = core::cmp::min(cnt, write_buffer.len());

        // Set chip select
        self.cs.map(|pin| pin.clear());

        // If a read-buffer was supplied too, we also start a read transaction
        if let Some(read_buf) = read_buffer {
            self.operating_mode.set(OperatingMode::WriteRead);
            cnt = core::cmp::min(cnt, read_buf.len());

            let rx_reg = self.registers.rxbuf() as *const ReadOnly<u16> as *const ();
            self.rx_dma
                .map(move |dma| dma.transfer_periph_to_mem(rx_reg, read_buf, cnt));
        } else {
            self.operating_mode.set(OperatingMode::Write);
        }

        // Start a write transaction
        let tx_reg = self.registers.txbuf() as *const ReadWrite<u16> as *const ();
        self.tx_dma
            .map(move |dma| dma.transfer_mem_to_periph(tx_reg, write_buffer, cnt));

        Ok(())
    }

    fn write_byte(&self, val: u8) {
        if self.is_busy() {
            return;
        }

        while self.registers.statw().is_set(UCSPIxSTATW::UCBUSY) {}
        self.registers.txbuf().set(val as u16);
        while self.registers.statw().is_set(UCSPIxSTATW::UCBUSY) {}
    }

    fn read_byte(&self) -> u8 {
        if self.is_busy() {
            return 0;
        }

        while self.registers.statw().is_set(UCSPIxSTATW::UCBUSY) {}
        self.registers.txbuf().set(0);
        while self.registers.statw().is_set(UCSPIxSTATW::UCBUSY) {}
        self.registers.rxbuf().get() as u8
    }

    fn read_write_byte(&self, val: u8) -> u8 {
        if self.is_busy() {
            return 0;
        }

        while self.registers.statw().is_set(UCSPIxSTATW::UCBUSY) {}
        self.registers.txbuf().set(val as u16);
        while self.registers.statw().is_set(UCSPIxSTATW::UCBUSY) {}
        self.registers.rxbuf().get() as u8
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        cs.make_output();
        cs.set();
        self.cs.set(cs);
    }

    fn set_rate(&self, rate: u32) -> u32 {
        let clk = SpiClock::from(rate);

        self.set_module_to_reset();
        self.registers.brw().set(clk as u16);
        self.clear_module_reset();

        self.clock.set(clk);
        clk.into()
    }

    fn get_rate(&self) -> u32 {
        self.clock.get().into()
    }

    fn set_clock(&self, polarity: spi::ClockPolarity) {
        self.set_module_to_reset();

        match polarity {
            spi::ClockPolarity::IdleLow => self
                .registers
                .ctlw0()
                .modify(UCSPIxCTLW0::UCCKPL::InactiveLow),
            spi::ClockPolarity::IdleHigh => self
                .registers
                .ctlw0()
                .modify(UCSPIxCTLW0::UCCKPL::InactiveHigh),
        }

        self.clear_module_reset();
    }

    fn get_clock(&self) -> spi::ClockPolarity {
        match self.registers.ctlw0().is_set(UCSPIxCTLW0::UCCKPL) {
            false => spi::ClockPolarity::IdleLow,
            true => spi::ClockPolarity::IdleHigh,
        }
    }

    fn set_phase(&self, phase: spi::ClockPhase) {
        self.set_module_to_reset();

        match phase {
            spi::ClockPhase::SampleLeading => self
                .registers
                .ctlw0()
                .modify(UCSPIxCTLW0::UCCKPH::CaptureFirstChangeFollowing),
            spi::ClockPhase::SampleTrailing => self
                .registers
                .ctlw0()
                .modify(UCSPIxCTLW0::UCCKPH::ChangeFirstCaptureFollowing),
        }

        self.clear_module_reset();
    }

    fn get_phase(&self) -> spi::ClockPhase {
        match self.registers.ctlw0().is_set(UCSPIxCTLW0::UCCKPH) {
            false => spi::ClockPhase::SampleTrailing,
            true => spi::ClockPhase::SampleLeading,
        }
    }

    fn hold_low(&self) {
        self.hold_cs.set(true);
    }

    fn release_low(&self) {
        self.hold_cs.set(false);
    }
}
