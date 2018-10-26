// To use this test, you need to modify FieldValue.value to be public
// and uncomment test

static CCFG_CONF: [u32; 22] = [
    0x01800000, 0xFF820010, 0x0058FFFE, 0xF3FBFF3A, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
    0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0x00FFFFFF, 0xFFFFFFFF, 0xFFFFFF00, 0xFFC5C5C5,
    0xFFC5C5C5, 0x00000000, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF,
];

use cc26x2::ccfg;
use kernel::common::registers::ReadWrite;

// unfortunately, I was unable to insert this into launchxl/src/ccfg.rs bc it forces you to include the cc26x2 crate
// the cc26x2 crate brings in crt1.rs which has a link definition which freaks out the compiler when you're building
const CCFG: ccfg::Registers = ccfg::Registers::new(ccfg::RegisterInitializer {
    ext_lf_clk: ReadWrite::new(0x01800000),
    mode_conf0: ReadWrite::new(0xF3FBFF3A),
    mode_conf1: ReadWrite::new(0xFF820010),
    bl_config: ReadWrite::new(0x00FFFFFF),
});

// pub fn test() {
//     unsafe {
//         let raw_array: *const u32 = &CCFG_CONF[0] as *const u32;
//         let constructed = &CCFG.ext_lf_clk.value as *const u32;

//         for n in 0..22 {
//             let raw = *(raw_array.offset(n));
//             let new = *(constructed.offset(n));
//             if raw != new {
//                 debug!(
//                     "Mismatch at location {:} : OLD {:x} != {:x} NEW",
//                     n, raw, new
//                 );
//             }
//         }
//     }
// }
