#![no_std]
#![feature(in_band_lifetimes)]

extern crate capsules;
#[macro_use] extern crate kernel;

#[cfg(feature = "sam4l-test")]
extern crate sam4l;

#[cfg(feature = "nrf52-test")]
extern crate nrf52;

#[cfg(feature = "nrf51-test")]
extern crate nrf51;

#[cfg(any(feature = "nrf51-test", feature = "nrf52-test"))]
extern crate nrf5x;

// SAM4L Tests
#[cfg(feature = "sam4l-test")] mod sam4l_tests;
#[cfg(feature = "sam4l-test")] use sam4l_tests as tests;

// NRF5X Tests
#[cfg(any(feature = "nrf51-test", feature = "nrf52-test"))] pub mod nrf5x_tests;

// NRF52 Tests
#[cfg(feature = "nrf52-test")] mod nrf52_tests;
#[cfg(feature = "nrf52-test")] use nrf52_tests as tests;

// NRF51 Tests
#[cfg(feature = "nrf51-test")] mod nrf51_tests;
#[cfg(feature = "nrf51-test")] use nrf51_tests as tests;

pub mod test_capsules;

/// Run all unit tests
// FIXME: Create helpers for all unit test modules to be executed from here! 
pub unsafe fn run_all_tests() {
    tests::run_all();
}
