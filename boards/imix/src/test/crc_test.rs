//! Test that CRC is working properly.
//!
//! To test, add the following line to the imix boot sequence:
//! ```
//!     test::crc_test::run_crc();
//! ```
//! You should see the following output:
//! ```
//!     crc_test passed (Crc32)
//!     crc_test passed (Crc32C)
//!     crc_test passed (Crc16CITT)
//!
//! ```
//!
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
    let data = static_init!([u8; 387], [0; 387]);

    static_init!(
        TestCrc<'static, Crccu>,
        TestCrc::new(&crc, data)
    )
}

