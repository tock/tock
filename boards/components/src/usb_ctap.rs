//! Component for CTAP over USB.

use capsules::usb::usb_ctap::CtapUsbSyscallDriver;
use capsules::usb::usbc_ctap_hid::ClientCtapHID;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::usb::Client;
use kernel::{static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! usb_ctap_component_buf {
    ($C:ty) => {{
        use capsules::usb::usb_ctap::CtapUsbSyscallDriver;
        use capsules::usb::usbc_ctap_hid::ClientCtapHID;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<ClientCtapHID<'static, 'static, $C>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<CtapUsbSyscallDriver<'static>> = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct UsbCtapComponent<C: 'static + hil::usb::UsbController<'static>> {
    board_kernel: &'static kernel::Kernel,
    controller: &'static C,
    max_ctrl_packet_size: u8,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str],
}

impl<C: 'static + hil::usb::UsbController<'static>> UsbCtapComponent<C> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        controller: &'static C,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str],
    ) -> Self {
        Self {
            board_kernel,
            controller,
            max_ctrl_packet_size,
            vendor_id,
            product_id,
            strings,
        }
    }
}

impl<C: 'static + hil::usb::UsbController<'static>> Component for UsbCtapComponent<C> {
    type StaticInput = (
        &'static mut MaybeUninit<ClientCtapHID<'static, 'static, C>>,
        &'static mut MaybeUninit<CtapUsbSyscallDriver<'static>>,
    );
    type Output = &'static CtapUsbSyscallDriver<'static>;

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let usb_ctap = static_init_half!(
            static_buffer.0,
            ClientCtapHID<'static, 'static, C>,
            ClientCtapHID::new(
                self.controller,
                self.max_ctrl_packet_size,
                self.vendor_id,
                self.product_id,
                self.strings,
            )
        );
        self.controller.set_client(usb_ctap);

        let send_buffer = static_init!([u8; 64], [0; 64]);
        let recv_buffer = static_init!([u8; 64], [0; 64]);

        // Configure the USB userspace driver
        let usb_driver = static_init_half!(
            static_buffer.1,
            CtapUsbSyscallDriver<'static>,
            CtapUsbSyscallDriver::new(
                usb_ctap,
                self.board_kernel.create_grant(&grant_cap),
                send_buffer,
                recv_buffer
            )
        );
        usb_ctap.set_client(usb_driver);

        usb_ctap.enable();
        usb_ctap.attach();

        usb_driver
    }
}
