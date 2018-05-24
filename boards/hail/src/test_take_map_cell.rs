use kernel::common::cells::MapCell;

pub unsafe fn test_take_map_cell() {
    static FOO: u32 = 1234;

    static mut MC_REF: MapCell<&'static u32> = MapCell::new(&FOO);
    test_map_cell(&MC_REF);

    static mut MC1: MapCell<[[u8; 256]; 1]> = MapCell::new([[125; 256]; 1]);
    test_map_cell(&MC1);

    static mut MC2: MapCell<[[u8; 256]; 2]> = MapCell::new([[125; 256]; 2]);
    test_map_cell(&MC2);

    static mut MC3: MapCell<[[u8; 256]; 3]> = MapCell::new([[125; 256]; 3]);
    test_map_cell(&MC3);

    static mut MC4: MapCell<[[u8; 256]; 4]> = MapCell::new([[125; 256]; 4]);
    test_map_cell(&MC4);

    static mut MC5: MapCell<[[u8; 256]; 5]> = MapCell::new([[125; 256]; 5]);
    test_map_cell(&MC5);

    static mut MC6: MapCell<[[u8; 256]; 6]> = MapCell::new([[125; 256]; 6]);
    test_map_cell(&MC6);

    static mut MC7: MapCell<[[u8; 256]; 7]> = MapCell::new([[125; 256]; 7]);
    test_map_cell(&MC7);
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
    debug!("time: {}, size: {}", end, ::core::mem::size_of_val(tc));
}
