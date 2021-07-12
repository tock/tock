//! Components for collections of HMACs.
//!
//! Usage
//! -----
//! ```rust
//!    let hmac_data_buffer = static_init!([u8; 64], [0; 64]);
//!    let hmac_dest_buffer = static_init!([u8; 32], [0; 32]);
//!
//!    let mux_hmac = components::hmac::HmacMuxComponent::new(&earlgrey::hmac::HMAC).finalize(
//!        components::hmac_mux_component_helper!(lowrisc::hmac::Hmac, [u8; 32]),
//!    );
//!
//!    let hmac = components::hmac::HmacComponent::new(
//!        board_kernel,
//!        &mux_hmac,
//!        hmac_data_buffer,
//!        hmac_dest_buffer,
//!    )
//!    .finalize(components::hmac_component_helper!(
//!        lowrisc::hmac::Hmac,
//!        [u8; 32]
//!    ));
//! ```

use capsules;
use capsules::hmac::HmacDriver;
use capsules::virtual_hmac::MuxHmac;
use capsules::virtual_hmac::VirtualMuxHmac;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! hmac_mux_component_helper {
    ($A:ty, $L:expr $(,)?) => {{
        use capsules::virtual_hmac::MuxHmac;
        use capsules::virtual_hmac::VirtualMuxHmac;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxHmac<'static, $A, $L>> = MaybeUninit::uninit();
        &mut BUF1
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
            + digest::HMACSha256
            + digest::HMACSha384
            + digest::HMACSha512,
        const L: usize,
    > Component for HmacMuxComponent<A, L>
{
    type StaticInput = &'static mut MaybeUninit<MuxHmac<'static, A, L>>;
    type Output = &'static MuxHmac<'static, A, L>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_hmac = static_init_half!(s, MuxHmac<'static, A, L>, MuxHmac::new(self.hmac));

        mux_hmac
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! hmac_component_helper {
    ($A:ty, $L:expr $(,)?) => {{
        use capsules::hmac::HmacDriver;
        use capsules::virtual_hmac::MuxHmac;
        use capsules::virtual_hmac::VirtualMuxHmac;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxHmac<'static, $A, $L>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<HmacDriver<'static, VirtualMuxHmac<'static, $A, $L>, $L>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct HmacComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mux_hmac: &'static MuxHmac<'static, A, L>,
    key_buffer: &'static mut [u8],
    data_buffer: &'static mut [u8],
    dest_buffer: &'static mut [u8; L],
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> HmacComponent<A, L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mux_hmac: &'static MuxHmac<'static, A, L>,
        key_buffer: &'static mut [u8],
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8; L],
    ) -> HmacComponent<A, L> {
        HmacComponent {
            board_kernel,
            driver_num,
            mux_hmac,
            key_buffer,
            data_buffer,
            dest_buffer,
        }
    }
}

impl<
        A: kernel::hil::digest::HMACSha256
            + digest::HMACSha384
            + digest::HMACSha512
            + 'static
            + digest::Digest<'static, L>,
        const L: usize,
    > Component for HmacComponent<A, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxHmac<'static, A, L>>,
        &'static mut MaybeUninit<HmacDriver<'static, VirtualMuxHmac<'static, A, L>, L>>,
    );

    type Output = &'static HmacDriver<'static, VirtualMuxHmac<'static, A, L>, L>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_hmac_user = static_init_half!(
            s.0,
            VirtualMuxHmac<'static, A, L>,
            VirtualMuxHmac::new(self.mux_hmac, self.key_buffer)
        );

        let hmac = static_init_half!(
            s.1,
            capsules::hmac::HmacDriver<'static, VirtualMuxHmac<'static, A, L>, L>,
            capsules::hmac::HmacDriver::new(
                virtual_hmac_user,
                self.data_buffer,
                self.dest_buffer,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            )
        );

        hmac
    }
}
