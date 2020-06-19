//! Component for IEEE 802.15.4 radio syscall interface.
//!
//! This provides one Component, `Ieee802154Component`, which implements a
//! userspace syscall interface to a full 802.15.4 stack with a
//! always-on MAC implementation, as well as multiplexed access to that MAC implementation.
//!
//! Usage
//! -----
//! ```rust
//! let (radio, mux_mac) = components::ieee802154::Ieee802154Component::new(
//!     board_kernel,
//!     &nrf52::ieee802154_radio::RADIO,
//!     &nrf52::aes::AESECB,
//!     PAN_ID,
//!     SRC_MAC,
//! )
//! .finalize(components::ieee802154_component_helper!(
//!     nrf52::ieee802154_radio::Radio,
//!     nrf52::aes::AesECB<'static>
//! ));
//! ```

use capsules;
use capsules::ieee802154::device::MacDevice;
use capsules::ieee802154::mac::{AwakeMac, Mac};
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::radio;
use kernel::hil::symmetric_encryption::{self, AES128Ctr, AES128, AES128CBC, AES128CCM};
use kernel::{create_capability, static_init, static_init_half};

// Setup static space for the objects.
#[macro_export]
macro_rules! ieee802154_component_helper {
    ($R:ty, $A:ty) => {{
        use capsules::ieee802154::mac::AwakeMac;
        use core::mem::MaybeUninit;
        use kernel::hil::symmetric_encryption::{AES128Ctr, AES128, AES128CBC, AES128CCM};

        static mut BUF1: MaybeUninit<capsules::aes_ccm::AES128CCM<'static, $A>> =
            MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<AwakeMac<'static, $R>> = MaybeUninit::uninit();
        static mut BUF3: MaybeUninit<
            capsules::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, $R>,
                capsules::aes_ccm::AES128CCM<'static, $A>,
            >,
        > = MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2, &mut BUF3)
    };};
}

pub struct Ieee802154Component<
    R: 'static + kernel::hil::radio::Radio,
    A: 'static + AES128<'static> + AES128Ctr + AES128CBC,
> {
    board_kernel: &'static kernel::Kernel,
    radio: &'static R,
    aes: &'static A,
    pan_id: capsules::net::ieee802154::PanID,
    short_addr: u16,
}

impl<
        R: 'static + kernel::hil::radio::Radio,
        A: 'static + AES128<'static> + AES128Ctr + AES128CBC,
    > Ieee802154Component<R, A>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static R,
        aes: &'static A,
        pan_id: capsules::net::ieee802154::PanID,
        short_addr: u16,
    ) -> Self {
        Self {
            board_kernel,
            radio,
            aes,
            pan_id,
            short_addr,
        }
    }
}

static mut RADIO_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// The buffer packets are received into.
static mut RADIO_RX_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// This buffer is used as an intermediate buffer for AES CCM encryption
// An upper bound on the required size is 3 * BLOCK_SIZE + radio::MAX_BUF_SIZE
const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;
static mut CRYPT_BUF: [u8; CRYPT_SIZE] = [0x00; CRYPT_SIZE];

impl<
        R: 'static + kernel::hil::radio::Radio,
        A: 'static + AES128<'static> + AES128Ctr + AES128CBC,
    > Component for Ieee802154Component<R, A>
{
    type StaticInput = (
        &'static mut MaybeUninit<capsules::aes_ccm::AES128CCM<'static, A>>,
        &'static mut MaybeUninit<capsules::ieee802154::mac::AwakeMac<'static, R>>,
        &'static mut MaybeUninit<
            capsules::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, R>,
                capsules::aes_ccm::AES128CCM<'static, A>,
            >,
        >,
    );
    type Output = (
        &'static capsules::ieee802154::RadioDriver<'static>,
        &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    );

    unsafe fn finalize(self, static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let aes_ccm = static_init_half!(
            static_buffer.0,
            capsules::aes_ccm::AES128CCM<'static, A>,
            capsules::aes_ccm::AES128CCM::new(self.aes, &mut CRYPT_BUF)
        );

        self.aes.set_client(aes_ccm);
        self.aes.enable();

        // Keeps the radio on permanently; pass-through layer
        let awake_mac = static_init_half!(
            static_buffer.1,
            AwakeMac<'static, R>,
            AwakeMac::new(self.radio)
        );
        self.radio.set_transmit_client(awake_mac);
        self.radio.set_receive_client(awake_mac, &mut RADIO_RX_BUF);

        let mac_device = static_init_half!(
            static_buffer.2,
            capsules::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, R>,
                capsules::aes_ccm::AES128CCM<'static, A>,
            >,
            capsules::ieee802154::framer::Framer::new(awake_mac, aes_ccm)
        );
        aes_ccm.set_client(mac_device);
        awake_mac.set_transmit_client(mac_device);
        awake_mac.set_receive_client(mac_device);
        awake_mac.set_config_client(mac_device);

        let mux_mac = static_init!(
            capsules::ieee802154::virtual_mac::MuxMac<'static>,
            capsules::ieee802154::virtual_mac::MuxMac::new(mac_device)
        );
        mac_device.set_transmit_client(mux_mac);
        mac_device.set_receive_client(mux_mac);

        let userspace_mac = static_init!(
            capsules::ieee802154::virtual_mac::MacUser<'static>,
            capsules::ieee802154::virtual_mac::MacUser::new(mux_mac)
        );
        mux_mac.add_user(userspace_mac);

        let radio_driver = static_init!(
            capsules::ieee802154::RadioDriver<'static>,
            capsules::ieee802154::RadioDriver::new(
                userspace_mac,
                self.board_kernel.create_grant(&grant_cap),
                &mut RADIO_BUF
            )
        );

        mac_device.set_key_procedure(radio_driver);
        mac_device.set_device_procedure(radio_driver);
        userspace_mac.set_transmit_client(radio_driver);
        userspace_mac.set_receive_client(radio_driver);
        userspace_mac.set_pan(self.pan_id);
        userspace_mac.set_address(self.short_addr);

        (radio_driver, mux_mac)
    }
}
