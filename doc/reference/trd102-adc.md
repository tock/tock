Kernel Analog-to-Digital Conversion HIL
========================================

**TRD:** 102 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis and Branden Ghena<br/>
**Draft-Created:** Dec 18, 2016<br/>
**Draft-Modified:** May 5, 2017<br/>
**Draft-Version:** 2<br/>
**Draft-Discuss:** tock-dev@googlegroups.com</br>

Abstract
-------------------------------

This document describes the hardware independent layer interface (HIL) for
analog-to-digital conversion in the Tock operating system kernel. It describes
the Rust traits and other definitions for this service as well as the reasoning
behind them. This document also describes an implementation of the ADC HIL for
the SAM4L. This document is in full compliance with <a href="#trd1">TRD1</a>.

1 Introduction
========================================

Analog-to-digital converters (ADCs) are devices that convert analog input
signals to discrete digital output signals, typically voltage to a binary
number. While different microcontrollers can have very different control
registers and operating modes, the basic high-level interface they provide
is very uniform. Software that wishes to use more advanced features can
directly use the per-chip implementations, which may export these features.

The ADC HIL is the kernel crate, in module hil::adc. It
provides three traits:

  * kernel::hil::adc::Adc - provides basic interface for individual analog samples
  * kernel::hil::adc::Client - receives individual analog samples from the ADC
  * kernel::hil::adc::AdcHighSpeed - provides high speed buffered analog sampling interface
  * kernel::hil::adc::HighSpeedClient - receives buffers of analog samples from the ADC

The rest of this document discusses each in turn.


2 Adc trait
========================================

The Adc trait is for requesting individual analog to digital conversions,
either one-shot or repeatedly. It has four functions and one associated type:

```
/// Simple interface for reading an ADC sample on any channel.
pub trait Adc {
    /// The chip-dependent type of an ADC channel.
    type Channel;

    /// Initialize must be called before taking a sample.
    fn initialize(&self) -> ReturnCode;

    /// Request a single ADC sample on a particular channel.
    /// Used for individual samples that have no timing requirements.
    fn sample(&self, channel: &Self::Channel) -> ReturnCode;

    /// Request repeated ADC samples on a particular channel.
    /// Callbacks will occur at the given frequency with low jitter and can be
    /// set to any frequency supported by the chip implementation. However
    /// callbacks may be limited based on how quickly the system can service
    /// individual samples, leading to missed samples at high frequencies.
    fn sample_continuous(&self, channel: &Self::Channel, frequency: u32) -> ReturnCode;

    /// Stop a sampling operation.
    /// Can be used to stop any simple or high-speed sampling operation. No
    /// further callbacks will occur.
    fn stop_sampling(&self) -> ReturnCode;
}
```

The `initialize` function configures the hardware to perform analog sampling.
It MUST be called at least once before any samples are taken. It only needs to
be called once, not once per sample. This function MUST return SUCCESS upon
correct initialization or FAIL if the hardware fails to initialize
successfully. If the driver is already initialized, the function SHOULD return
SUCCESS.

The `sample` function starts a single conversion on the specified ADC channel.
The exact binding of this channel to external or internal analog inputs is
board-dependent. The function MUST return SUCCESS if the analog conversion has
been started, EOFF if the ADC is not initialized or enabled, EBUSY if a
conversion is already in progress, or EINVAL if the specified channel is
invalid. The `sample_ready` callback of the client MUST be called when the
conversion is complete.

The `sample_continuous` function begins repeated individual conversions on a
specified channel. Conversions MUST continue at the specified frequency until
`stop_sampling` is called. The `sample_ready` callback of the client MUST be
called when each conversion is complete. The channels and frequency ranges
supported are board-dependent. The function MUST return SUCCESS if repeated
analog conversions have been started,  EOFF if the ADC is not initialized or
enabled, EBUSY if a conversion is already in progress, or EINVAL if the
specified channel or frequency are invalid.


The `cancel_sample` function may be used to try to cancel an outstanding
conversion request. Because the conversion may have already begun, or
even have already completed but be enqueued within the kernel, this
call may not succeed and the ADC may still issue a callback with the
sample via the `Client` trait. This function MUST return SUCCESS, ERESERVE
or FAIL. SUCCESS indicates that a callback WILL NOT be issued, while
a failure code (FAIL or ERESERVE) indicate that the callback WILL be
issued normally.



3 AdcContinuous
========================================

The AdcContinuous trait is for requesting a stream of ADC conversions
at a fixed frequency. These samples are expected to be taken with
low jitter (e.g., clocked by an underyling hardware clock and not
controlled by software).

Because the AdcContinuous trait takes an interval between samples,
it needs a specify a time unit for this interval. As platforms can
vary by orders of magnitude in the interval they can support, the
trait instance should specify the time unit that represents the
highest possible precision. For example, if an ADC can sample at
maximum rate of 10kHz, the time unit should be 10kHz or less.
Software trying to sample at a significantly higher precision
(e.g., 1MHz ticks) should not be able to compile.

The `Frequency` trait specifies the time unit of continuous
sampling precision.

    pub trait Frequency {
        fn frequency() -> u32;
    }

    pub trait AdcContinuous {
        type Frequency: Frequency;
        fn compute_interval(&self, interval:u32) -> u32;
        fn sample_continuous(&self, channel: u8, interval:u32) -> Result;
        fn cancel_sampling(&self) -> Result;
    }

The AdcContinuous trait has three functions. The first,
`compute_interval` takes a specified interval and returns
the actual interval that the ADC will provide. Whenever
possible, `compute_interval` SHOULD be the identity function.
However, underlying clock rates and hardware details MAY cause
it to be a slightly different value.

The second, `sample_continuous`, starts a series of continuous
samples on `channel` with an interval of `interval`. The
ADC might not sample at exactly the interval specified, due
to underlying clock rates and hardware details. The value
passed in `interval` is effectively passed to `compute_interval`
to calculate the actual interval used. If a call to `sample_continuous`
is passed an `interval` generated by calling `compute_interval`,
then the implementation SHOULD match this interval exactly.

The third, `cancel_sampling`, stops the continuous sampling.

The `Frequency` type defines the frequency of the intervals. There
are three default values of `Frequency`:
  * 1kHz
  * 32kHz
  * 1MHz

4 Client
========================================

The Client trait is how a caller provides a callback to the ADC
implementation. Using a function defined outside the ADC trait, it
registers a reference implementing the Client trait with the ADC
implementation.

    pub trait Client {
        /// Called when a sample is ready.
        fn sample_done(&self, sample: u16, result: Result);
    }

Whenever the ADC completes a sample, it invokes the `sample_done`
function. If the sample was taken successfully, then `sample` MUST
contain the value and `result` MUST be SUCCESS. If `sample` contains a
value, the value MUST be left shifted so the most significant bit of
`sample` is the most significant bit of the value. Possible values for
`result` are SUCCESS, FAIL, EBUSY, EOFF, ERESERVE, EINVAL, or ECANCEL.


5 Example Implementation: SAM4L
========================================

The SAM4L ADC has a flexible ADC, supporting differential
and single-ended inputs, 8 or 12 bit samples, configurable clocks, 
reference voltages, and grounds. It supports periodic sampling supported
by an internal timer.  The SAM4L ADC uses generic clock 10 (GCLK10). 
The ADC is peripheral 38, so its control registers are found at
address 0x40038000. A complete description of the ADC can be
found in chapter 38 of the SAM4L datasheet.

The current implementation, found in `chips/sam4l/adc.rs`, implements 
the `AdcSingle` trait.


5.1 Initialization
---------------------------------

Initialization follows the process outlined in section 38.6.1 of the 
SAM4L datasheet.  The first step of initialization is to configure 
the ADC's clock.  The implementation configures GCLK10 to use the 
RCSYS clock, which operates at 115kHz.

    unsafe {
        pm::enable_clock(Clock::PBA(PBAClock::ADCIFE));
        nvic::enable(nvic::NvicIdx::ADCIFE);
        scif::generic_clock_enable(scif::GenericClock::GCLK10, scif::ClockSource::RCSYS);
    }

It then has a fixed delay, to ensure the ADC cell is ready. Next,
it enables the ADC and waits until it is ready. Once the ADC
is ready, it turns on the bandgap and reference buffers. Once
those buffers are enabled, it sets the ADC to takes samples against a
reference voltage of VCC/2 and the clock divider to be 4. Since
the clock frequency is 115kHz and it takes 6 clock cyles to 
take a 12-bit sample, the maximum default sampling rate is
115kHz / (4 * 6) ~= 4800ksps.

5.2 AdcSingle: Taking a sample
---------------------------------

A call to `AdcSingle.sample` has a few basic checks before
initiating a sample. It checks that the channel is valid
and that the ADC is enabled.

The sample configures the ADC to use the ground pad as the
ground (for reference to VCC/2) and take a 12-bit sample.
The sample is not left-justified (it is in the 12 least
significant bits).

The implementation does not support cancelling outstanding
samples: `cancel_sample` always returns `ReturnCode::FAIL`.

5.3 AdcSingle: Handling a sample
---------------------------------

The ADCIFE interrupt registers `handle_interrupt` bottom half
to run on `Adc`. This function clears the interrupt, reads
the sample result from the LCV register, and issues a callback
to an ADC client, if one has ben registered with `set_client`:

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


6 Authors' Address
========================================

```
Philip Levis
409 Gates Hall
Stanford University
Stanford, CA 94305

phone - +1 650 725 9046

email - pal@cs.stanford.edu
```

```
Branden Ghena

email - brghena@umich.edu
```

7 Citations
========================================

<a name="trd1"/>[TRD1] <a href="trd1-trds.md">Tock Reference Document (TRD) Structure and Keywords</a>
