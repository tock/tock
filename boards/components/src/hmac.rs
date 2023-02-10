//! Components for collections of HMACs.
//!
//! Usage
//! -----
//! ```rust
//!    let mux_hmac = components::hmac::HmacMuxComponent::new(&earlgrey::hmac::HMAC).finalize(
//!        components::hmac_mux_component_static!(lowrisc::hmac::Hmac, 32),
//!    );
//!
//!    let hmac = components::hmac::HmacComponent::new(
//!        board_kernel,
//!        &mux_hmac,
//!    )
//!    .finalize(components::hmac_component_static!(
//!        lowrisc::hmac::Hmac,
//!        32
//!    ));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_hmac::MuxHmac;
use core_capsules::virtual_hmac::VirtualMuxHmac;
use extra_capsules::hmac::HmacDriver;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;

#[macro_export]
macro_rules! hmac_mux_component_static {
    ($A:ty, $L:expr $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_hmac::MuxHmac<'static, $A, $L>)
    };};
}

#[macro_export]
macro_rules! hmac_component_static {
    ($A:ty, $L:expr $(,)?) => {{
        let virtual_mux =
            kernel::static_buf!(core_capsules::virtual_hmac::VirtualMuxHmac<'static, $A, $L>);
        let hmac = kernel::static_buf!(
            extra_capsules::hmac::HmacDriver<
                'static,
                core_capsules::virtual_hmac::VirtualMuxHmac<'static, $A, $L>,
                $L,
            >
        );

        let key_buffer = kernel::static_buf!([u8; 32]);
        let data_buffer = kernel::static_buf!([u8; 64]);
        let dest_buffer = kernel::static_buf!([u8; $L]);

        (virtual_mux, hmac, key_buffer, data_buffer, dest_buffer)
    };};
}

pub struct HmacMuxComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    hmac: &'static A,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> HmacMuxComponent<A, L> {
    pub fn new(hmac: &'static A) -> HmacMuxComponent<A, L> {
        HmacMuxComponent { hmac }
    }
}

impl<
        A: 'static
            + digest::Digest<'static, L>
            + digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512,
        const L: usize,
    > Component for HmacMuxComponent<A, L>
{
    type StaticInput = &'static mut MaybeUninit<MuxHmac<'static, A, L>>;
    type Output = &'static MuxHmac<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(MuxHmac::new(self.hmac))
    }
}

pub struct HmacComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mux_hmac: &'static MuxHmac<'static, A, L>,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> HmacComponent<A, L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mux_hmac: &'static MuxHmac<'static, A, L>,
    ) -> HmacComponent<A, L> {
        HmacComponent {
            board_kernel,
            driver_num,
            mux_hmac,
        }
    }
}

impl<
        A: kernel::hil::digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + 'static
            + digest::Digest<'static, L>,
        const L: usize,
    > Component for HmacComponent<A, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxHmac<'static, A, L>>,
        &'static mut MaybeUninit<HmacDriver<'static, VirtualMuxHmac<'static, A, L>, L>>,
        &'static mut MaybeUninit<[u8; 32]>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; L]>,
    );
    type Output = &'static HmacDriver<'static, VirtualMuxHmac<'static, A, L>, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let key_buffer = s.2.write([0; 32]);
        let data_buffer = s.3.write([0; 64]);
        let dest_buffer = s.4.write([0; L]);

        let virtual_hmac_user = s.0.write(VirtualMuxHmac::new(self.mux_hmac, key_buffer));

        let hmac = s.1.write(extra_capsules::hmac::HmacDriver::new(
            virtual_hmac_user,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        hmac
    }
}
