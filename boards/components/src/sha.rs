//! Components for collections of SHAa.
//!
//! Usage
//! -----
//! ```rust
//!    let sha_data_buffer = static_init!([u8; 64], [0; 64]);
//!    let sha_dest_buffer = static_init!([u8; 32], [0; 32]);
//!
//!    let mux_sha = components::sha::ShaMuxComponent::new(&earlgrey::sha::HMAC).finalize(
//!        components::sha_mux_component_helper!(lowrisc::sha::Sha, [u8; 32]),
//!    );
//!
//!    let sha = components::sha::ShaComponent::new(
//!        board_kernel,
//!        &mux_sha,
//!        sha_data_buffer,
//!        sha_dest_buffer,
//!    )
//!    .finalize(components::sha_component_helper!(
//!        lowrisc::sha::Sha,
//!        32,
//!    ));
//! ```

use capsules;
use capsules::sha::ShaDriver;
use capsules::virtual_sha::MuxSha;
use capsules::virtual_sha::VirtualMuxSha;
use core::mem::MaybeUninit;
use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::digest;
use kernel::static_init_half;

// Setup static space for the objects.
#[macro_export]
macro_rules! sha_mux_component_helper {
    ($A:ty, $L:expr $(,)?) => {{
        use capsules::virtual_sha::MuxSha;
        use capsules::virtual_sha::VirtualMuxSha;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<MuxSha<'static, $A, $L>> = MaybeUninit::uninit();
        &mut BUF1
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

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let mux_sha = static_init_half!(s, MuxSha<'static, A, L>, MuxSha::new(self.sha));

        mux_sha
    }
}

// Setup static space for the objects.
#[macro_export]
macro_rules! sha_component_helper {
    ($A:ty, $L:expr$(,)?) => {{
        use capsules::sha::ShaDriver;
        use capsules::virtual_sha::MuxSha;
        use capsules::virtual_sha::VirtualMuxSha;
        use core::mem::MaybeUninit;
        static mut BUF1: MaybeUninit<VirtualMuxSha<'static, $A, $L>> = MaybeUninit::uninit();
        static mut BUF2: MaybeUninit<ShaDriver<'static, VirtualMuxSha<'static, $A, $L>, $L>> =
            MaybeUninit::uninit();
        (&mut BUF1, &mut BUF2)
    };};
}

pub struct ShaComponent<A: 'static + digest::Digest<'static, L>, const L: usize> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    mux_sha: &'static MuxSha<'static, A, L>,
    data_buffer: &'static mut [u8],
    dest_buffer: &'static mut [u8; L],
}

impl<A: 'static + digest::Digest<'static, L>, const L: usize> ShaComponent<A, L> {
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        mux_sha: &'static MuxSha<'static, A, L>,
        data_buffer: &'static mut [u8],
        dest_buffer: &'static mut [u8; L],
    ) -> ShaComponent<A, L> {
        ShaComponent {
            board_kernel,
            driver_num,
            mux_sha,
            data_buffer,
            dest_buffer,
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
    );

    type Output = &'static ShaDriver<'static, VirtualMuxSha<'static, A, L>, L>;

    unsafe fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let virtual_sha_user = static_init_half!(
            s.0,
            VirtualMuxSha<'static, A, L>,
            VirtualMuxSha::new(self.mux_sha)
        );

        let sha = static_init_half!(
            s.1,
            capsules::sha::ShaDriver<'static, VirtualMuxSha<'static, A, L>, L>,
            capsules::sha::ShaDriver::new(
                virtual_sha_user,
                self.data_buffer,
                self.dest_buffer,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
            )
        );

        sha
    }
}
