//! PWM driver for RP2040.

use kernel::debug;
use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::{ReadWriteable, Readable, Writeable};
use kernel::utilities::registers::{register_bitfields, ReadWrite, ReadOnly, WriteOnly};
use kernel::utilities::StaticRef;

use crate::clocks;
use crate::gpio::RPGpio;

register_bitfields![u32,
    CSR [
        /// Enable PWM channel
        EN OFFSET(0) NUMBITS(1) [],
        /// Enable phase-correct modulation
        PH_CORRECT OFFSET(1) NUMBITS(1) [],
        /// Invert output A
        A_INV OFFSET(2) NUMBITS(1) [],
        /// Invert output B
        B_INV OFFSET(3) NUMBITS(1) [],
        /// PWM slice event selection for fractional clock divider
        /// Default value = FREE_RUNNING (always on)
        /// If the event is different from FREE_RUNNING, then pin B becomes
        /// an input pin
        DIVMOD OFFSET(4) NUMBITS(2) [
            /// Free-running counting at rate dictated by fractional divider
            FREE_RUNNING = 0,
            /// Fractional divider operation is gated by the PWM B pin
            B_HIGH = 1,
            /// Counter advances with each rising edge of the PWM B pin
            B_RISING = 2,
            /// Counter advances with each falling edge of the PWM B pin
            B_FALLING = 3
        ],
        /// Retard the phase of the counter by 1 count, while it is running
        /// Self-clearing. Write a 1, and poll until low. Counter must be running.
        PH_RET OFFSET(6) NUMBITS(1) [],
        /// Advance the phase of the counter by 1 count, while it is running
        /// Self clearing. Write a 1, and poll until low. Counter must be running.
        PH_ADV OFFSET(7) NUMBITS(1) []
    ],

    /// DIV register
    /// INT and FRAC form a fixed-point fractional number.
    /// Counting rate is system clock frequency divided by this number.
    /// Fractional division uses simple 1st-order sigma-delta.
    DIV [
        FRAC OFFSET(0) NUMBITS(4) [],
        INT OFFSET(4) NUMBITS(8) []
    ],

    /// Direct access to the PWM counter
    CTR [
        CTR OFFSET(0) NUMBITS(16) []
    ],

    /// Counter compare values
    CC [
        A OFFSET(0) NUMBITS(16) [],
        B OFFSET(16) NUMBITS(16) []
    ],

    /// Counter top value
    /// When the value of the counter reaches the top value, depending on the
    /// ph_correct value, the counter will either:
    /// + wrap to 0 if ph_correct == 0
    /// + it starts counting downward until it reaches 0 again if ph_correct == 0
    TOP [
        TOP OFFSET(0) NUMBITS(16) []
    ],

    /// Control multiple channels at once.
    /// Each bit controls one channel.
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

#[repr(C)]
struct Ch {
    /// Control and status register
    csr: ReadWrite<u32, CSR::Register>,
    /// Division register
    div: ReadWrite<u32, DIV::Register>,
    /// Direct access to the PWM counter register
    ctr: ReadWrite<u32, CTR::Register>,
    /// Counter compare values register
    cc: ReadWrite<u32, CC::Register>,
    /// Counter wrap value register
    top: ReadWrite<u32, TOP::Register>
}

#[repr(C)]
struct PwmRegisters {
    /// Channel registers
    // TODO: Remove hard coding of the number of channels
    // core::mem::variant_count::<ChannenlNumber>() can't be used since it is not stable
    ch: [Ch; 8],
    /// Enable register
    /// This register aliases the CSR_EN bits for all channels.
    /// Writing to this register allows multiple channels to be enabled or disabled
    /// or disables simultaneously, so they can run in perfect sync.
    en: ReadWrite<u32, CH::Register>,
    /// Raw interrupts register
    intr: WriteOnly<u32, CH::Register>,
    /// Interrupt enable register
    inte: ReadWrite<u32, CH::Register>,
    /// Interrupt force register
    intf: ReadWrite<u32, CH::Register>,
    /// Interrupt status after masking & forcing
    ints: ReadOnly<u32, CH::Register>
}

#[derive(Clone, Copy)]
pub enum DivMode {
    FreeRunning,
    High,
    Rising,
    Falling
}

#[derive(Clone, Copy)]
pub enum ChannelNumber {
    Ch0,
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7
}

impl From<RPGpio> for ChannelNumber {
    fn from(gpio: RPGpio) -> Self {
        match gpio as u8 >> 1 & 7 {
            0 => ChannelNumber::Ch0,
            1 => ChannelNumber::Ch1,
            2 => ChannelNumber::Ch2,
            3 => ChannelNumber::Ch3,
            4 => ChannelNumber::Ch4,
            5 => ChannelNumber::Ch5,
            6 => ChannelNumber::Ch6,
            7 => ChannelNumber::Ch7,
            // This branch can't be reached due to logical AND
            _ => panic!("Unreachable branch")
        }
    }
}

// Each channel has two output pins associated
// **Note**: an output pin corresponds to two GPIOs, except for the 7th channel
#[derive(Clone, Copy, PartialEq)]
pub enum ChannelPin {
    A,
    B
}

impl From<RPGpio> for ChannelPin {
    fn from(gpio: RPGpio) -> Self {
        match gpio as u8 & 1 {
            0 => ChannelPin::A,
            1 => ChannelPin::B,
            // This branch can't be reached due to logical AND
            _ => panic!("Unreachable branch")
        }
    }
}

pub struct PwmChannelConfiguration {
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

impl PwmChannelConfiguration {
    /// Create a set of default values to use for configuring a PWM channel:
    /// + enabled = false
    /// + ph_correct = false
    /// + a_inv = false (no pin A polarity inversion)
    /// + b_inv = false (no pin B polarity inversion)
    /// + divmode = DivMode::FreeRunning (clock divider is always enabled)
    /// + int = 1 (integral part of the clock divider)
    /// + frac = 0 (fractional part of the clock divider)
    /// + cc_a = None (don't change current compare value for pin A)
    /// + cc_b = None (don't change current compare value for pin B)
    /// + top = u16::MAX (counter top value)
    pub fn default_config() -> Self {
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
            top: u16::MAX
        }
    }

    // enable == false ==> disable channel
    // enable == true ==> enable channel
    pub fn set_enabled(&mut self, enable: bool) {
        self.en = enable;
    }

    // ph_correct == false ==> trailing-edge modulation
    // ph_correct == true ==> phase-correct modulation
    pub fn set_ph_correct(&mut self, ph_correct: bool) {
        self.ph_correct = ph_correct;
    }

    // a_inv == true ==> invert polarity for pin A
    // b_inv == true ==> invert polarity for pin B
    pub fn set_invert_polarity(&mut self, a_inv: bool, b_inv: bool) {
        self.a_inv = a_inv;
        self.b_inv = b_inv;
    }

    // divmode == FreeRunning ==> always enable clock divider
    // divmode == High ==> enable clock divider when pin B is high
    // divmode == Rising ==> enable clock divider when pin B is rising
    // divmode == Falling ==> enable clock divider when pin B is falling
    pub fn set_div_mode(&mut self, divmode: DivMode) {
        self.divmode = divmode;
    }

    // RP 2040 uses a 8.4 fractional clock divider
    // The minimum value of the divider is   1 (int) +  0 / 16 (frac)
    // The maximum value of the divider is 255 (int) + 15 / 16 (frac)
    pub fn set_divider_int_frac(&mut self, int: u8, frac: u8) {
        // No need to check the upper bound, since the int parameter is u8
        assert!(int >= 1);
        // No need to check the lower bound, since the frac parameter is u8
        assert!(frac <= 15);
        self.int = int;
        self.frac = frac;
    }

    // Set compare value for channel A
    // If counter value < compare value ==> pin A high
    pub fn set_compare_value_a(&mut self, cc_a: u16) {
        self.cc_a = cc_a;
    }

    // Set compare value for channel B
    // If counter value < compare value ==> pin B high (if divmode == FreeRuning)
    pub fn set_compare_value_b(&mut self, cc_b: u16) {
        self.cc_b = cc_b;
    }

    pub fn set_compare_values_a_and_b(&mut self, cc_a: u16, cc_b: u16) {
        self.set_compare_value_a(cc_a);
        self.set_compare_value_b(cc_b);
    }

    // Set counter top value
    pub fn set_top(&mut self, top: u16) {
        self.top = top;
    }
}

const PWM_BASE: StaticRef<PwmRegisters> =
    unsafe { StaticRef::new(0x40050000 as *const PwmRegisters) };

pub struct Pwm<'a> {
    registers: StaticRef<PwmRegisters>,
    clocks: OptionalCell<&'a clocks::Clocks>
}

impl<'a> Pwm<'a> {
    pub fn new() -> Self {
        Self {
            registers: PWM_BASE,
            clocks: OptionalCell::empty()
        }
    }

    // enable == false ==> disable channel
    // enable == true ==> enable channel
    pub fn set_enabled(&self, channel_number: ChannelNumber, enable: bool) {
        self.registers.ch[channel_number as usize].csr.modify(match enable {
            true => CSR::EN::SET,
            false => CSR::EN::CLEAR
        });
    }

    // This function allows multiple channels to be enabled or disabled
    // simultaneously, so they can run in perfect sync.
    // Bits 0-7 enable channels 0-7 respectively
    pub fn set_mask_enabled(&self, mask: u8) {
        self.registers.en.modify(CH::CH.val(mask as u32));
    }

    // ph_correct == false ==> trailing-edge modulation
    // ph_correct == true ==> phase-correct modulation
    pub fn set_ph_correct(&self, channel_number: ChannelNumber, ph_correct: bool) {
        self.registers.ch[channel_number as usize].csr.modify(match ph_correct {
            true => CSR::PH_CORRECT::SET,
            false => CSR::PH_CORRECT::CLEAR
        });
    }

    // a_inv == true ==> invert polarity for pin A
    // b_inv == true ==> invert polarity for pin B
    pub fn set_invert_polarity(&self, channel_number: ChannelNumber, a_inv: bool, b_inv: bool) {
        self.registers.ch[channel_number as usize].csr.modify(match a_inv {
            true => CSR::A_INV::SET,
            false => CSR::A_INV::CLEAR
        });
        self.registers.ch[channel_number as usize].csr.modify(match b_inv {
            true => CSR::B_INV::SET,
            false => CSR::B_INV::CLEAR
        });
    }

    // divmode == FreeRunning ==> always enable clock divider
    // divmode == High ==> enable clock divider when pin B is high
    // divmode == Rising ==> enable clock divider when pin B is rising
    // divmode == Falling ==> enable clock divider when pin B is falling
    pub fn set_div_mode(&self, channel_number: ChannelNumber, divmode: DivMode) {
        self.registers.ch[channel_number as usize].csr.modify(match divmode {
            DivMode::FreeRunning => CSR::DIVMOD::FREE_RUNNING,
            DivMode::High => CSR::DIVMOD::B_HIGH,
            DivMode::Rising => CSR::DIVMOD::B_RISING,
            DivMode::Falling => CSR::DIVMOD::B_FALLING
        });
    }

    // RP 2040 uses a 8.4 fractional clock divider
    // The minimum value of the divider is   1 (int) +  0 / 16 (frac)
    // The maximum value of the divider is 255 (int) + 15 / 16 (frac)
    pub fn set_divider_int_frac(&self, channel_number: ChannelNumber, int: u8, frac: u8) {
        // No need to check the upper bound, since the int parameter is u8
        assert!(int >= 1);
        // No need to check the lower bound, since the frac parameter is u8
        assert!(frac <= 15);
        self.registers.ch[channel_number as usize].div.modify(DIV::INT.val(int as u32));
        self.registers.ch[channel_number as usize].div.modify(DIV::FRAC.val(frac as u32));
    }

    // Set output pin A compare value
    // If counter value < compare value A ==> pin A high
    pub fn set_compare_value_a(&self, channel_number: ChannelNumber, cc_a: u16) {
        self.registers.ch[channel_number as usize].cc.modify(CC::A.val(cc_a as u32));
    }

    // Set output pin B compare value
    // If counter value < compare value B ==> pin B high (if divmode == FreeRunning)
    pub fn set_compare_value_b(&self, channel_number: ChannelNumber, cc_b: u16) {
        self.registers.ch[channel_number as usize].cc.modify(CC::B.val(cc_b as u32));
    }

    pub fn set_compare_values_a_and_b(&self, channel_number: ChannelNumber, cc_a: u16, cc_b: u16) {
        self.set_compare_value_a(channel_number, cc_a);
        self.set_compare_value_b(channel_number, cc_b);
    }

    // Set counter top value
    pub fn set_top(&self, channel_number: ChannelNumber, top: u16) {
        self.registers.ch[channel_number as usize].top.modify(TOP::TOP.val(top as u32));
    }

    pub fn get_counter(&self, channel_number: ChannelNumber) -> u16 {
        self.registers.ch[channel_number as usize].ctr.read(CTR::CTR) as u16
    }

    pub fn set_counter(&self, channel_number: ChannelNumber, value: u16) {
        self.registers.ch[channel_number as usize].ctr.modify(CTR::CTR.val(value as u32));
    }

    pub fn advance_count(&self, channel_number: ChannelNumber) {
        self.registers.ch[channel_number as usize].csr.modify(CSR::PH_ADV::SET);
        while self.registers.ch[channel_number as usize].csr.read(CSR::PH_ADV) == 1 {}
    }

    pub fn retard_count(&self, channel_number: ChannelNumber) {
        self.registers.ch[channel_number as usize].csr.modify(CSR::PH_RET::SET);
        while self.registers.ch[channel_number as usize].csr.read(CSR::PH_RET) == 1 {}
    }

    pub fn enable_interrupt(&self, channel_number: ChannelNumber) {
        // What about adding a new method to the register interface which performs
        // a bitwise OR and another one for AND?
        let mask = self.registers.inte.read(CH::CH);
        self.registers.inte.modify(CH::CH.val(mask | 1 << channel_number as u32));
    }

    pub fn disable_interrupt(&self, channel_number: ChannelNumber) {
        let mask = self.registers.inte.read(CH::CH);
        self.registers.inte.modify(CH::CH.val(mask & !(1 << channel_number as u32)));
    }

    pub fn enable_mask_interrupt(&self, mask: u8) {
        self.registers.inte.modify(CH::CH.val(mask as u32));
    }

    pub fn clear_interrupt(&self, channel_number: ChannelNumber) {
        self.registers.intr.write(CH::CH.val(channel_number as u32));
    }

    pub fn force_interrupt(&self, channel_number: ChannelNumber) {
        let mask = self.registers.intf.read(CH::CH);
        self.registers.intf.modify(CH::CH.val(mask | 1 << channel_number as u32));
    }

    pub fn unforce_interrupt(&self, channel_number: ChannelNumber) {
        let mask = self.registers.intf.read(CH::CH);
        self.registers.intf.modify(CH::CH.val(mask & !(1 << channel_number as u32)));
    }

    pub fn get_interrupt_status_mask(&self) -> u8 {
        self.registers.ints.read(CH::CH) as u8
    }

    pub fn configure_channel(&self, channel_number: ChannelNumber, config: &PwmChannelConfiguration) {
        self.set_ph_correct(channel_number, config.ph_correct);
        self.set_invert_polarity(channel_number, config.a_inv, config.b_inv);
        self.set_div_mode(channel_number, config.divmode);
        self.set_divider_int_frac(channel_number, config.int, config.frac);
        self.set_compare_value_a(channel_number, config.cc_a);
        self.set_compare_value_b(channel_number, config.cc_b);
        self.set_top(channel_number, config.top);
        self.set_enabled(channel_number, config.en);
    }

    pub fn init(&self) {
        let channel_numbers = [
            ChannelNumber::Ch0,
            ChannelNumber::Ch1,
            ChannelNumber::Ch2,
            ChannelNumber::Ch3,
            ChannelNumber::Ch4,
            ChannelNumber::Ch5,
            ChannelNumber::Ch6,
            ChannelNumber::Ch7,
        ];
        let default_config = PwmChannelConfiguration::default_config();
        for channel_number in channel_numbers {
            self.configure_channel(channel_number, &default_config);
        }
    }

    // This method should be called when resolving dependencies for the
    // default peripherals
    pub fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.set(clocks);
    }

    // Given a channel number and a channel pin, return a struct that allows controlling its pins
    fn new_pwm_pin(&'a self, channel_number: ChannelNumber, channel_pin: ChannelPin) -> PwmPin<'a> {
        PwmPin {pwm_struct: self, channel_number, channel_pin}
    }

    // Even GPIO pins are mapped to output pin A, and odd GPIO pins are mapped to output pin B
    fn gpio_to_pwm(&self, gpio: RPGpio) -> (ChannelNumber, ChannelPin) {
        (ChannelNumber::from(gpio), ChannelPin::from(gpio))
    }

    // For a given GPIO, return the corresponding PwmPin struct to control it
    pub fn gpio_to_pwm_pin(&'a self, gpio: RPGpio) -> PwmPin {
        let (channel_number, channel_pin) = self.gpio_to_pwm(gpio);
        self.new_pwm_pin(channel_number, channel_pin)
    }

    // Helper function to compute top, int and frac values
    // selected_freq_hz ==> user's desired frequency
    //
    // Return value: Ok(top, int, frac) in case of no error, otherwise Err(())
    fn compute_top_int_frac(&self, selected_freq_hz: usize) -> Result<(u16, u8, u8), ()> {
        // If the selected frequency is high enough, then there is no need for a divider
        // Note that unwrap can never fail.
        let max_freq_hz = hil::pwm::Pwm::get_maximum_frequency_hz(self);
        let threshold_freq_hz = max_freq_hz / hil::pwm::Pwm::get_maximum_duty_cycle(self);
        if selected_freq_hz >= threshold_freq_hz {
            return Ok(((max_freq_hz / selected_freq_hz - 1) as u16, 1, 0));
        }
        // If the selected frequency is below the threshold frequency, then a divider is necessary

        // Set top to max
        let top = u16::MAX;
        // Get the corresponding divider value
        let divider = threshold_freq_hz as f32 / selected_freq_hz as f32;
        // If the desired frequency is too low, then it can't be achieved using the divider.
        // In this case, notify the caller with an error. Otherwise, get the integral part
        // of the divider.
        let int = if divider as u32 > u8::MAX as u32 {
           return Err(());
        } else {
            divider as u8
        };
        // Now that the integral part of the divider has been computed, frac can be too.
        let frac = ((divider - int as f32) * 16.0) as u8;

        // Return the final result
        Ok((top, int, frac))
    }

    fn start_pwm_pin(
        &self,
        channel_number: ChannelNumber,
        channel_pin: ChannelPin,
        frequency_hz: usize,
        duty_cycle: usize
    ) -> Result<(), ErrorCode>
    {
        let (top, int, frac) = match self.compute_top_int_frac(frequency_hz) {
            Ok(result) => result,
            Err(_) => return Result::from(ErrorCode::INVAL)
        };

        // If top value is equal to u16::MAX, then it is impossible to
        // have a 100% duty cycle, so an error will be generated. A solution
        // to this would be setting a higher frequency than the threshold frequency,
        // since the lose in precision is insignificant for very high top values.
        let max_duty_cycle = hil::pwm::Pwm::get_maximum_duty_cycle(self);
        let compare_value = if duty_cycle == max_duty_cycle {
            if top == u16::MAX {
                return Result::from(ErrorCode::INVAL);
            }
            else {
                // counter compare value for 100% glitch-free duty cycle
                top + 1
            }
        } else {
            // Normally, no overflow should occur if duty_cycle is less than or
            // equal to get_maximum_duty_cycle().
            (top as usize * duty_cycle / max_duty_cycle) as u16
        };

        // Create a channel configuration and set the corresponding parameters
        self.set_top(channel_number, top);
        self.set_divider_int_frac(channel_number, int, frac);
        // Set the compare value corresponding to the pin
        if channel_pin == ChannelPin::A {
            self.set_compare_value_a(channel_number, compare_value);
        }
        else {
            self.set_compare_value_b(channel_number, compare_value);
        };
        self.set_enabled(channel_number, true);
        Ok(())
    }

    fn stop_pwm_channel(&self, channel_number: ChannelNumber) -> Result<(), ErrorCode> {
        self.set_enabled(channel_number, false);
        Ok(())
    }
}

impl hil::pwm::Pwm for Pwm<'_> {
    type Pin = RPGpio;

    fn start(&self, pin: &Self::Pin, frequency_hz: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        let (channel_number, channel_pin) = self.gpio_to_pwm(*pin);
        self.start_pwm_pin(channel_number, channel_pin, frequency_hz, duty_cycle)
    }

    fn stop(&self, pin: &Self::Pin) -> Result<(), ErrorCode> {
        let (channel_number, _) = self.gpio_to_pwm(*pin);
        self.stop_pwm_channel(channel_number)
    }

    fn get_maximum_frequency_hz(&self) -> usize {
        self.clocks.unwrap_or_panic().get_frequency(clocks::Clock::System) as usize
    }

    fn get_maximum_duty_cycle(&self) -> usize {
        return u16::MAX as usize
    }
}


pub struct PwmPin<'a> {
    pwm_struct: &'a Pwm<'a>,
    channel_number: ChannelNumber,
    channel_pin: ChannelPin
}

impl PwmPin<'_> {
    pub fn get_channel_number(&self) -> ChannelNumber {
        self.channel_number
    }

    pub fn get_channel_pin(&self) -> ChannelPin {
        self.channel_pin
    }
}

impl hil::pwm::PwmPin for PwmPin<'_> {
    // Starts the pin with the given frequency and the given duty cycle.
    // If the pin was already running, the new values for the frequency and
    // the duty cycle will take effect at the end of the current PWM period.
    //
    // Starting the PWM pin might fail if the given frequency is too low. The minimal
    // value for the frequency is get_maximum_frequency_hz() / 16_773_120.
    // **Note**: the actual duty cycle value may vary due to precission errors. For
    // maximum precision, the frequency should be set to as close as get_maximum_frequency_hz() /
    // u16::MAX (higher the frequency, lower the precision). Beyond this value,
    // the precision will not increase.
    // **Note**: if a 100% duty cycle is desired, then the selected frequency should be higher than
    // the threshold frequency. One easy way to achieve this is to set frequency_hz to
    // get_maximum_frequency_hz() and duty_cycle to get_maximum_duty_cycle().
    fn start(&self, frequency_hz: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        self.pwm_struct.start_pwm_pin(self.channel_number, self.channel_pin, frequency_hz, duty_cycle)
    }

    // Stops the pin. If the pin was already stopped, this function does nothing.
    // **Note**: any subsequent start of the pin will continue where it stopped.
    fn stop(&self) -> Result<(), ErrorCode> {
        self.pwm_struct.stop_pwm_channel(self.channel_number)
    }

    // unwrap_or_panic() should never panic if peripherals were correctly configured.
    // If it panics, then it means that the clock dependency was not configured
    // correctly inside Rp2040DefaultPeripherals.resolve_dependencies() or that
    // clocks were misconfigured.
    // Using a default value makes no sense, since the PWM counter frequency depends
    // on the system clock frequency.
    //
    // The resulting frequency has *u32* type, so it is necessary to convert it to a *usize*.
    // The conversion cannot fail since all MCUs supported by Tock are 32-bit.
    // TODO: Maybe change the return type of Clocks::get_frequency() to usize
    fn get_maximum_frequency_hz(&self) -> usize {
        hil::pwm::Pwm::get_maximum_frequency_hz(self.pwm_struct)
    }

    // Since the PWM counter is a 16-bit counter, its maximum value is used as a
    // reference to compute the duty cycle.
    fn get_maximum_duty_cycle(&self) -> usize {
        hil::pwm::Pwm::get_maximum_duty_cycle(self.pwm_struct)
    }
}

pub mod tests {
    use super::*;

    fn test_channel(pwm: &Pwm, channel_number: ChannelNumber) {
        debug!("Starting testing channel {}...", channel_number as usize);

        // Testing set_enabled()
        pwm.set_enabled(channel_number, true);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::EN), 1);
        pwm.set_enabled(channel_number, false);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::EN), 0);

        // Testing set_ph_correct()
        pwm.set_ph_correct(channel_number, true);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::PH_CORRECT), 1);
        pwm.set_ph_correct(channel_number, false);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::PH_CORRECT), 0);

        // Testing set_invert_polarity()
        pwm.set_invert_polarity(channel_number, true, true);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::A_INV), 1);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::B_INV), 1);
        pwm.set_invert_polarity(channel_number, true, false);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::A_INV), 1);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::B_INV), 0);
        pwm.set_invert_polarity(channel_number, false, true);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::A_INV), 0);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::B_INV), 1);
        pwm.set_invert_polarity(channel_number, false, false);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::A_INV), 0);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::B_INV), 0);

        // Testing set_div_mode()
        pwm.set_div_mode(channel_number, DivMode::FreeRunning);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::DIVMOD), DivMode::FreeRunning as u32);
        pwm.set_div_mode(channel_number, DivMode::High);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::DIVMOD), DivMode::High as u32);
        pwm.set_div_mode(channel_number, DivMode::Rising);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::DIVMOD), DivMode::Rising as u32);
        pwm.set_div_mode(channel_number, DivMode::Falling);
        assert_eq!(pwm.registers.ch[channel_number as usize].csr.read(CSR::DIVMOD), DivMode::Falling as u32);

        // Testing set_divider_int_frac()
        pwm.set_divider_int_frac(channel_number, 123, 4);
        assert_eq!(pwm.registers.ch[channel_number as usize].div.read(DIV::INT), 123);
        assert_eq!(pwm.registers.ch[channel_number as usize].div.read(DIV::FRAC), 4);

        // Testing set_compare_value() methods
        pwm.set_compare_value_a(channel_number, 2022);
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::A), 2022);
        pwm.set_compare_value_b(channel_number, 12);
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::B), 12);
        pwm.set_compare_values_a_and_b(channel_number, 2023, 1);
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::A), 2023);
        assert_eq!(pwm.registers.ch[channel_number as usize].cc.read(CC::B), 1);

        // Testing set_top()
        pwm.set_top(channel_number, 12345);
        assert_eq!(pwm.registers.ch[channel_number as usize].top.read(TOP::TOP), 12345);

        // Testing get_counter() and set_counter()
        pwm.set_counter(channel_number, 1);
        assert_eq!(pwm.registers.ch[channel_number as usize].ctr.read(CTR::CTR), 1);
        assert_eq!(pwm.get_counter(channel_number), 1);

        // Testing advance_count and retard_count()
        // The counter must be running to pass retard_count()
        // The counter must run at less than full speed (div_int + div_frac / 16 > 1) to pass
        // advance_count()
        pwm.set_div_mode(channel_number, DivMode::FreeRunning);
        pwm.advance_count(channel_number);
        assert_eq!(pwm.get_counter(channel_number), 2);
        pwm.set_enabled(channel_number, true);
        // No assert for retard count since it is impossible to predict how much the counter
        // will advance while running. However, the fact that the function returns is a good
        // indicator that it does its job.
        pwm.retard_count(channel_number);
        // Disabling PWM to prevent it from generating interrupts signals for next tests
        pwm.set_enabled(channel_number, false);

        // Testing enable_interrupt() and disable_interrupt()
        pwm.enable_interrupt(channel_number);
        assert_eq!(pwm.registers.inte.read(CH::CH), 1 << (channel_number as u32));
        pwm.disable_interrupt(channel_number);
        assert_eq!(pwm.registers.inte.read(CH::CH), 0);

        // Testing force_interrupt(), unforce_interrupt() and get_interrupt_status_mask()
        pwm.force_interrupt(channel_number);
        assert_eq!(pwm.registers.intf.read(CH::CH), 1 << (channel_number as u32));
        assert_eq!(pwm.get_interrupt_status_mask(), 1 << (channel_number as u8));
        pwm.unforce_interrupt(channel_number);
        assert_eq!(pwm.registers.intf.read(CH::CH), 0);
        assert_eq!(pwm.get_interrupt_status_mask(), 0);

        debug!("Channel {} works!", channel_number as usize);
    }

    pub fn run(pwm: &Pwm) {
        let channel_number_list = [
            // Pins 0 and 1 are kept available for UART
            ChannelNumber::Ch1,
            ChannelNumber::Ch2,
            ChannelNumber::Ch3,
            ChannelNumber::Ch4,
            ChannelNumber::Ch5,
            ChannelNumber::Ch6,
            ChannelNumber::Ch7
        ];

        for channel_number in channel_number_list {
            test_channel(pwm, channel_number);
        }
    }
}
