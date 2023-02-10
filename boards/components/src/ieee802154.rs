//! Component for IEEE 802.15.4 radio syscall interface.
//!
//! This provides one Component, `Ieee802154Component`, which implements a
//! userspace syscall interface to a full 802.15.4 stack with a always-on MAC
//! implementation, as well as multiplexed access to that MAC implementation.
//!
//! Usage
//! -----
//! ```rust
//! let aes_mux = components::ieee802154::MuxAes128ccmComponent::new(
//!     &base_peripherals.ecb,
//!     dynamic_deferred_caller,
//! )
//!  .finalize(components::mux_aes128ccm_component_static!(
//!     nrf52840::aes::AesECB
//! ));
//!
//! let (radio, mux_mac) = components::ieee802154::Ieee802154Component::new(
//!     board_kernel,
//!     extra_capsules::ieee802154::DRIVER_NUM,
//!     &nrf52::ieee802154_radio::RADIO,
//!     aes_mux,
//!     PAN_ID,
//!     SRC_MAC,
//!     deferred_caller,
//! )
//! .finalize(components::ieee802154_component_static!(
//!     nrf52::ieee802154_radio::Radio,
//!     nrf52::aes::AesECB<'static>
//! ));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_aes_ccm::MuxAES128CCM;
use extra_capsules::ieee802154::device::MacDevice;
use extra_capsules::ieee802154::mac::{AwakeMac, Mac};
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::dynamic_deferred_call::DynamicDeferredCall;
use kernel::hil::radio;
use kernel::hil::symmetric_encryption::{self, AES128Ctr, AES128, AES128CBC, AES128CCM, AES128ECB};

// This buffer is used as an intermediate buffer for AES CCM encryption. An
// upper bound on the required size is `3 * BLOCK_SIZE + radio::MAX_BUF_SIZE`.
pub const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;

#[macro_export]
macro_rules! mux_aes128ccm_component_static {
    ($A:ty $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_aes_ccm::MuxAES128CCM<'static, $A>)
    };};
}

pub struct MuxAes128ccmComponent<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> {
    aes: &'static A,
    deferred_caller: &'static DynamicDeferredCall,
}

impl<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> MuxAes128ccmComponent<A> {
    pub fn new(aes: &'static A, deferred_caller: &'static DynamicDeferredCall) -> Self {
        Self {
            aes,
            deferred_caller,
        }
    }
}

impl<A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> Component
    for MuxAes128ccmComponent<A>
{
    type StaticInput = &'static mut MaybeUninit<MuxAES128CCM<'static, A>>;
    type Output = &'static MuxAES128CCM<'static, A>;

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let aes_mux = static_buffer.write(MuxAES128CCM::new(self.aes, self.deferred_caller));
        self.aes.set_client(aes_mux);
        aes_mux.initialize_callback_handle(
            self.deferred_caller.register(aes_mux).unwrap(), // Unwrap fail = no deferred call slot available for ccm mux
        );

        aes_mux
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! ieee802154_component_static {
    ($R:ty, $A:ty $(,)?) => {{
        let virtual_aes =
            kernel::static_buf!(core_capsules::virtual_aes_ccm::VirtualAES128CCM<'static, $A>);
        let awake_mac = kernel::static_buf!(extra_capsules::ieee802154::mac::AwakeMac<'static, $R>);
        let framer = kernel::static_buf!(
            extra_capsules::ieee802154::framer::Framer<
                'static,
                extra_capsules::ieee802154::mac::AwakeMac<'static, $R>,
                core_capsules::virtual_aes_ccm::VirtualAES128CCM<'static, $A>,
            >
        );

        let mux_mac = kernel::static_buf!(extra_capsules::ieee802154::virtual_mac::MuxMac<'static>);
        let mac_user =
            kernel::static_buf!(extra_capsules::ieee802154::virtual_mac::MacUser<'static>);
        let radio_driver = kernel::static_buf!(extra_capsules::ieee802154::RadioDriver<'static>);

        let radio_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let radio_rx_buf = kernel::static_buf!([u8; kernel::hil::radio::MAX_BUF_SIZE]);
        let crypt_buf = kernel::static_buf!([u8; components::ieee802154::CRYPT_SIZE]);

        (
            virtual_aes,
            awake_mac,
            framer,
            mux_mac,
            mac_user,
            radio_driver,
            radio_buf,
            radio_rx_buf,
            crypt_buf,
        )
    };};
}

pub struct Ieee802154Component<
    R: 'static + kernel::hil::radio::Radio,
    A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    radio: &'static R,
    aes_mux: &'static MuxAES128CCM<'static, A>,
    pan_id: extra_capsules::net::ieee802154::PanID,
    short_addr: u16,
    deferred_caller: &'static DynamicDeferredCall,
}

impl<
        R: 'static + kernel::hil::radio::Radio,
        A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
    > Ieee802154Component<R, A>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        radio: &'static R,
        aes_mux: &'static MuxAES128CCM<'static, A>,
        pan_id: extra_capsules::net::ieee802154::PanID,
        short_addr: u16,
        deferred_caller: &'static DynamicDeferredCall,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            radio,
            aes_mux,
            pan_id,
            short_addr,
            deferred_caller,
        }
    }
}

impl<
        R: 'static + kernel::hil::radio::Radio,
        A: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
    > Component for Ieee802154Component<R, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<core_capsules::virtual_aes_ccm::VirtualAES128CCM<'static, A>>,
        &'static mut MaybeUninit<extra_capsules::ieee802154::mac::AwakeMac<'static, R>>,
        &'static mut MaybeUninit<
            extra_capsules::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, R>,
                core_capsules::virtual_aes_ccm::VirtualAES128CCM<'static, A>,
            >,
        >,
        &'static mut MaybeUninit<extra_capsules::ieee802154::virtual_mac::MuxMac<'static>>,
        &'static mut MaybeUninit<extra_capsules::ieee802154::virtual_mac::MacUser<'static>>,
        &'static mut MaybeUninit<extra_capsules::ieee802154::RadioDriver<'static>>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; radio::MAX_BUF_SIZE]>,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
    );
    type Output = (
        &'static extra_capsules::ieee802154::RadioDriver<'static>,
        &'static extra_capsules::ieee802154::virtual_mac::MuxMac<'static>,
    );

    fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let crypt_buf = static_buffer.8.write([0; CRYPT_SIZE]);
        let aes_ccm = static_buffer
            .0
            .write(core_capsules::virtual_aes_ccm::VirtualAES128CCM::new(
                self.aes_mux,
                crypt_buf,
            ));
        aes_ccm.setup();

        // Keeps the radio on permanently; pass-through layer.
        let radio_rx_buf = static_buffer.7.write([0; radio::MAX_BUF_SIZE]);
        let awake_mac = static_buffer.1.write(AwakeMac::new(self.radio));
        self.radio.set_transmit_client(awake_mac);
        self.radio.set_receive_client(awake_mac, radio_rx_buf);

        let mac_device = static_buffer
            .2
            .write(extra_capsules::ieee802154::framer::Framer::new(
                awake_mac, aes_ccm,
            ));
        AES128CCM::set_client(aes_ccm, mac_device);
        awake_mac.set_transmit_client(mac_device);
        awake_mac.set_receive_client(mac_device);
        awake_mac.set_config_client(mac_device);

        let mux_mac = static_buffer
            .3
            .write(extra_capsules::ieee802154::virtual_mac::MuxMac::new(
                mac_device,
            ));
        mac_device.set_transmit_client(mux_mac);
        mac_device.set_receive_client(mux_mac);

        let userspace_mac =
            static_buffer
                .4
                .write(extra_capsules::ieee802154::virtual_mac::MacUser::new(
                    mux_mac,
                ));
        mux_mac.add_user(userspace_mac);

        let radio_buffer = static_buffer.6.write([0; radio::MAX_BUF_SIZE]);
        let radio_driver = static_buffer
            .5
            .write(extra_capsules::ieee802154::RadioDriver::new(
                userspace_mac,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                radio_buffer,
                self.deferred_caller,
            ));

        mac_device.set_key_procedure(radio_driver);
        mac_device.set_device_procedure(radio_driver);
        userspace_mac.set_transmit_client(radio_driver);
        userspace_mac.set_receive_client(radio_driver);
        userspace_mac.set_pan(self.pan_id);
        userspace_mac.set_address(self.short_addr);
        radio_driver.initialize_callback_handle(
            self.deferred_caller.register(radio_driver).unwrap(), // Unwrap fail = no deferred call slot available for ieee802154 driver
        );

        (radio_driver, mux_mac)
    }
}
