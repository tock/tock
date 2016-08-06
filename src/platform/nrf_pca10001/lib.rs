#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(lang_items)]

extern crate drivers;
extern crate hil;
extern crate nrf51822;
extern crate support;
extern crate process;
extern crate common;

use drivers::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use hil::gpio::GPIOPin;
use drivers::timer::TimerDriver;
use nrf51822::timer::TimerAlarm;
use nrf51822::timer::ALARM1;

pub mod systick;

pub struct Platform {
    chip: nrf51822::chip::Nrf51822,
    gpio: &'static drivers::gpio::GPIO<'static, nrf51822::gpio::GPIOPin>,
    timer: &'static TimerDriver<'static, VirtualMuxAlarm<'static, TimerAlarm>>,
}

pub struct DummyMPU;

impl DummyMPU {
    pub fn set_mpu(&mut self, _: u32, _: u32, _: u32, _: bool, _: u32) {
    }
}

impl Platform {
    pub unsafe fn service_pending_interrupts(&mut self) {
        self.chip.service_pending_interrupts()
    }

    pub unsafe fn has_pending_interrupts(&mut self) -> bool {
        self.chip.has_pending_interrupts()
    }

    pub fn mpu(&mut self) -> DummyMPU {
        DummyMPU
    }

    #[inline(never)]
    pub fn with_driver<F, R>(&mut self, driver_num: usize, f: F) -> R where
            F: FnOnce(Option<&hil::Driver>) -> R {
        match driver_num {
            1 => f(Some(self.gpio)),
           // 3 => f(Some(self.timer)),
            _ => f(None)
        }
    }
}

macro_rules! static_init {
    ($V:ident : $T:ty = $e:expr, $size:expr) => {
        // Ideally we could use mem::size_of<$T> here instead of $size, however
        // that is not currently possible in rust. Instead we write the size as
        // a constant in the code and use compile-time verification to see that
        // we got it right
        let $V : &'static mut $T = {
            use core::{mem, ptr};
            // This is our compile-time assertion. The optimizer should be able
            // to remove it from the generated code.
            let assert_buf: [u8; $size] = mem::uninitialized();
            let assert_val: $T = mem::transmute(assert_buf);
            mem::forget(assert_val);

            // Statically allocate a read-write buffer for the value, write our
            // initial value into it (without dropping the initial zeros) and
            // return a reference to it.
            static mut BUF: [u8; $size] = [0; $size];
            let mut tmp : &mut $T = mem::transmute(&mut BUF);
            ptr::write(tmp as *mut $T, $e);
            tmp
        };
    }
}

pub unsafe fn init() -> &'static mut Platform {
    use core::mem;
    use nrf51822::gpio::PORT;

    static_init!(gpio_pins : [&'static nrf51822::gpio::GPIOPin; 10] = [
            &nrf51822::gpio::PORT[18], // LED_0
            &nrf51822::gpio::PORT[19], // LED_1
            &nrf51822::gpio::PORT[0], // Top left header on EK board
            &nrf51822::gpio::PORT[1], //   |
            &nrf51822::gpio::PORT[2], //   V 
            &nrf51822::gpio::PORT[3], // 
            &nrf51822::gpio::PORT[4], //
            &nrf51822::gpio::PORT[5], // 
            &nrf51822::gpio::PORT[6], // 
            &nrf51822::gpio::PORT[7], // 
            ], 4 * 10);
    static_init!(gpio: drivers::gpio::GPIO<'static, nrf51822::gpio::GPIOPin> =
                     drivers::gpio::GPIO::new(gpio_pins), 20);
    for pin in gpio_pins.iter() {
        pin.set_client(gpio);
    }

    let alarm = &nrf51822::timer::ALARM1;
    static_init!(mux_alarm : MuxAlarm<'static, TimerAlarm> = MuxAlarm::new(&ALARM1), 16);
    alarm.set_client(mux_alarm);

    static_init!(virtual_alarm1 : VirtualMuxAlarm<'static, TimerAlarm> =
                                  VirtualMuxAlarm::new(mux_alarm), 24);
    static_init!(timer : TimerDriver<'static, VirtualMuxAlarm<'static, TimerAlarm>> =
                         TimerDriver::new(virtual_alarm1, process::Container::create()), 12);
    virtual_alarm1.set_client(timer);

    nrf51822::clock::CLOCK.low_stop();
    nrf51822::clock::CLOCK.high_stop();

    nrf51822::clock::CLOCK.low_set_source(nrf51822::clock::LowClockSource::RC);
    nrf51822::clock::CLOCK.low_start();
    nrf51822::clock::CLOCK.high_start();
    while !nrf51822::clock::CLOCK.low_started() {}
    while !nrf51822::clock::CLOCK.high_started() {}

    static_init!(platform: Platform = Platform {
        chip: nrf51822::chip::Nrf51822::new(),
        gpio: gpio,
        timer: timer
    }, 8);

    // The systick implementation currently directly accesses the low clock;
    // it should go through clock::CLOCK instead.
    systick::reset();
    systick::enable(true);
    platform
}


use core::fmt::Arguments;
#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
pub unsafe extern fn rust_begin_unwind(_args: &Arguments,
    _file: &'static str, _line: usize) -> ! {
    use support::nop;
    use hil::gpio::GPIOPin;

    let led0 = &nrf51822::gpio::PORT[18];
    let led1 = &nrf51822::gpio::PORT[19];

    led0.enable_output();
    led1.enable_output();
    loop {
        for _ in 0..100000 {
            led0.set();
            led1.set();
            nop();
        }
        for _ in 0..100000 {
            led0.clear();
            led1.clear();
            nop();
        }
    }
}
