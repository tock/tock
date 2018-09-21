use capsules::test::rng::TestRng;
use capsules::rng;
use kernel::hil::rng::Rng;
use kernel::hil::entropy::{Entropy8, Entropy32};
use sam4l::trng::{TRNG};

/// This tests a platform with an underlying 32-bit entropy generator
/// by taking that generator, putting into a 32-8 conversion to be an
/// 8-bit generator, putting that into an 8-to-32 conversion to be a
/// 32-bit generator again, and making that 32-bit entropy source be
/// the tested RNG. This therefore tests not only the underlying entropy
/// source but also the conversion library.
pub unsafe fn run_entropy32() {
    let t = static_init_test_entropy32();
    t.run();
}

unsafe fn static_init_test_entropy32() -> &'static TestRng<'static> {
    let e1 = static_init!(rng::Entropy32To8<'static>, rng::Entropy32To8::new(&TRNG));
    TRNG.set_client(e1);
    let e2 = static_init!(rng::Entropy8To32<'static>, rng::Entropy8To32::new(e1));
    e1.set_client(e2);
    let er = static_init!(rng::Entropy32ToRandom<'static>, rng::Entropy32ToRandom::new(e2));
    e2.set_client(er);
    let test = static_init!(TestRng<'static>, TestRng::new(er));
    er.set_client(test);
    test
}
