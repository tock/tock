//! This tests an underlying 32-bit entropy generator and the library
//! transformations between 8-bit and 32-bit entropy. To run this test,
//! add this line to the imix boot sequence:
//! ```
//!     rng_test::run_entropy32();
//! ```
//! This test takes a 32-bit entropy generator, puts its output into a 
//! 32-8 conversion to be an 8-bit generator, puts that output into an 
//! 8-to-32 conversion to be a 32-bit generator again, and makes this final
//! 32-bit entropy source be the tested RNG. This therefore tests not only 
//! the underlying entropy source but also the conversion library.
//!
//! The expected output is a series of random numbers that should be
//! different on each invocation. Rigorous entropy tests are outside
//! the scope of this test.

use capsules::rng;
use capsules::test::rng::TestRng;
use kernel::hil::entropy::{Entropy32, Entropy8};
use kernel::hil::rng::Rng;
use sam4l::trng::TRNG;

pub unsafe fn run_entropy32() {
    let t = static_init_test_entropy32();
    t.run();
}

unsafe fn static_init_test_entropy32() -> &'static TestRng<'static> {
    let e1 = static_init!(rng::Entropy32To8<'static>, rng::Entropy32To8::new(&TRNG));
    TRNG.set_client(e1);
    let e2 = static_init!(rng::Entropy8To32<'static>, rng::Entropy8To32::new(e1));
    e1.set_client(e2);
    let er = static_init!(
        rng::Entropy32ToRandom<'static>,
        rng::Entropy32ToRandom::new(e2)
    );
    e2.set_client(er);
    let test = static_init!(TestRng<'static>, TestRng::new(er));
    er.set_client(test);
    test
}
