use core::prelude::*;
use core::intrinsics;
use hil::{adc};
use pm::{self, Clock, PBAClock};

/* This is a first cut at an implementation of the SAM4L ADC.
   It only allows a single sample at a time, and has three known bugs,
   all serious:
     1) It has not yet been tested (I have no hardware)
     2) Interrupts are not hooked up yet
     3) You cannot request a sample in a callback.

    I've added it mostly as an example of how we might implement
    these services.
*/

#[repr(C, packed)]
#[allow(dead_code,missing_copy_implementations)]
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
  registers: &'static mut AdcRegisters,
  enabled: bool,
  request: Option<&'static mut adc::Request>
}

impl Adc {
    pub fn new() -> Adc {
        let address = BASE_ADDRESS;
        Adc {
            registers: unsafe { intrinsics::transmute(address) },
            enabled: false,
            request: None
        }
    }
}

impl adc::AdcInternal for Adc {
    fn initialize(&mut self) -> bool {
        if !self.enabled {
            self.enabled = true;
            unsafe {pm::enable_clock(Clock::PBA(PBAClock::ADCIFE));}
            volatile!(self.registers.cr |= 1 << 8);  // Enable ADC
            volatile!(self.registers.cr |= 1 << 10); // Enable bandgap buffer
            volatile!(self.registers.cr |= 1 << 4);  // Enable reference buffer
            if (volatile!(self.registers.sr) & (1 << 24)) != 0 { // ADC is enabled
                // Setting all 0s in the configuration register sets
                //   - the clock divider to be 4,
                //   - the source to be the Generic clock,
                //   - the max speed to be 300 ksps, and
                //   - the reference voltage to be 1.0V
                volatile!(self.registers.cfg = 0);
            }
        }
        return true;
    }
    
    fn sample(&mut self, request: &'static mut adc::Request) -> bool {
        if self.enabled || request.channel > 14 {
            return false;
        } else {
            self.enabled = true;
            self.request = Some(request);
 
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

            let mut channel:usize = request.channel as usize;
            channel = channel << 16;
            volatile!(self.registers.seqcfg = 0x00708081 | channel);
/*                    00708081 =  7      << 20 | // MUXNEG
                                  channel << 16 | // MUXPOS
                                  2       << 14 | // Internal
                                  0       << 12 | // Resolution
                                  0       << 8  | // TRGSEL
                                  1       << 7  | // GCOMP
                                  0       << 4  | // GAIN
                                  0       << 2  | // BIPOLAR
                                  1));*/
            // Enable end of conversion interrupt
            volatile!(self.registers.ier = 1);
            // Initiate conversion
            volatile!(self.registers.cr = 2);
            return true;
        }
    }
    
    fn handle_interrupt(&mut self) {
        // Disable further interrupts
        volatile!(self.registers.idr = 1);
        match self.request {
            Some(ref mut request) => {
                let val = volatile!(self.registers.lcv) & 0xffff;         
                request.callback.read_done(val as u16);
            }
            None => {}

        }
        self.request = None;
    }
}
