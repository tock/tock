Kernel Analog-to-Digital Conversion HIL
========================================

**TRD:** 102 <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Philip Levis and Branden Ghena<br/>
**Draft-Created:** Dec 18, 2016<br/>
**Draft-Modified:** June 12, 2017<br/>
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
either one-shot or repeatedly. It is implemented by chip drivers to provide ADC
functionality. Data is provided through the Client trait. It has four functions
and one associated type:

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

The `stop_sampling` function can be used to stop any sampling operation,
single, continuous, or high speed. Conversions which have already begun are
canceled. `stop_sampling` MUST be safe to call from any callback in the Client
or HighSpeedClient traits. The function MUST return SUCCESS, EOFF, or EINVAL.
SUCCESS indicates that all conversions are stopped and no further callbacks
will occur, EOFF means the ADC is not initialized or enabled, and EINVAL means
the ADC was not active.

The `channel` type is used to signify which ADC channel to sample data on for
various commands. What it maps to is implementation-specific, possibly an I/O
pin number or abstract notion of a channel. One approach used for channels by
the SAM4L implementation is for the capsule to keep an array of possible
channels, which are connected to pins by the board `main.rs` file, and selected
from by userland applications.


3 Client trait
========================================

The Client trait handles responses from Adc trait sampling commands. It is
implemented by capsules to receive chip driver responses. It has one function:

```
/// Trait for handling callbacks from simple ADC calls.
pub trait Client {
    /// Called when a sample is ready.
    fn sample_ready(&self, sample: u16);
}
```

The `sample_ready` function is called whenever data is available from a
`sample` or `sample_continuous` call. It is safe to call `stop_sampling` within
the `sample_ready` callback. The sample data returned is a maximum of 16 bits
in resolution, with the exact data resolution being chip-specific. If data is
less than 16 bits (for example 12-bits on the SAM4L), it SHOULD be placed in
the least significant bits of the `sample` value.


4 AdcHighSpeed trait
========================================

The AdcHighSpeed trait is used for sampling data at high frequencies such that
receiving individual samples would be untenable. Instead, it provides an
interface that returns buffers filled with samples. This trait relies on the
Adc trait being implemented as well in order to provide primitives like
`intialize` and `stop_sampling` which are used for ADCs in this mode as well.
While we expect many chips to support the Adc trait, we expect the AdcHighSpeed
trait to be implemented due to a high-speed sampling need on a platform. The
trait has three functions:

```
/// Interface for continuously sampling at a given frequency on a channel.
/// Requires the AdcSimple interface to have been implemented as well.
pub trait AdcHighSpeed: Adc {
    /// Start sampling continuously into buffers.
    /// Samples are double-buffered, going first into `buffer1` and then into
    /// `buffer2`. A callback is performed to the client whenever either buffer
    /// is full, which expects either a second buffer to be sent via the
    /// `provide_buffer` call. Length fields correspond to the number of
    /// samples that should be collected in each buffer. If an error occurs,
    /// the buffers will be returned.
    fn sample_highspeed(&self,
                        channel: &Self::Channel,
                        frequency: u32,
                        buffer1: &'static mut [u16],
                        length1: usize,
                        buffer2: &'static mut [u16],
                        length2: usize)
                        -> (ReturnCode, Option<&'static mut [u16]>,
                            Option<&'static mut [u16]>);

    /// Provide a new buffer to fill with the ongoing `sample_continuous`
    /// configuration.
    /// Expected to be called in a `buffer_ready` callback. Note that if this
    /// is not called before the second buffer is filled, samples will be
    /// missed. Length field corresponds to the number of samples that should
    /// be collected in the buffer. If an error occurs, the buffer will be
    /// returned.
    fn provide_buffer(&self,
                      buf: &'static mut [u16],
                      length: usize)
                      -> (ReturnCode, Option<&'static mut [u16]>);

    /// Reclaim ownership of buffers.
    /// Can only be called when the ADC is inactive, which occurs after a
    /// successful `stop_sampling`. Used to reclaim buffers after a sampling
    /// operation is complete. Returns success if the ADC was inactive, but
    /// there may still be no buffers that are `some` if the driver had already
    /// returned all buffers.
    fn retrieve_buffers(&self)
                        -> (ReturnCode, Option<&'static mut [u16]>,
                            Option<&'static mut [u16]>);
}
```

The `sample_highspeed` function is used to perform high-speed double-buffered
sampling. After the first buffer is filled with samples, the `samples_ready`
function will be called and sampling will immediately continue into the second
buffer in order to reduce jitter between samples. Additional buffers SHOULD be
passed through the `provide_buffer` call. However, if none are provided, the
driver MUST cease sampling once it runs out of buffers. In case of an error,
the buffers will be immediately returned from the function. The channels and
frequencies acceptable are chip-specific. The return code MUST be SUCCESS if
sampling has begun successfully, EOFF if the ADC is not enabled or initialized,
EBUSY if the ADC is in use, or EINVAL if the channel or frequency are invalid.

The `provide_buffer` function is used to provide additional buffers to an
ongoing high-speed sampling operation. It is expected to be called within a
`samples_ready` callback in order to keep sampling running without delay. In
case of an error, the buffer will be immediately returned from the function. It
is not an error to fail to call `provide_buffer` and the underlying driver MUST
cease sampling if no buffers are remaining. It is an error to call
`provide_buffer` twice without having received a buffer through
`samples_ready`. The prior settings for channel and frequency will persist. The
return code MUST be SUCCESS if the buffer has been saved for later use, EOFF if
the ADC is not initialized or enabled, EINVAL if there is no currently running
continuous sampling operation, or EBUSY if an additional buffer has already
been provided.

The `retrieve_buffers` function returns ownership of all buffers owned by the
chip implementation. All ADC operations MUST be stopped before buffers are
returned. Any data within the buffers SHOULD be considered invalid. It is
expected that `retrieve_buffers` will be called from within a `samples_ready`
callback after calling `stop_sampling`. Up to two buffers will be returned by
the function. The return code MUST be SUCCESS if the ADC is not in operation
(although as few as zero buffers may be returned), EINVAL MUST be returned if
an ADC operation is still in progress.


5 HighSpeedClient trait
========================================

The HighSpeedClient trait is used to receive samples from a call to
`sample_highspeed`. It is implemented by a capsule to receive chip driver
responses. It has one function:

```
/// Trait for handling callbacks from high-speed ADC calls.
pub trait HighSpeedClient {
    /// Called when a buffer is full.
    /// The length provided will always be less than or equal to the length of
    /// the buffer. Expects an additional call to either provide another buffer
    /// or stop sampling
    fn samples_ready(&self, buf: &'static mut [u16], length: usize);
}
```

The `samples_ready` function receives a buffer filled with up to `length`
number of samples. Each sample MAY be up to 16 bits in size. Smaller samples
SHOULD be aligned such that the data is in the least significant bits of each
value. The length field MUST match the length passed in with the buffer
(through either `sample_highspeed` or `provide_buffer`). Within the
`samples_ready` callback, the capsule SHOULD call `provide_buffer` if it wishes
to continue sampling. Alternatively, `stop_sampling` and `retrieve_buffers`
SHOULD be called to stop the ongoing ADC operation. 


6 Example Implementation: SAM4L
========================================

The SAM4L ADC has a flexible ADC, supporting differential and single-ended
inputs, 8 or 12 bit samples, configurable clocks, reference voltages, and
grounds. It supports periodic sampling supported by an internal timer.  The
SAM4L ADC uses generic clock 10 (GCLK10). The ADC is peripheral 38, so its
control registers are found at address 0x40038000. A complete description of
the ADC can be found in Chapter 38 (Page 995) of the
[SAM4L datasheet](http://www.atmel.com/images/atmel-42023-arm-microcontroller-atsam4l-low-power-lcd_datasheet.pdf).

The current implementation, found in `chips/sam4l/adc.rs`, implements 
the `Adc` and `AdcHighSpeed` traits.

6.1 ADC Channels
---------------------------------

In order to provide a list of ADC channels to the capsule and userland, the
SAM4L implementation creates an AdcChannel struct which contains and enum
defining its value. Each possible ADC channel is then statically created. Other
chips may want to consider a similar system.

```
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
    ...
    ReferenceGround = 0x17,
}

/// Initialization of an ADC channel.
impl AdcChannel {
    /// Create a new ADC channel.
    /// channel - Channel enum representing the channel number and whether it is
    ///           internal
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
...
pub static mut CHANNEL_REFERENCE_GROUND: AdcChannel = AdcChannel::new(Channel::ReferenceGround);
```

6.2 Client Type
---------------------------------

It is difficult in Rust to require a argument that implements two types.
However, it is convenient for the implementation to expect a single client that
implements both the `adc::Client` and `adc::HighSpeedClient` interfaces. It is
possible to do so by defining a new trait that requires each.

```
/// Create a trait of both client types to allow a single client reference to
/// act as both
pub trait EverythingClient: hil::adc::Client + hil::adc::HighSpeedClient {}
impl<C: hil::adc::Client + hil::adc::HighSpeedClient> EverythingClient for C {}
```

6.3 Clock Initialization
---------------------------------

The ADC clock on the SAM4L is poorly documented. It is required to both
generate a clock based on the PBA clock as well as GCLK10. However, the clock
used for samples by the ADC run at 1.5 MHz at the highest (for single sampling
mode). In order to handle this, the SAM4L ADC implementation first divides down
the clock to reach a value less than or equal to 1.5 MHz (exactly 1.5 MHz in
practice for a CPU clock running at 48 MHz).

6.4 ADC Initialization
---------------------------------

The process of initializing the ADC is well documented in the SAM4L datasheet,
unfortunately it seems to be entirely false. While following the documentation
allows for single sampling, high speed sampling fails in practice after a small
number of samples (order less than 100) have been collected. After much
experimentation and comparison to other SAM4L code available online, it was
determined that the initialization process should be:

1. Enable clock
2. Configure ADC
3. Reset ADC
4. Enable ADC
5. Wait until ADC status is set to enabled
6. Enable the Bandgap and Reference Buffers
7. Wait until the buffers are enabled

It is quite possible that other orders of initialization are valid, however
proceed with caution.


7 Authors' Address
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

8 Citations
========================================

<a name="trd1"/>[TRD1] <a href="trd1-trds.md">Tock Reference Document (TRD) Structure and Keywords</a>
