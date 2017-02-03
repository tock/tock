// adc.rs -- Implementation of SAM4L ADCIFE.
//
// This is a bare-bones implementation of the SAM4L ADC. It is bare-bones
// because it provides little flexibility on how samples are taken. Currently,
// all samples
//   - are 12 bits
//   - use the ground pad as the negative reference
//   - use a VCC/2 positive reference
//   - are right justified
//
// NOTE: The pin labels/assignments on the Firestorm schematic are
// incorrect. The mappings should be
//   AD5 -> ADCIFE channel 6
//   AD4 -> ADCIFE channel 5
//   AD3 -> ADCIFE channel 4
//   AD2 -> ADCIFE channel 3
//   AD1 -> ADCIFE channel 2
//   AD0 -> ADCIFE channel 1
//
// but in reality they are
//   AD5 -> ADCIFE channel 1
//   AD4 -> ADCIFE channel 2
//   AD3 -> ADCIFE channel 3
//   AD2 -> ADCIFE channel 4
//   AD1 -> ADCIFE channel 5
//   AD0 -> ADCIFE channel 6
//
//
//
// Author: Philip Levis <pal@cs.stanford.edu>
// Date: August 5, 2015
//

use core::cell::Cell;
use core::mem;
use kernel::common::volatile_cell::VolatileCell;
use kernel::hil;
use kernel::hil::adc;
use kernel::returncode::ReturnCode;
use nvic;
use pm::{self, Clock, PBAClock};
use scif;

#[repr(C, packed)]
pub struct AdcRegisters {
    // From page 1005 of SAM4L manual
    cr: VolatileCell<u32>, // Control               (0x00)
    cfg: VolatileCell<u32>, // Configuration        (0x04)
    sr: VolatileCell<u32>, // Status                (0x08)
    scr: VolatileCell<u32>, // Status clear         (0x0c)
    pad: VolatileCell<u32>, // padding/reserved
    seqcfg: VolatileCell<u32>, // Sequencer config  (0x14)
    cdma: VolatileCell<u32>, // Config DMA          (0x18)
    tim: VolatileCell<u32>, // Timing config        (0x1c)
    itimer: VolatileCell<u32>, // Internal timer    (0x20)
    wcfg: VolatileCell<u32>, // Window config       (0x24)
    wth: VolatileCell<u32>, // Window threshold     (0x28)
    lcv: VolatileCell<u32>, // Last converted value (0x2c)
    ier: VolatileCell<u32>, // Interrupt enable     (0x30)
    idr: VolatileCell<u32>, // Interrupt disable    (0x34)
    imr: VolatileCell<u32>, // Interrupt mask       (0x38)
    calib: VolatileCell<u32>, // Calibration        (0x3c)
    version: VolatileCell<u32>, // Version          (0x40)
    parameter: VolatileCell<u32>, // Parameter      (0x44)
}

// Page 59 of SAM4L data sheet
const BASE_ADDRESS: *mut AdcRegisters = 0x40038000 as *mut AdcRegisters;

pub struct Adc {
    registers: *mut AdcRegisters,
    enabled: Cell<bool>,
    channel: Cell<u8>,
    client: Cell<Option<&'static hil::adc::Client>>,
}

pub static mut ADC: Adc = Adc::new(BASE_ADDRESS);

impl Adc {
    const fn new(base_address: *mut AdcRegisters) -> Adc {
        Adc {
            registers: base_address,
            enabled: Cell::new(false),
            channel: Cell::new(0),
            client: Cell::new(None),
        }
    }

    pub fn set_client<C: hil::adc::Client>(&self, client: &'static C) {
        self.client.set(Some(client));
    }

    pub fn handle_interrupt(&mut self) {
        let val: u16;
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };
        // Make sure this is the SEOC (Sequencer end-of-conversion) interrupt
        let status = regs.sr.get();
        if status & 0x01 == 0x01 {
            // Clear SEOC interrupt
            regs.scr.set(0x0000001);
            // Disable SEOC interrupt
            regs.idr.set(0x00000001);
            // Read the value from the LCV register.
            // The sample is 16 bits wide
            val = (regs.lcv.get() & 0xffff) as u16;
            self.client.get().map(|client| { client.sample_done(val); });
        }
    }
}

impl adc::AdcSingle for Adc {
    fn initialize(&self) -> ReturnCode {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };
        if !self.enabled.get() {
            self.enabled.set(true);
            // This logic is from 38.6.1 "Initializing the ADCIFE" of
            // the SAM4L data sheet
            // 1. Start the clocks, ADC uses GCLK10, choose to
            // source it from RCSYS (115Khz)
            unsafe {
                pm::enable_clock(Clock::PBA(PBAClock::ADCIFE));
                nvic::enable(nvic::NvicIdx::ADCIFE);
                scif::generic_clock_enable(scif::GenericClock::GCLK10, scif::ClockSource::RCSYS);
            }
            // 2. Insert a fixed delay
            for _ in 1..10000 {
                let _ = regs.cr.get();
            }

            // 3, Enable the ADC
            let mut cr: u32 = regs.cr.get();
            cr |= 1 << 8;
            regs.cr.set(cr);

            // 4. Wait until ADC ready
            while regs.sr.get() & (1 << 24) == 0 {}
            // 5. Turn on bandgap and reference buffer
            let cr2: u32 = (1 << 10) | (1 << 8) | (1 << 4);
            regs.cr.set(cr2);

            // 6. Configure the ADCIFE
            // Setting below in the configuration register sets
            //   - the clock divider to be 4,
            //   - the source to be the Generic clock,
            //   - the max speed to be 300 ksps, and
            //   - the reference voltage to be VCC/2
            regs.cfg.set(0x00000008);
            while regs.sr.get() & (0x51000000) != 0x51000000 {}
        }
        return ReturnCode::SUCCESS;
    }

    fn sample(&self, channel: u8) -> ReturnCode {
        let regs: &mut AdcRegisters = unsafe { mem::transmute(self.registers) };
        if !self.enabled.get() {
            return ReturnCode::EOFF;
        } else if channel > 14 {
            return ReturnCode::EINVAL;
        } else {
            self.channel.set(channel);
            // This configuration sets the ADC to use Pad Ground as the
            // negative input, and the ADC channel as the positive. Since
            // this is a single-ended sample, the bipolar bit is set to zero.
            // Trigger select is set to zero because this denotes a software
            // sample. Gain is 0.5x (set to 111). Resolution is set to 12 bits
            // (set to 0).

            let chan_field: u32 = (self.channel.get() as u32) << 16;
            let mut cfg: u32 = chan_field;
            cfg |= 0x00700000; // MUXNEG   = 111 (ground pad)
            cfg |= 0x00008000; // INTERNAL =  10 (int neg, ext pos)
            cfg |= 0x00000000; // RES      =   0 (12-bit)
            cfg |= 0x00000000; // TRGSEL   =   0 (software)
            cfg |= 0x00000000; // GCOMP    =   0 (no gain error corr)
            cfg |= 0x00000070; // GAIN     = 111 (0.5x gain)
            cfg |= 0x00000000; // BIPOLAR  =   0 (not bipolar)
            cfg |= 0x00000000; // HWLA     =   0 (no left justify value)
            regs.seqcfg.set(cfg);
            // Enable end of conversion interrupt
            regs.ier.set(1);
            // Initiate conversion
            regs.cr.set(8);
            return ReturnCode::SUCCESS;
        }
    }

    fn cancel_sample(&self) -> ReturnCode {
        return ReturnCode::FAIL;
    }
}

/// Not implemented yet. -pal 12/22/16
impl adc::AdcContinuous for Adc {
    type Frequency = adc::Freq1KHz;

    fn compute_interval(&self, interval: u32) -> u32 {
        interval
    }

    fn sample_continuous(&self, _channel: u8, _interval: u32) -> ReturnCode {
        ReturnCode::FAIL
    }

    fn cancel_sampling(&self) -> ReturnCode {
        ReturnCode::FAIL
    }
}

interrupt_handler!(adcife_handler, ADCIFE);
