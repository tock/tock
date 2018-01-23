#[cfg_attr(rustfmt, rustfmt_skip)]
#[no_mangle]
#[link_section = ".ccfg"]
pub static CCFG_CONF: [u32; 22] = [
    0xFFFFFFFF,
    0xFFFBFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xC5FFFFFF,
    0xFFFFFFFF,
    0xFFFFFFC5,
    0xFFC5C5C5,
    0xFFC5C5C5,
    0x00000000, // Set the image as valid
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFF,
];
