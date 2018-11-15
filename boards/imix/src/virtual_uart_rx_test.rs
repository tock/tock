use capsules::test::virtual_uart::TestVirtualUartReceive;
use capsules::virtual_uart::{UartDevice, UartMux};
use kernel::hil::uart::Receive;

pub unsafe fn run_virtual_uart_receive(mux: &'static UartMux<'static>) {
    debug!("Starting virtual reads.");
    let small = static_init_test_receive_small(mux);
    let large = static_init_test_receive_large(mux);
    small.run();
    large.run();
}

unsafe fn static_init_test_receive_small(
    mux: &'static UartMux<'static>,
) -> &'static TestVirtualUartReceive {
    static mut SMALL: [u8; 3] = [0; 3];
    let device = static_init!(UartDevice<'static>, UartDevice::new(mux, true));
    device.setup();
    let test = static_init!(
        TestVirtualUartReceive,
        TestVirtualUartReceive::new(device, &mut SMALL)
    );
    device.set_receive_client(test);
    test
}

unsafe fn static_init_test_receive_large(
    mux: &'static UartMux<'static>,
) -> &'static TestVirtualUartReceive {
    static mut BUFFER: [u8; 7] = [0; 7];
    let device = static_init!(UartDevice<'static>, UartDevice::new(mux, true));
    device.setup();
    let test = static_init!(
        TestVirtualUartReceive,
        TestVirtualUartReceive::new(device, &mut BUFFER)
    );
    device.set_receive_client(test);
    test
}
