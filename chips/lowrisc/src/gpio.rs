//! General Purpose Input/Output driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{
    register_bitfields, register_structs, Field, ReadOnly, ReadWrite, WriteOnly,
};
use kernel::common::StaticRef;
use kernel::hil::gpio;

register_structs! {
    pub GpioRegisters {
        (0x00 => intr_state: ReadWrite<u32, pins::Register>),
        (0x04 => intr_enable: ReadWrite<u32, pins::Register>),
        (0x08 => intr_test: WriteOnly<u32, pins::Register>),
        (0x0c => data_in: ReadOnly<u32, pins::Register>),
        (0x10 => direct_out: ReadWrite<u32, pins::Register>),
        (0x14 => masked_out_lower: ReadWrite<u32, mask_half::Register>),
        (0x18 => masked_out_upper: ReadWrite<u32, mask_half::Register>),
        (0x1c => direct_oe: ReadWrite<u32, pins::Register>),
        (0x20 => masked_oe_lower: ReadWrite<u32, mask_half::Register>),
        (0x24 => masked_oe_upper: ReadWrite<u32, mask_half::Register>),
        (0x28 => intr_ctrl_en_rising: ReadWrite<u32, pins::Register>),
        (0x2c => intr_ctrl_en_falling: ReadWrite<u32, pins::Register>),
        (0x30 => intr_ctrl_en_lvlhigh: ReadWrite<u32, pins::Register>),
        (0x34 => intr_ctrl_en_lvllow: ReadWrite<u32, pins::Register>),
        (0x38 => ctrl_en_input_filter: ReadWrite<u32, pins::Register>),
        (0x3c => @END),
    }
}

register_bitfields![u32,
    pub pins [
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
    ],
    mask_half [
        data OFFSET(0) NUMBITS(16) [],
        mask OFFSET(16) NUMBITS(16) []
    ]
];

pub struct GpioPin {
    registers: StaticRef<GpioRegisters>,
    pin: Field<u32, pins::Register>,
    client: OptionalCell<&'static dyn gpio::Client>,
}

impl GpioPin {
    pub const fn new(base: StaticRef<GpioRegisters>, pin: Field<u32, pins::Register>) -> GpioPin {
        GpioPin {
            registers: base,
            pin: pin,
            client: OptionalCell::empty(),
        }
    }

    #[inline(always)]
    fn half_set(
        val: bool,
        field: Field<u32, pins::Register>,
        lower: &ReadWrite<u32, mask_half::Register>,
        upper: &ReadWrite<u32, mask_half::Register>,
    ) {
        let shift = field.shift;
        let bit = if val { 1u32 } else { 0u32 };
        if shift < 16 {
            lower.write(mask_half::data.val(bit << shift) + mask_half::mask.val(1u32 << shift));
        } else {
            let upper_shift = shift - 16;
            upper.write(
                mask_half::data.val(bit << upper_shift) + mask_half::mask.val(1u32 << upper_shift),
            );
        }
    }

    pub fn handle_interrupt(&self) {
        let regs = self.registers;
        let pin = self.pin;

        if regs.intr_state.is_set(pin) {
            regs.intr_state.modify(pin.val(1));
            self.client.map(|client| {
                client.fired();
            });
        }
    }
}

impl gpio::Configure for GpioPin {
    fn configuration(&self) -> gpio::Configuration {
        match self.registers.direct_oe.is_set(self.pin) {
            true => gpio::Configuration::InputOutput,
            false => gpio::Configuration::Input,
        }
    }

    fn set_floating_state(&self, _mode: gpio::FloatingState) {
        panic!("OpenTitan does not allow configuration of floating state");
    }

    fn floating_state(&self) -> gpio::FloatingState {
        // TODO: check this against the design
        gpio::FloatingState::PullNone
    }

    fn deactivate_to_low_power(&self) {
        self.disable_input();
        self.disable_output();
    }

    fn make_output(&self) -> gpio::Configuration {
        let regs = self.registers;
        GpioPin::half_set(true, self.pin, &regs.masked_oe_lower, &regs.masked_oe_upper);
        gpio::Configuration::InputOutput
    }

    fn disable_output(&self) -> gpio::Configuration {
        let regs = self.registers;
        GpioPin::half_set(
            false,
            self.pin,
            &regs.masked_oe_lower,
            &regs.masked_oe_upper,
        );
        gpio::Configuration::Input
    }

    fn make_input(&self) -> gpio::Configuration {
        self.configuration()
    }

    fn disable_input(&self) -> gpio::Configuration {
        self.configuration()
    }
}

impl gpio::Input for GpioPin {
    fn read(&self) -> bool {
        self.registers.data_in.is_set(self.pin)
    }
}

impl gpio::Output for GpioPin {
    fn toggle(&self) -> bool {
        let regs = self.registers;
        let pin = self.pin;
        let new_state = !regs.direct_out.is_set(pin);

        GpioPin::half_set(
            new_state,
            self.pin,
            &regs.masked_out_lower,
            &regs.masked_out_upper,
        );
        new_state
    }

    fn set(&self) {
        let regs = self.registers;
        GpioPin::half_set(
            true,
            self.pin,
            &regs.masked_out_lower,
            &regs.masked_out_upper,
        );
    }

    fn clear(&self) {
        let regs = self.registers;
        GpioPin::half_set(
            false,
            self.pin,
            &regs.masked_out_lower,
            &regs.masked_out_upper,
        );
    }
}

impl gpio::Interrupt for GpioPin {
    fn set_client(&self, client: &'static dyn gpio::Client) {
        self.client.set(client);
    }

    fn enable_interrupts(&self, mode: gpio::InterruptEdge) {
        let regs = self.registers;
        let pin = self.pin;

        match mode {
            gpio::InterruptEdge::RisingEdge => {
                regs.intr_ctrl_en_rising.modify(pin.val(1));
                regs.intr_ctrl_en_falling.modify(pin.val(0));
            }
            gpio::InterruptEdge::FallingEdge => {
                regs.intr_ctrl_en_rising.modify(pin.val(0));
                regs.intr_ctrl_en_falling.modify(pin.val(1));
            }
            gpio::InterruptEdge::EitherEdge => {
                regs.intr_ctrl_en_rising.modify(pin.val(1));
                regs.intr_ctrl_en_falling.modify(pin.val(1));
            }
        }
        regs.intr_state.modify(pin.val(1));
        regs.intr_enable.modify(pin.val(1));
    }

    fn disable_interrupts(&self) {
        let regs = self.registers;
        let pin = self.pin;

        regs.intr_enable.modify(pin.val(0));
        // Clear any pending interrupt
        regs.intr_state.modify(pin.val(1));
    }

    fn is_pending(&self) -> bool {
        self.registers.intr_state.is_set(self.pin)
    }
}

impl gpio::Pin for GpioPin {}
impl gpio::InterruptPin for GpioPin {}
