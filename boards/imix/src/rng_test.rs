use capsules::test::rng::{TestRng32, TestRng8};
use capsules::rng;
use kernel::hil::rng::{Rng32, Rng8};
use sam4l::trng::{TRNG};

/// This tests a platform with an underlying 32-bit random number generator
/// by taking that generator, putting into a 32-8 conversion to be an 8-bit
/// RNG, putting that into an 8-to-32 conversion to be a 32-bit RNG again,
/// and making that 32-bit RNG be the tested RNG. This therefore tests not
/// only the underlying RNG but also the conversion library.
pub unsafe fn run_rng32() {
    let t = static_init_test_rng32();
    t.run();
}

unsafe fn static_init_test_rng32() -> &'static TestRng32<'static> {
    let r1 = static_init!(rng::Rng32To8<'static>, rng::Rng32To8::new(&TRNG));
    TRNG.set_client(r1);
    let r2 = static_init!(rng::Rng8To32<'static>, rng::Rng8To32::new(r1));
    r1.set_client(r2);
    let test = static_init!(TestRng32<'static>, TestRng32::new(r2));
    r2.set_client(test);
    test
}

/// This tests a platform with an underlying 32-bit random number generator
/// by taking that generator, putting into a 32-8 conversion to be an 8-bit
/// RNG,and making that 8-bit RNG be the tested RNG. This therefore tests not
/// only the underlying RNG but also the conversion library.
pub unsafe fn run_rng8() {
    let t = static_init_test_rng8();
    t.run();
}

unsafe fn static_init_test_rng8() -> &'static TestRng8<'static> {
    let r1 = static_init!(rng::Rng32To8<'static>, rng::Rng32To8::new(&TRNG));
    TRNG.set_client(r1);
    let test = static_init!(TestRng8<'static>, TestRng8::new(r1));
    r1.set_client(test);

    test
}
