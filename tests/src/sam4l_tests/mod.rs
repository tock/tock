/// Unit Tests for drivers.

mod aes_ccm_test;
mod aes_test;
mod i2c_dummy;
mod rng_test;

// FIXME: Not enabled yet because requires some initilization outside the module
// mod sixlowpan_dummy;
// mod spi_dummy;
// mod spi_slave_dummy;
// mod udp_lowpan_test;
// mod virtual_uart_rx_test;

/// Run all unit tests
// FIXME: Create helpers for all unit test modules to be executed from here! 
pub unsafe fn run_all() {
    aes_test::run_aes128_cbc();
    aes_test::run_aes128_ctr();

    aes_ccm_test::run();
    
    i2c_dummy::i2c_accel_test();
    i2c_dummy::i2c_li_test();
    i2c_dummy::i2c_scan_slaves();

    rng_test::run_entropy32();
}
