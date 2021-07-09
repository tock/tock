//! Test that CRC is working properly.
//!
//! To test, add the following line to the imix boot sequence:
//! ```
//!     test::crc_test::run_crc();
//! ```
//! You should see the following output:
//! ```
//!     CRC32: 0xcbf43926
//!     CRC32C: 0xe3069283
//!     CRC16CITT: 0x89f6
//!
//! ```
//!
//! These results are for computing the CRC over the string
//! "123456789" (not including the quotes). The result values were
//! taken from
//! <https://reveng.sourceforge.io/crc-catalogue/17plus.htm>

use capsules::test::crc::TestCrc;
use kernel::hil::crc::Crc;
use kernel::static_init;
use sam4l::crccu::Crccu;

pub unsafe fn run_crc(crc: &'static Crccu) {
    let t = static_init_crc(crc);
    crc.set_client(t);

    t.run();
}

unsafe fn static_init_crc(crc: &'static Crccu) -> &'static TestCrc<'static, Crccu<'static>> {
    let data = static_init!([u8; 9], [0; 9]);

    for i in 0..9 {
        data[i] = i as u8 + ('1' as u8);
    }
    static_init!(TestCrc<'static, Crccu>, TestCrc::new(&crc, data))
}
