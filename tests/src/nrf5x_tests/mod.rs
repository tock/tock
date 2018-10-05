mod aes;

/// Run all tests for nrf5x
pub unsafe fn run_all() {
    aes::run();
}
