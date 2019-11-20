//! General Purpose Input/Output driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, Field, FieldValue, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

#[repr(C)]
pub struct GpioRegisters {
    intr_state: ReadOnly<u32, pins::Register>,
    intr_enable: ReadOnly<u32, pins::Register>,
    intr_test: ReadOnly<u32, pins::Register>,
    // GPIO Input data read value
    data_in: ReadOnly<u32, pins::Register>,
    // GPIO direct output data write value
    direct_out: ReadWrite<u32, pins::Register>,
    // GPIO write data lower with mask
    masked_out_low: ReadWrite<u32, pins::Register>,
    // GPIO write data upper with mask
    masked_out_high: ReadWrite<u32, pins::Register>,
    // GPIO Output Enable
    direct_oe: ReadWrite<u32, pins::Register>,
    // GPIO write Output Enable lower with mask
    masked_oe_low: ReadWrite<u32, pins::Register>,
    // GPIO write Output Enable upper with mask
    masked_oe_upper: ReadWrite<u32, pins::Register>,
    // GPIO interrupt enable for GPIO, rising edge
    intr_ctrl_en_rise: ReadWrite<u32, pins::Register>,
    // GPIO interrupt enable for GPIO, falling edge
    intr_ctrl_en_fall: ReadWrite<u32, pins::Register>,
    // GPIO interrupt enable for GPIO, level high
    intr_ctrl_en_lvlhigh: ReadWrite<u32, pins::Register>,
    // GPIO interrupt enable for GPIO, level low
    intr_ctrl_en_lvllow: ReadWrite<u32, pins::Register>,
    // filter enable for GPIO input bits
    intr_ctrl_en_input_filer: ReadWrite<u32, pins::Register>,
}

register_bitfields![u32,
	pins [
	    pin0 0,
	    pin1 1,
	    pin2 2,
	    pin3 3,
	    pin4 4,
	    pin5 5,
	    pin6 6,
	    pin7 7,
	    pin8 8,
	    pin9 9,
	    pin10 10,
	    pin11 11,
	    pin12 12,
	    pin13 13,
	    pin14 14,
	    pin15 15,
	    pin16 16,
	    pin17 17,
	    pin18 18,
	    pin19 19,
	    pin20 20,
	    pin21 21,
	    pin22 22,
	    pin23 23,
	    pin24 24,
	    pin25 25,
	    pin26 26,
	    pin27 27,
	    pin28 28,
	    pin29 29,
	    pin30 30,
	    pin31 31
	]
];

pub struct GpioPin {
    registers: StaticRef<GpioRegisters>,
    pin: Field<u32, pins::Register>,
    set: FieldValue<u32, pins::Register>,
    clear: FieldValue<u32, pins::Register>,
    client: OptionalCell<&'static dyn hil::gpio::Client>,
}

impl GpioPin {
    pub const fn new(
        base: StaticRef<GpioRegisters>,
        pin: Field<u32, pins::Register>,
        set: FieldValue<u32, pins::Register>,
        clear: FieldValue<u32, pins::Register>,
    ) -> GpioPin {
        GpioPin {
            registers: base,
            pin: pin,
            set: set,
            clear: clear,
            client: OptionalCell::empty(),
        }
    }

    /// There are separate interrupts in PLIC for each pin, so the interrupt
    /// handler only needs to exist on each pin.
    pub fn handle_interrupt(&self) {
        // TODO
    }
}

impl hil::gpio::Configure for GpioPin {
    fn configuration(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        let output = regs.direct_oe.is_set(self.pin);
        let input;
        if self.pin.shift < 15 {
            input = regs.masked_oe_low.is_set(self.pin);
        } else {
            input = regs.masked_oe_upper.is_set(self.pin);
        }

        return match (input, output) {
            (true, true) => hil::gpio::Configuration::InputOutput,
            (true, false) => hil::gpio::Configuration::Input,
            (false, true) => hil::gpio::Configuration::Output,
            (false, false) => hil::gpio::Configuration::LowPower,
        };
    }

    /* OpenTitan doesn't appear to support floating state GPIOs */
    fn set_floating_state(&self, _mode: hil::gpio::FloatingState) {
        // TODO
    }

    fn floating_state(&self) -> hil::gpio::FloatingState {
        // TODO
        hil::gpio::FloatingState::PullNone
    }

    fn deactivate_to_low_power(&self) {
        self.disable_input();
        self.disable_output();
    }

    fn make_output(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        regs.direct_oe.modify(self.set);

        if self.set.value < 15 {
            regs.masked_oe_low.set(self.set.value << 16);
            regs.masked_oe_low.modify(self.set);
        } else {
            regs.masked_oe_upper.set((self.set.value - 16) << 16);
            regs.masked_oe_upper.set(self.set.value - 16);
        }

        self.configuration()
    }

    fn disable_output(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        regs.direct_oe.modify(self.clear);

        if self.set.value < 15 {
            regs.masked_oe_low.set(self.set.value << 16);
            regs.masked_oe_low.modify(self.clear);
        } else {
            regs.masked_oe_upper.set((self.set.value - 16) << 16);
            regs.masked_oe_upper.set(self.clear.value - 16);
        }

        self.configuration()
    }

    fn make_input(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        regs.direct_oe.modify(self.clear);

        if self.set.value < 15 {
            regs.masked_oe_low.set(self.set.value << 16);
            regs.masked_oe_low.modify(self.clear);
        } else {
            regs.masked_oe_upper.set((self.set.value - 16) << 16);
            regs.masked_oe_upper.set(self.clear.value - 16);
        }

        self.configuration()
    }

    fn disable_input(&self) -> hil::gpio::Configuration {
        let regs = self.registers;

        regs.direct_oe.modify(self.clear);

        self.configuration()
    }
}

impl hil::gpio::Input for GpioPin {
    fn read(&self) -> bool {
        let regs = self.registers;

        regs.data_in.is_set(self.pin)
    }
}

impl hil::gpio::Output for GpioPin {
    fn toggle(&self) -> bool {
        let regs = self.registers;

        let current_outputs = regs.direct_out.extract();
        if current_outputs.is_set(self.pin) {
            regs.direct_out.modify_no_read(current_outputs, self.clear);
        } else {
            regs.direct_out.modify_no_read(current_outputs, self.set);
        }
        regs.direct_out.extract().is_set(self.pin)
    }

    fn set(&self) {
        let regs = self.registers;

        regs.direct_out.modify(self.set);
    }

    fn clear(&self) {
        let regs = self.registers;

        regs.direct_out.modify(self.clear);
    }
}

/* This actually needs to be done */
impl hil::gpio::Interrupt for GpioPin {
    fn set_client(&self, client: &'static dyn hil::gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, _mode: hil::gpio::InterruptEdge) {
        // TODO
    }

    fn disable_interrupts(&self) {
        // TODO
    }

    fn is_pending(&self) -> bool {
        // TODO
        false
    }
}

impl hil::gpio::Pin for GpioPin {}
impl hil::gpio::InterruptPin for GpioPin {}
