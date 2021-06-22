//! Components for collections of Digests.
//!
//! Usage
//! -----
//! ```rust
//!    let digest_data_buffer = static_init!([u8; 64], [0; 64]);
//!    let digest_dest_buffer = static_init!([u8; 32], [0; 32]);
//!
//!    let mux_digest = components::digest::DigestMuxComponent::new(&earlgrey::digest::Digest).finalize(
//!        components::digest_mux_component_helper!(lowrisc::digest::Digest, [u8; 32]),
//!    );
//!
//!    let digest = components::digest::DigestComponent::new(
//!        board_kernel,
//!        &mux_digest,
//!        digest_data_buffer,
//!        digest_dest_buffer,
//!    )
//!    .finalize(components::digest_component_helper!(
//!        lowrisc::digest::Digest,
//!        [u8; 32]
//!    ));
//! ```

use capsules;
use capsules::virtual_digest::MuxDigest;
use capsules::virtual_digest::VirtualMuxDigest;
use core::mem::MaybeUninit;
use kernel::component::Component;
use kernel::hil::digest;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! digest_mux_component_helper {
    ($A:ty, $L:expr $(,)?) => {{
        use capsules::virtual_digest::MuxDigest;
        use capsules::virtual_digest::VirtualMuxDigest;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxDigest<'static, $A, $L>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct DigestMuxComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    digest: &'static A,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> DigestMuxComponent<A, L> {
    pub fn new(digest: &'static A) -> DigestMuxComponent<A, L> {
        DigestMuxComponent { digest }
    }
}

impl<
        A: 'static
            + digest::Digest<'static, L>
            + digest::HMACSha256
            + digest::HMACSha384
            + digest::HMACSha512
            + digest::Sha256
            + digest::Sha384
            + digest::Sha512,
        const L: usize,
    > Component for DigestMuxComponent<A, L>
{
    type StaticInput = &'static mut MaybeUninit<MuxDigest<'static, A, L>>;
    type Output = &'static MuxDigest<'static, A, L>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_digest =
            static_init_half!(s, MuxDigest<'static, A, L>, MuxDigest::new(self.digest));

        mux_digest
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! digest_component_helper {
    ($A:ty, $L:expr $(,)?) => {{
        use capsules::virtual_digest::MuxDigest;
        use capsules::virtual_digest::VirtualMuxDigest;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxDigest<'static, $A, $L>> = MaybeUninit::uninit();
        &mut BUF1
    };};
}

pub struct DigestComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    mux_digest: &'static MuxDigest<'static, A, L>,
    key_buffer: &'static mut [u8],
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> DigestComponent<A, L> {
    pub fn new(
        mux_digest: &'static MuxDigest<'static, A, L>,
        key_buffer: &'static mut [u8],
    ) -> DigestComponent<A, L> {
        DigestComponent {
            mux_digest,
            key_buffer,
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
    > Component for DigestComponent<A, L>
{
    type StaticInput = &'static mut MaybeUninit<VirtualMuxDigest<'static, A, L>>;

    type Output = &'static VirtualMuxDigest<'static, A, L>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let virtual_digest_user = static_init_half!(
            s,
            VirtualMuxDigest<'static, A, L>,
            VirtualMuxDigest::new(self.mux_digest, self.key_buffer)
        );

        virtual_digest_user
    }
}
