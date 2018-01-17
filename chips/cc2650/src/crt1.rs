use cortexm4::{generic_isr, nvic, systick_handler, SVC_Handler};

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
    /* SysTick */       systick_handler
];

#[link_section = ".vectors"]
#[no_mangle] // Ensures that the symbol is kept until the final binary
pub static IRQS: [unsafe extern "C" fn(); 50] = [generic_isr; 50];

#[cfg(target_os = "none")]
#[no_mangle]
pub unsafe extern "C" fn enable_gpio() {
    asm!("\
        mov r0, 0x500
        bl 0x100001D4
    ")
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let mut current_block;
    let mut p_src: *mut u32;
    let mut p_dest: *mut u32;

    const ROM_API_TABLE: u32 = 0x10000180;
    const ROM_API_PRCM_TABLE: u32 = ROM_API_TABLE + 14*4;
    let prc_table_ptr = ROM_API_PRCM_TABLE + 7*4;

    enable_gpio();

    let iocbase = 0x40081000;
    let iocfg10 = iocbase + 0x28;

    let gpiobase = 0x40022000;
    let doe = gpiobase + 0xD0;
    //let dio8to10 = gpiobase + 0x08;

    // Set DIO10 to output
    *(iocfg10 as *mut u16) = 0x7000;
    // Set DataEnable to 1
    *(doe as *mut u32) = 0x400;
    *((gpiobase + 0x00000090) as *mut u32) = 1 << 10;
    loop { }

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
