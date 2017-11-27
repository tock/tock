//! Implementation of the SAM4L ADCIFE.
//!
//! This is an implementation of the SAM4L analog to digital converter. It is
//! bare-bones because it provides little flexibility on how samples are taken.
//! Currently, all samples:
//!
//! - are 12 bits
//! - use the ground pad as the negative reference
//! - use a VCC/2 positive reference
//! - are right justified
//!
//! Samples can either be collected individually or continuously at a specified
//! frequency.
//!
//! - Author: Philip Levis <pal@cs.stanford.edu>, Branden Ghena <brghena@umich.edu>
//! - Updated: May 1, 2017

use core::{cmp, mem, slice};
use core::cell::Cell;
use dma;
use kernel::ReturnCode;
use kernel::common::VolatileCell;
use kernel::common::math;
use kernel::common::take_cell::TakeCell;
use kernel::hil;
use pm::{self, Clock, PBAClock};
use scif;

/// Representation of an ADC channel on the SAM4L.
pub struct AdcChannel {
    chan_num: u32,
    internal: u32,
}

/// SAM4L ADC channels.
#[derive(Copy,Clone,Debug)]
#[repr(u8)]
enum Channel {
    AD0 = 0x00,
    AD1 = 0x01,
    AD2 = 0x02,
    AD3 = 0x03,
    AD4 = 0x04,
    AD5 = 0x05,
    AD6 = 0x06,
    AD7 = 0x07,
    AD8 = 0x08,
    AD9 = 0x09,
    AD10 = 0x0A,
    AD11 = 0x0B,
    AD12 = 0x0C,
    AD13 = 0x0D,
    AD14 = 0x0E,
    Bandgap = 0x0F,
    ScaledVCC = 0x12,
    DAC = 0x13,
    Vsingle = 0x16,
    ReferenceGround = 0x17,
}

/// Initialization of an ADC channel.
impl AdcChannel {
    /// Create a new ADC channel.
    ///
    /// - `channel`: Channel enum representing the channel number and whether it
    ///   is internal
    const fn new(channel: Channel) -> AdcChannel {
        AdcChannel {
            chan_num: ((channel as u8) & 0x0F) as u32,
            internal: (((channel as u8) >> 4) & 0x01) as u32,
        }
    }
}

/// Statically allocated ADC channels. Used in board configurations to specify
/// which channels are used on the platform.
pub static mut CHANNEL_AD0: AdcChannel = AdcChannel::new(Channel::AD0);
pub static mut CHANNEL_AD1: AdcChannel = AdcChannel::new(Channel::AD1);
pub static mut CHANNEL_AD2: AdcChannel = AdcChannel::new(Channel::AD2);
pub static mut CHANNEL_AD3: AdcChannel = AdcChannel::new(Channel::AD3);
pub static mut CHANNEL_AD4: AdcChannel = AdcChannel::new(Channel::AD4);
pub static mut CHANNEL_AD5: AdcChannel = AdcChannel::new(Channel::AD5);
pub static mut CHANNEL_AD6: AdcChannel = AdcChannel::new(Channel::AD6);
pub static mut CHANNEL_AD7: AdcChannel = AdcChannel::new(Channel::AD7);
pub static mut CHANNEL_AD8: AdcChannel = AdcChannel::new(Channel::AD8);
pub static mut CHANNEL_AD9: AdcChannel = AdcChannel::new(Channel::AD9);
pub static mut CHANNEL_AD10: AdcChannel = AdcChannel::new(Channel::AD10);
pub static mut CHANNEL_AD11: AdcChannel = AdcChannel::new(Channel::AD11);
pub static mut CHANNEL_AD12: AdcChannel = AdcChannel::new(Channel::AD12);
pub static mut CHANNEL_AD13: AdcChannel = AdcChannel::new(Channel::AD13);
pub static mut CHANNEL_AD14: AdcChannel = AdcChannel::new(Channel::AD14);
pub static mut CHANNEL_BANDGAP: AdcChannel = AdcChannel::new(Channel::Bandgap);
pub static mut CHANNEL_SCALED_VCC: AdcChannel = AdcChannel::new(Channel::ScaledVCC);
pub static mut CHANNEL_DAC: AdcChannel = AdcChannel::new(Channel::DAC);
pub static mut CHANNEL_VSINGLE: AdcChannel = AdcChannel::new(Channel::Vsingle);
pub static mut CHANNEL_REFERENCE_GROUND: AdcChannel = AdcChannel::new(Channel::ReferenceGround);

/// Create a trait of both client types to allow a single client reference to
/// act as both
pub trait EverythingClient: hil::adc::Client + hil::adc::HighSpeedClient {}
impl<C: hil::adc::Client + hil::adc::HighSpeedClient> EverythingClient for C {}

/// ADC driver code for the SAM4L.
pub struct Adc {
    registers: *mut AdcRegisters,

    // state tracking for the ADC
    enabled: Cell<bool>,
    adc_clk_freq: Cell<u32>,
    active: Cell<bool>,
    continuous: Cell<bool>,
    dma_running: Cell<bool>,
    cpu_clock: Cell<bool>,

    // timer fire counting for slow sampling rates
    timer_repeats: Cell<u8>,
    timer_counts: Cell<u8>,

    // DMA peripheral, buffers, and length
    rx_dma: Cell<Option<&'static dma::DMAChannel>>,
    rx_dma_peripheral: dma::DMAPeripheral,
    rx_length: Cell<usize>,
    next_dma_buffer: TakeCell<'static, [u16]>,
    next_dma_length: Cell<usize>,
    stopped_buffer: TakeCell<'static, [u16]>,

    // ADC client to send sample complete notifications to
    client: Cell<Option<&'static EverythingClient>>,
}

/// Memory mapped registers for the ADC.
#[repr(C, packed)]
pub struct AdcRegisters {
    // From page 1005 of SAM4L manual
    pub cr: VolatileCell<u32>, // Control               (0x00)
    pub cfg: VolatileCell<u32>, // Configuration        (0x04)
    pub sr: VolatileCell<u32>, // Status                (0x08)
    pub scr: VolatileCell<u32>, // Status clear         (0x0c)
    pub pad: VolatileCell<u32>, // padding/reserved
    pub seqcfg: VolatileCell<u32>, // Sequencer config  (0x14)
    pub cdma: VolatileCell<u32>, // Config DMA          (0x18)
    pub tim: VolatileCell<u32>, // Timing config        (0x1c)
    pub itimer: VolatileCell<u32>, // Internal timer    (0x20)
    pub wcfg: VolatileCell<u32>, // Window config       (0x24)
    pub wth: VolatileCell<u32>, // Window threshold     (0x28)
    pub lcv: VolatileCell<u32>, // Last converted value (0x2c)
    pub ier: VolatileCell<u32>, // Interrupt enable     (0x30)
    pub idr: VolatileCell<u32>, // Interrupt disable    (0x34)
    pub imr: VolatileCell<u32>, // Interrupt mask       (0x38)
    pub calib: VolatileCell<u32>, // Calibration        (0x3c)
    pub version: VolatileCell<u32>, // Version          (0x40)
    pub parameter: VolatileCell<u32>, // Parameter      (0x44)
}
// Page 59 of SAM4L data sheet
pub const BASE_ADDRESS: *mut AdcRegisters = 0x40038000 as *mut AdcRegisters;

/// Statically allocated ADC driver. Used in board configurations to connect to
/// various capsules.
pub static mut ADC0: Adc = Adc::new(BASE_ADDRESS, dma::DMAPeripheral::ADCIFE_RX);

/// Functions for initializing the ADC.
impl Adc {
    /// Create a new ADC driver.
    ///
    /// - `base_address`: pointer to the ADC's memory mapped I/O registers
    /// - `rx_dma_peripheral`: type used for DMA transactions
    const fn new(base_address: *mut AdcRegisters, rx_dma_peripheral: dma::DMAPeripheral) -> Adc {
        Adc {
            // pointer to memory mapped I/O registers
            registers: base_address,

            // status of the ADC peripheral
            enabled: Cell::new(false),
            adc_clk_freq: Cell::new(0),
            active: Cell::new(false),
            continuous: Cell::new(false),
            dma_running: Cell::new(false),
            cpu_clock: Cell::new(false),

            // timer repeating state for slow sampling rates
            timer_repeats: Cell::new(0),
            timer_counts: Cell::new(0),

            // DMA status and stuff
            rx_dma: Cell::new(None),
            rx_dma_peripheral: rx_dma_peripheral,
            rx_length: Cell::new(0),
            next_dma_buffer: TakeCell::empty(),
            next_dma_length: Cell::new(0),
            stopped_buffer: TakeCell::empty(),

            // higher layer to send responses to
            client: Cell::new(None),
        }
    }

    /// Sets the client for this driver.
    ///
    /// - `client`: reference to capsule which handles responses
    pub fn set_client<C: EverythingClient>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    /// Sets the DMA channel for this driver.
    ///
    /// - `rx_dma`: reference to the DMA channel the ADC should use
    pub fn set_dma(&self, rx_dma: &'static dma::DMAChannel) {
        self.rx_dma.set(Some(rx_dma));
    }

    /// Interrupt handler for the ADC.
    pub fn handle_interrupt(&mut self) {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };
        let status = regs.sr.get();

        if self.enabled.get() && self.active.get() {
            if status & 0x01 == 0x01 {
                // sample complete interrupt

                // should we deal with this sample now, or wait for the next
                // one?
                if self.timer_counts.get() >= self.timer_repeats.get() {
                    // we actually care about this sample

                    // single sample complete. Send value to client
                    let val = (regs.lcv.get() & 0xffff) as u16;
                    self.client.get().map(|client| { client.sample_ready(val); });

                    // clean up state
                    if self.continuous.get() {
                        // continuous sampling, reset counts and keep going
                        self.timer_counts.set(0);

                    } else {
                        // single sampling, disable interrupt and set inactive
                        self.active.set(false);
                        regs.idr.set(1);
                    }

                } else {
                    // increment count and wait for next sample
                    self.timer_counts.set(self.timer_counts.get() + 1);
                }

                // clear status
                regs.scr.set(0x00000001);
            }

        } else {
            // we are inactive, why did we get an interrupt?
            // disable all interrupts, clear status, and just ignore it
            regs.idr.set(0x2F);
            regs.scr.set(0x2F);
        }
    }

    // Configures the ADC with the slowest clock that can provide continuous sampling at
    // the desired frequency and enables the ADC. Subsequent calls with the same frequency
    // value have no effect. Using the slowest clock also ensures efficient discrete
    // sampling.
    fn config_and_enable(&self, frequency: u32) -> ReturnCode {
        if self.active.get() {
            // disallow reconfiguration during sampling
            ReturnCode::EBUSY
        } else if frequency == self.adc_clk_freq.get() {
            // already configured to work on this frequency
            ReturnCode::SUCCESS
        } else {
            let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };

            // disabling the ADC before switching clocks is necessary to avoid leaving it
            // in undefined state
            // disable ADC
            regs.cr.set(1 << 9);

            // wait until status is disabled
            let mut timeout = 10000;
            while regs.sr.get() & (0x1 << 24) == (0x1 << 24) {
                timeout -= 1;
                if timeout == 0 {
                    // ADC never disabled
                    return ReturnCode::FAIL;
                }
            }

            self.enabled.set(true);

            // First, enable the clocks
            // Both the ADCIFE clock and GCLK10 are needed
            let mut clock_divisor;
            unsafe {
                // turn on ADCIFE bus clock. Already set to the same frequency
                // as the CPU clock
                pm::enable_clock(Clock::PBA(PBAClock::ADCIFE));
                // the maximum sampling frequency with the RC clocks is 1/32th of their clock
                // frequency. This is because of the minimum PRESCAL by a factor of 4 and the
                // 7+1 cycles needed for conversion in continuous mode. Hence, 4*(7+1)=32.
                if frequency <= 113600 / 32 {
                    // RC oscillator
                    self.cpu_clock.set(false);
                    let max_freq: u32;
                    if frequency <= 32000 / 32 {
                        // frequency of the RC32K is 32KHz.
                        scif::generic_clock_enable(scif::GenericClock::GCLK10,
                                                   scif::ClockSource::RC32K);
                        max_freq = 32000 / 32;
                    } else {
                        // frequency of the RCSYS is 115KHz.
                        scif::generic_clock_enable(scif::GenericClock::GCLK10,
                                                   scif::ClockSource::RCSYS);
                        max_freq = 113600 / 32;
                    }
                    let divisor = (frequency + max_freq - 1) / frequency; // ceiling of division
                    clock_divisor = math::log_base_two(math::closest_power_of_two(divisor));
                    clock_divisor = cmp::min(cmp::max(clock_divisor, 0), 7); // keep in bounds
                    self.adc_clk_freq.set(max_freq / (1 << (clock_divisor)));
                } else {
                    // CPU clock
                    self.cpu_clock.set(true);
                    scif::generic_clock_enable(scif::GenericClock::GCLK10,
                                               scif::ClockSource::CLK_CPU);
                    // determine clock divider
                    // we need the ADC_CLK to be a maximum of 1.5 MHz in frequency,
                    // so we need to find the PRESCAL value that will make this
                    // happen.
                    // Formula: f(ADC_CLK) = f(CLK_CPU)/2^(N+2) <= 1.5 MHz
                    // and we solve for N
                    // becomes: N <= ceil(log_2(f(CLK_CPU)/1500000)) - 2
                    let cpu_frequency = pm::get_system_frequency();
                    let divisor = (cpu_frequency + (1500000 - 1)) / 1500000; // ceiling of division
                    clock_divisor = math::log_base_two(math::closest_power_of_two(divisor)) - 2;
                    clock_divisor = cmp::min(cmp::max(clock_divisor, 0), 7); // keep in bounds
                    self.adc_clk_freq.set(cpu_frequency / (1 << (clock_divisor + 2)));
                }
            }

            // configure the ADC
            let clksel;
            if self.cpu_clock.get() {
                clksel = 1 // CLKSEL: use ADCIFE clock
            } else {
                clksel = 0 // CLKSEL: use GCLOCK clock
            }
            let cfg_val = (clock_divisor << 8) | // PRESCAL: clock divider
                              (clksel << 6) | // CLKSEL
                              (0x0 << 4) | // SPEED: maximum 300 ksps
                              (0x4 << 1); // REFSEL: VCC/2 reference


            regs.cfg.set(cfg_val);

            let tim_val = (0x1 << 8) | // ENSTUP
                          (0x17 << 0); // wait 24 cycles
            regs.tim.set(tim_val);

            // software reset (does not clear registers)
            regs.cr.set(1);

            // enable ADC
            regs.cr.set(1 << 8);

            // wait until status is enabled
            let mut timeout = 10000;
            while regs.sr.get() & (0x1 << 24) != (0x1 << 24) {
                timeout -= 1;
                if timeout == 0 {
                    // ADC never enabled
                    return ReturnCode::FAIL;
                }
            }

            // enable Bandgap buffer and Reference buffer. I don't actually
            // know what these do, but you need to turn them on
            let cr_val = (0x1 << 10) | // BGREQEN: Enable bandgap buffer request
                         (0x1 <<  4); // REFBUFEN: Enable reference buffer
            regs.cr.set(cr_val);

            // wait until buffers are enabled
            timeout = 100000;
            while regs.sr.get() & (0x51000000) != 0x51000000 {
                timeout -= 1;
                if timeout == 0 {
                    // ADC buffers never enabled
                    return ReturnCode::FAIL;
                }
            }

            ReturnCode::SUCCESS
        }
    }
}

/// Implements an ADC capable reading ADC samples on any channel.
impl hil::adc::Adc for Adc {
    type Channel = AdcChannel;

    /// Enable and configure the ADC.
    /// This can be called multiple times with no side effects.
    fn initialize(&self) -> ReturnCode {
        // always configure to 1KHz to get the slowest clock
        self.config_and_enable(1000)
    }

    /// Capture a single analog sample, calling the client when complete.
    /// Returns an error if the ADC is already sampling.
    ///
    /// - `channel`: the ADC channel to sample
    fn sample(&self, channel: &Self::Channel) -> ReturnCode {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };

        // always configure to 1KHz to get the slowest clock with single sampling
        let res = self.config_and_enable(1000);

        if res != ReturnCode::SUCCESS {
            return res;

        } else if !self.enabled.get() {
            ReturnCode::EOFF

        } else if self.active.get() {
            // only one operation at a time
            ReturnCode::EBUSY

        } else {
            self.active.set(true);
            self.continuous.set(false);
            self.timer_repeats.set(0);
            self.timer_counts.set(0);

            let cfg = (0x7 << 20) | // MUXNEG: ground pad
                      (channel.chan_num << 16) | // MUXPOS: selection
                      (0x1 << 15) | // INTERNAL: internal neg
                      (channel.internal << 14) | // INTERNAL: pos selection
                      (0x0 << 12) | // RES: 12-bit resolution
                      (0x0 <<  8) | // TRGSEL: software trigger
                      (0x0 <<  7) | // GCOMP: no gain compensation
                      (0x7 <<  4) | // GAIN: 0.5x gain
                      (0x0 <<  2) | // BIPOLAR: unipolar mode
                      (0x0 <<  0); // HWLA: right justify value
            regs.seqcfg.set(cfg);

            // clear any current status
            regs.scr.set(0x2F);

            // enable end of conversion interrupt
            regs.ier.set(0x01);

            // initiate conversion
            regs.cr.set(0x08);

            ReturnCode::SUCCESS
        }
    }

    /// Request repeated analog samples on a particular channel, calling after
    /// each sample. In order to not unacceptably slow down the system
    /// collecting samples, this interface is limited to one sample every 100
    /// microseconds (10000 samples per second). To sample faster, use the
    /// sample_highspeed function.
    ///
    /// - `channel`: the ADC channel to sample
    /// - `frequency`: the number of samples per second to collect
    fn sample_continuous(&self, channel: &Self::Channel, frequency: u32) -> ReturnCode {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };

        let res = self.config_and_enable(frequency);

        if res != ReturnCode::SUCCESS {
            return res;

        } else if !self.enabled.get() {
            ReturnCode::EOFF

        } else if self.active.get() {
            // only one sample at a time
            ReturnCode::EBUSY

        } else if frequency == 0 || frequency > 10000 {
            // limit sampling frequencies to a valid range
            ReturnCode::EINVAL

        } else {
            self.active.set(true);
            self.continuous.set(true);

            let trgsel;
            if self.cpu_clock.get() {
                trgsel = 1; // internal timer trigger
            } else {
                trgsel = 3; // continuous mode
            }

            let cfg = (0x7 << 20) | // MUXNEG: ground pad
                      (channel.chan_num << 16) | // MUXPOS: selection
                      (0x1 << 15) | // INTERNAL: internal neg
                      (channel.internal << 14) | // INTERNAL: pos selection
                      (0x0 << 12) | // RES: 12-bit resolution
                      (trgsel <<  8) | // TRGSEL
                      (0x0 <<  7) | // GCOMP: no gain compensation
                      (0x7 <<  4) | // GAIN: 0.5x gain
                      (0x0 <<  2) | // BIPOLAR: unipolar mode
                      (0x0 <<  0); // HWLA: right justify value
            regs.seqcfg.set(cfg);

            // stop timer if running
            regs.cr.set(0x02);

            if self.cpu_clock.get() {
                // This logic only applies for sampling off the CPU
                // setup timer for low-frequency samples. Based on the ADC clock,
                // the minimum timer frequency is:
                // 1500000 / (0xFFFF + 1) = 22.888 Hz.
                // So for any frequency less than 23 Hz, we will keep our own
                // counter in addition and only actually perform a callback every N
                // timer fires. This is important to enable low-jitter sampling in
                // the 1-22 Hz range.
                let timer_frequency;
                if frequency < 23 {
                    // set a number of timer repeats before the callback is
                    // performed. 60 here is an arbitrary number which limits the
                    // actual itimer frequency to between 42 and 60 in the desired
                    // range of 1-22 Hz, which seems slow enough to keep the system
                    // from getting bogged down with interrupts
                    let counts = 60 / frequency;
                    self.timer_repeats.set(counts as u8);
                    self.timer_counts.set(0);
                    timer_frequency = frequency * counts;
                } else {
                    // we can sample at this frequency directly with the timer
                    self.timer_repeats.set(0);
                    self.timer_counts.set(0);
                    timer_frequency = frequency;
                }

                // set timer, limit to bounds
                // f(timer) = f(adc) / (counter + 1)
                let mut counter = (self.adc_clk_freq.get() / timer_frequency) - 1;
                counter = cmp::max(cmp::min(counter, 0xFFFF), 0);
                regs.itimer.set(counter);
            } else {
                // we can sample at this frequency directly with the timer
                self.timer_repeats.set(0);
                self.timer_counts.set(0);
            }

            // clear any current status
            regs.scr.set(0x2F);

            // enable end of conversion interrupt
            regs.ier.set(0x01);

            // start timer
            regs.cr.set(0x04);

            ReturnCode::SUCCESS
        }
    }

    /// Stop continuously sampling the ADC.
    /// This is expected to be called to stop continuous sampling operations,
    /// but can be called to abort any currently running operation. The buffer,
    /// if any, will be returned via the `samples_ready` callback.
    fn stop_sampling(&self) -> ReturnCode {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };

        if !self.enabled.get() {
            ReturnCode::EOFF

        } else if !self.active.get() {
            // cannot cancel sampling that isn't running
            ReturnCode::EINVAL

        } else {
            // clean up state
            self.active.set(false);
            self.continuous.set(false);
            self.dma_running.set(false);

            // stop internal timer
            regs.cr.set(0x02);

            // disable sample interrupts
            regs.idr.set(0x01);

            // reset the ADC peripheral
            regs.cr.set(0x01);

            // stop DMA transfer if going. This should safely return a None if
            // the DMA was not being used
            let dma_buffer = self.rx_dma.get().map_or(None, |rx_dma| {
                let dma_buf = rx_dma.abort_xfer();
                rx_dma.disable();
                dma_buf
            });
            self.rx_length.set(0);

            // store the buffer if it exists
            dma_buffer.map(|dma_buf| {
                // change buffer back into a [u16]
                // the buffer was originally a [u16] so this should be okay
                let buf_ptr = unsafe { mem::transmute::<*mut u8, *mut u16>(dma_buf.as_mut_ptr()) };
                let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, dma_buf.len() / 2) };

                // we'll place it here so we can return it to the higher level
                // later in a `retrieve_buffers` call
                self.stopped_buffer.replace(buf);
            });

            ReturnCode::SUCCESS
        }
    }
}

/// Implements an ADC capable of continuous sampling
impl hil::adc::AdcHighSpeed for Adc {
    /// Capture buffered samples from the ADC continuously at a given
    /// frequency, calling the client whenever a buffer fills up. The client is
    /// then expected to either stop sampling or provide an additional buffer
    /// to sample into. Note that due to hardware constraints the maximum
    /// frequency range of the ADC is from 187 kHz to 23 Hz (although its
    /// precision is limited at higher frequencies due to aliasing).
    ///
    /// - `channel`: the ADC channel to sample
    /// - `frequency`: frequency to sample at
    /// - `buffer1`: first buffer to fill with samples
    /// - `length1`: number of samples to collect (up to buffer length)
    /// - `buffer2`: second buffer to fill once the first is full
    /// - `length2`: number of samples to collect (up to buffer length)
    fn sample_highspeed(&self,
                        channel: &Self::Channel,
                        frequency: u32,
                        buffer1: &'static mut [u16],
                        length1: usize,
                        buffer2: &'static mut [u16],
                        length2: usize)
                        -> (ReturnCode, Option<&'static mut [u16]>, Option<&'static mut [u16]>) {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };

        let res = self.config_and_enable(frequency);

        if res != ReturnCode::SUCCESS {
            return (res, Some(buffer1), Some(buffer2));

        } else if !self.enabled.get() {
            (ReturnCode::EOFF, Some(buffer1), Some(buffer2))

        } else if self.active.get() {
            // only one sample at a time
            (ReturnCode::EBUSY, Some(buffer1), Some(buffer2))

        } else if frequency <= (self.adc_clk_freq.get() / (0xFFFF + 1)) || frequency > 250000 {
            // can't sample faster than the max sampling frequency or slower
            // than the timer can be set to
            (ReturnCode::EINVAL, Some(buffer1), Some(buffer2))

        } else if length1 == 0 {
            // at least need a valid length for the for the first buffer full of
            // samples. Otherwise, what are we doing here?
            (ReturnCode::EINVAL, Some(buffer1), Some(buffer2))

        } else {
            self.active.set(true);
            self.continuous.set(true);

            // store the second buffer for later use
            self.next_dma_buffer.replace(buffer2);
            self.next_dma_length.set(length2);

            let trgsel;
            if self.cpu_clock.get() {
                trgsel = 1; // internal timer trigger
            } else {
                trgsel = 3; // continuous mode
            }

            // adc sequencer configuration
            let cfg = (0x7 << 20) | // MUXNEG: ground pad
                      (channel.chan_num << 16) | // MUXPOS: selection
                      (0x1 << 15) | // INTERNAL: internal neg
                      (channel.internal << 14) | // INTERNAL: pos selection
                      (0x0 << 12) | // RES: 12-bit resolution
                      (trgsel <<  8) | // TRGSEL
                      (0x0 <<  7) | // GCOMP: no gain compensation
                      (0x7 <<  4) | // GAIN: 0.5x gain
                      (0x0 <<  2) | // BIPOLAR: unipolar mode
                      (0x0 <<  0); // HWLA: right justify value
            regs.seqcfg.set(cfg);

            // stop timer if running
            regs.cr.set(0x02);

            if self.cpu_clock.get() {
                // set timer, limit to bounds
                // f(timer) = f(adc) / (counter + 1)
                let mut counter = (self.adc_clk_freq.get() / frequency) - 1;
                counter = cmp::max(cmp::min(counter, 0xFFFF), 0);
                regs.itimer.set(counter);
            }

            // clear any current status
            regs.scr.set(0x2F);

            // receive up to the buffer's length samples
            let dma_len = cmp::min(buffer1.len(), length1);

            // change buffer into a [u8]
            // this is unsafe but acceptable for the following reasons
            //  * the buffer is aligned based on 16-bit boundary, so the 8-bit
            //    alignment is fine
            //  * the DMA is doing checking based on our expected data width to
            //    make sure we don't go past dma_buf.len()/width
            //  * we will transmute the array back to a [u16] after the DMA
            //    transfer is complete
            let dma_buf_ptr = unsafe { mem::transmute::<*mut u16, *mut u8>(buffer1.as_mut_ptr()) };
            let dma_buf = unsafe { slice::from_raw_parts_mut(dma_buf_ptr, buffer1.len() * 2) };

            // set up the DMA
            self.rx_dma.get().map(move |dma| {
                self.dma_running.set(true);
                dma.enable();
                self.rx_length.set(dma_len);
                dma.do_xfer(self.rx_dma_peripheral, dma_buf, dma_len);
            });

            // start timer
            regs.cr.set(0x04);

            (ReturnCode::SUCCESS, None, None)
        }
    }

    /// Provide a new buffer to send on-going buffered continuous samples to.
    /// This is expected to be called after the `samples_ready` callback.
    ///
    /// - `buf`: buffer to fill with samples
    /// - `length`: number of samples to collect (up to buffer length)
    fn provide_buffer(&self,
                      buf: &'static mut [u16],
                      length: usize)
                      -> (ReturnCode, Option<&'static mut [u16]>) {
        if !self.enabled.get() {
            (ReturnCode::EOFF, Some(buf))

        } else if !self.active.get() {
            // cannot continue sampling that isn't running
            (ReturnCode::EINVAL, Some(buf))

        } else if !self.continuous.get() {
            // cannot continue a single sample operation
            (ReturnCode::EINVAL, Some(buf))

        } else if self.next_dma_buffer.is_some() {
            // we've already got a second buffer, we don't need a third yet
            (ReturnCode::EBUSY, Some(buf))

        } else {

            // store the buffer for later use
            self.next_dma_buffer.replace(buf);
            self.next_dma_length.set(length);

            (ReturnCode::SUCCESS, None)
        }
    }

    /// Reclaim buffers after the ADC is stopped.
    /// This is expected to be called after `stop_sampling`.
    fn retrieve_buffers(&self)
                        -> (ReturnCode, Option<&'static mut [u16]>, Option<&'static mut [u16]>) {

        if self.active.get() {
            // cannot return buffers while running
            (ReturnCode::EINVAL, None, None)
        } else {
            // we're not running, so give back whatever we've got
            (ReturnCode::SUCCESS, self.next_dma_buffer.take(), self.stopped_buffer.take())
        }
    }
}

/// Implements a client of a DMA.
impl dma::DMAClient for Adc {
    /// Handler for DMA transfer completion.
    ///
    /// - `pid`: the DMA peripheral that is complete
    fn xfer_done(&self, pid: dma::DMAPeripheral) {
        // check if this was an RX transfer
        if pid == self.rx_dma_peripheral {
            // RX transfer was completed

            // get buffer filled with samples from DMA
            let dma_buffer = self.rx_dma.get().map_or(None, |rx_dma| {
                self.dma_running.set(false);
                let dma_buf = rx_dma.abort_xfer();
                rx_dma.disable();
                dma_buf
            });

            // get length of received buffer
            let length = self.rx_length.get();

            // start a new transfer with the next buffer
            // we need to do this quickly in order to keep from missing samples.
            // At 175000 Hz, we only have 5.8 us (~274 cycles) to do so
            self.next_dma_buffer.take().map(|buf| {

                // first determine the buffer's length in samples
                let dma_len = cmp::min(buf.len(), self.next_dma_length.get());

                // only continue with a nonzero length. If we were given a
                // zero-length buffer or length field, assume that the user knew
                // what was going on, and just don't use the buffer
                if dma_len > 0 {
                    // change buffer into a [u8]
                    // this is unsafe but acceptable for the following reasons
                    //  * the buffer is aligned based on 16-bit boundary, so the
                    //    8-bit alignment is fine
                    //  * the DMA is doing checking based on our expected data
                    //    width to make sure we don't go past
                    //    dma_buf.len()/width
                    //  * we will transmute the array back to a [u16] after the
                    //    DMA transfer is complete
                    let dma_buf_ptr =
                        unsafe { mem::transmute::<*mut u16, *mut u8>(buf.as_mut_ptr()) };
                    let dma_buf = unsafe { slice::from_raw_parts_mut(dma_buf_ptr, buf.len() * 2) };

                    // set up the DMA
                    self.rx_dma.get().map(move |dma| {
                        self.dma_running.set(true);
                        dma.enable();
                        self.rx_length.set(dma_len);
                        dma.do_xfer(self.rx_dma_peripheral, dma_buf, dma_len);
                    });

                } else {
                    // if length was zero, just keep the buffer in the takecell
                    // so we can return it when `stop_sampling` is called
                    self.next_dma_buffer.replace(buf);
                }
            });

            // alert client
            self.client.get().map(|client| {
                dma_buffer.map(|dma_buf| {

                    // change buffer back into a [u16]
                    // the buffer was originally a [u16] so this should be okay
                    let buf_ptr =
                        unsafe { mem::transmute::<*mut u8, *mut u16>(dma_buf.as_mut_ptr()) };
                    let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, dma_buf.len() / 2) };

                    // pass the buffer up to the next layer. It will then either
                    // send down another buffer to continue sampling, or stop
                    // sampling
                    client.samples_ready(buf, length);
                });
            });
        }
    }
}
