mod uart;

/// Run all tests for nrf52
pub unsafe fn run_all() {
    uart::run();
    super::nrf5x_tests::run_all();
}
