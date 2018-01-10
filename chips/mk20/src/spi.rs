use regs::spi::*;
use kernel::hil::spi::*;
use kernel::ReturnCode;
use kernel::common::take_cell::TakeCell;
use core::cell::Cell;
use core::mem;
use clock;
use nvic::{self, NvicIdx};

pub enum SpiRole {
    Master,
    Slave
}

pub struct Spi<'a> {
    regs: *mut Registers,
    client: Cell<Option<&'a SpiMasterClient>>,
    index: usize,
    chip_select_settings: [Cell<u32>; 6],
    write: TakeCell<'static, [u8]>,
    read: TakeCell<'static, [u8]>,
    transfer_len: Cell<usize>,
}

pub static mut SPI0: Spi<'static> = Spi::new(0);
pub static mut SPI1: Spi<'static> = Spi::new(1);
pub static mut SPI2: Spi<'static> = Spi::new(2);

impl<'a> Spi<'a> {
    pub const fn new(index: usize) -> Spi<'a> {
        Spi {
            regs: SPI_ADDRS[index],
            client: Cell::new(None),
            index: index,
            chip_select_settings: [Cell::new(0),
                                   Cell::new(0),
                                   Cell::new(0),
                                   Cell::new(0),
                                   Cell::new(0),
                                   Cell::new(0)],
            write: TakeCell::empty(),
            read: TakeCell::empty(),
            transfer_len: Cell::new(0),
        }
    }

    fn regs(&self) -> &mut Registers {
        unsafe { mem::transmute(self.regs) }
    }

    pub fn enable(&self) {
        self.regs().mcr.modify(MCR::MDIS::CLEAR);
    }

    pub fn disable(&self) {
        self.regs().mcr.modify(MCR::MDIS::SET);
    }

    pub fn is_running(&self) -> bool {
        self.regs().sr.is_set(SR::TXRS)
    }

    pub fn halt(&self) {
        self.regs().mcr.modify(MCR::HALT::SET);
        while self.is_running() {}
    }

    pub fn resume(&self) {
        self.regs().mcr.modify(MCR::HALT::CLEAR);
    }

    fn enable_clock(&self) {
        use sim::{clocks, Clock};
        match self.index {
            0 => clocks::SPI0.enable(),
            1 => clocks::SPI1.enable(),
            2 => clocks::SPI2.enable(),
            _ => unreachable!()
        };
    }

    fn set_client(&self, client: &'a SpiMasterClient) {
        self.client.set(Some(client));
    }

    fn set_role(&self, role: SpiRole) {
        self.halt();
        match role {
            SpiRole::Master => {
                self.regs().mcr.modify(MCR::MSTR::Master);
            },
            SpiRole::Slave => {
                self.regs().mcr.modify(MCR::MSTR::Slave);
            }
        }
        self.resume();
    }

    fn set_polarity(&self, polarity: ClockPolarity) {
        let cpol = match polarity {
            ClockPolarity::IdleHigh => CTAR::CPOL::IdleHigh,
            ClockPolarity::IdleLow => CTAR::CPOL::IdleLow
        };
        self.halt();
        self.regs().ctar0.modify(cpol);
        self.resume();
    }

    fn get_polarity(&self) -> ClockPolarity {
        if self.regs().ctar0.matches(CTAR::CPOL::IdleHigh) {
            ClockPolarity::IdleHigh
        } else {
            ClockPolarity::IdleLow
        }
    }

    fn set_phase(&self, phase: ClockPhase) {
        let cpha = match phase {
            ClockPhase::SampleLeading => CTAR::CPHA::SampleLeading,
            ClockPhase::SampleTrailing => CTAR::CPHA::SampleTrailing
        };
        self.halt();
        self.regs().ctar0.modify(cpha);
        self.resume();
    }

    fn get_phase(&self) -> ClockPhase {
        if self.regs().ctar0.matches(CTAR::CPHA::SampleLeading) {
            ClockPhase::SampleLeading
        } else {
            ClockPhase::SampleTrailing
        }
    }

    pub fn set_data_order(&self, order: DataOrder) {
        let order = match order {
            DataOrder::LSBFirst => CTAR::LSBFE::LsbFirst,
            DataOrder::MSBFirst => CTAR::LSBFE::MsbFirst
        };
        self.halt();
        self.regs().ctar0.modify(order);
        self.resume();
    }

    pub fn get_data_order(&self) -> DataOrder {
        if self.regs().ctar0.matches(CTAR::LSBFE::LsbFirst) {
            DataOrder::LSBFirst
        } else {
            DataOrder::MSBFirst
        }
    }

    fn fifo_depth(&self) -> u32 {
        // SPI0 has a FIFO with 4 entries, all others have a 1 entry "FIFO".
        match self.index {
            0 => 4,
            1 | 2 => 1,
            _ => unreachable!()
        }
    }

    fn num_chip_selects(&self) -> u32 {
        match self.index {
            0 => 6,
            1 => 4,
            2 => 2,
            _ => unreachable!()
        }
    }

    fn flush_tx_fifo(&self) {
        self.halt();
        self.regs().mcr.modify(MCR::CLR_TXF::SET);
        self.resume();
    }

    fn flush_rx_fifo(&self) {
        self.halt();
        self.regs().mcr.modify(MCR::CLR_RXF::SET);
        self.resume();
    }

    fn tx_fifo_ready(&self) -> bool {
        !(self.regs().sr.read(SR::TXCTR) >= self.fifo_depth())
    }

    fn rx_fifo_ready(&self) -> bool {
        self.regs().sr.read(SR::RXCTR) > 0
    }

    fn baud_rate(dbl: u32, prescaler: u32, scaler: u32) -> u32 {
        (clock::bus_clock_hz() * (1 + dbl)) / (prescaler * scaler)
    }

    fn set_baud_rate(&self, rate: u32) -> u32 {
        let prescalers: [u32; 4] = [ 2, 3, 5, 7 ];
        let scalers: [u32; 16] = [2, 4, 6, 8,
                                  1<<4, 1<<5, 1<<6, 1<<7,
                                  1<<8, 1<<9, 1<<10, 1<<11,
                                  1<<12, 1<<13, 1<<14, 1<<15];
        let dbls: [u32; 2] = [0, 1];

        let mut rate_diff = rate;
        let mut prescaler = 0;
        let mut scaler = 0;
        let mut dbl = 0;

        // Since there are only 128 unique settings, just iterate over possible
        // configurations until we find the best match. If baud rate can be
        // matched exactly, this loop will terminate early.
        for d in 0..dbls.len() { // 0 is preferred for DBL, as it affects duty cycle
            for p in 0..prescalers.len() {
                for s in 0..scalers.len() {
                    let curr_rate = Spi::baud_rate(dbls[d],
                                                   prescalers[p],
                                                   scalers[s]);

                    // Determine the distance from the target baud rate.
                    let curr_diff = if curr_rate > rate { curr_rate - rate }
                                    else { rate - curr_rate };

                    // If we've improved the best configuration, use it.
                    if curr_diff < rate_diff {
                        rate_diff = curr_diff;
                        scaler = s;
                        prescaler = p;
                        dbl = d;
                    }

                    // Terminate if we've found an exact match.
                    if rate_diff == 0 { break }
                }
            }
        }

        self.halt();
        self.regs().ctar0.modify(CTAR::DBR.val(dbl as u32) +
                                 CTAR::PBR.val(prescaler as u32) +
                                 CTAR::BR.val(scaler as u32));
        self.resume();

        Spi::baud_rate(dbls[dbl], prescalers[prescaler], scalers[scaler])
    }

    fn get_baud_rate(&self) -> u32 {
        let prescaler = match self.regs().ctar0.read(CTAR::PBR) {
            0 => 2,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => panic!("Impossible value for baud rate field!")
        };

        let scaler = match self.regs().ctar0.read(CTAR::BR) {
            0 => 2,
            1 => 4,
            2 => 6,
            s @ _ => 1 << s
        };

        let dbl = self.regs().ctar0.read(CTAR::DBR);

        Spi::baud_rate(dbl, prescaler, scaler)
    }

    pub fn transfer_count(&self) -> u32 {
        self.regs().sr.read(SR::TXCTR)
    }

    pub fn start_of_queue(&self) {
        self.regs().pushr_cmd.modify(PUSHR_CMD::EOQ::CLEAR);
    }

    fn end_of_queue(&self) {
        self.regs().pushr_cmd.modify(PUSHR_CMD::EOQ::SET);
    }

    fn configure_timing(&self) {
        self.halt();
        // Set maximum delay after transfer.
        self.regs().ctar0.modify(CTAR::DT.val(0x0) + CTAR::PDT::Delay7);
        self.resume();
    }

    fn set_frame_size(&self, size: u32) {
        if size > 16 || size < 4 { return }

        self.halt();
        self.regs().ctar0.modify(CTAR::FMSZ.val(size - 1));
        self.resume();
    }

    fn enable_interrupt(&self) {
        let idx = match self.index {
            0 => NvicIdx::SPI0,
            1 => NvicIdx::SPI1,
            2 => NvicIdx::SPI2,
            _ => unreachable!()
        };

        self.halt();
        unsafe {
            nvic::enable(idx);
        }
        self.regs().rser.modify(RSER::EOQF_RE::SET);
        self.resume();
    }

    pub fn handle_interrupt(&self) {
        // TODO: Determine why the extra interrupt is called

        // End of transfer
        if self.regs().sr.is_set(SR::EOQF) {
            self.regs().sr.modify(SR::EOQF::SET);

            self.client.get().map(|client| {
                match self.write.take() {
                    Some(wbuf) => client.read_write_done(wbuf, self.read.take(), self.transfer_len.get()),
                    None => ()
                };
            });
        }
    }
}

impl<'a> SpiMaster for Spi<'a> {
    type ChipSelect = u32;

    fn set_client(&self, client: &'static SpiMasterClient) {
        Spi::set_client(self, client);
    }

    fn init(&self) {
        // Section 57.6.2
        self.enable_clock();
        self.flush_rx_fifo();
        self.flush_tx_fifo();
        self.set_role(SpiRole::Master);
        self.enable_interrupt();
        self.enable();

        self.set_frame_size(8);
        self.configure_timing();
        self.regs().mcr.modify(MCR::PCSIS::AllInactiveHigh);
        self.regs().pushr_cmd.modify(PUSHR_CMD::PCS.val(0));
    }

    fn is_busy(&self) -> bool {
        self.is_running()
    }

    /// Perform an asynchronous read/write operation, whose
    /// completion is signaled by invoking SpiMasterClient on
    /// the initialized client. write_buffer must be Some,
    /// read_buffer may be None. If read_buffer is Some, the
    /// length of the operation is the minimum of the size of
    /// the two buffers.
    fn read_write_bytes(&self,
                        write_buffer: &'static mut [u8],
                        read_buffer: Option<&'static mut [u8]>,
                        len: usize)
                        -> ReturnCode {

        self.start_of_queue();
        if let Some(rbuf) = read_buffer {
            for i in 0..len {
                while !self.tx_fifo_ready() {}

                if i == len - 1 {
                    self.end_of_queue();
                }

                self.regs().pushr_data.set(write_buffer[i]);

                // TODO: this is pretty hacky
                while !self.rx_fifo_ready() {}
                rbuf[i] = self.regs().popr.get() as u8;
            }

            self.read.put(Some(rbuf));
        } else {
            for i in 0..len {
                while !self.tx_fifo_ready() {}

                if i == len - 1 {
                    self.end_of_queue();
                }

                self.regs().pushr_data.set(write_buffer[i]);
            }
            self.read.put(None);
        }

        self.write.put(Some(write_buffer));
        self.transfer_len.set(len);

        ReturnCode::SUCCESS
    }

    fn write_byte(&self, _val: u8) {
        unimplemented!();
    }

    fn read_byte(&self) -> u8 {
        unimplemented!();
    }

    fn read_write_byte(&self, _val: u8) -> u8 {
        unimplemented!();
    }

    /// Tell the SPI peripheral what to use as a chip select pin.
    /// The type of the argument is based on what makes sense for the
    /// peripheral when this trait is implemented.
    fn specify_chip_select(&self, cs: Self::ChipSelect) {
        if cs >= self.num_chip_selects() {
            return;
        }

        // The PCS field is one-hot (the way this interface uses it).
        let pcs = self.regs().pushr_cmd.read(PUSHR_CMD::PCS);
        let old_cs = match pcs {
            0 | 0b000001 => 0,
            0b000010 => 1,
            0b000100 => 2,
            0b001000 => 3,
            0b010000 => 4,
            0b100000 => 5,
            _ => panic!("Unexpected PCS: {:?}", pcs),
        };

        let new_cs = cs as usize;

        // Swap in the new configuration.
        self.halt();
        self.chip_select_settings[old_cs].set(self.regs().ctar0.get());
        self.regs().ctar0.set(self.chip_select_settings[new_cs].get());
        self.resume();
        self.regs().pushr_cmd.modify(PUSHR_CMD::PCS.val(1 << new_cs));
    }

    /// Returns the actual rate set
    fn set_rate(&self, rate: u32) -> u32 {
        self.set_baud_rate(rate)
    }

    fn get_rate(&self) -> u32 {
        self.get_baud_rate()
    }

    fn set_clock(&self, polarity: ClockPolarity) {
        self.set_polarity(polarity);
    }

    fn get_clock(&self) -> ClockPolarity {
        self.get_polarity()
    }

    fn set_phase(&self, phase: ClockPhase) {
        Spi::set_phase(self, phase);
    }

    fn get_phase(&self) -> ClockPhase {
        Spi::get_phase(self)
    }

    // These two functions determine what happens to the chip
    // select line between transfers. If hold_low() is called,
    // then the chip select line is held low after transfers
    // complete. If release_low() is called, then the chip select
    // line is brought high after a transfer completes. A "transfer"
    // is any of the read/read_write calls. These functions
    // allow an application to manually control when the
    // CS line is high or low, such that it can issue multi-byte
    // requests with single byte operations.
    fn hold_low(&self) {
        self.regs().pushr_cmd.modify(PUSHR_CMD::CONT::ChipSelectInactiveBetweenTxfers);
    }

    fn release_low(&self) {
        self.regs().pushr_cmd.modify(PUSHR_CMD::CONT::ChipSelectAssertedBetweenTxfers);
    }
}

interrupt_handler!(spi0_interrupt_handler, SPI0);
interrupt_handler!(spi1_interrupt_handler, SPI1);
interrupt_handler!(spi2_interrupt_handler, SPI2);
