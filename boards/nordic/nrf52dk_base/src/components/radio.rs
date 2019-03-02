//! Component forRadio syscall interface on imix board.
//!
//! This provides one Component, RadioComponent, which implements a
//! userspace syscall interface to a full 802.15.4 stack with a
//! always-on MAC implementation.
//!
//! Usage
//! -----
//! ```rust
//! let (radio_driver, mux_mac) = RadioComponent::new(rf233, PAN_ID, 0x1008).finalize();
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

extern crate kernel;
extern crate nrf52;
extern crate nrf5x;

use capsules;
use capsules::ieee802154::device::MacDevice;
use capsules::ieee802154::mac::{AwakeMac, Mac};

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::radio;
use kernel::hil::radio::RadioData;
use kernel::hil::symmetric_encryption::AES128CCM;
use kernel::{create_capability, static_init};

// Save some deep nesting

pub struct RadioComponent {
    board_kernel: &'static kernel::Kernel,
    radio: &'static nrf52::nrf_radio::Radio,
    pan_id: capsules::net::ieee802154::PanID,
    short_addr: u16,
}

impl RadioComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static nrf52::nrf_radio::Radio,
        pan_id: capsules::net::ieee802154::PanID,
        addr: u16,
    ) -> RadioComponent {
        RadioComponent {
            board_kernel: board_kernel,
            radio: radio,
            pan_id: pan_id,
            short_addr: addr,
        }
    }
}

static mut RADIO_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// The buffer RF233 packets are received into.
static mut RADIO_RX_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// This buffer is used as an intermediate buffer for AES CCM encryption
// An upper bound on the required size is 3 * BLOCK_SIZE + radio::MAX_BUF_SIZE
const CRYPT_SIZE: usize = 1;
static mut CRYPT_BUF: [u8; CRYPT_SIZE] = [0x00; CRYPT_SIZE];

impl Component for RadioComponent {
    type Output = (
        &'static capsules::ieee802154::RadioDriver<'static>,
        &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    );

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let aes_ccm = static_init!(
            capsules::aes_ccm::AES128CCM<'static, nrf5x::aes::AesECB<'static>>,
            capsules::aes_ccm::AES128CCM::new(&nrf5x::aes::AESECB, &mut CRYPT_BUF)
        );

        // Keeps the radio on permanently; pass-through layer
        let awake_mac: &AwakeMac<nrf52::nrf_radio::Radio> = static_init!(
            AwakeMac<'static, nrf52::nrf_radio::Radio>,
            AwakeMac::new(self.radio)
        );
        self.radio.set_transmit_client(awake_mac);
        self.radio.set_receive_client(awake_mac, &mut RADIO_RX_BUF);

        let mac_device = static_init!(
            capsules::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, nrf52::nrf_radio::Radio>,
                capsules::aes_ccm::AES128CCM<'static, nrf5x::aes::AesECB<'static>>,
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

        let radio_mac = static_init!(
            capsules::ieee802154::virtual_mac::MacUser<'static>,
            capsules::ieee802154::virtual_mac::MacUser::new(mux_mac)
        );
        mux_mac.add_user(radio_mac);

        let radio_driver = static_init!(
            capsules::ieee802154::RadioDriver<'static>,
            capsules::ieee802154::RadioDriver::new(
                radio_mac,
                self.board_kernel.create_grant(&grant_cap),
                &mut RADIO_BUF
            )
        );

        mac_device.set_key_procedure(radio_driver);
        mac_device.set_device_procedure(radio_driver);

        radio_mac.set_transmit_client(radio_driver);
        radio_mac.set_receive_client(radio_driver);

        radio_mac.set_pan(self.pan_id);
        radio_mac.set_address(self.short_addr);

        (radio_driver, mux_mac)
    }
}
