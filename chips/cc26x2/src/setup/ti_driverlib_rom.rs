use kernel::common::StaticRef;

#[repr(C)]
pub struct RomFuncTable {
    crc32: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    flag_get_size: unsafe extern "C" fn() -> u32,
    get_chip_id: unsafe extern "C" fn() -> u32,
    _reserved_location_1: unsafe extern "C" fn(u32) -> u32,
    _reserved_location_2: unsafe extern "C" fn() -> u32,
    _reserved_location_3: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    reset_device: unsafe extern "C" fn(),
    fletcher32: unsafe extern "C" fn(*mut u16, u16, u16) -> u32,
    min_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    max_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    mean_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    standard_deviation_value: unsafe extern "C" fn(*mut u32, u32) -> u32,
    _reserved_location_4: unsafe extern "C" fn(u32),
    _reserved_location_5: unsafe extern "C" fn(u32),
    pub hf_source_safe_switch: unsafe extern "C" fn(),
    pub select_comp_a_input: unsafe extern "C" fn(u8),
    pub select_comp_a_ref: unsafe extern "C" fn(u8),
    pub select_adc_comp_b_input: unsafe extern "C" fn(u8),
    pub select_dac_vref: unsafe extern "C" fn(u8),
}

const ROM_HAPI_TABLE_ADDR: usize = 0x1000_0048;

pub const ROM_HAPI: StaticRef<RomFuncTable> =
    unsafe { StaticRef::new(ROM_HAPI_TABLE_ADDR as *const RomFuncTable) };


