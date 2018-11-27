use core::cell::Cell;

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{Field, FieldValue, ReadOnly, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;

#[repr(C)]
pub struct GpioRegisters {
    /// Pin value.
    value: ReadOnly<u32, pins::Register>,
    /// Pin Input Enable Register
    input_en: ReadWrite<u32, pins::Register>,
    /// Pin Output Enable Register
    output_en: ReadWrite<u32, pins::Register>,
    /// Output Port Value Register
    port: ReadWrite<u32, pins::Register>,
    /// Internal Pull-Up Enable Register
    pullup: ReadWrite<u32, pins::Register>,
    /// Drive Strength Register
    drive: ReadWrite<u32, pins::Register>,
    /// Rise Interrupt Enable Register
    rise_ie: ReadWrite<u32, pins::Register>,
    /// Rise Interrupt Pending Register
    rise_ip: ReadWrite<u32, pins::Register>,
    /// Fall Interrupt Enable Register
    fall_ie: ReadWrite<u32, pins::Register>,
    /// Fall Interrupt Pending Register
    fall_ip: ReadWrite<u32, pins::Register>,
    /// High Interrupt Enable Register
    high_ie: ReadWrite<u32, pins::Register>,
    /// High Interrupt Pending Register
    high_ip: ReadWrite<u32, pins::Register>,
    /// Low Interrupt Enable Register
    low_ie: ReadWrite<u32, pins::Register>,
    /// Low Interrupt Pending Register
    low_ip: ReadWrite<u32, pins::Register>,
    /// HW I/O Function Enable Register
    iof_en: ReadWrite<u32, pins::Register>,
    /// HW I/O Function Select Register
    iof_sel: ReadWrite<u32, pins::Register>,
    /// Output XOR (invert) Register
    out_xor: ReadWrite<u32, pins::Register>,
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
    client_data: Cell<usize>,
    client: OptionalCell<&'static hil::gpio::Client>,
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
            client_data: Cell::new(0),
            client: OptionalCell::empty(),
        }
    }

    pub fn set_client<C: hil::gpio::Client>(&self, client: &'static C) {
        self.client.set(client);
    }

    /// Configure this pin as IO Function 0. What that maps to is chip- and pin-
    /// specific.
    pub fn iof0(&self) {
        let regs = self.registers;

        regs.out_xor.modify(self.clear);
        regs.iof_sel.modify(self.clear);
        regs.iof_en.modify(self.set);
    }

    /// Configure this pin as IO Function 1. What that maps to is chip- and pin-
    /// specific.
    pub fn iof1(&self) {
        let regs = self.registers;

        regs.out_xor.modify(self.clear);
        regs.iof_sel.modify(self.set);
        regs.iof_en.modify(self.set);
    }

    /// There are separate interrupts in PLIC for each pin, so the interrupt
    /// handler only needs to exist on each pin.
    pub fn handle_interrupt(&self) {
        self.client.map(|client| {
            client.fired(self.client_data.get());
        });
    }
}

impl hil::gpio::PinCtl for GpioPin {
    fn set_input_mode(&self, mode: hil::gpio::InputMode) {
        let regs = self.registers;

        match mode {
            hil::gpio::InputMode::PullUp => {
                regs.pullup.modify(self.set);
            }
            hil::gpio::InputMode::PullDown => {
                regs.pullup.modify(self.clear);
            }
            hil::gpio::InputMode::PullNone => {
                regs.pullup.modify(self.clear);
            }
        }
    }
}

impl hil::gpio::Pin for GpioPin {
    fn disable(&self) {
        // NOP. There does not seem to be a way to "disable" a GPIO pin on this
        // chip.
    }

    fn make_output(&self) {
        let regs = self.registers;

        regs.drive.modify(self.clear);
        regs.out_xor.modify(self.clear);
        regs.output_en.modify(self.set);
        regs.iof_en.modify(self.clear);
    }

    fn make_input(&self) {
        let regs = self.registers;

        regs.pullup.modify(self.clear);
        regs.input_en.modify(self.set);
        regs.iof_en.modify(self.clear);
    }

    fn read(&self) -> bool {
        let regs = self.registers;

        regs.value.is_set(self.pin)
    }

    fn toggle(&self) {
        let regs = self.registers;

        let current_outputs = regs.port.extract();
        if current_outputs.is_set(self.pin) {
            regs.port.modify_no_read(current_outputs, self.clear);
        } else {
            regs.port.modify_no_read(current_outputs, self.set);
        }
    }

    fn set(&self) {
        let regs = self.registers;

        regs.port.modify(self.set);
    }

    fn clear(&self) {
        let regs = self.registers;

        regs.port.modify(self.clear);
    }

    fn enable_interrupt(&self, client_data: usize, mode: hil::gpio::InterruptMode) {
        let regs = self.registers;

        regs.pullup.modify(self.clear);
        regs.input_en.modify(self.set);
        regs.iof_en.modify(self.clear);

        self.client_data.set(client_data);

        match mode {
            hil::gpio::InterruptMode::RisingEdge => {
                regs.rise_ie.modify(self.set);
            }
            hil::gpio::InterruptMode::FallingEdge => {
                regs.fall_ie.modify(self.set);
            }
            hil::gpio::InterruptMode::EitherEdge => {
                regs.rise_ie.modify(self.set);
                regs.fall_ie.modify(self.set);
            }
        }
    }

    fn disable_interrupt(&self) {
        let regs = self.registers;

        regs.rise_ie.modify(self.clear);
        regs.fall_ie.modify(self.clear);
    }
}
