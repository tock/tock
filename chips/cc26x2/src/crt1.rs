use cortexm4::{
    disable_specific_nvic, generic_isr, hard_fault_handler, nvic, set_privileged_thread,
    stash_process_state, svc_handler, systick_handler,
};
use setup;

extern "C" {
    // Symbols defined in the linker file
    static mut _erelocate: u32;
    static mut _etext: u32;
    static mut _ezero: u32;
    static mut _srelocate: u32;
    static mut _szero: u32;
    fn reset_handler();

    // _estack is not really a function, but it makes the types work
    // You should never actually invoke it!!
    fn _estack();
}

use events::set_event_flag_from_isr;
macro_rules! generic_isr {
    ($label:tt, $priority:expr) => {
        #[cfg(target_os = "none")]
        #[naked]
        unsafe extern "C" fn $label() {
            stash_process_state();
            set_event_flag_from_isr($priority);
            disable_specific_nvic();
            set_privileged_thread();
        }
    };
}

macro_rules! custom_isr {
    ($label:tt, $priority:expr, $isr:ident) => {
        #[cfg(target_os = "none")]
        #[naked]
        unsafe extern "C" fn $label() {
            stash_process_state();
            set_event_flag_from_isr($priority);
            $isr();
            set_privileged_thread();
        }
    };
}

use event_priority::EVENT_PRIORITY;
generic_isr!(gpio_nvic, EVENT_PRIORITY::GPIO);
generic_isr!(i2c0_nvic, EVENT_PRIORITY::I2C0);
generic_isr!(aon_rtc_nvic, EVENT_PRIORITY::AON_RTC);
generic_isr!(rfc_cpe0_isr, EVENT_PRIORITY::RF_CORE_CPE0);
generic_isr!(rfc_cpe1_isr, EVENT_PRIORITY::RF_CORE_CPE1);
generic_isr!(rfc_hw_isr, EVENT_PRIORITY::RF_CORE_HW);
generic_isr!(rfc_cmd_ack_isr, EVENT_PRIORITY::RF_CMD_ACK);
generic_isr!(osc_isr, EVENT_PRIORITY::OSC);
generic_isr!(adc_complete, EVENT_PRIORITY::AUX_ADC);

use uart::{uart0_isr, uart1_isr};
custom_isr!(uart0_nvic, EVENT_PRIORITY::UART0, uart0_isr);
custom_isr!(uart1_nvic, EVENT_PRIORITY::UART1, uart1_isr);

unsafe extern "C" fn unhandled_interrupt() {
    'loop0: loop {}
}

#[link_section = ".vectors"]
// used Ensures that the symbol is kept until the final binary
#[used]
pub static BASE_VECTORS: [unsafe extern "C" fn(); 54] = [
    _estack,
    reset_handler,
    unhandled_interrupt, // NMI
    hard_fault_handler,  // Hard Fault
    unhandled_interrupt, // MPU fault
    unhandled_interrupt, // Bus fault
    unhandled_interrupt, // Usage fault
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // Reserved
    svc_handler,         // SVC
    unhandled_interrupt, // Debug monitor,
    unhandled_interrupt, // Reserved
    unhandled_interrupt, // PendSV
    systick_handler,     // Systick
    gpio_nvic,           // GPIO Int handler
    i2c0_nvic,           // I2C0
    rfc_cpe1_isr,        // RF Core Command & Packet Engine 1
    generic_isr,         // AON SpiSplave Rx, Tx and CS
    aon_rtc_nvic,        // AON RTC
    uart0_nvic,          // UART0 Rx and Tx
    generic_isr,         // AUX software event 0
    generic_isr,         // SSI0 Rx and Tx
    generic_isr,         // SSI1 Rx and Tx
    rfc_cpe0_isr,        // RF Core Command & Packet Engine 0
    rfc_hw_isr,          // RF Core Hardware
    rfc_cmd_ack_isr,     // RF Core Command Acknowledge
    generic_isr,         // I2S
    generic_isr,         // AUX software event 1
    generic_isr,         // Watchdog timer
    generic_isr,         // Timer 0 subtimer A
    generic_isr,         // Timer 0 subtimer B
    generic_isr,         // Timer 1 subtimer A
    generic_isr,         // Timer 1 subtimer B
    generic_isr,         // Timer 2 subtimer A
    generic_isr,         // Timer 2 subtimer B
    generic_isr,         // Timer 3 subtimer A
    generic_isr,         // Timer 3 subtimer B
    generic_isr,         // Crypto Core Result available
    generic_isr,         // uDMA Software
    generic_isr,         // uDMA Error
    generic_isr,         // Flash controller
    generic_isr,         // Software Event 0
    generic_isr,         // AUX combined event
    generic_isr,         // AON programmable 0
    generic_isr,         // Dynamic Programmable interrupt
    // source (Default: PRCM)
    generic_isr,  // AUX Comparator A
    adc_complete, // AUX ADC new sample or ADC DMA
    // done, ADC underflow, ADC overflow
    generic_isr, // TRNG event (hw_ints.h 49)
    osc_isr,
    generic_isr,
    uart1_nvic, //uart1_generic_isr,//uart::uart1_isr, // 52 allegedly UART1 (http://e2e.ti.com/support/wireless_connectivity/proprietary_sub_1_ghz_simpliciti/f/156/t/662981?CC1312R-UART1-can-t-work-correctly-in-sensor-oad-cc1312lp-example-on-both-cc1312-launchpad-and-cc1352-launchpad)
    generic_isr,
];

#[no_mangle]
pub unsafe extern "C" fn init() {
    let mut current_block;
    let mut p_src: *mut u32;
    let mut p_dest: *mut u32;

    // Move the relocate segment. This assumes it is located after the text
    // segment, which is where the storm linker file puts it
    p_src = &mut _etext as (*mut u32);
    p_dest = &mut _srelocate as (*mut u32);
    if p_src != p_dest {
        current_block = 1;
    } else {
        current_block = 2;
    }
    'loop1: loop {
        if current_block == 1 {
            if !(p_dest < &mut _erelocate as (*mut u32)) {
                current_block = 2;
                continue;
            }
            *{
                let _old = p_dest;
                p_dest = p_dest.offset(1isize);
                _old
            } = *{
                let _old = p_src;
                p_src = p_src.offset(1isize);
                _old
            };
            current_block = 1;
        } else {
            p_dest = &mut _szero as (*mut u32);
            break;
        }
    }
    'loop3: loop {
        if !(p_dest < &mut _ezero as (*mut u32)) {
            break;
        }
        *{
            let _old = p_dest;
            p_dest = p_dest.offset(1isize);
            _old
        } = 0u32;
    }

    setup::perform();
    nvic::enable_all();
}
