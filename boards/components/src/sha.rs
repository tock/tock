//! Components for collections of SHA.
//!
//! Usage
//! -----
//! ```rust
//!    let mux_sha = components::sha::ShaMuxComponent::new(&earlgrey::sha::HMAC).finalize(
//!        components::sha_mux_component_static!(lowrisc::sha::Sha, 32),
//!    );
//!
//!    let sha = components::sha::ShaComponent::new(
//!        board_kernel,
//!        &mux_sha,
//!    )
//!    .finalize(components::sha_component_static!(
//!        lowrisc::sha::Sha,
//!        32,
//!    ));
//! ```

use core::mem::MaybeUninit;
use core_capsules::virtual_sha::MuxSha;
use core_capsules::virtual_sha::VirtualMuxSha;
use extra_capsules::sha::ShaDriver;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;

// Setup static space for the objects.
#[macro_export]
macro_rules! sha_mux_component_static {
    ($A:ty, $L:expr $(,)?) => {{
        kernel::static_buf!(core_capsules::virtual_sha::MuxSha<'static, $A, $L>)
    };};
}

pub struct ShaMuxComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    sha: &'static A,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> ShaMuxComponent<A, L> {
    pub fn new(sha: &'static A) -> ShaMuxComponent<A, L> {
        ShaMuxComponent { sha }
    }
}

impl<
        A: 'static + digest::Digest<'static, L> + digest::Sha256 + digest::Sha384 + digest::Sha512,
        const L: usize,
    > Component for ShaMuxComponent<A, L>
{
    type StaticInput = &'static mut MaybeUninit<MuxSha<'static, A, L>>;
    type Output = &'static MuxSha<'static, A, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        s.write(MuxSha::new(self.sha))
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! sha_component_static {
    ($A:ty, $L:expr$(,)?) => {{
        let sha_mux =
            kernel::static_buf!(core_capsules::virtual_sha::VirtualMuxSha<'static, $A, $L>);
        let sha_driver = kernel::static_buf!(
            extra_capsules::sha::ShaDriver<
                'static,
                core_capsules::virtual_sha::VirtualMuxSha<'static, $A, $L>,
                $L,
            >
        );

        let data_buffer = kernel::static_buf!([u8; 64]);
        let dest_buffer = kernel::static_buf!([u8; $L]);

        (sha_mux, sha_driver, data_buffer, dest_buffer)
    };};
}

pub struct ShaComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mux_sha: &'static MuxSha<'static, A, L>,
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> ShaComponent<A, L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mux_sha: &'static MuxSha<'static, A, L>,
    ) -> ShaComponent<A, L> {
        ShaComponent {
            board_kernel,
            driver_num,
            mux_sha,
        }
    }
}

impl<
        A: kernel::hil::digest::Sha256
            + digest::Sha384
            + digest::Sha512
            + 'static
            + digest::Digest<'static, L>,
        const L: usize,
    > Component for ShaComponent<A, L>
{
    type StaticInput = (
        &'static mut MaybeUninit<VirtualMuxSha<'static, A, L>>,
        &'static mut MaybeUninit<ShaDriver<'static, VirtualMuxSha<'static, A, L>, L>>,
        &'static mut MaybeUninit<[u8; 64]>,
        &'static mut MaybeUninit<[u8; L]>,
    );

    type Output = &'static ShaDriver<'static, VirtualMuxSha<'static, A, L>, L>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_sha_user = s.0.write(VirtualMuxSha::new(self.mux_sha));

        let data_buffer = s.2.write([0; 64]);
        let dest_buffer = s.3.write([0; L]);

        let sha = s.1.write(extra_capsules::sha::ShaDriver::new(
            virtual_sha_user,
            data_buffer,
            dest_buffer,
            self.board_kernel.create_grant(self.driver_num, &grant_cap),
        ));

        sha
    }
}
