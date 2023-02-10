//! Components for collections of Digests.
//!
//! Usage
//! -----
//! ```rust
//!    let mux_digest = components::digest::DigestMuxComponent::new(&earlgrey::digest::Digest)
//!        .finalize(
//!            components::digest_mux_component_static!(lowrisc::digest::Digest, [u8; 32]),
//!        );
//!
//!    let digest = components::digest::DigestComponent::new(
//!        board_kernel,
//!        &mux_digest,
//!    )
//!    .finalize(components::digest_component_static!(
//!        lowrisc::digest::Digest,
//!        [u8; 32]
//!    ));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_digest::MuxDigest;
use core_capsules::virtual_digest::VirtualMuxDigest;
use kernel::component::Component;
use kernel::hil::digest;

#[macro_export]
macro_rules! digest_mux_component_static {
    ($A:ty, $L:expr $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_digest::MuxDigest<'static, $A, $L>)
    };};
}

#[macro_export]
macro_rules! digest_component_static {
    ($A:ty, $L:expr $(,)?) => {{
        let virtual_mux =
            kernel::static_buf!(core_capsules::virtual_digest::VirtualMuxDigest<'static, $A, $L>);
        let key_buffer = kernel::static_buf!([u8; $L]);

        (virtual_mux, key_buffer)
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
            + digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + digest::Sha256
            + digest::Sha384
            + digest::Sha512,
        const L: usize,
    > Component for DigestMuxComponent<A, L>
{
    type StaticInput = &'static mut MaybeUninit<MuxDigest<'static, A, L>>;
    type Output = &'static MuxDigest<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(MuxDigest::new(self.digest))
    }
}

pub struct DigestComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    mux_digest: &'static MuxDigest<'static, A, L>,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> DigestComponent<A, L> {
    pub fn new(mux_digest: &'static MuxDigest<'static, A, L>) -> DigestComponent<A, L> {
        DigestComponent { mux_digest }
    }
}

impl<
        A: kernel::hil::digest::HmacSha256
            + digest::HmacSha384
            + digest::HmacSha512
            + 'static
            + digest::Digest<'static, L>,
        const L: usize,
    > Component for DigestComponent<A, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxDigest<'static, A, L>>,
        &'static mut MaybeUninit<[u8; L]>,
    );
    type Output = &'static VirtualMuxDigest<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let key_buffer = s.1.write([0; L]);
        let virtual_digest_user =
            s.0.write(VirtualMuxDigest::new(self.mux_digest, key_buffer));

        virtual_digest_user
    }
}
