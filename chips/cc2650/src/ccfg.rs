#[cfg_attr(rustfmt, rustfmt_skip)]
#[no_mangle]
#[link_section = ".ccfg"]
pub static CCFG_CONF: [u32; 22] = [
    0x01800000,
    0xFF820010,
    0x0058FFFD,
    0xF3BFFF3A,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0x00FFFFFF,
    0xFFFFFFFF,
    0xFFFFFF00,
    0xFFC500C5,
    0xFF000000,
    0x00000000, // Set image as valid
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
];
