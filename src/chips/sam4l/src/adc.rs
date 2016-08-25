// adc.rs -- Implementation of SAM4L ADCIFE.
//
// This is a bare-bones implementation of the SAM4L ADC. It is bare-bones
// because it provides little flexibility on how samples are taken. Currently,
// all samples
//   - are 12 bits
//   - use the ground pad as the negative reference
//   - use a 1V positive reference
//   - are hardware left justified (16 bits wide, bottom 4 bits empty)
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
use core::intrinsics;
use hil::adc::{Request, AdcInternal};
use nvic;
use pm::{self, Clock, PBAClock};
use scif;


#[repr(C, packed)]
#[allow(dead_code,missing_copy_implementations)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub struct AdcRegisters { // From page 1005 of SAM4L manual
    cr:        usize,   // Control               (0x00)
    cfg:       usize,   // Configuration         (0x04)
    sr:        usize,   // Status                (0x08)
    scr:       usize,   // Status clear          (0x0c)
    pad:       usize,   // padding/reserved
    seqcfg:    usize,   // Sequencer config      (0x14)
    cdma:      usize,   // Config DMA            (0x18)
    tim:       usize,   // Timing config         (0x1c)
    itimer:    usize,   // Internal timer        (0x20)
    wcfg:      usize,   // Window config         (0x24)
    wth:       usize,   // Window threshold      (0x28)
    lcv:       usize,   // Last converted value  (0x2c)
    ier:       usize,   // Interrupt enable      (0x30)
    idr:       usize,   // Interrupt disable     (0x34)
    imr:       usize,   // Interrupt mask        (0x38)
    calib:     usize,   // Calibration           (0x3c)
    version:   usize,   // Version               (0x40)
    parameter: usize,   // Parameter             (0x44)
}

// Page 59 of SAM4L data sheet
pub const BASE_ADDRESS: usize = 0x40038000;

pub struct Adc {
    registers: *mut AdcRegisters,
    enabled: bool,
    channel: Cell<u8>,
    request: Cell<Option<&'static Request>>,
}

impl Adc {
    pub fn new() -> Adc {
        let address = BASE_ADDRESS;
        Adc {
            registers: unsafe { intrinsics::transmute(address) },
            enabled: false,
            channel: Cell::new(0),
            request: Cell::new(None),
        }
    }
    pub fn handle_interrupt(&mut self) {
        let val: u16;
        unsafe {
            // Clear SEOC interrupt
            intrinsics::volatile_store(&mut (*self.registers).scr, 0x0000001);
            // Disable SEOC interrupt
            intrinsics::volatile_store(&mut (*self.registers).idr, 0x00000001);
            // Read the value from the LCV register.
            // Note that since samples are left-justified (HWLA mode)
            // the sample is 16 bits wide
            val = (intrinsics::volatile_load(&(*self.registers).lcv) & 0xffff) as u16;
        }
        if self.request.get().is_none() {
            return;
        }
        let opt = self.request.get().take();
        let copt: &'static Request = opt.unwrap();
        self.request = Cell::new(None);
        copt.sample_done(val, copt);
    }
}

impl AdcInternal for Adc {
    fn initialize(&'static mut self) -> bool {
        if !self.enabled {
            self.enabled = true;
            unsafe {
                // This logic is from 38.6.1 "Initializing the ADCIFE" of
                // the SAM4L data sheet
                // 1. Start the clocks
                pm::enable_clock(Clock::PBA(PBAClock::ADCIFE));
                nvic::enable(nvic::NvicIdx::ADCIFE);
                scif::generic_clock_enable(scif::GenericClock::GCLK10, scif::ClockSource::RCSYS);
                // 2. Insert a fixed delay
                for _ in 1..10000 {
                    let _ = intrinsics::volatile_load(&(*self.registers).cr);
                }

                // 3, Enable the ADC
                let mut cr: usize = intrinsics::volatile_load(&(*self.registers).cr);
                cr |= 1 << 8;
                intrinsics::volatile_store(&mut (*self.registers).cr, cr);

                // 4. Wait until ADC ready
                while intrinsics::volatile_load(&(*self.registers).sr) & (1 << 24) == 0 {}
                // 5. Turn on bandgap and reference buffer
                let cr2: usize = (1 << 10) | (1 << 8) | (1 << 4);
                intrinsics::volatile_store(&mut (*self.registers).cr, cr2);

                // 6. Configure the ADCIFE
                // Setting all 0s in the configuration register sets
                //   - the clock divider to be 4,
                //   - the source to be the Generic clock,
                //   - the max speed to be 300 ksps, and
                //   - the reference voltage to be 1.0V
                intrinsics::volatile_store(&mut (*self.registers).cfg, 0x00000030 as usize);
                while intrinsics::volatile_load(&(*self.registers).sr) & (0x51000000) !=
                      0x51000000 {}
            }
        }
        return true;
    }

    fn sample(&self, channel: u8, request: &'static Request) -> bool {
        if !self.enabled || self.request.get().is_some() || channel > 14 {
            return false;
        } else {
            self.request.set(Some(request));
            self.channel.set(channel);
            // This configuration sets the ADC to use Pad Ground as the
            // negative input, and the ADC channel as the positive. Since
            // this is a single-ended sample, the bipolar bit is set to zero.
            // Trigger select is set to zero because this denotes a software
            // sample. Gain is 1x (set to 0). Resolution is set to 12 bits
            // (set to 0). The one trick is that the half word left adjust
            // (HWLA) is set to 1. This means that both 12-bit and 8-bit
            // samples are left justified to the lower 16 bits. So they share
            // the same most significant bit but for 8 bit samples the lower
            // 8 bits are zero and for 12 bits the lower 4 bits are zero.

            let chan_field: usize = (self.channel.get() as usize) << 16;
            unsafe {
                let mut cfg: usize = chan_field;
                cfg |= 0x00700000; // MUXNEG   = 111 (ground pad)
                cfg |= 0x00008000; // INTERNAL =  10 (int neg, ext pos)
                cfg |= 0x00000000; // RES      =   0 (12-bit)
                cfg |= 0x00000000; // TRGSEL   =   0 (software)
                cfg |= 0x00000000; // GCOMP    =   0 (no gain error corr)
                cfg |= 0x00000000; // GAIN     =   0 (1x gain)
                cfg |= 0x00000000; // BIPOLAR  =   0 (not bipolar)
                cfg |= 0x00000001; // HWLA     =   1 (left justify value)
                intrinsics::volatile_store(&mut (*self.registers).seqcfg, cfg);
                // Enable end of conversion interrupt
                intrinsics::volatile_store(&mut (*self.registers).ier, 1);
                // Initiate conversion
                intrinsics::volatile_store(&mut (*self.registers).cr, 8);
                return true;
            }
        }
    }
}

interrupt_handler!(adcife_handler, ADCIFE);
