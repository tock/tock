#[repr(C)]
struct GpioRegisters {
    _r0: [u8; 0x90],
    _dout_set31_0: VolatileCell<u32>,
    _r1: [u8; 0xC],
    _dout_clr31_0: Volatile<u32>,
    _r2: [u8; 0xC],
    _dout_tgl31_0: Volatile<u32>,
    _r3: [u8; 0xC],
    _din31_0: Volatile<u32>,
    _r4: [u8; 0xC],
    _doe31_0: Volatile<u32>,
    _r5: [u8; 0xC],
    _evflags31_0: Volatile<u32>,
}
