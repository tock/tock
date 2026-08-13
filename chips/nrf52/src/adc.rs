// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! ADC driver for the nRF52. Uses the SAADC peripheral.

use core::cell::Cell;
use core::cmp;
use core::ptr::addr_of_mut;
use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{MapCell, OptionalCell, TakeCell};
use kernel::utilities::dma_slice::DmaSliceMut;
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_bitfields};

#[repr(C)]
struct AdcRegisters {
    /// Start the ADC and prepare the result buffer in RAM
    tasks_start: WriteOnly<u32, TASK::Register>,
    /// Take one ADC sample, if scan is enabled all channels are sampled
    tasks_sample: WriteOnly<u32, TASK::Register>,
    /// Stop the ADC and terminate any on-going conversion
    tasks_stop: WriteOnly<u32, TASK::Register>,
    /// Starts offset auto-calibration
    tasks_calibrateoffset: WriteOnly<u32, TASK::Register>,
    _reserved0: [u8; 240],
    /// The ADC has started
    events_started: ReadWrite<u32, EVENT::Register>,
    /// The ADC has filled up the Result buffer
    events_end: ReadWrite<u32, EVENT::Register>,
    /// A conversion task has been completed. Depending on the mode, multiple conversion
    events_done: ReadWrite<u32, EVENT::Register>,
    /// A result is ready to get transferred to RAM
    events_resultdone: ReadWrite<u32, EVENT::Register>,
    /// Calibration is complete
    events_calibratedone: ReadWrite<u32, EVENT::Register>,
    /// The ADC has stopped
    events_stopped: ReadWrite<u32, EVENT::Register>,
    /// Last result is equal or above `CH[X].LIMIT`
    events_ch: [AdcEventChRegisters; 8],
    _reserved1: [u8; 424],
    /// Enable or disable interrupt
    inten: ReadWrite<u32, INTEN::Register>,
    /// Enable interrupt
    intenset: ReadWrite<u32, INTEN::Register>,
    /// Disable interrupt
    intenclr: ReadWrite<u32, INTEN::Register>,
    _reserved2: [u8; 244],
    /// Status
    status: ReadOnly<u32>,
    _reserved3: [u8; 252],
    /// Enable or disable ADC
    enable: ReadWrite<u32, ENABLE::Register>,
    _reserved4: [u8; 12],
    ch: [AdcChRegisters; 8],
    _reserved5: [u8; 96],
    /// Resolution configuration
    resolution: ReadWrite<u32, RESOLUTION::Register>,
    /// Oversampling configuration. OVERSAMPLE should not be combined with SCAN. The RES
    oversample: ReadWrite<u32>,
    /// Controls normal or continuous sample rate
    samplerate: ReadWrite<u32, SAMPLERATE::Register>,
    _reserved6: [u8; 48],
    /// Pointer to store samples to
    result_ptr: ReadWrite<u32>,
    /// Number of 16 bit samples to save in RAM
    result_maxcnt: ReadWrite<u32, RESULT_MAXCNT::Register>,
    /// Number of 16 bit samples recorded to RAM
    result_amount: ReadWrite<u32, RESULT_AMOUNT::Register>,
}

#[repr(C)]
struct AdcEventChRegisters {
    limith: ReadWrite<u32, EVENT::Register>,
    limitl: ReadWrite<u32, EVENT::Register>,
}

#[repr(C)]
struct AdcChRegisters {
    pselp: ReadWrite<u32, PSEL::Register>,
    pseln: ReadWrite<u32, PSEL::Register>,
    config: ReadWrite<u32, CONFIG::Register>,
    limit: ReadWrite<u32, LIMIT::Register>,
}

register_bitfields![u32,
    INTEN [
        /// Enable or disable interrupt on EVENTS_STARTED event
        STARTED 0,
        /// Enable or disable interrupt on EVENTS_END event
        END 1,
        /// Enable or disable interrupt on EVENTS_DONE event
        DONE 2,
        /// Enable or disable interrupt on EVENTS_RESULTDONE event
        RESULTDONE 3,
        /// Enable or disable interrupt on EVENTS_CALIBRATEDONE event
        CALIBRATEDONE 4,
        /// Enable or disable interrupt on EVENTS_STOPPED event
        STOPPED 5,
        /// Enable or disable interrupt on EVENTS_CH[0].LIMITH event
        CH0LIMITH 6,
        /// Enable or disable interrupt on EVENTS_CH[0].LIMITL event
        CH0LIMITL 7,
        /// Enable or disable interrupt on EVENTS_CH[1].LIMITH event
        CH1LIMITH 8,
        /// Enable or disable interrupt on EVENTS_CH[1].LIMITL event
        CH1LIMITL 9,
        /// Enable or disable interrupt on EVENTS_CH[2].LIMITH event
        CH2LIMITH 10,
        /// Enable or disable interrupt on EVENTS_CH[2].LIMITL event
        CH2LIMITL 11,
        /// Enable or disable interrupt on EVENTS_CH[3].LIMITH event
        CH3LIMITH 12,
        /// Enable or disable interrupt on EVENTS_CH[3].LIMITL event
        CH3LIMITL 13,
        /// Enable or disable interrupt on EVENTS_CH[4].LIMITH event
        CH4LIMITH 14,
        /// Enable or disable interrupt on EVENTS_CH[4].LIMITL event
        CH4LIMITL 15,
        /// Enable or disable interrupt on EVENTS_CH[5].LIMITH event
        CH5LIMITH 16,
        /// Enable or disable interrupt on EVENTS_CH[5].LIMITL event
        CH5LIMITL 17,
        /// Enable or disable interrupt on EVENTS_CH[6].LIMITH event
        CH6LIMITH 18,
        /// Enable or disable interrupt on EVENTS_CH[6].LIMITL event
        CH6LIMITL 19,
        /// Enable or disable interrupt on EVENTS_CH[7].LIMITH event
        CH7LIMITH 20,
        /// Enable or disable interrupt on EVENTS_CH[7].LIMITL event
        CH7LIMITL 21
    ],
    ENABLE [
        ENABLE 0
    ],
    SAMPLERATE [
        /// Capture and compare value. Sample rate is 16 MHz/CC
        CC OFFSET(0) NUMBITS(11) [],
        /// Select mode for sample rate control
        MODE OFFSET(12) NUMBITS(1) [
            /// Rate is controlled from SAMPLE task
            Task = 0,
            /// Rate is controlled from local timer (use CC to control the rate)
            Timers = 1
        ]
    ],
    EVENT [
        EVENT 0
    ],
    TASK [
        TASK 0
    ],
    PSEL [
        PSEL OFFSET(0) NUMBITS(5) [
            NotConnected = 0,
            AnalogInput0 = 1,
            AnalogInput1 = 2,
            AnalogInput2 = 3,
            AnalogInput3 = 4,
            AnalogInput4 = 5,
            AnalogInput5 = 6,
            AnalogInput6 = 7,
            AnalogInput7 = 8,
            VDD = 9,
            VDDHDIV5 = 0xD
        ]
    ],
    CONFIG [
        RESP OFFSET(0) NUMBITS(2) [
            Bypass = 0,
            Pulldown = 1,
            Pullup = 2,
            VDD1_2 = 3
        ],
        RESN OFFSET(4) NUMBITS(2) [
            Bypass = 0,
            Pulldown = 1,
            Pullup = 2,
            VDD1_2 = 3
        ],
        GAIN OFFSET(8) NUMBITS(3) [
            Gain1_6 = 0,
            Gain1_5 = 1,
            Gain1_4 = 2,
            Gain1_3 = 3,
            Gain1_2 = 4,
            Gain1 = 5,
            Gain2 = 6,
            Gain4 = 7
        ],
        REFSEL OFFSET(12) NUMBITS(1) [
            Internal = 0,
            VDD1_4 = 1
        ],
        TACQ OFFSET(16) NUMBITS(3) [
            us3 = 0,
            us5 = 1,
            us10 = 2,
            us15 = 3,
            us20 = 4,
            us40 = 5
        ],
        MODE OFFSET(20) NUMBITS(1) [
            SE = 0,
            Diff = 1
        ],
        BURST OFFSET(24) NUMBITS(1) [
            Disable = 0,
            Enable = 1
        ]
    ],
    LIMIT [
        LOW OFFSET(0) NUMBITS(16) [],
        HIGH OFFSET(16) NUMBITS(16) []
    ],
    RESOLUTION [
        VAL OFFSET(0) NUMBITS(3) [
            bit8 = 0,
            bit10 = 1,
            bit12 = 2,
            bit14 = 3
        ]
    ],
    RESULT_MAXCNT [
        MAXCNT OFFSET(0) NUMBITS(16) []
    ],
    RESULT_AMOUNT [
        AMOUNT OFFSET(0) NUMBITS(16) []
    ]
];

/// Wrapper for managing MMIO for the ADC's EasyDMA result buffer.
///
/// This type encapsulates every access to the DMA-related `RESULT.PTR` and
/// `RESULT.MAXCNT` registers, and to the `TASKS_SAMPLE` and `TASKS_STOP`
/// tasks. A buffer is only ever handed to the hardware (by writing its
/// address to `RESULT.PTR`) while it is held here, in `dma_buf1` or
/// `dma_buf2`, which guarantees the buffer cannot be accessed from Rust while
/// EasyDMA may be writing to it.
///
/// High-speed sampling double-buffers: `dma_buf1` holds the buffer actively
/// being filled (matched to the most recent `TASKS_START`), while `dma_buf2`
/// holds a second buffer already queued to replace it. Per the nRF52 product
/// specification, `RESULT.PTR` may be safely repointed at the next buffer as
/// soon as `EVENTS_STARTED` fires for the active buffer, without disturbing
/// its in-progress transfer; `dma_buf2` exists to hold that next buffer
/// between being queued and being promoted to active.
struct AdcRegistersManager {
    /// MMIO registers for the ADC peripheral.
    registers: StaticRef<AdcRegisters>,
    /// The buffer (and requested sample count) currently targeted by
    /// `RESULT.PTR`/`RESULT.MAXCNT` that EasyDMA is actively filling.
    dma_buf1: MapCell<(DmaSliceMut<'static, u16>, usize)>,
    /// The buffer (and requested sample count) queued to become active the
    /// next time sampling (re)starts.
    dma_buf2: MapCell<(DmaSliceMut<'static, u16>, usize)>,
}

impl AdcRegistersManager {
    fn new_saadc() -> Self {
        Self {
            registers: SAADC_BASE,
            dma_buf1: MapCell::empty(),
            dma_buf2: MapCell::empty(),
        }
    }

    /// Point `RESULT.PTR`/`RESULT.MAXCNT` at `buf`, to sample `count` values
    /// into it, and hold on to it until `finish_buffer()` is called.
    fn start_buffer(&self, buf: &'static mut [u16], count: usize) {
        // To create a DmaFence we must trust the implementation.
        //
        // SAFETY: The architecture-provided version is correct for the nRF52.
        let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

        // Create a DmaSlice for the result buffer. This ensures that we can
        // soundly share it with the DMA hardware.
        let dma_slice = DmaSliceMut::new_static(buf, fence);

        self.registers.result_ptr.set(dma_slice.ptr_addr() as u32);
        self.registers
            .result_maxcnt
            .write(RESULT_MAXCNT::MAXCNT.val(count as u32));

        self.dma_buf1.replace((dma_slice, count));
    }

    /// Point `RESULT.PTR` at `buf`, queuing it to become the active buffer
    /// once `promote_queued_buffer()` is called. `RESULT.MAXCNT` is left
    /// alone; it is only written once the buffer is promoted.
    fn queue_buffer(&self, buf: &'static mut [u16], count: usize) {
        let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };
        let dma_slice = DmaSliceMut::new_static(buf, fence);

        // The underlying ADC hardware is double buffered, so we can set this in
        // the DMA hardware once we have started the previous DMA use.
        self.registers.result_ptr.set(dma_slice.ptr_addr() as u32);

        self.dma_buf2.replace((dma_slice, count));
    }

    /// Promote the buffer queued with `queue_buffer()`, if any, to be the
    /// active buffer, writing `RESULT.MAXCNT` for it. The caller is
    /// responsible for re-triggering `TASKS_START`. Returns `true` if a
    /// queued buffer was promoted.
    fn promote_queued_buffer(&self) -> bool {
        match self.dma_buf2.take() {
            Some((dma_slice, count)) => {
                self.registers
                    .result_maxcnt
                    .write(RESULT_MAXCNT::MAXCNT.val(count as u32));
                self.dma_buf1.replace((dma_slice, count));
                true
            }
            None => false,
        }
    }

    /// Trigger a single sample conversion.
    fn start_sample(&self) {
        self.registers.tasks_sample.write(TASK::TASK::SET);
    }

    /// Stop any in-progress sample conversion.
    fn stop_sample(&self) {
        self.registers.tasks_stop.write(TASK::TASK::SET);
    }

    /// Reclaim the active buffer and the sample count it was started with.
    ///
    /// Callers must only invoke this once they have observed, via
    /// `EVENTS_END`, that EasyDMA is done writing to the buffer.
    fn finish_buffer(&self) -> Option<(&'static mut [u16], usize)> {
        Self::take_and_fence(&self.dma_buf1)
    }

    /// Reclaim the queued buffer (and count), if any, without it ever having
    /// become active. Used to recover a buffer that was queued but never
    /// promoted, e.g. when sampling is stopped.
    fn finish_queued_buffer(&self) -> Option<(&'static mut [u16], usize)> {
        Self::take_and_fence(&self.dma_buf2)
    }

    fn take_and_fence(
        cell: &MapCell<(DmaSliceMut<'static, u16>, usize)>,
    ) -> Option<(&'static mut [u16], usize)> {
        cell.take().map(|(dma_slice, count)| {
            // To create a DmaFence we must trust the implementation.
            //
            // # Safety
            //
            // The architecture-provided version is correct for the nRF52.
            let fence = unsafe { cortexm4f::dma_fence::CortexMDmaFence::new() };

            // # Safety
            //
            // We only reclaim a buffer after observing, through `EVENTS_END`
            // (or after sampling has stopped), that EasyDMA will not write to
            // it further.
            (unsafe { dma_slice.take(fence) }, count)
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdcChannel {
    AnalogInput0 = 1,
    AnalogInput1 = 2,
    AnalogInput2 = 3,
    AnalogInput3 = 4,
    AnalogInput4 = 5,
    AnalogInput5 = 6,
    AnalogInput6 = 7,
    AnalogInput7 = 8,
    VDD = 9,
    VDDHDIV5 = 0xD,
}

const SAADC_BASE: StaticRef<AdcRegisters> =
    unsafe { StaticRef::new(0x40007000 as *const AdcRegisters) };

// Buffer to save completed sample to.
static mut SAMPLE: [u16; 1] = [0; 1];

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum AdcChannelGain {
    Gain1_6 = 0,
    Gain1_5 = 1,
    Gain1_4 = 2,
    Gain1_3 = 3,
    Gain1_2 = 4,
    Gain1 = 5,
    Gain2 = 6,
    Gain4 = 7,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum AdcChannelResistor {
    Bypass = 0,
    Pulldown = 1,
    Pullup = 2,
    VDD1_2 = 3,
}

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum AdcChannelSamplingTime {
    us3 = 0,
    us5 = 1,
    us10 = 2,
    us15 = 3,
    us20 = 4,
    us40 = 5,
}

#[derive(Copy, Clone, Debug)]
pub struct AdcChannelSetup {
    channel: AdcChannel,
    gain: AdcChannelGain,
    resp: AdcChannelResistor,
    resn: AdcChannelResistor,
    sampling_time: AdcChannelSamplingTime,
}

impl PartialEq for AdcChannelSetup {
    fn eq(&self, other: &Self) -> bool {
        self.channel == other.channel
    }
}

impl AdcChannelSetup {
    pub fn new(channel: AdcChannel) -> AdcChannelSetup {
        AdcChannelSetup {
            channel,
            gain: AdcChannelGain::Gain1_4,
            resp: AdcChannelResistor::Bypass,
            resn: AdcChannelResistor::Pulldown,
            sampling_time: AdcChannelSamplingTime::us10,
        }
    }

    pub fn setup(
        channel: AdcChannel,
        gain: AdcChannelGain,
        resp: AdcChannelResistor,
        resn: AdcChannelResistor,
        sampling_time: AdcChannelSamplingTime,
    ) -> AdcChannelSetup {
        AdcChannelSetup {
            channel,
            gain,
            resp,
            resn,
            sampling_time,
        }
    }
}

#[derive(Clone, Copy)]
enum AdcMode {
    Idle,
    Calibrate,
    Single,
    HighSpeed,
}

pub struct Adc<'a> {
    registers: AdcRegistersManager,
    reference: Cell<usize>,
    mode: Cell<AdcMode>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
    highspeed_client: OptionalCell<&'a dyn hil::adc::HighSpeedClient>,

    /// Scratch buffer used for both offset calibration and single
    /// (non-high-speed) samples, which only ever need to hold one `u16`.
    single_sample_buffer: TakeCell<'static, [u16]>,

    /// The second buffer provided for high-speed sampling, before its
    /// address has been handed to `registers` (which happens at the
    /// following `EVENTS_STARTED`, via `queue_buffer`).
    next_buffer: TakeCell<'static, [u16]>,
    next_length: Cell<usize>,
}

impl Adc<'_> {
    pub fn new(voltage_reference_in_mv: usize) -> Self {
        Self {
            registers: AdcRegistersManager::new_saadc(),
            reference: Cell::new(voltage_reference_in_mv),
            mode: Cell::new(AdcMode::Idle),
            client: OptionalCell::empty(),
            highspeed_client: OptionalCell::empty(),
            // Safety: `SAMPLE` is only ever accessed through this reference,
            // taken once here for the lifetime of the `Adc` instance.
            single_sample_buffer: TakeCell::new(unsafe { &mut *addr_of_mut!(SAMPLE) }),
            next_buffer: TakeCell::empty(),
            next_length: Cell::new(0),
        }
    }

    // Calibrate and measure the actual VDD of the board.
    pub fn calibrate(&self) {
        self.mode.set(AdcMode::Calibrate);

        // Enable the ADC
        self.registers.registers.enable.write(ENABLE::ENABLE::SET);
        self.registers
            .registers
            .inten
            .write(INTEN::CALIBRATEDONE::SET);
        self.registers
            .registers
            .tasks_calibrateoffset
            .write(TASK::TASK::SET);
    }

    pub fn handle_interrupt(&self) {
        match self.mode.get() {
            AdcMode::Calibrate => {
                if self
                    .registers
                    .registers
                    .events_calibratedone
                    .is_set(EVENT::EVENT)
                {
                    self.registers
                        .registers
                        .events_calibratedone
                        .write(EVENT::EVENT::CLEAR);

                    // After calibration, read VDD to set our voltage reference.
                    self.registers.registers.ch[0].pselp.write(PSEL::PSEL::VDD);
                    self.registers.registers.ch[0]
                        .pseln
                        .write(PSEL::PSEL::NotConnected);

                    // Configure the ADC for a single read.
                    self.registers.registers.ch[0].config.write(
                        CONFIG::GAIN::Gain1_6
                            + CONFIG::REFSEL::Internal
                            + CONFIG::TACQ::us10
                            + CONFIG::RESP::Bypass
                            + CONFIG::RESN::Bypass
                            + CONFIG::MODE::SE,
                    );

                    self.setup_resolution();

                    // Where to put the reading.
                    if let Some(buf) = self.single_sample_buffer.take() {
                        self.registers.start_buffer(buf, 1);
                    }

                    // No automatic sampling, will trigger manually.
                    self.registers
                        .registers
                        .samplerate
                        .write(SAMPLERATE::MODE::Task);

                    // Enable the ADC
                    self.registers.registers.enable.write(ENABLE::ENABLE::SET);

                    // Enable started, sample end, and stopped interrupts.
                    self.registers
                        .registers
                        .inten
                        .write(INTEN::STARTED::SET + INTEN::END::SET + INTEN::STOPPED::SET);

                    self.registers.registers.tasks_start.write(TASK::TASK::SET);
                } else if self.registers.registers.events_started.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_started
                        .write(EVENT::EVENT::CLEAR);
                    // ADC has started, now issue the sample.
                    self.registers.start_sample();
                } else if self.registers.registers.events_end.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_end
                        .write(EVENT::EVENT::CLEAR);

                    // Reading finished; EasyDMA is done writing the buffer.
                    if let Some((buf, _count)) = self.registers.finish_buffer() {
                        let reading = buf[0] as i16 as usize;

                        // reading = val * (gain/ref) * 2^12
                        //         = val * ((1/6)/0.6 V) * 2^12
                        //         = val * 1/3600 mV * 2^12
                        // val = (reading * 3600 mV) / 2^12
                        let val = (reading * 3600) / (1 << 12);

                        // If the reading looks like it exists in a reasonable
                        // range than save this as the reference.
                        if val > 1000 && val < 5100 {
                            self.reference.set(val);
                        }

                        self.single_sample_buffer.replace(buf);
                    }

                    // Turn off the ADC.
                    self.registers.stop_sample();
                } else if self.registers.registers.events_stopped.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_stopped
                        .write(EVENT::EVENT::CLEAR);
                    // ADC is stopped. Disable it.
                    self.registers.registers.enable.write(ENABLE::ENABLE::CLEAR);
                }
            }

            AdcMode::Single => {
                // Determine what event occurred.
                if self
                    .registers
                    .registers
                    .events_calibratedone
                    .is_set(EVENT::EVENT)
                {
                    self.registers
                        .registers
                        .events_calibratedone
                        .write(EVENT::EVENT::CLEAR);
                    self.registers.registers.enable.write(ENABLE::ENABLE::CLEAR);
                } else if self.registers.registers.events_started.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_started
                        .write(EVENT::EVENT::CLEAR);
                    // ADC has started, now issue the sample.
                    self.registers.start_sample();
                } else if self.registers.registers.events_end.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_end
                        .write(EVENT::EVENT::CLEAR);

                    // Reading finished; EasyDMA is done writing the buffer.
                    if let Some((buf, _count)) = self.registers.finish_buffer() {
                        let val = buf[0] as i16;

                        self.single_sample_buffer.replace(buf);

                        self.client.map(|client| {
                            // shift left to meet the ADC HIL requirement
                            client.sample_ready(if val < 0 { 0 } else { val << 4 } as u16);
                        });
                    }

                    // Turn off the ADC.
                    self.registers.stop_sample();
                } else if self.registers.registers.events_stopped.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_stopped
                        .write(EVENT::EVENT::CLEAR);
                    // ADC is stopped. Disable it.
                    self.registers.registers.enable.write(ENABLE::ENABLE::CLEAR);
                }
            }

            AdcMode::HighSpeed => {
                if self.registers.registers.events_started.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_started
                        .write(EVENT::EVENT::CLEAR);

                    // According to PS1.7 Section 6.23.4, we can set the new
                    // buffer address after we get the start event, without
                    // disturbing the transfer already in progress.
                    if let Some(buf) = self.next_buffer.take() {
                        let length2 = self.next_length.get();
                        let dma_len = cmp::min(buf.len(), length2);
                        if dma_len > 0 {
                            self.registers.queue_buffer(buf, length2);
                        } else {
                            // Nothing to sample into; keep it for later.
                            self.next_buffer.replace(buf);
                        }
                    }

                    // Trigger sample task to start taking samples.
                    self.registers.start_sample();
                } else if self.registers.registers.events_end.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_end
                        .write(EVENT::EVENT::CLEAR);

                    let (ret_buf, length) = self.registers.finish_buffer().unwrap();

                    // Left shift all samples to the MSB. This handles
                    // differences in resolution between ADC chips and meets the
                    // ADC HIL requirement.
                    for i in 0..length {
                        ret_buf[i] <<= 4;
                    }

                    self.highspeed_client.map(|client| {
                        client.samples_ready(ret_buf, length);
                    });

                    // If a buffer was queued (above, at the last
                    // EVENTS_STARTED), promote it to active and resume
                    // sampling.
                    if self.registers.promote_queued_buffer() {
                        self.registers.registers.tasks_start.write(TASK::TASK::SET);
                    }
                } else if self.registers.registers.events_stopped.is_set(EVENT::EVENT) {
                    self.registers
                        .registers
                        .events_stopped
                        .write(EVENT::EVENT::CLEAR);
                }
            }

            AdcMode::Idle => {}
        }
    }

    fn setup_channel(&self, channel: &AdcChannelSetup) {
        // Positive goes to the channel passed in, negative not connected.
        self.registers.registers.ch[0]
            .pselp
            .write(PSEL::PSEL.val(channel.channel as u32));
        self.registers.registers.ch[0]
            .pseln
            .write(PSEL::PSEL::NotConnected);

        // Configure the ADC for a single read.
        self.registers.registers.ch[0].config.write(
            CONFIG::GAIN.val(channel.gain as u32)
                + CONFIG::REFSEL::VDD1_4
                + CONFIG::TACQ.val(channel.sampling_time as u32)
                + CONFIG::RESP.val(channel.resp as u32)
                + CONFIG::RESN.val(channel.resn as u32)
                + CONFIG::MODE::SE,
        );
    }

    fn setup_resolution(&self) {
        // Set max resolution (with oversampling).
        self.registers
            .registers
            .resolution
            .write(RESOLUTION::VAL::bit12);
    }

    fn setup_frequency(&self, frequency: u32) {
        let raw_cc = 16000000 / frequency;
        let cc = raw_cc.clamp(80, 2047);

        self.registers
            .registers
            .samplerate
            .write(SAMPLERATE::MODE::Timers + SAMPLERATE::CC.val(cc));
    }
}

/// Implements an ADC capable reading ADC samples on any channel.
impl<'a> hil::adc::Adc<'a> for Adc<'a> {
    type Channel = AdcChannelSetup;

    fn sample(&self, channel: &Self::Channel) -> Result<(), ErrorCode> {
        let buf = self.single_sample_buffer.take().ok_or(ErrorCode::BUSY)?;

        self.setup_channel(channel);
        self.setup_resolution();

        // Do one measurement.
        self.registers.start_buffer(buf, 1);

        // No automatic sampling, will trigger manually.
        self.registers
            .registers
            .samplerate
            .write(SAMPLERATE::MODE::Task);

        // Enable the ADC
        self.registers.registers.enable.write(ENABLE::ENABLE::SET);

        // Enable started, sample end, and stopped interrupts.
        self.registers
            .registers
            .inten
            .write(INTEN::STARTED::SET + INTEN::END::SET + INTEN::STOPPED::SET);

        self.mode.set(AdcMode::Single);

        // Start the SAADC and wait for the started interrupt.
        self.registers.registers.tasks_start.write(TASK::TASK::SET);

        Ok(())
    }

    fn sample_continuous(
        &self,
        _channel: &Self::Channel,
        _frequency: u32,
    ) -> Result<(), ErrorCode> {
        Err(ErrorCode::FAIL)
    }

    fn stop_sampling(&self) -> Result<(), ErrorCode> {
        self.registers.stop_sample();
        Ok(())
    }

    fn get_resolution_bits(&self) -> usize {
        12
    }

    fn get_voltage_reference_mv(&self) -> Option<usize> {
        Some(self.reference.get())
    }

    fn set_client(&self, client: &'a dyn hil::adc::Client) {
        self.client.set(client);
    }
}

impl<'a> hil::adc::AdcHighSpeed<'a> for Adc<'a> {
    fn sample_highspeed(
        &self,
        channel: &Self::Channel,
        frequency: u32,
        buffer1: &'static mut [u16],
        length1: usize,
        buffer2: &'static mut [u16],
        length2: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u16], &'static mut [u16])> {
        if length1 == 0 {
            // At least need to take one sample.
            Err((ErrorCode::INVAL, buffer1, buffer2))
        } else {
            // Store the second buffer for later use.
            self.next_buffer.replace(buffer2);
            self.next_length.set(length2);

            self.setup_channel(channel);
            self.setup_resolution();

            // Use EasyDMA to save the samples to our buffer.
            self.registers.start_buffer(buffer1, length1);

            // Set the frequency best we can.
            self.setup_frequency(frequency);

            // Enable the ADC
            self.registers.registers.enable.write(ENABLE::ENABLE::SET);

            // Enable started, sample end, and stopped interrupts.
            self.registers
                .registers
                .inten
                .write(INTEN::STARTED::SET + INTEN::END::SET + INTEN::STOPPED::SET);

            self.mode.set(AdcMode::HighSpeed);

            // Start the SAADC and wait for the started interrupt.
            self.registers.registers.tasks_start.write(TASK::TASK::SET);

            Ok(())
        }
    }

    fn provide_buffer(
        &self,
        buf: &'static mut [u16],
        length: usize,
    ) -> Result<(), (ErrorCode, &'static mut [u16])> {
        if self.next_buffer.is_some() {
            // we've already got a second buffer, we don't need a third yet
            Err((ErrorCode::BUSY, buf))
        } else {
            // store the buffer for later use
            self.next_buffer.replace(buf);
            self.next_length.set(length);

            Ok(())
        }
    }

    fn retrieve_buffers(
        &self,
    ) -> Result<(Option<&'static mut [u16]>, Option<&'static mut [u16]>), ErrorCode> {
        let active = self.registers.finish_buffer().map(|(buf, _)| buf);
        let queued = self
            .registers
            .finish_queued_buffer()
            .map(|(buf, _)| buf)
            .or_else(|| self.next_buffer.take());
        Ok((active, queued))
    }

    fn set_highspeed_client(&self, client: &'a dyn hil::adc::HighSpeedClient) {
        self.highspeed_client.set(client);
    }
}
