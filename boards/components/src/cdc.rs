//! Component for CDC-ACM over USB support.
//!
//! This provides a component for using the CDC-ACM driver. This allows for
//! serial communication over USB.
//!
//! Usage
//! -----
//! ```rust
//! static STRINGS: &'static [&str; 3] = &[
//!     "XYZ Corp.",      // Manufacturer
//!     "The Zorpinator", // Product
//!     "Serial No. 5",   // Serial number
//! ];
//! let cdc_acm = components::cdc::CdcAcmComponent::new(
//!     &nrf52::usbd::USBD,
//!     capsules::usb::usbc_client::MAX_CTRL_PACKET_SIZE_NRF52840,
//!     0x2341,
//!     0x005a,
//!     STRINGS)
//! .finalize(components::usb_cdc_acm_component_helper!(nrf52::usbd::Usbd));
//! ```

use core::mem::MaybeUninit;

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use kernel::common::dynamic_deferred_call::DynamicDeferredCall;
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! usb_cdc_acm_component_helper {
    ($U:ty, $A:ty $(,)?) => {{
        use capsules::virtual_alarm::VirtualMuxAlarm;
        use core::mem::MaybeUninit;
        static mut BUF0: MaybeUninit<VirtualMuxAlarm<'static, $A>> = MaybeUninit::uninit();
        static mut BUF1: MaybeUninit<
            capsules::usb::cdc::CdcAcm<'static, $U, VirtualMuxAlarm<'static, $A>>,
        > = MaybeUninit::uninit();
        (&mut BUF0, &mut BUF1)
    };};
}

pub struct CdcAcmComponent<
    U: 'static + hil::usb::UsbController<'static>,
    A: 'static + Alarm<'static>,
> {
    usb: &'static U,
    max_ctrl_packet_size: u8,
    vendor_id: u16,
    product_id: u16,
    strings: &'static [&'static str; 3],
    alarm_mux: &'static MuxAlarm<'static, A>,
    deferred_caller: &'static DynamicDeferredCall,
    host_initiated_function: Option<&'static (dyn Fn() + 'static)>,
}

impl<U: 'static + hil::usb::UsbController<'static>, A: 'static + Alarm<'static>>
    CdcAcmComponent<U, A>
{
    pub fn new(
        usb: &'static U,
        max_ctrl_packet_size: u8,
        vendor_id: u16,
        product_id: u16,
        strings: &'static [&'static str; 3],
        alarm_mux: &'static MuxAlarm<'static, A>,
        deferred_caller: &'static DynamicDeferredCall,
        host_initiated_function: Option<&'static (dyn Fn() + 'static)>,
    ) -> Self {
        Self {
            usb,
            max_ctrl_packet_size,
            vendor_id,
            product_id,
            strings,
            alarm_mux,
            deferred_caller,
            host_initiated_function,
        }
    }
}

impl<U: 'static + hil::usb::UsbController<'static>, A: 'static + Alarm<'static>> Component
    for CdcAcmComponent<U, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxAlarm<'static, A>>,
        &'static mut MaybeUninit<
            capsules::usb::cdc::CdcAcm<'static, U, VirtualMuxAlarm<'static, A>>,
        >,
    );
    type Output = &'static capsules::usb::cdc::CdcAcm<'static, U, VirtualMuxAlarm<'static, A>>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let cdc_alarm = static_init_half!(
            s.0,
            VirtualMuxAlarm<'static, A>,
            VirtualMuxAlarm::new(self.alarm_mux)
        );
        let cdc = static_init_half!(
            s.1,
            capsules::usb::cdc::CdcAcm<'static, U, VirtualMuxAlarm<'static, A>>,
            capsules::usb::cdc::CdcAcm::new(
                self.usb,
                self.max_ctrl_packet_size,
                self.vendor_id,
                self.product_id,
                self.strings,
                cdc_alarm,
                self.deferred_caller,
                self.host_initiated_function,
            )
        );
        self.usb.set_client(cdc);
        cdc.initialize_callback_handle(
            self.deferred_caller
                .register(cdc)
                .expect("no deferred call slot available for USB-CDC"),
        );
        cdc_alarm.set_alarm_client(cdc);

        cdc
    }
}
