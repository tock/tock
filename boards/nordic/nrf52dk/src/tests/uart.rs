use kernel::hil::uart::Transmit;
use kernel::static_init;
use nrf52832::uart::UARTE0;

const BUFFER_SIZE_2048: usize = 2048;

/// To run the tests add the following `main.rs::reset_handler` somewhere after that the UART
/// peripheral has been initilized:
///
/// ```rustc
///     tests::uart::run();
/// ```
///
/// Make sure you don't are running any user-space processes and remove all `debug!` prints
/// in `main.rs::reset_handler()` otherwise race-conditions in the UART will occur.
/// Then enable the test you want run in `run()`
///
pub unsafe fn run() {
    // Note: you can only one of these tests at the time because
    //  1. It will generate race-conitions in the UART because we don't have any checks against that
    //  2. `buf` can only be `borrowed` once and avoid allocate four different buffers

    let buf = static_init!([u8; BUFFER_SIZE_2048], [0; BUFFER_SIZE_2048]);

    // create an iterator of printable ascii characters and write to the uart buffer
    for (ascii_char, b) in (33..126).cycle().zip(buf.iter_mut()) {
        *b = ascii_char;
    }

    transmit_entire_buffer(buf);
    // transmit_512(buf);
    // should_not_transmit(buf);
    // transmit_254(buf);
}

#[allow(unused)]
unsafe fn transmit_entire_buffer(buf: &'static mut [u8]) {
    &UARTE0.transmit_buffer(buf, BUFFER_SIZE_2048);
}

#[allow(unused)]
unsafe fn should_not_transmit(buf: &'static mut [u8]) {
    &UARTE0.transmit_buffer(buf, 0);
}

#[allow(unused)]
unsafe fn transmit_512(buf: &'static mut [u8]) {
    &UARTE0.transmit_buffer(buf, 512);
}

#[allow(unused)]
unsafe fn transmit_254(buf: &'static mut [u8]) {
    &UARTE0.transmit_buffer(buf, 254);
}
