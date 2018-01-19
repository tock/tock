#[repr(C)]
struct IocRegisters {
    _iocfg: [VolatileCell<u32>; 32],
}
