// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! ADC driver for the nRF52. Uses the SAADC peripheral.

use core::cell::Cell;
use core::cmp;
use core::ptr::addr_of;
use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::StaticRef;
use kernel::utilities::cells::{OptionalCell, TakeCell};
use kernel::utilities::registers::interfaces::{Readable, Writeable};
use kernel::utilities::registers::{
    ReadOnly, ReadWrite, WriteOnly, register_bitfields, register_structs,
};

register_structs! {
    pub AdcRegisters {
        /// Start the ADC and prepare the result buffer in RAM
        (0x000 => tasks_start: WriteOnly<u32, TASK::Register>),
        /// Take one ADC sample, if scan is enabled all channels are sampled
        (0x004 => tasks_sample: WriteOnly<u32, TASK::Register>),
        /// Stop the ADC and terminate any on-going conversion
        (0x008 => tasks_stop: WriteOnly<u32, TASK::Register>),
        /// Starts offset auto-calibration
        (0x00C => tasks_calibrateoffset: WriteOnly<u32, TASK::Register>),
        (0x010 => _reserved0),
        /// The ADC has started
        (0x100 => events_started: ReadWrite<u32, EVENT::Register>),
        /// The ADC has filled up the Result buffer
        (0x104 => events_end: ReadWrite<u32, EVENT::Register>),
        /// A conversion task has been completed. Depending on the mode, multiple conversion
        (0x108 => events_done: ReadWrite<u32, EVENT::Register>),
        /// A result is ready to get transferred to RAM
        (0x10C => events_resultdone: ReadWrite<u32, EVENT::Register>),
        /// Calibration is complete
        (0x110 => events_calibratedone: ReadWrite<u32, EVENT::Register>),
        /// The ADC has stopped
        (0x114 => events_stopped: ReadWrite<u32, EVENT::Register>),
        /// Last result is equal or above `CH[X].LIMIT`
        (0x118 => events_ch: [AdcEventChRegisters; 8]),
        (0x158 => _reserved1),
        /// Enable or disable interrupt
        (0x300 => inten: ReadWrite<u32, INTEN::Register>),
        /// Enable interrupt
        (0x304 => intenset: ReadWrite<u32, INTEN::Register>),
        /// Disable interrupt
        (0x308 => intenclr: ReadWrite<u32, INTEN::Register>),
        (0x30C => _reserved2),
        /// Status
        (0x400 => status: ReadOnly<u32>),
        (0x404 => _reserved3),
        /// Enable or disable ADC
        (0x500 => enable: ReadWrite<u32, ENABLE::Register>),
        (0x504 => _reserved4),
        (0x510 => ch: [AdcChRegisters; 8]),
        (0x590 => _reserved5),
        /// Resolution configuration
        (0x5F0 => resolution: ReadWrite<u32, RESOLUTION::Register>),
        /// Oversampling configuration. OVERSAMPLE should not be combined with SCAN. The RES
        (0x5F4 => oversample: ReadWrite<u32>),
        /// Controls normal or continuous sample rate
        (0x5F8 => samplerate: ReadWrite<u32, SAMPLERATE::Register>),
        (0x5FC => _reserved6),
        /// Pointer to store samples to
        (0x62C => result_ptr: ReadWrite<u32>),
        /// Number of 16 bit samples to save in RAM
        (0x630 => result_maxcnt: ReadWrite<u32, RESULT_MAXCNT::Register>),
        /// Number of 16 bit samples recorded to RAM
        (0x634 => result_amount: ReadWrite<u32, RESULT_AMOUNT::Register>),
        (0x638 => @END),
    }
}

register_structs! {
    AdcEventChRegisters {
        (0x0 => limith: ReadWrite<u32, EVENT::Register>),
        (0x4 => limitl: ReadWrite<u32, EVENT::Register>),
        (0x8 => @END),
    }
}

register_structs! {
    AdcChRegisters {
        (0x0 => pselp: ReadWrite<u32, PSEL::Register>),
        (0x4 => pseln: ReadWrite<u32, PSEL::Register>),
        (0x8 => config: ReadWrite<u32, CONFIG::Register>),
        (0xC => limit: ReadWrite<u32, LIMIT::Register>),
        (0x10 => @END),
    }
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
    registers: StaticRef<AdcRegisters>,
    reference: Cell<usize>,
    mode: Cell<AdcMode>,
    client: OptionalCell<&'a dyn hil::adc::Client>,
    highspeed_client: OptionalCell<&'a dyn hil::adc::HighSpeedClient>,

    buffer: TakeCell<'static, [u16]>,
    length: Cell<usize>,
    next_buffer: TakeCell<'static, [u16]>,
    next_length: Cell<usize>,
}

impl Adc<'_> {
    pub const fn new(registers: StaticRef<AdcRegisters>, voltage_reference_in_mv: usize) -> Self {
        Self {
            registers,
            reference: Cell::new(voltage_reference_in_mv),
            mode: Cell::new(AdcMode::Idle),
            client: OptionalCell::empty(),
            highspeed_client: OptionalCell::empty(),
            buffer: TakeCell::empty(),
            length: Cell::new(0),
            next_buffer: TakeCell::empty(),
            next_length: Cell::new(0),
        }
    }

    // Calibrate and measure the actual VDD of the board.
    pub fn calibrate(&self) {
        self.mode.set(AdcMode::Calibrate);

        // Enable the ADC
        self.registers.enable.write(ENABLE::ENABLE::SET);
        self.registers.inten.write(INTEN::CALIBRATEDONE::SET);
        self.registers.tasks_calibrateoffset.write(TASK::TASK::SET);
    }

    pub fn handle_interrupt(&self) {
        match self.mode.get() {
            AdcMode::Calibrate => {
                if self.registers.events_calibratedone.is_set(EVENT::EVENT) {
                    self.registers
                        .events_calibratedone
                        .write(EVENT::EVENT::CLEAR);

                    // After calibration, read VDD to set our voltage reference.
                    self.registers.ch[0].pselp.write(PSEL::PSEL::VDD);
                    self.registers.ch[0].pseln.write(PSEL::PSEL::NotConnected);

                    // Configure the ADC for a single read.
                    self.registers.ch[0].config.write(
                        CONFIG::GAIN::Gain1_6
                            + CONFIG::REFSEL::Internal
                            + CONFIG::TACQ::us10
                            + CONFIG::RESP::Bypass
                            + CONFIG::RESN::Bypass
                            + CONFIG::MODE::SE,
                    );

                    self.setup_resolution();
                    self.setup_sample_count(1);

                    // Where to put the reading.
                    let sample: *const [u16; 1] = addr_of!(SAMPLE);
                    let sample: *const u16 = sample.cast();
                    self.registers.result_ptr.set(sample as u32);

                    // No automatic sampling, will trigger manually.
                    self.registers.samplerate.write(SAMPLERATE::MODE::Task);

                    // Enable the ADC
                    self.registers.enable.write(ENABLE::ENABLE::SET);

                    // Enable started, sample end, and stopped interrupts.
                    self.registers
                        .inten
                        .write(INTEN::STARTED::SET + INTEN::END::SET + INTEN::STOPPED::SET);

                    self.registers.tasks_start.write(TASK::TASK::SET);

                    // self.registers.enable.write(ENABLE::ENABLE::CLEAR);
                } else if self.registers.events_started.is_set(EVENT::EVENT) {
                    self.registers.events_started.write(EVENT::EVENT::CLEAR);
                    // ADC has started, now issue the sample.
                    self.registers.tasks_sample.write(TASK::TASK::SET);
                } else if self.registers.events_end.is_set(EVENT::EVENT) {
                    self.registers.events_end.write(EVENT::EVENT::CLEAR);
                    // Reading finished. Turn off the ADC.
                    self.registers.tasks_stop.write(TASK::TASK::SET);
                } else if self.registers.events_stopped.is_set(EVENT::EVENT) {
                    self.registers.events_stopped.write(EVENT::EVENT::CLEAR);
                    // ADC is stopped. Disable and return value.
                    self.registers.enable.write(ENABLE::ENABLE::CLEAR);

                    let reading = unsafe { SAMPLE[0] as i16 } as usize;

                    // reading = val * (gain/ref) * 2^12
                    //         = val * ((1/6)/0.6 V) * 2^12
                    //         = val * 1/3600 mV * 2^12
                    // val = (reading * 3600 mV) / 2^12
                    let val = (reading * 3600) / (1 << 12);

                    // If the reading looks like it exists in a reasonable range
                    // than save this as the reference.
                    if val > 1000 && val < 5100 {
                        self.reference.set(val);
                    }
                }
            }

            AdcMode::Single => {
                // Determine what event occurred.
                if self.registers.events_calibratedone.is_set(EVENT::EVENT) {
                    self.registers
                        .events_calibratedone
                        .write(EVENT::EVENT::CLEAR);
                    self.registers.enable.write(ENABLE::ENABLE::CLEAR);
                } else if self.registers.events_started.is_set(EVENT::EVENT) {
                    self.registers.events_started.write(EVENT::EVENT::CLEAR);
                    // ADC has started, now issue the sample.
                    self.registers.tasks_sample.write(TASK::TASK::SET);
                } else if self.registers.events_end.is_set(EVENT::EVENT) {
                    self.registers.events_end.write(EVENT::EVENT::CLEAR);
                    // Reading finished. Turn off the ADC.
                    self.registers.tasks_stop.write(TASK::TASK::SET);
                } else if self.registers.events_stopped.is_set(EVENT::EVENT) {
                    self.registers.events_stopped.write(EVENT::EVENT::CLEAR);
                    // ADC is stopped. Disable and return value.
                    self.registers.enable.write(ENABLE::ENABLE::CLEAR);

                    let val = unsafe { SAMPLE[0] as i16 };
                    self.client.map(|client| {
                        // shift left to meet the ADC HIL requirement
                        client.sample_ready(if val < 0 { 0 } else { val << 4 } as u16);
                    });
                }
            }

            AdcMode::HighSpeed => {
                if self.registers.events_started.is_set(EVENT::EVENT) {
                    self.registers.events_started.write(EVENT::EVENT::CLEAR);

                    // According to PS1.7 Section 6.23.4, we can set the new
                    // buffer address after we get the start event.
                    self.next_buffer.map(|buf| {
                        // First determine the buffer's length in samples.
                        let dma_len = cmp::min(buf.len(), self.next_length.get());
                        if dma_len > 0 {
                            self.registers.result_ptr.set(buf.as_ptr() as u32);
                        }
                    });

                    // Trigger sample task to start taking samples.
                    self.registers.tasks_sample.write(TASK::TASK::SET);
                } else if self.registers.events_end.is_set(EVENT::EVENT) {
                    self.registers.events_end.write(EVENT::EVENT::CLEAR);

                    let ret_buf = self.buffer.take().unwrap();

                    // Left shift all samples to the MSB. This handles
                    // differences in resolution between ADC chips and meets the
                    // ADC HIL requirement.
                    let length = self.length.get();
                    for i in 0..length {
                        ret_buf[i] <<= 4;
                    }

                    self.highspeed_client.map(|client| {
                        client.samples_ready(ret_buf, length);
                    });

                    // Optionally setup to continue reading. We already
                    // configured the address if valid.
                    let length2 = self.next_length.get();
                    if length2 > 0 {
                        self.length.set(length2);
                        self.buffer.put(self.next_buffer.take());
                        self.registers
                            .result_maxcnt
                            .write(RESULT_MAXCNT::MAXCNT.val(length2 as u32));
                        kernel::debug!("len2 {}", length2);

                        // self.registers.tasks_sample.write(TASK::TASK::SET);
                        self.registers.tasks_start.write(TASK::TASK::SET);
                    }
                } else if self.registers.events_stopped.is_set(EVENT::EVENT) {
                    self.registers.events_stopped.write(EVENT::EVENT::CLEAR);
                }
            }

            AdcMode::Idle => {}
        }
    }

    fn setup_channel(&self, channel: &AdcChannelSetup) {
        // Positive goes to the channel passed in, negative not connected.
        self.registers.ch[0]
            .pselp
            .write(PSEL::PSEL.val(channel.channel as u32));
        self.registers.ch[0].pseln.write(PSEL::PSEL::NotConnected);

        // Configure the ADC for a single read.
        self.registers.ch[0].config.write(
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
        self.registers.resolution.write(RESOLUTION::VAL::bit12);
    }

    fn setup_sample_count(&self, count: usize) {
        self.registers
            .result_maxcnt
            .write(RESULT_MAXCNT::MAXCNT.val(count as u32));
    }

    fn setup_frequency(&self, frequency: u32) {
        let raw_cc = 16000000 / frequency;
        let cc = raw_cc.clamp(80, 2047);

        self.registers
            .samplerate
            .write(SAMPLERATE::MODE::Timers + SAMPLERATE::CC.val(cc));
    }
}

/// Implements an ADC capable reading ADC samples on any channel.
impl<'a> hil::adc::Adc<'a> for Adc<'a> {
    type Channel = AdcChannelSetup;

    fn sample(&self, channel: &Self::Channel) -> Result<(), ErrorCode> {
        self.setup_channel(channel);
        self.setup_resolution();

        // Do one measurement.
        self.registers
            .result_maxcnt
            .write(RESULT_MAXCNT::MAXCNT.val(1));
        // Where to put the reading.
        let sample: *const [u16; 1] = addr_of!(SAMPLE);
        let sample: *const u16 = sample.cast();
        self.registers.result_ptr.set(sample as u32);

        // No automatic sampling, will trigger manually.
        self.registers.samplerate.write(SAMPLERATE::MODE::Task);

        // Enable the ADC
        self.registers.enable.write(ENABLE::ENABLE::SET);

        // Enable started, sample end, and stopped interrupts.
        self.registers
            .inten
            .write(INTEN::STARTED::SET + INTEN::END::SET + INTEN::STOPPED::SET);

        self.mode.set(AdcMode::Single);

        // Start the SAADC and wait for the started interrupt.
        self.registers.tasks_start.write(TASK::TASK::SET);

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
        self.registers.tasks_stop.write(TASK::TASK::SET);
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
            // Store the second buffer for later use
            self.next_buffer.replace(buffer2);
            self.next_length.set(length2);

            self.setup_channel(channel);
            self.setup_resolution();

            // Use EasyDMA to save the samples to our buffer.
            self.registers.result_ptr.set(buffer1.as_ptr() as u32);

            // Also need to save these to return to the caller.
            self.buffer.replace(buffer1);
            self.length.set(length1);

            // Number of measurements.
            self.setup_sample_count(length1);

            // Set the frequency best we can.
            self.setup_frequency(frequency);

            // Enable the ADC
            self.registers.enable.write(ENABLE::ENABLE::SET);

            // Enable started, sample end, and stopped interrupts.
            self.registers
                .inten
                .write(INTEN::STARTED::SET + INTEN::END::SET + INTEN::STOPPED::SET);

            self.mode.set(AdcMode::HighSpeed);

            // Start the SAADC and wait for the started interrupt.
            self.registers.tasks_start.write(TASK::TASK::SET);

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
        Ok((self.buffer.take(), self.next_buffer.take()))
    }

    fn set_highspeed_client(&self, client: &'a dyn hil::adc::HighSpeedClient) {
        self.highspeed_client.set(client);
    }
}
