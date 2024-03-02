// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Pulse wave modulation (PWM) driver for RP2040.
//!
//! # Hardware Interface Layer (HIL)
//!
//! The driver implements both Pwm and PwmPin HILs. The following features are available when using
//! the driver through HIL:
//!
//! + Configurable top and compare values
//! + Independent configuration for each channel and for each output/input pin
//! + Duty cycle from 0% to 100% **inclusive**
//!
//! # Examples
//!
//! The integration tests for Raspberry Pi Pico provide some examples using the driver.
//! See boards/raspberry_pi_pico/src/test/pwm.rs

use kernel::debug;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{
    register_bitfields, register_structs, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::utilities::StaticRef;
use kernel::ErrorCode;

use crate::clocks;
use crate::gpio::RPGpio;

register_bitfields![u32,
    CSR [
        // Enable PWM channel
        EN OFFSET(0) NUMBITS(1) [],
        // Enable phase-correct modulation
        PH_CORRECT OFFSET(1) NUMBITS(1) [],
        // Invert output A
        A_INV OFFSET(2) NUMBITS(1) [],
        // Invert output B
        B_INV OFFSET(3) NUMBITS(1) [],
        // PWM slice event selection for fractional clock divider
        // Default value = FREE_RUNNING (always on)
        // If the event is different from FREE_RUNNING, then pin B becomes
        // an input pin
        DIVMOD OFFSET(4) NUMBITS(2) [
            // Free-running counting at rate dictated by fractional divider
            FREE_RUNNING = 0,
            // Fractional divider operation is gated by the PWM B pin
            B_HIGH = 1,
            // Counter advances with each rising edge of the PWM B pin
            B_RISING = 2,
            // Counter advances with each falling edge of the PWM B pin
            B_FALLING = 3
        ],
        // Retard the phase of the counter by 1 count, while it is running
        // Self-clearing. Write a 1, and poll until low. Counter must be running.
        PH_RET OFFSET(6) NUMBITS(1) [],
        // Advance the phase of the counter by 1 count, while it is running
        // Self clearing. Write a 1, and poll until low. Counter must be running.
        PH_ADV OFFSET(7) NUMBITS(1) []
    ],

    // DIV register
    // INT and FRAC form a fixed-point fractional number.
    // Counting rate is system clock frequency divided by this number.
    // Fractional division uses simple 1st-order sigma-delta.
    DIV [
        FRAC OFFSET(0) NUMBITS(4) [],
        INT OFFSET(4) NUMBITS(8) []
    ],

    // Direct access to the PWM counter
    CTR [
        CTR OFFSET(0) NUMBITS(16) []
    ],

    // Counter compare values
    CC [
        A OFFSET(0) NUMBITS(16) [],
        B OFFSET(16) NUMBITS(16) []
    ],

    // Counter top value
    // When the value of the counter reaches the top value, depending on the
    // ph_correct value, the counter will either:
    // + wrap to 0 if ph_correct == 0
    // + it starts counting downward until it reaches 0 again if ph_correct == 0
    TOP [
        TOP OFFSET(0) NUMBITS(16) []
    ],

    // Control multiple channels at once.
    // Each bit controls one channel.
    CH [
        CH OFFSET(0) NUMBITS(8) [
            CH0 = 1,
            CH1 = 2,
            CH2 = 4,
            CH3 = 8,
            CH4 = 16,
            CH5 = 32,
            CH6 = 64,
            CH7 = 128
        ]
    ]
];

const NUMBER_CHANNELS: usize = 8;

#[repr(C)]
struct Channel {
    // Control and status register
    csr: ReadWrite<u32, CSR::Register>,
    // Division register
    div: ReadWrite<u32, DIV::Register>,
    // Direct access to the PWM counter register
    ctr: ReadWrite<u32, CTR::Register>,
    // Counter compare values register
    cc: ReadWrite<u32, CC::Register>,
    // Counter wrap value register
    top: ReadWrite<u32, TOP::Register>,
}

register_structs! {
    PwmRegisters {
        // Channel registers
        (0x0000 => ch: [Channel; NUMBER_CHANNELS]),
        // Enable register
        // This register aliases the CSR_EN bits for all channels.
        // Writing to this register allows multiple channels to be enabled or disabled
        // or disables simultaneously, so they can run in perfect sync.
        (0x00A0 => en: ReadWrite<u32, CH::Register>),
        // Raw interrupts register
        (0x00A4 => intr: WriteOnly<u32, CH::Register>),
        // Interrupt enable register
        (0x00A8 => inte: ReadWrite<u32, CH::Register>),
        // Interrupt force register
        (0x00AC => intf: ReadWrite<u32, CH::Register>),
        // Interrupt status after masking & forcing
        (0x00B0 => ints: ReadOnly<u32, CH::Register>),
        (0x00B4 => @END),
    }
}

#[derive(Clone, Copy)]
/// Fractional clock divider running mode
///
/// Each channel can be configured to run in four different ways:
///
/// + Free running: The fractional clock divider is always enabled. In this mode,
/// pins A and B are configured as output pins. In other modes, pin B becomes
/// an input pin.
/// + High: The fractional clock divider is enabled when pin B is high.
/// + Rising: The fractional clock divider is enabled when a rising-edge is
/// detected on pin B.
/// + Falling: The fractional clock divider is enabled when a falling-edge
/// is detected on pin B.
pub enum DivMode {
    FreeRunning,
    High,
    Rising,
    Falling,
}

/// Channel identifier
///
/// There are a total of 8 eight PWM channels.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChannelNumber {
    Ch0,
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
}

const CHANNEL_NUMBERS: [ChannelNumber; NUMBER_CHANNELS] = [
    ChannelNumber::Ch0,
    ChannelNumber::Ch1,
    ChannelNumber::Ch2,
    ChannelNumber::Ch3,
    ChannelNumber::Ch4,
    ChannelNumber::Ch5,
    ChannelNumber::Ch6,
    ChannelNumber::Ch7,
];

/// Each GPIO pin can be configured as a PWM pin.
/// The following table shows the mapping between GPIO pins and PWM pins:
///
/// | GPIO  | PWM |
/// | ----- | --- |
/// | 0     | 0A  |
/// | 1     | 0B  |
/// | 2     | 1A  |
/// | 3     | 1B  |
/// | 4     | 2A  |
/// | 5     | 2B  |
/// | 6     | 3A  |
/// | 7     | 3B  |
/// | 8     | 4A  |
/// | 9     | 4B  |
/// | 10    | 5A  |
/// | 11    | 5B  |
/// | 12    | 6A  |
/// | 13    | 6B  |
/// | 14    | 7A  |
/// | 15    | 7B  |
/// | 16    | 0A  |
/// | 17    | 0B  |
/// | 18    | 1A  |
/// | 19    | 1B  |
/// | 20    | 2A  |
/// | 21    | 2B  |
/// | 22    | 3A  |
/// | 23    | 3B  |
/// | 24    | 4A  |
/// | 25    | 4B  |
/// | 26    | 5A  |
/// | 27    | 5B  |
/// | 28    | 6A  |
/// | 29    | 6B  |
///
/// **Note**:
///
/// + The same PWM output can be selected on two GPIO pins. The same signal will appear on each
/// GPIO.
/// + If a PWM B pin is used as an input, and is selected on multiple GPIO pins, then the PWM
/// channel will see the logical OR of those two GPIO inputs
impl From<RPGpio> for ChannelNumber {
    fn from(gpio: RPGpio) -> Self {
        match gpio as u8 >> 1 & 0b111 {
            // Because of the bitwise AND, there are only eight possible values
            0 => ChannelNumber::Ch0,
            1 => ChannelNumber::Ch1,
            2 => ChannelNumber::Ch2,
            3 => ChannelNumber::Ch3,
            4 => ChannelNumber::Ch4,
            5 => ChannelNumber::Ch5,
            6 => ChannelNumber::Ch6,
            _ => ChannelNumber::Ch7,
        }
    }
}

/// Identifier for a channel pin
///
/// Each PWM channel has two pins: A and B.
/// Pin A is always configured as an output pin.
/// Pin B is configured as an output pin when running in free running mode. Otherwise, it is
/// configured as an input pin.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChannelPin {
    A,
    B,
}

/// Check ChannelNumber implementation for more details
impl From<RPGpio> for ChannelPin {
    fn from(gpio: RPGpio) -> Self {
        match gpio as u8 & 0b0000_0001 {
            // Because of the bitwise AND, there are only two possible values
            0 => ChannelPin::A,
            _ => ChannelPin::B,
        }
    }
}

// PWM channel configuration structure
//
// This helper struct allows multiple channels to share the same configuration.
struct PwmChannelConfiguration {
    en: bool,
    ph_correct: bool,
    a_inv: bool,
    b_inv: bool,
    divmode: DivMode,
    int: u8,
    frac: u8,
    cc_a: u16,
    cc_b: u16,
    top: u16,
}

impl Default for PwmChannelConfiguration {
    // Create a set of default values to use for configuring a PWM channel:
    // + the channel is disabled
    // + trailing-edge modulation configured
    // + no pin A and B polarity inversion
    // + free running mode for the fractional clock divider
    // + integral part of the divider is 1 and the fractional part is 0
    // + compare values for both pins are set 0 (0% duty cycle)
    // + top value is set to its maximum value
    fn default() -> Self {
        PwmChannelConfiguration {
            en: false,
            ph_correct: false,
            a_inv: false,
            b_inv: false,
            divmode: DivMode::FreeRunning,
            int: 1,
            frac: 0,
            cc_a: 0,
            cc_b: 0,
            top: u16::MAX,
        }
    }
}

const PWM_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x40050000 as *const PwmRegisters) };

/// Main struct for controlling PWM peripheral
pub struct Pwm<'a> {
    registers: StaticRef<PwmRegisters>,
    clocks: OptionalCell<&'a clocks::Clocks>,
}

impl<'a> Pwm<'a> {
    /// Create a new Pwm struct
    ///
    /// **Note**:
    /// + This method must be called only once when setting up the kernel peripherals.
    /// + This peripheral depends on the chip's clocks.
    /// + Also, if interrupts are required, then an interrupt handler must be set. Otherwise, all
    /// the interrupts will be ignored.
    pub fn new() -> Self {
        let pwm = Self {
            registers: PWM_BASE,
            clocks: OptionalCell::empty(),
        };
        pwm.init();
        pwm
    }

    // Enable or disable the given PWM channel
    //
    // enable == false ==> disable channel
    // enable == true ==> enable channel
    fn set_enabled(&self, channel_number: ChannelNumber, enable: bool) {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(match enable {
                true => CSR::EN::SET,
                false => CSR::EN::CLEAR,
            });
    }

    // Set phase correct (dual slope) modulation for the givem PWM channel
    //
    // ph_correct == false ==> trailing-edge modulation
    // ph_correct == true ==> phase-correct modulation
    fn set_ph_correct(&self, channel_number: ChannelNumber, ph_correct: bool) {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(match ph_correct {
                true => CSR::PH_CORRECT::SET,
                false => CSR::PH_CORRECT::CLEAR,
            });
    }

    // Invert polarity for pin A
    // a_inv == true ==> invert polarity for pin A
    fn set_invert_polarity_a(&self, channel_number: ChannelNumber, inv: bool) {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(match inv {
                true => CSR::A_INV::SET,
                false => CSR::A_INV::CLEAR,
            });
    }

    // Invert polarity for pin B
    // b_inv == true ==> invert polarity for pin B
    fn set_invert_polarity_b(&self, channel_number: ChannelNumber, inv: bool) {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(match inv {
                true => CSR::B_INV::SET,
                false => CSR::B_INV::CLEAR,
            });
    }

    // Invert polarity for both pins
    fn set_invert_polarity(&self, channel_number: ChannelNumber, a_inv: bool, b_inv: bool) {
        self.set_invert_polarity_a(channel_number, a_inv);
        self.set_invert_polarity_b(channel_number, b_inv);
    }

    // Set running mode for the givel channel
    //
    // divmode == FreeRunning ==> always enable clock divider
    // divmode == High ==> enable clock divider when pin B is high
    // divmode == Rising ==> enable clock divider when pin B is rising
    // divmode == Falling ==> enable clock divider when pin B is falling
    fn set_div_mode(&self, channel_number: ChannelNumber, div_mode: DivMode) {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(match div_mode {
                DivMode::FreeRunning => CSR::DIVMOD::FREE_RUNNING,
                DivMode::High => CSR::DIVMOD::B_HIGH,
                DivMode::Rising => CSR::DIVMOD::B_RISING,
                DivMode::Falling => CSR::DIVMOD::B_FALLING,
            });
    }

    // Set integral and fractional part of the clock divider
    // RP 2040 uses a 8.4 fractional clock divider.
    // The minimum value of the divider is   1 (int) +  0 / 16 (frac).
    // The maximum value of the divider is 255 (int) + 15 / 16 (frac).
    //
    // **Note**: this method will do nothing if int == 0 || frac > 15.
    fn set_divider_int_frac(&self, channel_number: ChannelNumber, int: u8, frac: u8) {
        if int == 0 || frac > 15 {
            return;
        }
        self.registers.ch[channel_number as usize]
            .div
            .modify(DIV::INT.val(int as u32));
        self.registers.ch[channel_number as usize]
            .div
            .modify(DIV::FRAC.val(frac as u32));
    }

    // Set output pin A compare value
    // If counter value < compare value A ==> pin A high
    fn set_compare_value_a(&self, channel_number: ChannelNumber, cc_a: u16) {
        self.registers.ch[channel_number as usize]
            .cc
            .modify(CC::A.val(cc_a as u32));
    }

    // Set output pin B compare value
    // If counter value < compare value B ==> pin B high (if divmode == FreeRunning)
    fn set_compare_value_b(&self, channel_number: ChannelNumber, cc_b: u16) {
        self.registers.ch[channel_number as usize]
            .cc
            .modify(CC::B.val(cc_b as u32));
    }

    // Set compare values for both pins
    fn set_compare_values_a_and_b(&self, channel_number: ChannelNumber, cc_a: u16, cc_b: u16) {
        self.set_compare_value_a(channel_number, cc_a);
        self.set_compare_value_b(channel_number, cc_b);
    }

    // Set counter top value
    fn set_top(&self, channel_number: ChannelNumber, top: u16) {
        self.registers.ch[channel_number as usize]
            .top
            .modify(TOP::TOP.val(top as u32));
    }

    // Get the current value of the counter
    fn get_counter(&self, channel_number: ChannelNumber) -> u16 {
        self.registers.ch[channel_number as usize]
            .ctr
            .read(CTR::CTR) as u16
    }

    // Set the value of the counter
    fn set_counter(&self, channel_number: ChannelNumber, value: u16) {
        self.registers.ch[channel_number as usize]
            .ctr
            .modify(CTR::CTR.val(value as u32));
    }

    fn wait_for(count: usize, f: impl Fn() -> bool) -> bool {
        for _ in 0..count {
            if f() {
                return true;
            }
        }

        false
    }

    // Increments the value of the counter
    //
    // The counter must be running at less than full speed. The method will return
    // once the increment is complete.
    fn advance_count(&self, channel_number: ChannelNumber) -> bool {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(CSR::PH_ADV::SET);
        Self::wait_for(100, || {
            self.registers.ch[channel_number as usize]
                .csr
                .read(CSR::PH_ADV)
                == 0
        })
    }

    // Retards the phase of the counter by 1 count
    //
    // The counter must be running. The method will return once the retardation
    // is complete.
    fn retard_count(&self, channel_number: ChannelNumber) -> bool {
        self.registers.ch[channel_number as usize]
            .csr
            .modify(CSR::PH_RET::SET);
        Self::wait_for(100, || {
            self.registers.ch[channel_number as usize]
                .csr
                .read(CSR::PH_RET)
                == 0
        })
    }

    // Enable interrupt on the given PWM channel
    fn enable_interrupt(&self, channel_number: ChannelNumber) {
        // What about adding a new method to the register interface which performs
        // a bitwise OR and another one for AND?
        let mask = self.registers.inte.read(CH::CH);
        self.registers
            .inte
            .modify(CH::CH.val(mask | 1 << channel_number as u32));
    }

    // Disable interrupt on the given PWM channel
    fn disable_interrupt(&self, channel_number: ChannelNumber) {
        let mask = self.registers.inte.read(CH::CH);
        self.registers
            .inte
            .modify(CH::CH.val(mask & !(1 << channel_number as u32)));
    }

    // Enable multiple channel interrupts at once.
    //
    // Bits 0 to 7 ==> enable channel 0-7 interrupts.
    fn enable_mask_interrupt(&self, mask: u8) {
        let old_mask = self.registers.inte.read(CH::CH);
        self.registers
            .inte
            .modify(CH::CH.val(old_mask | mask as u32));
    }

    // Disable multiple channel interrupts at once.
    //
    // Bits 0 to 7 ==> disable channel 0-7 interrupts.
    fn disable_mask_interrupt(&self, mask: u8) {
        let old_mask = self.registers.inte.read(CH::CH);
        self.registers
            .inte
            .modify(CH::CH.val(old_mask & !mask as u32));
    }

    // Clear interrupt flag
    fn clear_interrupt(&self, channel_number: ChannelNumber) {
        self.registers
            .intr
            .write(CH::CH.val(1 << channel_number as u32));
    }

    // Force interrupt on the given channel
    fn force_interrupt(&self, channel_number: ChannelNumber) {
        let mask = self.registers.intf.read(CH::CH);
        self.registers
            .intf
            .modify(CH::CH.val(mask | 1 << channel_number as u32));
    }

    // Unforce interrupt
    fn unforce_interrupt(&self, channel_number: ChannelNumber) {
        let mask = self.registers.intf.read(CH::CH);
        self.registers
            .intf
            .modify(CH::CH.val(mask & !(1 << channel_number as u32)));
    }

    // Get interrupt status
    fn get_interrupt_status(&self, channel_number: ChannelNumber) -> bool {
        (self.registers.ints.read(CH::CH) & 1 << channel_number as u32) != 0
    }

    // Configure the given channel using the given configuration
    fn configure_channel(&self, channel_number: ChannelNumber, config: &PwmChannelConfiguration) {
        self.set_ph_correct(channel_number, config.ph_correct);
        self.set_invert_polarity(channel_number, config.a_inv, config.b_inv);
        self.set_div_mode(channel_number, config.divmode);
        self.set_divider_int_frac(channel_number, config.int, config.frac);
        self.set_compare_value_a(channel_number, config.cc_a);
        self.set_compare_value_b(channel_number, config.cc_b);
        self.set_top(channel_number, config.top);
        self.set_enabled(channel_number, config.en);
    }

    // Initialize the struct
    fn init(&self) {
        let default_config: PwmChannelConfiguration = PwmChannelConfiguration::default();
        for channel_number in CHANNEL_NUMBERS {
            self.configure_channel(channel_number, &default_config);
            self.set_counter(channel_number, 0);
            self.disable_interrupt(channel_number);
        }
        self.registers.intr.write(CH::CH.val(0));
    }

    // This method should be called when resolving dependencies for the
    // default peripherals. See [crate::chip::Rp2040DefaultPeripherals::resolve_dependencies]
    pub(crate) fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.set(clocks);
    }

    // Given a channel number and a channel pin, return a struct that allows controlling it
    fn new_pwm_pin(&'a self, channel_number: ChannelNumber, channel_pin: ChannelPin) -> PwmPin<'a> {
        PwmPin {
            pwm_struct: self,
            channel_number,
            channel_pin,
        }
    }

    // Map the given GPIO to a PWM channel and a PWM pin
    fn gpio_to_pwm(&self, gpio: RPGpio) -> (ChannelNumber, ChannelPin) {
        (ChannelNumber::from(gpio), ChannelPin::from(gpio))
    }

    /// Map the GPIO to a PwmPin struct
    ///
    /// The returned structure can be used to control the PWM pin.
    ///
    /// See [PwmPin]
    pub fn gpio_to_pwm_pin(&'a self, gpio: RPGpio) -> PwmPin {
        let (channel_number, channel_pin) = self.gpio_to_pwm(gpio);
        self.new_pwm_pin(channel_number, channel_pin)
    }

    // Helper function to compute top, int and frac values
    // selected_freq_hz ==> user's desired frequency
    //
    // Return value: Ok(top, int, frac) in case of no error, otherwise Err(())
    fn compute_top_int_frac(&self, selected_freq_hz: usize) -> Result<(u16, u8, u8), ()> {
        let max_freq_hz = hil::pwm::Pwm::get_maximum_frequency_hz(self);
        let threshold_freq_hz = max_freq_hz / hil::pwm::Pwm::get_maximum_duty_cycle(self);
        // If the desired frequency doesn't make sense, return directly an error
        if selected_freq_hz > max_freq_hz || selected_freq_hz == 0 {
            return Err(());
        }

        // If the selected frequency is high enough, then there is no need for a divider
        if selected_freq_hz > threshold_freq_hz {
            return Ok(((max_freq_hz / selected_freq_hz - 1) as u16, 1, 0));
        }
        // If the selected frequency is below the threshold frequency, then a divider is necessary

        // Set top to max
        let top = u16::MAX;
        // Get the corresponding integral part of the divider
        let int = threshold_freq_hz / selected_freq_hz;
        // If the desired frequency is too low, then it can't be achieved using the divider.
        // In this case, notify the caller with an error.
        if int >= 256 {
            return Err(());
        }
        // Now that the integral part is valid, the fractional part can be computed as well.
        // The fractional part is on 4 bits.
        let frac = ((threshold_freq_hz << 4) / selected_freq_hz - (int << 4)) as u8;

        // Return the final result
        // Since int < 256, the cast will not truncate the value.
        Ok((top, int as u8, frac))
    }

    // Starts a PWM pin with the given frequency and duty cycle.
    //
    // Note: the actual values may vary due to rounding errors.
    fn start_pwm_pin(
        &self,
        channel_number: ChannelNumber,
        channel_pin: ChannelPin,
        frequency_hz: usize,
        duty_cycle: usize,
    ) -> Result<(), ErrorCode> {
        let (top, int, frac) = match self.compute_top_int_frac(frequency_hz) {
            Ok(result) => result,
            Err(()) => return Result::from(ErrorCode::INVAL),
        };

        let max_duty_cycle = hil::pwm::Pwm::get_maximum_duty_cycle(self);
        // Return an error if the selected duty cycle is higher than the maximum value
        if duty_cycle > max_duty_cycle {
            return Err(ErrorCode::INVAL);
        }
        // If top value is equal to u16::MAX, then it is impossible to
        // have a 100% duty cycle, so an error will be returned.
        let compare_value = if duty_cycle == max_duty_cycle {
            if top == u16::MAX {
                return Result::from(ErrorCode::INVAL);
            } else {
                // counter compare value for 100% glitch-free duty cycle
                top + 1
            }
        } else {
            // Normally, no overflow should occur if duty_cycle is less than or
            // equal to get_maximum_duty_cycle(). It is in user's responsability to
            // ensure the value is valid.
            ((top as usize + 1) * duty_cycle / max_duty_cycle) as u16
        };

        // Configure the channel accordingly
        self.set_top(channel_number, top);
        self.set_divider_int_frac(channel_number, int, frac);
        // Configure the pin accordingly
        if channel_pin == ChannelPin::A {
            self.set_compare_value_a(channel_number, compare_value);
        } else {
            self.set_compare_value_b(channel_number, compare_value);
        };
        // Finally, enable the channel
        self.set_enabled(channel_number, true);
        Ok(())
    }

    // Stop a PWM channel.
    //
    // This method does nothing if the PWM channel was already disabled.
    //
    // Note that disabling a PWM channel may result in disabling multiple PWM pins.
    fn stop_pwm_channel(&self, channel_number: ChannelNumber) -> Result<(), ErrorCode> {
        self.set_enabled(channel_number, false);
        Ok(())
    }
}

/// Implementation of the Hardware Interface Layer (HIL)
impl hil::pwm::Pwm for Pwm<'_> {
    type Pin = RPGpio;

    /// Start a PWM pin
    ///
    /// Start the given PWM pin with the given frequency and the given duty cycle.
    /// The actual values may vary due to rounding errors. For high precision duty cycles,
    /// the frequency should be set less than:
    ///
    /// ```rust,ignore
    /// let threshold_freq = pwm_struct.get_maximum_frequency_hz() / pwm_struct.get_maximum_duty_cycle()
    /// ```
    ///
    /// ## Errors
    ///
    /// This method may fail in one of the following situations:
    ///
    /// + selected frequency and duty cycle higher than the maximum possible values
    /// + 100% duty cycle demand for low frequencies (close to or below threshold_freq)
    /// + very low frequencies
    ///
    /// ## Safety
    ///
    /// It is safe to call multiples times this method with different values while the pin is
    /// running.
    ///
    /// **Note**: the pin must be set as a PWM pin prior to calling this method.
    fn start(
        &self,
        pin: &Self::Pin,
        frequency_hz: usize,
        duty_cycle: usize,
    ) -> Result<(), ErrorCode> {
        let (channel_number, channel_pin) = self.gpio_to_pwm(*pin);
        self.start_pwm_pin(channel_number, channel_pin, frequency_hz, duty_cycle)
    }

    /// Stop the given pin
    ///
    /// ## Errors
    ///
    /// This method may never fail.
    ///
    /// ## Safety
    ///
    /// It is safe to call this method multiple times on the same pin. If the pin is already
    /// stopped, then it does nothing.
    fn stop(&self, pin: &Self::Pin) -> Result<(), ErrorCode> {
        let (channel_number, _) = self.gpio_to_pwm(*pin);
        self.stop_pwm_channel(channel_number)
    }

    /// Return the maximum value of the frequency in Hz
    ///
    /// ## Panics
    ///
    /// This method will panic if the dependencies are not resolved.
    fn get_maximum_frequency_hz(&self) -> usize {
        self.clocks
            .unwrap_or_panic()
            .get_frequency(clocks::Clock::System) as usize
    }

    /// Return an opaque value representing 100% duty cycle
    fn get_maximum_duty_cycle(&self) -> usize {
        u16::MAX as usize + 1
    }
}

/// Helper structure to control a PWM pin
pub struct PwmPin<'a> {
    pwm_struct: &'a Pwm<'a>,
    channel_number: ChannelNumber,
    channel_pin: ChannelPin,
}

impl PwmPin<'_> {
    /// Returns the PWM channel the pin belongs to
    pub fn get_channel_number(&self) -> ChannelNumber {
        self.channel_number
    }

    /// Returns the PWM pin the pin belongs to
    pub fn get_channel_pin(&self) -> ChannelPin {
        self.channel_pin
    }

    // See [Pwm::set_invert_polarity_a] and [Pwm::set_invert_polarity_b]
    fn set_invert_polarity(&self, inv: bool) {
        if self.channel_pin == ChannelPin::A {
            self.pwm_struct
                .set_invert_polarity_a(self.channel_number, inv);
        } else {
            self.pwm_struct
                .set_invert_polarity_b(self.channel_number, inv);
        }
    }

    // See [Pwm::set_compare_value_a] and [Pwm::set_compare_value_b]
    fn set_compare_value(&self, compare_value: u16) {
        if self.channel_pin == ChannelPin::A {
            self.pwm_struct
                .set_compare_value_a(self.channel_number, compare_value);
        } else {
            self.pwm_struct
                .set_compare_value_b(self.channel_number, compare_value);
        }
    }
}

impl hil::pwm::PwmPin for PwmPin<'_> {
    /// Same as Pwm::start
    fn start(&self, frequency_hz: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        self.pwm_struct.start_pwm_pin(
            self.channel_number,
            self.channel_pin,
            frequency_hz,
            duty_cycle,
        )
    }

    /// Same as Pwm::stop
    fn stop(&self) -> Result<(), ErrorCode> {
        self.pwm_struct.stop_pwm_channel(self.channel_number)
    }

    /// Same as Pwm::get_maximum_frequency_hz
    fn get_maximum_frequency_hz(&self) -> usize {
        hil::pwm::Pwm::get_maximum_frequency_hz(self.pwm_struct)
    }

    /// Same as Pwm::get_maximum_duty_cycle
    fn get_maximum_duty_cycle(&self) -> usize {
        hil::pwm::Pwm::get_maximum_duty_cycle(self.pwm_struct)
    }
}

/// Unit tests
///
/// This module provides unit tests for the PWM driver.
///
/// To run the tests, add the following line before loading processes:
///
/// ```rust,ignore
/// rp2040::pwm::unit_tests::run::(&peripherals.pwm);
/// ```
///
/// Compile and flash the kernel on the board. Then, connect to UART on GPIOs 1 and 2.
/// If everything goes right, the following output should be displayed:
///
/// ```txt
/// Testing ChannelNumber enum...
/// ChannelNumber enum OK
/// Testing ChannelPin enum...
/// ChannelPin enum OK
/// Testing PWM struct...
/// Starting testing channel 1...
/// Channel 1 works!
/// Starting testing channel 2...
/// Channel 2 works!
/// Starting testing channel 3...
/// Channel 3 works!
/// Starting testing channel 4...
/// Channel 4 works!
/// Starting testing channel 5...
/// Channel 5 works!
/// Starting testing channel 6...
/// Channel 6 works!
/// Starting testing channel 7...
/// Channel 7 works!
/// PWM struct OK
/// Testing PwmPinStruct...
/// PwmPin struct OK
/// Testing PWM HIL trait...
/// PWM HIL trait OK
/// ```

pub mod unit_tests {
    use super::*;

    fn test_channel_number() {
        debug!("Testing ChannelNumber enum...");
        assert_eq!(ChannelNumber::from(RPGpio::GPIO0), ChannelNumber::Ch0);
        assert_eq!(ChannelNumber::from(RPGpio::GPIO3), ChannelNumber::Ch1);
        assert_eq!(ChannelNumber::from(RPGpio::GPIO14), ChannelNumber::Ch7);
        assert_eq!(ChannelNumber::from(RPGpio::GPIO28), ChannelNumber::Ch6);
        debug!("ChannelNumber enum OK");
    }

    fn test_channel_pin() {
        debug!("Testing ChannelPin enum...");
        assert_eq!(ChannelPin::from(RPGpio::GPIO4), ChannelPin::A);
        assert_eq!(ChannelPin::from(RPGpio::GPIO5), ChannelPin::B);
        debug!("ChannelPin enum OK");
    }

    fn test_channel(pwm: &Pwm, channel_number: ChannelNumber) {
        debug!("Starting testing channel {}...", channel_number as usize);

        // Testing set_enabled()
        pwm.set_enabled(channel_number, true);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].csr.read(CSR::EN),
            1
        );
        pwm.set_enabled(channel_number, false);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].csr.read(CSR::EN),
            0
        );

        // Testing set_ph_correct()
        pwm.set_ph_correct(channel_number, true);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::PH_CORRECT),
            1
        );
        pwm.set_ph_correct(channel_number, false);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::PH_CORRECT),
            0
        );

        // Testing set_invert_polarity()
        pwm.set_invert_polarity(channel_number, true, true);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::A_INV),
            1
        );
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::B_INV),
            1
        );
        pwm.set_invert_polarity(channel_number, true, false);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::A_INV),
            1
        );
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::B_INV),
            0
        );
        pwm.set_invert_polarity(channel_number, false, true);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::A_INV),
            0
        );
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::B_INV),
            1
        );
        pwm.set_invert_polarity(channel_number, false, false);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::A_INV),
            0
        );
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::B_INV),
            0
        );

        // Testing set_div_mode()
        pwm.set_div_mode(channel_number, DivMode::FreeRunning);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::DIVMOD),
            DivMode::FreeRunning as u32
        );
        pwm.set_div_mode(channel_number, DivMode::High);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::DIVMOD),
            DivMode::High as u32
        );
        pwm.set_div_mode(channel_number, DivMode::Rising);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::DIVMOD),
            DivMode::Rising as u32
        );
        pwm.set_div_mode(channel_number, DivMode::Falling);
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .csr
                .read(CSR::DIVMOD),
            DivMode::Falling as u32
        );

        // Testing set_divider_int_frac()
        pwm.set_divider_int_frac(channel_number, 123, 4);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].div.read(DIV::INT),
            123
        );
        assert_eq!(
            pwm.registers.ch[channel_number as usize]
                .div
                .read(DIV::FRAC),
            4
        );

        // Testing set_compare_value() methods
        pwm.set_compare_value_a(channel_number, 2022);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].cc.read(CC::A),
            2022
        );
        pwm.set_compare_value_b(channel_number, 12);
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::B), 12);
        pwm.set_compare_values_a_and_b(channel_number, 2023, 1);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].cc.read(CC::A),
            2023
        );
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::B), 1);

        // Testing set_top()
        pwm.set_top(channel_number, 12345);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].top.read(TOP::TOP),
            12345
        );

        // Testing get_counter() and set_counter()
        pwm.set_counter(channel_number, 1);
        assert_eq!(
            pwm.registers.ch[channel_number as usize].ctr.read(CTR::CTR),
            1
        );
        assert_eq!(pwm.get_counter(channel_number), 1);

        // Testing advance_count and retard_count()
        // The counter must be running to pass retard_count()
        // The counter must run at less than full speed (div_int + div_frac / 16 > 1) to pass
        // advance_count()
        pwm.set_div_mode(channel_number, DivMode::FreeRunning);
        assert!(pwm.advance_count(channel_number));
        assert_eq!(pwm.get_counter(channel_number), 2);
        pwm.set_enabled(channel_number, true);
        // No assert for retard count since it is impossible to predict how much the counter
        // will advance while running. However, the fact that the function returns true is a
        // good indicator that it does its job.
        assert!(pwm.retard_count(channel_number));
        // Disabling PWM to prevent it from generating interrupts signals for next tests
        pwm.set_enabled(channel_number, false);

        // Testing enable_interrupt() and disable_interrupt()
        pwm.enable_interrupt(channel_number);
        assert_eq!(
            pwm.registers.inte.read(CH::CH),
            1 << (channel_number as u32)
        );
        pwm.disable_interrupt(channel_number);
        assert_eq!(pwm.registers.inte.read(CH::CH), 0);

        // Testing get_interrupt_status()
        pwm.enable_interrupt(channel_number);
        pwm.set_counter(channel_number, 12345);
        pwm.advance_count(channel_number);
        assert!(pwm.get_interrupt_status(channel_number));
        pwm.disable_interrupt(channel_number);

        // Testing clear_interrupt()
        pwm.clear_interrupt(channel_number);
        assert!(!pwm.get_interrupt_status(channel_number));

        // Testing force_interrupt(), unforce_interrupt()
        pwm.force_interrupt(channel_number);
        assert_eq!(
            pwm.registers.intf.read(CH::CH),
            1 << (channel_number as u32)
        );
        assert!(pwm.get_interrupt_status(channel_number));
        pwm.unforce_interrupt(channel_number);
        assert_eq!(pwm.registers.intf.read(CH::CH), 0);
        assert!(!pwm.get_interrupt_status(channel_number));

        debug!("Channel {} works!", channel_number as usize);
    }

    fn test_pwm_struct(pwm: &Pwm) {
        debug!("Testing PWM struct...");
        let channel_number_list = [
            // Pins 0 and 1 are kept available for UART
            ChannelNumber::Ch1,
            ChannelNumber::Ch2,
            ChannelNumber::Ch3,
            ChannelNumber::Ch4,
            ChannelNumber::Ch5,
            ChannelNumber::Ch6,
            ChannelNumber::Ch7,
        ];

        // Testing enable_mask_interrupt() and disable_mask_interrupt()
        pwm.enable_mask_interrupt(u8::MAX);
        assert_eq!(pwm.registers.inte.read(CH::CH), u8::MAX as u32);
        pwm.disable_mask_interrupt(u8::MAX);
        assert_eq!(pwm.registers.inte.read(CH::CH), 0);

        for channel_number in channel_number_list {
            test_channel(pwm, channel_number);
        }
        debug!("PWM struct OK");
    }

    fn test_pwm_pin_struct<'a>(pwm: &'a Pwm<'a>) {
        debug!("Testing PwmPin struct...");
        let pwm_pin = pwm.gpio_to_pwm_pin(RPGpio::GPIO13);
        assert_eq!(pwm_pin.get_channel_number(), ChannelNumber::Ch6);
        assert_eq!(pwm_pin.get_channel_pin(), ChannelPin::B);

        pwm_pin.set_invert_polarity(true);
        assert_eq!(
            pwm.registers.ch[pwm_pin.get_channel_number() as usize]
                .csr
                .read(CSR::B_INV),
            1
        );
        pwm_pin.set_invert_polarity(false);
        assert_eq!(
            pwm.registers.ch[pwm_pin.get_channel_number() as usize]
                .csr
                .read(CSR::B_INV),
            0
        );

        pwm_pin.set_compare_value(987);
        assert_eq!(
            pwm.registers.ch[pwm_pin.get_channel_number() as usize]
                .cc
                .read(CC::B),
            987
        );
        debug!("PwmPin struct OK");
    }

    fn test_pwm_trait(pwm: &Pwm) {
        debug!("Testing PWM HIL trait...");
        let max_freq_hz = hil::pwm::Pwm::get_maximum_frequency_hz(pwm);
        let max_duty_cycle = hil::pwm::Pwm::get_maximum_duty_cycle(pwm);

        let (top, int, frac) = pwm.compute_top_int_frac(max_freq_hz).unwrap();
        assert_eq!(top, 0);
        assert_eq!(int, 1);
        assert_eq!(frac, 0);

        let (top, int, frac) = pwm.compute_top_int_frac(max_freq_hz / 4).unwrap();
        assert_eq!(top, 3);
        assert_eq!(int, 1);
        assert_eq!(frac, 0);

        let (top, int, frac) = pwm
            .compute_top_int_frac(max_freq_hz / max_duty_cycle)
            .unwrap();
        assert_eq!(top, u16::MAX);
        assert_eq!(int, 1);
        assert_eq!(frac, 0);

        let (top, int, frac) = pwm
            .compute_top_int_frac(max_freq_hz / max_duty_cycle / 2)
            .unwrap();
        assert_eq!(top, u16::MAX);
        assert_eq!(int, 2);
        assert_eq!(frac, 0);

        let freq = ((max_freq_hz / max_duty_cycle) as f32 / 2.5) as usize;
        let (top, int, frac) = pwm.compute_top_int_frac(freq).unwrap();
        assert_eq!(top, u16::MAX);
        assert_eq!(int, 2);
        assert_eq!(frac, 8);

        let freq = ((max_freq_hz / max_duty_cycle) as f32 / 3.15) as usize;
        let (top, int, frac) = pwm.compute_top_int_frac(freq).unwrap();
        assert_eq!(top, u16::MAX);
        assert_eq!(int, 3);
        assert_eq!(frac, 2);

        assert!(pwm
            .compute_top_int_frac(max_freq_hz / max_duty_cycle / 256)
            .is_err());
        assert!(pwm.compute_top_int_frac(max_freq_hz + 1).is_err());

        let (channel_number, channel_pin) = pwm.gpio_to_pwm(RPGpio::GPIO24);
        assert!(pwm
            .start_pwm_pin(channel_number, channel_pin, max_freq_hz / 4, 0)
            .is_ok());
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::A), 0);

        assert!(pwm
            .start_pwm_pin(
                channel_number,
                channel_pin,
                max_freq_hz / 4,
                max_duty_cycle / 4 * 3
            )
            .is_ok());
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::A), 3);

        assert!(pwm
            .start_pwm_pin(channel_number, channel_pin, max_freq_hz / 4, max_duty_cycle)
            .is_ok());
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::A), 4);

        assert!(pwm
            .start_pwm_pin(
                channel_number,
                channel_pin,
                max_freq_hz / max_duty_cycle,
                max_duty_cycle
            )
            .is_err());
        assert!(pwm
            .start_pwm_pin(channel_number, channel_pin, max_freq_hz + 1, max_duty_cycle)
            .is_err());
        assert!(pwm
            .start_pwm_pin(channel_number, channel_pin, max_freq_hz, max_duty_cycle + 1)
            .is_err());
        debug!("PWM HIL trait OK")
    }

    /// Run all unit tests
    ///
    /// pwm must be initialized and its dependencies resolved.
    pub fn run<'a>(pwm: &'a Pwm<'a>) {
        test_channel_number();
        test_channel_pin();
        test_pwm_struct(pwm);
        test_pwm_pin_struct(pwm);
        test_pwm_trait(pwm);
    }
}
