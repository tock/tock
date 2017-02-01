use kernel::common::take_cell::{MapCell,TakeCell};

pub unsafe fn test_take_map_cell() {
    static FOO: u32 = 1234;
    static mut tc_ref: TakeCell<&'static u32> = TakeCell::new(&FOO);
    test_take_cell(&tc_ref);

    static mut tc1: TakeCell<[[u8; 256]; 1]> = TakeCell::new([[125; 256]; 1]);
    test_take_cell(&tc1);

    static mut tc2: TakeCell<[[u8; 256]; 2]> = TakeCell::new([[125; 256]; 2]);
    test_take_cell(&tc2);

    static mut tc3: TakeCell<[[u8; 256]; 3]> = TakeCell::new([[125; 256]; 3]);
    test_take_cell(&tc3);

    static mut tc4: TakeCell<[[u8; 256]; 4]> = TakeCell::new([[125; 256]; 4]);
    test_take_cell(&tc4);

    static mut tc5: TakeCell<[[u8; 256]; 5]> = TakeCell::new([[125; 256]; 5]);
    test_take_cell(&tc5);

    static mut tc6: TakeCell<[[u8; 256]; 6]> = TakeCell::new([[125; 256]; 6]);
    test_take_cell(&tc6);

    static mut tc7: TakeCell<[[u8; 256]; 7]> = TakeCell::new([[125; 256]; 7]);
    test_take_cell(&tc7);

    static mut mc_ref: MapCell<&'static u32> = MapCell::new(&FOO);
    test_map_cell(&mc_ref);

    static mut mc1: MapCell<[[u8; 256]; 1]> = MapCell::new([[125; 256]; 1]);
    test_map_cell(&mc1);

    static mut mc2: MapCell<[[u8; 256]; 2]> = MapCell::new([[125; 256]; 2]);
    test_map_cell(&mc2);

    static mut mc3: MapCell<[[u8; 256]; 3]> = MapCell::new([[125; 256]; 3]);
    test_map_cell(&mc3);

    static mut mc4: MapCell<[[u8; 256]; 4]> = MapCell::new([[125; 256]; 4]);
    test_map_cell(&mc4);

    static mut mc5: MapCell<[[u8; 256]; 5]> = MapCell::new([[125; 256]; 5]);
    test_map_cell(&mc5);

    static mut mc6: MapCell<[[u8; 256]; 6]> = MapCell::new([[125; 256]; 6]);
    test_map_cell(&mc6);

    static mut mc7: MapCell<[[u8; 256]; 7]> = MapCell::new([[125; 256]; 7]);
    test_map_cell(&mc7);
}

#[inline(never)]
#[allow(unused_unsafe)]
unsafe fn test_take_cell<'a, A>(tc: &TakeCell<A>) {
    let dwt_ctl: *mut u32 = 0xE0001000 as *mut u32;
    let dwt_cycles: *mut u32 = 0xE0001004 as *mut u32;
    let demcr: *mut u32 = 0xE000EDFC as *mut u32;

    ::core::ptr::write_volatile(demcr, 0x01000000);
    ::core::ptr::write_volatile(dwt_cycles, 0);
    ::core::ptr::write_volatile(dwt_ctl, ::core::ptr::read_volatile(dwt_ctl) | 1);
    tc.map(|_| ());
    let end = ::core::ptr::read_volatile(dwt_cycles);
    println!("time: {}, size: {}", end, ::core::mem::size_of_val(tc));
}

#[inline(never)]
#[allow(unused_unsafe)]
unsafe fn test_map_cell<'a, A>(tc: &MapCell<A>) {
    let dwt_ctl: *mut u32 = 0xE0001000 as *mut u32;
    let dwt_cycles: *mut u32 = 0xE0001004 as *mut u32;
    let demcr: *mut u32 = 0xE000EDFC as *mut u32;

    ::core::ptr::write_volatile(demcr, 0x01000000);
    ::core::ptr::write_volatile(dwt_cycles, 0);
    ::core::ptr::write_volatile(dwt_ctl, ::core::ptr::read_volatile(dwt_ctl) | 1);
    tc.map(|_| ());
    let end = ::core::ptr::read_volatile(dwt_cycles);
    println!("time: {}, size: {}", end, ::core::mem::size_of_val(tc));
}

