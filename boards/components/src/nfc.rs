//! Component for NFC Tag.
//!
//! Usage
//! -----
//! ```rust
//! let nfct = components::nfct::NfcComponent::new(board_kernel, &nrf52840::nfct::NFCT).finalize(());
//! ```

// Author: Mirna Al-Shetairy <mshetairy@google.com>

use capsules::nfc;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::nfc::NfcTag;
use kernel::static_init;

pub struct NfcComponent {
    board_kernel: &'static kernel::Kernel,
    nfct: &'static dyn NfcTag<'static>,
}

impl NfcComponent {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        nfct: &'static dyn NfcTag<'static>,
    ) -> NfcComponent {
        NfcComponent {
            board_kernel: board_kernel,
            nfct: nfct,
        }
    }
}

impl Component for NfcComponent {
    type StaticInput = ();
    type Output = &'static nfc::NfcDriver<'static>;

    unsafe fn finalize(self, _static_buffer: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let tx_buffer = static_init!([u8; nfc::MAX_LENGTH], [0u8; nfc::MAX_LENGTH]);
        let rx_buffer = static_init!([u8; nfc::MAX_LENGTH], [0u8; nfc::MAX_LENGTH]);

        let nfct = static_init!(
            // Supply to the capsule: the driver and a grant
            nfc::NfcDriver<'static>,
            nfc::NfcDriver::new(
                self.nfct,
                tx_buffer,
                rx_buffer,
                self.board_kernel.create_grant(&grant_cap)
            )
        );
        self.nfct.set_client(nfct);
        self.nfct.enable();
        nfct
    }
}
