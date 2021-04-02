//! Tock kernel for the Raspberry Pi Pico.
//!
//! It is based on RP2040SoC SoC (Cortex M0+).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![deny(missing_docs)]
#![feature(asm, naked_functions)]

use enum_primitive::cast::FromPrimitive;

use kernel::component::Component;
use kernel::{capabilities, create_capability, static_init, Kernel, Platform};
use kernel::hil::time::{AlarmClient,Time, Alarm};
use kernel::hil::gpio::Output;

use rp2040;
use rp2040::chip::{Rp2040, Rp2040DefaultPeripherals};
use rp2040::clocks::{
    AdcAuxiliaryClockSource, PeripheralAuxiliaryClockSource, PllClock,
    ReferenceAuxiliaryClockSource, ReferenceClockSource, RtcAuxiliaryClockSource,
    SystemAuxiliaryClockSource, SystemClockSource, UsbAuxiliaryClockSource,
};
use rp2040::gpio::{RPGpio, RPGpioPin};
use rp2040::resets::Peripheral;
use rp2040::timer::RPAlarm;
mod io;

mod flash_bootloader;

extern "C" {
    static _stext: *const u32;
}

/// Allocate memory for the stack
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

// Manually setting the boot header section that contains the FCB header
#[used]
#[link_section = ".flash_bootloader"]
static FLASH_BOOTLOADER: [u8; 256] = flash_bootloader::FLASH_BOOTLOADER;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 4;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static Rp2040<Rp2040DefaultPeripherals>> = None;

/// Supported drivers by the platform
pub struct RaspberryPiPico {
    ipc: kernel::ipc::IPC<NUM_PROCS>,
    
}

impl Platform for RaspberryPiPico {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            kernel::ipc::DRIVER_NUM => f(Some(&self.ipc)),
            _ => f(None),
        }
    }
}

// struct AlarmTest<'a>{
//     alarm: &'a RPAlarm<'a>,
//     led: RPGpioPin<'a>
// }

// impl AlarmClient for AlarmTest<'_>{
//     fn alarm (&self){
//         self.led.toggle();
//         self.alarm.set_alarm(self.alarm.now(), <RPAlarm as Time>::ticks_from_ms(1000));
//     }
// }

/// Entry point used for debuger
#[no_mangle]
#[naked]
pub unsafe extern "C" fn reset() {
    asm!(
        "
    movs r0, #0
    ldr r1, =(0xe0000000 + 0x0000ed08)
    str r0, [r1]
    ldmia r0!, {{r1, r2}}
    msr msp, r1
    bx r2
    ",
        options(noreturn)
    );
}

fn init_clocks(peripherals: &Rp2040DefaultPeripherals) {
    // Disable the Resus clock
    peripherals.clocks.disable_resus();

    // Setup the external Osciallator
    peripherals.xosc.init();

    // disable ref and sys clock aux sources
    peripherals.clocks.disable_sys_aux();
    peripherals.clocks.disable_ref_aux();

    peripherals
        .resets
        .reset(&[Peripheral::PllSys, Peripheral::PllUsb]);
    peripherals
        .resets
        .unreset(&[Peripheral::PllSys, Peripheral::PllUsb], true);

    // Configure PLLs (from Pico SDK)
    //                   REF     FBDIV VCO            POSTDIV
    // PLL SYS: 12 / 1 = 12MHz * 125 = 1500MHZ / 6 / 2 = 125MHz
    // PLL USB: 12 / 1 = 12MHz * 40  = 480 MHz / 5 / 2 =  48MHz

    // It seems that the external osciallator is clocked at 12 MHz

    peripherals
        .clocks
        .pll_init(PllClock::Sys, 12, 1, 1500 * 1000000, 6, 2);
    peripherals
        .clocks
        .pll_init(PllClock::Usb, 12, 1, 480 * 1000000, 5, 2);

    // pico-sdk: // CLK_REF = XOSC (12MHz) / 1 = 12MHz
    peripherals.clocks.configure_reference(
        ReferenceClockSource::Xosc,
        ReferenceAuxiliaryClockSource::PllUsb,
        12000000,
        12000000,
    );
    // pico-sdk: CLK SYS = PLL SYS (125MHz) / 1 = 125MHz
    peripherals.clocks.configure_system(
        SystemClockSource::Auxiliary,
        SystemAuxiliaryClockSource::PllSys,
        125000000,
        125000000,
    );
    // pico-sdk: CLK USB = PLL USB (48MHz) / 1 = 48MHz
    peripherals
        .clocks
        .configure_usb(UsbAuxiliaryClockSource::PllSys, 48000000, 48000000);
    // pico-sdk: CLK ADC = PLL USB (48MHZ) / 1 = 48MHz
    peripherals
        .clocks
        .configure_adc(AdcAuxiliaryClockSource::PllUsb, 48000000, 48000000);
    // pico-sdk: CLK RTC = PLL USB (48MHz) / 1024 = 46875Hz
    peripherals
        .clocks
        .configure_rtc(RtcAuxiliaryClockSource::PllSys, 48000000, 46875);
    // pico-sdk:
    // CLK PERI = clk_sys. Used as reference clock for Peripherals. No dividers so just select and enable
    // Normally choose clk_sys or clk_usb
    peripherals
        .clocks
        .configure_peripheral(PeripheralAuxiliaryClockSource::System, 125000000);
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn main() {
    // Loads relocations and clears BSS
    rp2040::init();

    let peripherals = static_init!(Rp2040DefaultPeripherals, Rp2040DefaultPeripherals::new());

    // Reset all peripherals except QSPI (we might be booting from Flash), PLL USB and PLL SYS
    peripherals.resets.reset_all_except(&[
        Peripheral::IOQSpi,
        Peripheral::PadsQSpi,
        Peripheral::PllUsb,
        Peripheral::PllSys,
    ]);

    // Unreset all the peripherals that do not require clock setup as they run using the sys_clk or ref_clk
    // Wait for the peripherals to reset
    peripherals.resets.unreset_all_except(
        &[
            Peripheral::Adc,
            Peripheral::Rtc,
            Peripheral::Spi0,
            Peripheral::Spi1,
            Peripheral::Uart0,
            Peripheral::Uart1,
            Peripheral::UsbCtrl,
        ],
        true,
    );

    init_clocks(&peripherals);

    // Unreset all peripherals
    peripherals.resets.unreset_all_except(&[], true);

    // Disable IE for pads 26-29 (the Pico SDK runtime does this, not sure why)
    for pin in 26..30 {
        let gpio = RPGpioPin::new(RPGpio::from_usize(pin).unwrap());
        gpio.deactivate_pads();
    }

    use kernel::hil::gpio::{Configure, Output};
    use kernel::hil::time::{Alarm, Time};

    // fn off (){
    //     let pin = RPGpioPin::new(RPGpio::GPIO25);
    //     pin.make_output();
    //     pin.clear();
    // }

    // let pin = RPGpioPin::new(RPGpio::GPIO25);
    // pin.make_output();
    // pin.set();

    // let pin = RPGpioPin::new(RPGpio::GPIO25);
    // pin.make_output();
    // pin.set();

    // let at = static_init!(AlarmTest, AlarmTest{
    //     alarm:&peripherals.alarm,
    //     led: pin
    // });
    // peripherals.alarm.set_alarm_client(at);
    // peripherals.alarm.set_alarm(peripherals.alarm.now(), <RPAlarm as Time>::ticks_from_ms(1000));

    let chip = static_init!(Rp2040<Rp2040DefaultPeripherals>, Rp2040::new(peripherals));

    CHIP = Some(chip);

    let board_kernel = static_init!(Kernel, Kernel::new(&PROCESSES));
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let raspberry_pi_pico = RaspberryPiPico {
        ipc: kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability),
    };

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));

    board_kernel.kernel_loop(
        &raspberry_pi_pico,
        chip,
        Some(&raspberry_pi_pico.ipc),
        scheduler,
        &main_loop_capability,
    );
}
