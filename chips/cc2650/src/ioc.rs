#[repr(C)]
struct IocRegisters {
    _iocfg: [VolatileCell<u32>; 32],
}

const IOC_BASE: usize = 0x4008_1000;
