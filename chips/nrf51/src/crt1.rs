use cortexm0::{nvic, generic_isr, SVC_Handler};

/*
 * Adapted from crt1.c which was relicensed by the original author from
 * GPLv3 to Apache 2.0.
 * The original version of the file, under GPL can be found at
 * https://github.com/SoftwareDefinedBuildings/
 *     stormport/blob/rebase0/tos/platforms/storm/stormcrt1.c
 *
 * Copyright 2016, Michael Andersen <m.andersen@eecs.berkeley.edu>
 */

/* https://github.com/NordicSemiconductor/nrf52-hardware-startup-hands-on/blob/master/
           pca10040/s132/arm5_no_packs/RTE/Device/nRF52832_xxAA/arm_startup_nrf52.s */
/* https://github.com/NordicSemiconductor/nRF52-ble-app-lbs/blob/master/
           pca10040/s132/arm5_no_packs/RTE/Device/nRF52832_xxAA/system_nrf52.c */

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

unsafe extern "C" fn unhandled_interrupt() {
    'loop0: loop {}
}


unsafe extern "C" fn hard_fault_handler() {
    'loop0: loop {}
}

#[link_section=".vectors"]
#[cfg_attr(rustfmt, rustfmt_skip)]
// no_mangle Ensures that the symbol is kept until the final binary
#[no_mangle]
pub static BASE_VECTORS: [unsafe extern fn(); 16] = [
    _estack, reset_handler,
    /* NMI */           unhandled_interrupt,
    /* Hard Fault */    hard_fault_handler,
    /* MemManage */     unhandled_interrupt,
    /* BusFault */      unhandled_interrupt,
    /* UsageFault*/     unhandled_interrupt,
    unhandled_interrupt, unhandled_interrupt, unhandled_interrupt,
    unhandled_interrupt,
    /* SVC */           SVC_Handler,
    /* DebugMon */      unhandled_interrupt,
    unhandled_interrupt,
    /* PendSV */        unhandled_interrupt,
    /* SysTick */       unhandled_interrupt,
];

#[link_section=".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 80] = [generic_isr; 80];

#[no_mangle]
pub unsafe extern "C" fn init() {
    let mut current_block;
    let mut p_src: *mut u32;
    let mut p_dest: *mut u32;

    // Apply early initialization workarounds for anomalies documented on
    // 2015-12-11 nRF52832 Errata v1.2
    // http://infocenter.nordicsemi.com/pdf/nRF52832_Errata_v1.2.pdf

    // Workaround for Errata 12
    // "COMP: Reference ladder not correctly callibrated" found at the Errate doc
    *(0x40013540i32 as (*mut u32)) = (*(0x10000324i32 as (*mut u32)) & 0x1f00u32) >> 8i32;

    // Workaround for Errata 16
    // "System: RAM may be corrupt on wakeup from CPU IDLE" found at the Errata doc
    *(0x4007c074i32 as (*mut u32)) = 3131961357u32;

    // Workaround for Errata 31
    // "CLOCK: Calibration values are not correctly loaded from FICR at reset"
    // found at the Errata doc
    *(0x4000053ci32 as (*mut u32)) = (*(0x10000244i32 as (*mut u32)) & 0xe000u32) >> 13i32;

    // Workaround for Errata 32
    // "DIF: Debug session automatically enables TracePort pins" found at the Errata doc
    //    CoreDebug->DEMCR &= ~CoreDebug_DEMCR_TRCENA_Msk;

    // Workaround for Errata 36
    // "CLOCK: Some registers are not reset when expected" found at the Errata doc
    //    NRF_CLOCK->EVENTS_DONE = 0;
    //    NRF_CLOCK->EVENTS_CTTO = 0;
    //    NRF_CLOCK->CTIV = 0;
    // }

    // Workaround for Errata 37
    // "RADIO: Encryption engine is slow by default" found at the Errata document doc
    *(0x400005a0i32 as (*mut u32)) = 0x3u32;

    // Workaround for Errata 57
    // "NFCT: NFC Modulation amplitude" found at the Errata doc
    *(0x40005610i32 as (*mut u32)) = 0x5u32;
    *(0x40005688i32 as (*mut u32)) = 0x1u32;
    *(0x40005618i32 as (*mut u32)) = 0x0u32;
    *(0x40005614i32 as (*mut u32)) = 0x3fu32;

    // Workaround for Errata 66
    // "TEMP: Linearity specification not met with default settings" found at the Errata doc
    //     NRF_TEMP->A0 = NRF_FICR->TEMP.A0;
    //     NRF_TEMP->A1 = NRF_FICR->TEMP.A1;
    //     NRF_TEMP->A2 = NRF_FICR->TEMP.A2;
    //     NRF_TEMP->A3 = NRF_FICR->TEMP.A3;
    //     NRF_TEMP->A4 = NRF_FICR->TEMP.A4;
    //     NRF_TEMP->A5 = NRF_FICR->TEMP.A5;
    //     NRF_TEMP->B0 = NRF_FICR->TEMP.B0;
    //     NRF_TEMP->B1 = NRF_FICR->TEMP.B1;
    //     NRF_TEMP->B2 = NRF_FICR->TEMP.B2;
    //     NRF_TEMP->B3 = NRF_FICR->TEMP.B3;
    //     NRF_TEMP->B4 = NRF_FICR->TEMP.B4;
    //     NRF_TEMP->B5 = NRF_FICR->TEMP.B5;
    //     NRF_TEMP->T0 = NRF_FICR->TEMP.T0;
    //     NRF_TEMP->T1 = NRF_FICR->TEMP.T1;
    //     NRF_TEMP->T2 = NRF_FICR->TEMP.T2;
    //     NRF_TEMP->T3 = NRF_FICR->TEMP.T3;
    //     NRF_TEMP->T4 = NRF_FICR->TEMP.T4;
    // }

    // Workaround for Errata 108
    // "RAM: RAM content cannot be trusted upon waking up from System ON Idle
    // or System OFF mode" found at the Errata doc
    *(0x40000ee4i32 as (*mut u32)) = *(0x10000258i32 as (*mut u32)) & 0x4fu32;


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
    nvic::enable_all();
}
