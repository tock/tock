// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.
//
// Author: Jason Hu <jasonhu2026@u.northwestern.edu>
//         Anthony Alvarez <anthonyalvarez2026@u.northwestern.edu>

//! SPI using the Programmable Input Output (PIO) hardware.
use crate::clocks::{self};
use crate::pio::{Pio, PioRxClient, PioTxClient, SMNumber, StateMachineConfiguration};
use core::cell::Cell;
use kernel::deferred_call::{DeferredCall, DeferredCallClient};
use kernel::hil::spi::cs::{ChipSelectPolar, Polarity};
use kernel::hil::spi::SpiMasterClient;
use kernel::hil::spi::{ClockPhase, ClockPolarity};
use kernel::utilities::cells::MapCell;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::leasable_buffer::SubSliceMut;
use kernel::{hil, ErrorCode};

// Since auto push / pull is set to 8 for the purposes of writing in bytes
// rather than words, values read in have to be bitshifted by 24
const AUTOPULL_SHIFT: usize = 24;

// Frequency of system clock, for rate changes
const SYSCLOCK_FREQ: u32 = 125_000_000;

// The following programs are in PIO asm
// SPI_CPHA0 and SPI_CPHA1 are sourced from pico examples
// https://github.com/raspberrypi/pico-examples/blob/master/pio/spi/spi.pio
//
// For the idle high clock programs, we took inspiration of how Zephyr did it
// which was the simple change of swapping when the side set pin outputs 0 or 1
// https://github.com/zephyrproject-rtos/zephyr/blob/main/drivers/spi/spi_rpi_pico_pio.c
//
// for further reference consult the RP2040 datasheet chapter 3 (especially sections 3.4 and 3.6)
// https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf
//
// One can compile pioasm programs locally using the official pico sdk,
// Or you can use the following website and copy the hex output
// https://wokwi.com/tools/pioasm

// Leading edge clock phase + Idle low clock
const SPI_CPHA0: [u16; 2] = [
    0x6101, /*  0: out    pins, 1         side 0 [1] */
    0x5101, /*  1: in     pins, 1         side 1 [1] */
];

// Trailing edge clock phase + Idle low clock
const SPI_CPHA1: [u16; 3] = [
    0x6021, /* 0: out    x, 1            side 0 */
    0xb101, /* 1: mov    pins, x         side 1 [1] */
    0x4001, /* 2: in     pins, 1         side 0 */
];

// Leading edge clock phase + Idle high clock
const SPI_CPHA0_HIGH_CLOCK: [u16; 2] = [
    0x7101, /*  0: out    pins, 1         side 1 [1] */
    0x4101, /*  1: in     pins, 1         side 0 [1] */
];

// Trailing edge clock phase + Idle high clock
const SPI_CPHA1_HIGH_CLOCK: [u16; 3] = [
    0x7021, /*  0: out    x, 1            side 1 */
    0xa101, /*  1: mov    pins, x         side 0 [1] */
    0x5001, /*  2: in     pins, 1         side 1 */
];

/*
Instantiation example
let _pio_spi: &'static mut PioSpi<'static> = static_init!(
        PioSpi,
        PioSpi::<'static>::new(
            &peripherals.pio0,
            &peripherals.clocks,
            10, // clock pin
            11, // in pin (MISO)
            12, // out pin (MOSI)
            SMNumber::SM0,
        )
    );

    // make the pio subscribe to interrupts
    peripherals.pio0.sm(SMNumber::SM0).set_rx_client(_pio_spi);
    peripherals.pio0.sm(SMNumber::SM0).set_tx_client(_pio_spi);
    _pio_spi.register(); // necessary for asynchronous transactions


By default it is in clock idle low, sample leading edge clock phase, 1 MHz clock frequency
*/

pub struct PioSpi<'a> {
    clocks: OptionalCell<&'a clocks::Clocks>,
    pio: &'a Pio,
    clock_pin: u32,
    out_pin: u32,
    in_pin: u32,
    sm_number: SMNumber,
    client: OptionalCell<&'a dyn SpiMasterClient>,
    tx_buffer: MapCell<SubSliceMut<'static, u8>>,
    tx_position: Cell<usize>,
    rx_buffer: MapCell<SubSliceMut<'static, u8>>,
    rx_position: Cell<usize>,
    len: Cell<usize>,
    state: Cell<PioSpiState>,
    deferred_call: DeferredCall,
    clock_div_int: Cell<u32>,
    clock_div_frac: Cell<u32>,
    clock_phase: Cell<ClockPhase>,
    clock_polarity: Cell<ClockPolarity>,
    chip_select: OptionalCell<ChipSelectPolar<'a, crate::gpio::RPGpioPin<'a>>>,
    hold_low: Cell<bool>,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PioSpiState {
    Free = 0b00,
    Writing = 0b01,
    Reading = 0b10,
    ReadingWriting = 0b11,
}

impl<'a> PioSpi<'a> {
    pub fn new(
        pio: &'a Pio,
        clocks: &'a clocks::Clocks,
        clock_pin: u32,
        in_pin: u32,
        out_pin: u32,
        sm_number: SMNumber,
    ) -> Self {
        Self {
            clocks: OptionalCell::new(clocks),
            pio,
            clock_pin,
            in_pin,
            out_pin,
            sm_number,
            client: OptionalCell::empty(),
            tx_buffer: MapCell::empty(),
            tx_position: Cell::new(0),
            rx_buffer: MapCell::empty(),
            rx_position: Cell::new(0),
            len: Cell::new(0),
            state: Cell::new(PioSpiState::Free),
            deferred_call: DeferredCall::new(),
            clock_div_int: Cell::new(31u32), // defaults to 1 MHz
            clock_div_frac: Cell::new(64u32),
            clock_phase: Cell::new(ClockPhase::SampleLeading), // defaults to mode 0 0
            clock_polarity: Cell::new(ClockPolarity::IdleLow),
            chip_select: OptionalCell::empty(),
            hold_low: Cell::new(false),
        }
    }

    // Helper function to read and writes to and from the buffers, returns whether the operation finished
    fn read_write_buffers(&self) -> bool {
        let mut finished = false;

        self.tx_buffer.map(|buf| {
            let length = self.len.get();

            // FIFOs are 4 units deep, so that is the max one can read/write before having to wait
            const FIFO_DEPTH: usize = 4;

            let left_to_do = self.len.get() - self.tx_position.get() + 1;
            let run_to = if FIFO_DEPTH > left_to_do {
                left_to_do
            } else {
                FIFO_DEPTH
            };

            for _i in 0..run_to {
                let mut errors = false;

                // Try to write one byte
                if self.tx_position.get() < length {
                    let res = self
                        .pio
                        .sm(self.sm_number)
                        .push(buf[self.tx_position.get()] as u32);
                    match res {
                        Err(_error) => errors = true,
                        _ => {
                            self.tx_position.set(self.tx_position.get() + 1);
                        }
                    }
                }

                // Try to read one byte
                if self.rx_position.get() < length {
                    let data = self.pio.sm(self.sm_number).pull();
                    match data {
                        Ok(val) => {
                            self.rx_buffer.map(|readbuf| {
                                readbuf[self.rx_position.get()] = (val >> AUTOPULL_SHIFT) as u8;
                                self.rx_position.set(self.rx_position.get() + 1);
                            });
                        }
                        _ => errors = true,
                    }
                }

                // If we are done reading and writing, then exit
                if self.tx_position.get() >= self.len.get()
                    && self.rx_position.get() >= self.len.get()
                {
                    finished = true;

                    break;
                }

                // If any read/write errors, then exit and stop writing
                if errors {
                    break;
                }
            }
        });

        finished
    }

    // reset the buffers and call the SPI client if any, to finish off a transaction
    // should only be called from the deferred call or interrupt handlers
    fn call_client_and_clean_up(&self) {
        self.state.set(PioSpiState::Free);

        let transaction_size = self.len.get();

        self.state.set(PioSpiState::Free);
        self.len.set(0);
        self.tx_position.set(0);
        self.rx_position.set(0);

        if !self.hold_low.get() {
            self.set_chip_select(false);
        }

        if let Some(tx_buffer) = self.tx_buffer.take() {
            self.client.map(|client| {
                client.read_write_done(tx_buffer, self.rx_buffer.take(), Ok(transaction_size));
            });
        }
    }

    fn set_chip_select(&self, active: bool) {
        if active {
            self.chip_select.map(|p| match p.polarity {
                Polarity::Low => {
                    p.activate();
                }
                _ => {
                    p.deactivate();
                }
            });
        } else {
            self.chip_select.map(|p| match p.polarity {
                Polarity::Low => {
                    p.deactivate();
                }
                _ => {
                    p.activate();
                }
            });
        }
    }
}

impl<'a> hil::spi::SpiMaster<'a> for PioSpi<'a> {
    type ChipSelect = ChipSelectPolar<'a, crate::gpio::RPGpioPin<'a>>;

    fn init(&self) -> Result<(), ErrorCode> {
        self.pio.init();

        // the trailing phase programs have a different length
        let mut wrap = 1;
        let program: &[u16] = if self.clock_phase.get() == ClockPhase::SampleLeading {
            if self.clock_polarity.get() == ClockPolarity::IdleLow {
                &SPI_CPHA0
            } else {
                &SPI_CPHA0_HIGH_CLOCK
            }
        } else {
            // sample trailing branch
            wrap = 2;
            if self.clock_polarity.get() == ClockPolarity::IdleLow {
                &SPI_CPHA1
            } else {
                &SPI_CPHA1_HIGH_CLOCK
            }
        };

        match self.pio.add_program16(None::<usize>, program) {
            Ok(_res) => {
                self.pio.sm(self.sm_number).exec_program(_res, true);
            }
            Err(_error) => return Err(ErrorCode::FAIL),
        }

        let custom_config = StateMachineConfiguration {
            div_int: self.clock_div_int.get(),
            div_frac: self.clock_div_frac.get(),
            // 8 bit mode on pio
            in_push_threshold: 8,
            out_pull_threshold: 8,
            side_set_base: self.clock_pin,
            in_pins_base: self.in_pin,
            out_pins_base: self.out_pin,
            side_set_bit_count: 1,
            wrap,
            // automatically push and pull from the fifos
            in_autopush: true,
            out_autopull: true,
            ..Default::default()
        };

        self.pio.spi_program_init(
            self.sm_number,
            self.clock_pin,
            self.in_pin,
            self.out_pin,
            &custom_config,
        );

        Ok(())
    }

    fn set_client(&self, client: &'a dyn SpiMasterClient) {
        self.client.set(client);
    }

    fn is_busy(&self) -> bool {
        match self.state.get() {
            PioSpiState::Free => false,
            _ => true,
        }
    }

    fn read_write_bytes(
        &self,
        write_buffer: SubSliceMut<'static, u8>,
        read_buffer: Option<SubSliceMut<'static, u8>>,
    ) -> Result<
        (),
        (
            ErrorCode,
            SubSliceMut<'static, u8>,
            Option<SubSliceMut<'static, u8>>,
        ),
    > {
        if self.is_busy() {
            return Err((ErrorCode::BUSY, write_buffer, read_buffer));
        }

        if write_buffer.len() < 1 {
            return Err((ErrorCode::INVAL, write_buffer, read_buffer));
        }

        self.set_chip_select(true);

        // Keep track of the new buffers
        self.len.replace(write_buffer.len());
        self.tx_buffer.replace(write_buffer);
        self.tx_position.set(0);

        self.state.replace(PioSpiState::Writing);

        if let Some(readbuf) = read_buffer {
            self.rx_buffer.replace(readbuf);
            self.state.replace(PioSpiState::ReadingWriting);
            self.rx_position.set(0);
        }

        // Begin reading/writing to/from buffers
        let done = self.read_write_buffers();

        // this call is likely coming from above, so set a deferred call if it gets done within here
        if done {
            self.deferred_call.set();
        }

        Ok(())
    }

    fn write_byte(&self, val: u8) -> Result<(), ErrorCode> {
        match self.read_write_byte(val) {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }

    fn read_byte(&self) -> Result<u8, ErrorCode> {
        self.read_write_byte(0)
    }

    fn read_write_byte(&self, val: u8) -> Result<u8, ErrorCode> {
        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }

        self.set_chip_select(true);

        let mut data: u32;

        // One byte operations can be synchronous
        self.pio.sm(self.sm_number).push_blocking(val as u32)?;

        data = match self.pio.sm(self.sm_number).pull_blocking() {
            Ok(val) => val,
            Err(error) => {
                return Err(error);
            }
        };

        data >>= AUTOPULL_SHIFT;

        if !self.hold_low.get() {
            self.set_chip_select(false);
        }

        Ok(data as u8)
    }

    fn specify_chip_select(&self, cs: Self::ChipSelect) -> Result<(), ErrorCode> {
        if !self.is_busy() {
            self.chip_select.set(cs);
            Ok(())
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn set_rate(&self, rate: u32) -> Result<u32, ErrorCode> {
        if rate == 0 {
            return Err(ErrorCode::FAIL);
        }

        if self.is_busy() {
            return Err(ErrorCode::BUSY);
        }

        let sysclock_freq = self.clocks.map_or(SYSCLOCK_FREQ, |clocks| {
            clocks.get_frequency(clocks::Clock::System)
        });

        // Program does two instructions per every SPI clock peak
        // Still runs at half of rate after that so multiply again by two
        let rate = rate * 4;

        // Max clock rate is the sys clock
        if rate > sysclock_freq {
            return Err(ErrorCode::INVAL);
        }

        let divint = sysclock_freq / rate;
        // Div frac is in units of 1/256
        let divfrac = (sysclock_freq % rate) * 256u32 / rate;

        self.clock_div_int.replace(divint);
        self.clock_div_frac.replace(divfrac);

        // Reinit the PIO so it updates the times
        self.pio.sm(self.sm_number).set_enabled(false);
        self.pio
            .sm(self.sm_number)
            .set_clkdiv_int_frac(divint, divfrac);
        self.pio.sm(self.sm_number).clkdiv_restart();
        self.pio.sm(self.sm_number).set_enabled(true);

        Ok(rate)
    }

    fn get_rate(&self) -> u32 {
        let sysclock_freq = self.clocks.map_or(SYSCLOCK_FREQ, |clocks| {
            clocks.get_frequency(clocks::Clock::Peripheral)
        });

        let divisor = self.clock_div_int.get() as f32 + (self.clock_div_frac.get() as f32 / 256f32);

        if divisor == 0f32 {
            return sysclock_freq / 65536u32;
        }

        (sysclock_freq as f32 / divisor) as u32 / 4u32
    }

    fn set_polarity(&self, polarity: ClockPolarity) -> Result<(), ErrorCode> {
        if !self.is_busy() {
            self.clock_polarity.replace(polarity);
            self.init()
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn get_polarity(&self) -> ClockPolarity {
        self.clock_polarity.get()
    }

    fn set_phase(&self, phase: ClockPhase) -> Result<(), ErrorCode> {
        if !self.is_busy() {
            self.clock_phase.replace(phase);
            self.init()
        } else {
            Err(ErrorCode::BUSY)
        }
    }

    fn get_phase(&self) -> ClockPhase {
        self.clock_phase.get()
    }

    fn hold_low(&self) {
        self.hold_low.replace(true);
    }

    fn release_low(&self) {
        self.hold_low.replace(false);
    }
}

impl PioTxClient for PioSpi<'_> {
    // Buffer space availble, so send next byte
    fn on_buffer_space_available(&self) {
        self.tx_position.set(self.tx_position.get() + 1);

        match self.state.get() {
            PioSpiState::Writing | PioSpiState::ReadingWriting => {
                let done = self.read_write_buffers();
                if done {
                    self.call_client_and_clean_up();
                }
            }
            _ => {}
        }
    }
}

impl PioRxClient for PioSpi<'_> {
    // Data received, so update buffer and continue reading/writing
    fn on_data_received(&self, data: u32) {
        let data = data >> AUTOPULL_SHIFT;

        if self.len.get() > self.rx_position.get() {
            self.rx_buffer.map(|buf| {
                buf[self.rx_position.get()] = data as u8;
                self.rx_position.set(self.rx_position.get() + 1);
            });
        }
        match self.state.get() {
            PioSpiState::Reading | PioSpiState::ReadingWriting => {
                let done = self.read_write_buffers();
                if done {
                    self.call_client_and_clean_up();
                }
            }
            _ => {}
        }
    }
}

impl DeferredCallClient for PioSpi<'_> {
    // deferred call to calling the client
    fn handle_deferred_call(&self) {
        self.call_client_and_clean_up();
    }

    fn register(&'static self) {
        self.deferred_call.register(self);
    }
}
