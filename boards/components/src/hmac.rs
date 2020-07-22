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
//! ));
//! ```

use capsules;
use capsules::hmac::HmacDriver;
use capsules::virtual_hmac::MuxHmac;
use capsules::virtual_hmac::VirtualMuxHmac;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! hmac_mux_component_helper {
    ($A:ty, $T:ty) => {{
        use capsules::virtual_hmac::MuxHmac;
        use capsules::virtual_hmac::VirtualMuxHmac;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxHmac<'static, $A, $T>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct HmacMuxComponent<
    A: 'static + digest::Digest<'static, T>,
    T: 'static + digest::DigestType,
> {
    hmac: &'static A,
    phantom: PhantomData<&'static T>,
}

impl<A: 'static + digest::Digest<'static, T>, T: 'static + digest::DigestType>
    HmacMuxComponent<A, T>
{
    pub fn new(hmac: &'static A) -> HmacMuxComponent<A, T> {
        HmacMuxComponent {
            hmac,
            phantom: PhantomData,
        }
    }
}

impl<A: 'static + digest::Digest<'static, T>, T: 'static + digest::DigestType> Component
    for HmacMuxComponent<A, T>
{
    type StaticInput = &'static mut MaybeUninit<MuxHmac<'static, A, T>>;
    type Output = &'static MuxHmac<'static, A, T>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_hmac = static_init_half!(s, MuxHmac<'static, A, T>, MuxHmac::new(self.hmac));

        mux_hmac
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! hmac_component_helper {
    ($A:ty, $T:ty) => {{
        use capsules::hmac::HmacDriver;
        use capsules::virtual_hmac::MuxHmac;
        use capsules::virtual_hmac::VirtualMuxHmac;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxHmac<'static, $A, $T>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<HmacDriver<'static, VirtualMuxHmac<'static, $A, $T>, $T>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct HmacComponent<A: 'static + digest::Digest<'static, T>, T: 'static + digest::DigestType> {
    board_kernel: &'static kernel::Kernel,
    mux_hmac: &'static MuxHmac<'static, A, T>,
    data_buffer: &'static mut [u8],
    dest_buffer: &'static mut T,
    phantom: PhantomData<&'static T>,
}

impl<A: 'static + digest::Digest<'static, T>, T: 'static + digest::DigestType> HmacComponent<A, T> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        mux_hmac: &'static MuxHmac<'static, A, T>,
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut T,
    ) -> HmacComponent<A, T> {
        HmacComponent {
            board_kernel,
            mux_hmac,
            data_buffer,
            dest_buffer,
            phantom: PhantomData,
        }
    }
}

impl<
        A: kernel::hil::digest::HMACSha256 + 'static + digest::Digest<'static, T>,
        T: 'static + digest::DigestType,
    > Component for HmacComponent<A, T>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxHmac<'static, A, T>>,
        &'static mut MaybeUninit<HmacDriver<'static, VirtualMuxHmac<'static, A, T>, T>>,
    );

    type Output = &'static HmacDriver<'static, VirtualMuxHmac<'static, A, T>, T>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_hmac_user = static_init_half!(
            s.0,
            VirtualMuxHmac<'static, A, T>,
            VirtualMuxHmac::new(self.mux_hmac)
        );

        let hmac = static_init_half!(
            s.1,
            capsules::hmac::HmacDriver<'static, VirtualMuxHmac<'static, A, T>, T>,
            capsules::hmac::HmacDriver::new(
                virtual_hmac_user,
                self.data_buffer,
                self.dest_buffer,
                self.board_kernel.create_grant(&grant_cap),
            )
        );

        digest::Digest::set_client(virtual_hmac_user, hmac);

        hmac
    }
}
