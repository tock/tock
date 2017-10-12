
/*
 * Adapted from crt1.c which was relicensed by the original author from
 * GPLv3 to Apache 2.0.
 * The original version of the file, under GPL can be found at
 * https://github.com/SoftwareDefinedBuildings/
 *     stormport/blob/rebase0/tos/platforms/storm/stormcrt1.c
 *
 * Copyright 2016, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

extern "C" {
    fn ECB_Handler();
    fn GPIOTE_Handler();
    fn RADIO_Handler();
    fn RNG_Handler();
    fn RTC1_Handler();
    fn SVC_Handler();
    fn TEMP_Handler();
    fn TIMER0_Handler();
    fn TIMER1_Handler();
    fn TIMER2_Handler();
    fn UART0_Handler();

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

#[no_mangle]
pub unsafe extern "C" fn dummy_handler() {
    'loop0: loop {}
}

#[no_mangle]
pub unsafe extern "C" fn hard_fault_handler() {
    'loop0: loop {}
}

#[link_section=".vectors"]
#[no_mangle]
pub static mut INTERRUPT_TABLE: [unsafe extern "C" fn(); 48] = [_estack,
                                                                reset_handler,
                                                                dummy_handler, // NMI_Handler
                                                                hard_fault_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                SVC_Handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler, // PendSV_Handler
                                                                dummy_handler, // SysTick_Handler
                                                                dummy_handler, // POWER_CLOCK
                                                                RADIO_Handler,
                                                                UART0_Handler,
                                                                dummy_handler, // SPI0_TWI0_Handler
                                                                dummy_handler, // SPI1_TWI1_Handler
                                                                dummy_handler,
                                                                GPIOTE_Handler,
                                                                dummy_handler, // ADC_Handler
                                                                TIMER0_Handler,
                                                                TIMER1_Handler,
                                                                TIMER2_Handler,
                                                                dummy_handler, // RTC0_Handler
                                                                TEMP_Handler,
                                                                RNG_Handler,
                                                                ECB_Handler,
                                                                dummy_handler, // CCM_AAR_Handler
                                                                dummy_handler, // WDT_Handler
                                                                RTC1_Handler,
                                                                dummy_handler, // QDEC_Handler
                                                                dummy_handler, // LPCOMP_Handler
                                                                dummy_handler, // SWI0_Handler
                                                                dummy_handler, // SWI1_Handler
                                                                dummy_handler, // SWI2_Handler
                                                                dummy_handler, // SWI3_Handler
                                                                dummy_handler, // SWI4_Handler
                                                                dummy_handler, // SWI5_Handler
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler,
                                                                dummy_handler];

#[no_mangle]
pub unsafe extern "C" fn init() {
    let mut current_block;
    let mut p_src: *mut u32;
    let mut p_dest: *mut u32;


    /* Apply early initialization workarounds for anomalies documented on
     * nRF51822-PAN v2.4. Note that they have been validated only for xxAA
     * variant. For other variants, please refer to the applicable
     * nRF51822-PAN. */

    /* Power on RAM blocks manually (PAN #16). Note that xxAA/xxAB variants
     * have only two RAM blocks. For xxAC, change to 0x0000000F. */
    *(0x40000524i32 as (*mut u32)) = 0x3u32;

    /* Setup peripherals manually (PAN #26) */
    *(0x40000504i32 as (*mut u32)) = 0xc007ffdfu32;
    *(0x40006c18i32 as (*mut u32)) = 0x8000u32;

    /* Move the relocate segment This assumes it is located after the text
     * segment, which is where the storm linker file puts it
     */
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
}
