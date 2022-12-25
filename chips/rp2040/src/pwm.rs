//! PWM driver for RP2040.

use kernel::ErrorCode;
use kernel::hil;
use kernel::utilities::cells::OptionalCell;
use kernel::utilities::registers::interfaces::Writeable;
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
            CH0 = 0,
            CH1 = 1,
            CH2 = 2,
            CH3 = 3,
            CH4 = 4,
            CH5 = 5,
            CH6 = 6,
            CH7 = 7
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
    /// + cc_a = 0 (counter compare value for pin A)
    /// + cc_b = 0 (counter compare value for pin B)
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

    // Set compare values
    // If counter value < compare value A ==> pin A high
    // If couter value < compare value B ==> pin B high (if divmode == FreeRunning)
    pub fn set_compare_values(&mut self, cc_a: u16, cc_b: u16) {
        self.cc_a = cc_a;
        self.cc_b = cc_b;
    }

    // Set counter top value
    pub fn set_top_value(&mut self, top: u16) {
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
        self.registers.ch[channel_number as usize].csr.write(match enable {
            true => CSR::EN::SET,
            false => CSR::EN::CLEAR
        });
    }

    // This function allows multiple channels to be enabled or disabled
    // simultaneously, so they can run in perfect sync.
    // Bits 0-7 enable channels 0-7 respectively
    pub fn set_mask_enabled(&self, mask: u8) {
        self.registers.en.write(CH::CH.val(mask as u32));
    }

    // ph_correct == false ==> trailing-edge modulation
    // ph_correct == true ==> phase-correct modulation
    pub fn set_ph_correct(&self, channel_number: ChannelNumber, ph_correct: bool) {
        self.registers.ch[channel_number as usize].csr.write(match ph_correct {
            true => CSR::PH_CORRECT::SET,
            false => CSR::PH_CORRECT::CLEAR
        });
    }

    // a_inv == true ==> invert polarity for pin A
    // b_inv == true ==> invert polarity for pin B
    pub fn set_invert_polarity(&self, channel_number: ChannelNumber, a_inv: bool, b_inv: bool) {
        self.registers.ch[channel_number as usize].csr.write(match a_inv {
            true => CSR::A_INV::SET,
            false => CSR::A_INV::CLEAR
        });
        self.registers.ch[channel_number as usize].csr.write(match b_inv {
            true => CSR::B_INV::SET,
            false => CSR::B_INV::CLEAR
        });
    }

    // divmode == FreeRunning ==> always enable clock divider
    // divmode == High ==> enable clock divider when pin B is high
    // divmode == Rising ==> enable clock divider when pin B is rising
    // divmode == Falling ==> enable clock divider when pin B is falling
    pub fn set_div_mode(&self, channel_number: ChannelNumber, divmode: DivMode) {
        self.registers.ch[channel_number as usize].csr.write(match divmode {
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
        self.registers.ch[channel_number as usize].div.write(DIV::INT.val(int as u32));
        self.registers.ch[channel_number as usize].div.write(DIV::FRAC.val(frac as u32));
    }

    // Set compare values
    // If counter value < compare value A ==> pin A high
    // If couter value < compare value B ==> pin B high (if divmode == FreeRunning)
    pub fn set_compare_values(&self, channel_number: ChannelNumber, cc_a: u16, cc_b: u16) {
        self.registers.ch[channel_number as usize].cc.write(CC::A.val(cc_a as u32));
        self.registers.ch[channel_number as usize].cc.write(CC::B.val(cc_b as u32));
    }

    // Set counter top value
    pub fn set_top(&self, channel_number: ChannelNumber, top: u16) {
        self.registers.ch[channel_number as usize].top.write(TOP::TOP.val(top as u32));
    }

    pub fn configure_channel(&self, channel_number: ChannelNumber, config: &PwmChannelConfiguration) {
        self.set_enabled(channel_number, config.en);
        self.set_ph_correct(channel_number, config.ph_correct);
        self.set_invert_polarity(channel_number, config.a_inv, config.b_inv);
        self.set_div_mode(channel_number, config.divmode);
        self.set_divider_int_frac(channel_number, config.int, config.frac);
        self.set_compare_values(channel_number, config.cc_a, config.cc_b);
        self.set_top(channel_number, config.top);
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

    pub fn set_clocks(&self, clocks: &'a clocks::Clocks) {
        self.clocks.set(clocks);
    }

    fn new_pwm_pin(&'a self, channel_number: ChannelNumber) -> PwmPin<'a> {
        PwmPin {pwm_struct: self, channel_number}
    }

    pub fn gpio_to_pwm_pin(&'a self, gpio: RPGpio) -> PwmPin {
        match gpio {
            RPGpio::GPIO0 | RPGpio::GPIO1 | RPGpio::GPIO16 | RPGpio::GPIO17 => self.new_pwm_pin(ChannelNumber::Ch0),
            RPGpio::GPIO2 | RPGpio::GPIO3 | RPGpio::GPIO18 | RPGpio::GPIO19 => self.new_pwm_pin(ChannelNumber::Ch1),
            RPGpio::GPIO4 | RPGpio::GPIO5 | RPGpio::GPIO20 | RPGpio::GPIO21 => self.new_pwm_pin(ChannelNumber::Ch2),
            RPGpio::GPIO6 | RPGpio::GPIO7 | RPGpio::GPIO22 | RPGpio::GPIO23 => self.new_pwm_pin(ChannelNumber::Ch3),
            RPGpio::GPIO8 | RPGpio::GPIO9 | RPGpio::GPIO24 | RPGpio::GPIO25 => self.new_pwm_pin(ChannelNumber::Ch4),
            RPGpio::GPIO10 | RPGpio::GPIO11 | RPGpio::GPIO26 | RPGpio::GPIO27 => self.new_pwm_pin(ChannelNumber::Ch5),
            RPGpio::GPIO12 | RPGpio::GPIO13 | RPGpio::GPIO28 | RPGpio::GPIO29 => self.new_pwm_pin(ChannelNumber::Ch6),
            RPGpio::GPIO14 | RPGpio::GPIO15 => self.new_pwm_pin(ChannelNumber::Ch7),
        }
    }
}

pub struct PwmPin<'a> {
    pwm_struct: &'a Pwm<'a>,
    channel_number: ChannelNumber
}

impl hil::pwm::PwmPin for PwmPin<'_> {
    // Starts the pin with the given frequency and the given duty cycle.
    // If the pin was already running, the new values for the frequency and
    // the duty cycle will take effect at the end of the current PWM period.
    //
    // Starting the PWM pin might fail if the given frequency is too low. Minimal
    // value for the frequency is get_maximum_frequency_hz() / get_maximum_duty_cycle().
    // **Note**: the actual duty cycle value may vary due to precission errors. For
    // maximum precision, frequency should be set to its minimal value.
    fn start(&self, frequency_hz: usize, duty_cycle: usize) -> Result<(), ErrorCode> {
        let top = self.get_maximum_frequency_hz() / frequency_hz - 1;
        // If frequency is too low, then report an error
        let top: u16 = match top.try_into() {
            Ok(top) => top,
            Err(_) => return Result::from(ErrorCode::INVAL)
        };

        // If top value is equal to u16::MAX, then it is impossible to
        // have a 100% duty cycle, so an error will be generated. A solution
        // to this would be setting a slightly higher frequency, since the lose in
        // precission is insignificant for very high top values
        let compare_value = if duty_cycle == self.get_maximum_duty_cycle() {
            if top == u16::MAX {
                return Result::from(ErrorCode::INVAL);
            }
            else {
                // compare value for 100% glitch-free duty cycle
                top + 1
            }
        } else {
            top * ((duty_cycle / self.get_maximum_duty_cycle()) as u16)
        };

        // Create a channel configuration and set the corresponding parameters
        let mut config = PwmChannelConfiguration::default_config();
        config.set_top_value(top);
        config.set_compare_values(compare_value, compare_value);
        config.set_enabled(true);
        self.pwm_struct.configure_channel(self.channel_number, &config);
        Ok(())
    }

    // Stops the pin. If the pin was already stopped, this function does nothing.
    // **Note**: any subsequent start of the pin will continue where it stopped.
    fn stop(&self) -> Result<(), ErrorCode> {
        self.pwm_struct.set_enabled(self.channel_number, false);
        Ok(())
    }

    // unwrap_or_panic() should never panic if peripherals were correctly configured.
    // If it panics, then it means that the clock dependency was not configured
    // correctly inside Rp2040DefaultPeripherals.resolve_dependencies() or that
    // clocks were misconfigured.
    // Using a default value makes no sense, since the PWM counter frequency depends
    // on the system clock frequency.
    //
    // The resulting frequency has *u32* type, so it is necessary to convert it to a *usize*.
    // The conversation cannot fail since all MCUs supported by Tock are 32-bit.
    // TODO: Maybe change the return type of Clocks::get_frequency() to usize
    fn get_maximum_frequency_hz(&self) -> usize {
        return self.pwm_struct.clocks
            .unwrap_or_panic()
            .get_frequency(clocks::Clock::System)
            .try_into()
            .unwrap();
    }

    // Since the PWM counter is a 16-bit counter, its maximum value is used as a
    // reference to compute the duty cycle.
    // **Note**: If maximum precision is required for the duty cycle, then the minimum
    // frequency should be chosen:
    // `freq = get_maximum_frequency_hz() / get_maximum_duty_cycle()`
    // Failing in doing so will result in a less precise duty cycle due to round errors
    fn get_maximum_duty_cycle(&self) -> usize {
        return u16::MAX as usize;
    }
}
