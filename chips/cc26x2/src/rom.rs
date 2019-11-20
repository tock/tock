use enum_primitive::cast::FromPrimitive;
use enum_primitive::enum_from_primitive;
use kernel::common::StaticRef;

// Pretty much this whole file is extracted from
//     ~/ti/simplelink_cc13x2_sdk_2_20_00_71/source/ti/devices/cc13x2_cc26x2_v1/driverlib/rom.h
// The basic idea is that there are some special "TI Driver Lib" functions that exist in the ROM

// From the datasheet: "The ROM contains a serial bootloader with SPI and UART support (see Chapter 10)
// as well as a Driver Library and an RF stack support. For details, see Section 5.6."

#[repr(C)]
pub struct HARD_API {
    crc32: unsafe extern "C" fn(*mut u8, u32, u32) -> u32,
    flag_get_size: unsafe extern "C" fn() -> u32,
    pub get_chip_id: unsafe extern "C" fn() -> u32,
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
    pub select_comp_a_input: unsafe extern "C" fn(CompaIn),
    pub select_comp_a_ref: unsafe extern "C" fn(CompaRef),
    pub select_adc_comp_b_input: unsafe extern "C" fn(AdcCompbIn),
    pub select_dac_vref: unsafe extern "C" fn(DacRef),
}

const ROM_HAPI_TABLE_ADDR: usize = 0x1000_0048;

// struct that carries the hardware API
pub const HAPI: StaticRef<HARD_API> =
    unsafe { StaticRef::new(ROM_HAPI_TABLE_ADDR as *const HARD_API) };

// Defines for input parameter to the select_comp_a_input function.
// The define values can not be changed!
enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum CompaIn {
    Nc = 0x00,
    Auxio7 = 0x09,
    Auxio6 = 0x0A,
    Auxio5 = 0x0B,
    Auxio4 = 0x0C,
    Auxio3 = 0x0D,
    Auxio2 = 0x0E,
    Auxio1 = 0x0F,
    Auxio0 = 0x10,
}
}

// Defines for input parameter to the select_comp_a_ref function.
// The define values can not be changed!
enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum CompaRef {
    Nc = 0x00,
    Dcoupl = 0x01,
    Vss = 0x02,
    Vdds = 0x03,
    Adcvrefp = 0x04,
    Auxio7 = 0x09,
    Auxio6 = 0x0A,
    Auxio5 = 0x0B,
    Auxio4 = 0x0C,
    Auxio3 = 0x0D,
    Auxio2 = 0x0E,
    Auxio1 = 0x0F,
    Auxio0 = 0x10,
}
}

// Defines for input parameter to the select_adc_comp_b_input function.
// The define values can not be changed!
enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum AdcCompbIn {
    Nc = 0x00,
    Dcoupl = 0x03,
    Vss = 0x04,
    Vdds = 0x05,
    Auxio7 = 0x09,
    Auxio6 = 0x0A,
    Auxio5 = 0x0B,
    Auxio4 = 0x0C,
    Auxio3 = 0x0D,
    Auxio2 = 0x0E,
    Auxio1 = 0x0F,
    Auxio0 = 0x10,
}
}

// Defines for input parameter to the select_dac_vref function.
// The define values can not be changed!
enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum DacRef {
    Nc = 0x00,
    Dcoupl = 0x01,
    Vss = 0x02,
    Vdds = 0x03,
}
}
