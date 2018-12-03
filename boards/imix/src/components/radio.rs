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

// Author: Philip Levis <pal@cs.stanford.edu>
// Last modified: 7/25/2018 (by Hudson Ayers)

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules::ieee802154::device::MacDevice;
use capsules::ieee802154::mac::{AwakeMac, Mac};
use capsules::virtual_spi::VirtualSpiMasterDevice;

use kernel::capabilities;
use kernel::component::Component;
use kernel::hil::radio;
use kernel::hil::radio::RadioData;
use kernel::hil::symmetric_encryption;
use kernel::hil::symmetric_encryption::{AES128, AES128CCM};

// Save some deep nesting
type RF233Device =
    capsules::rf233::RF233<'static, VirtualSpiMasterDevice<'static, sam4l::spi::SpiHw>>;

pub struct RadioComponent {
    board_kernel: &'static kernel::Kernel,
    rf233: &'static RF233Device,
    pan_id: capsules::net::ieee802154::PanID,
    short_addr: u16,
}

impl RadioComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        rf233: &'static RF233Device,
        pan_id: capsules::net::ieee802154::PanID,
        addr: u16,
    ) -> RadioComponent {
        RadioComponent {
            board_kernel: board_kernel,
            rf233: rf233,
            pan_id: pan_id,
            short_addr: addr,
        }
    }
}
// The RF233 system call interface ("radio") requires one buffer, which it
// copies application transmissions into or copies out to application buffers
// for reception.
static mut RADIO_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// The buffer RF233 packets are received into.
static mut RF233_RX_BUF: [u8; radio::MAX_BUF_SIZE] = [0x00; radio::MAX_BUF_SIZE];

// This buffer is used as an intermediate buffer for AES CCM encryption
// An upper bound on the required size is 3 * BLOCK_SIZE + radio::MAX_BUF_SIZE
const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;
static mut CRYPT_BUF: [u8; CRYPT_SIZE] = [0x00; CRYPT_SIZE];

impl Component for RadioComponent {
    type Output = (
        &'static capsules::ieee802154::RadioDriver<'static>,
        &'static capsules::ieee802154::virtual_mac::MuxMac<'static>,
    );

    unsafe fn finalize(&mut self) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let aes_ccm = static_init!(
            capsules::aes_ccm::AES128CCM<'static, sam4l::aes::Aes<'static>>,
            capsules::aes_ccm::AES128CCM::new(&sam4l::aes::AES, &mut CRYPT_BUF)
        );
        sam4l::aes::AES.set_client(aes_ccm);
        sam4l::aes::AES.enable();

        // Keeps the radio on permanently; pass-through layer
        let awake_mac: &AwakeMac<RF233Device> =
            static_init!(AwakeMac<'static, RF233Device>, AwakeMac::new(self.rf233));
        self.rf233.set_transmit_client(awake_mac);
        self.rf233.set_receive_client(awake_mac, &mut RF233_RX_BUF);

        let mac_device = static_init!(
            capsules::ieee802154::framer::Framer<
                'static,
                AwakeMac<'static, RF233Device>,
                capsules::aes_ccm::AES128CCM<'static, sam4l::aes::Aes<'static>>,
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
