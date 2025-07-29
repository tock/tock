// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the nRF52840-based dynamic processes and policies tutorial.

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::process::ProcessLoadingAsync;
use kernel::process::ShortId;
use kernel::{capabilities, create_capability, static_init};
use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

mod app_id_assigner_name_metadata;
mod checker_credentials_not_required;
mod system_call_filter;

// GPIO used for the screen shield
const SCREEN_I2C_SDA_PIN: Pin = Pin::P1_10;
const SCREEN_I2C_SCL_PIN: Pin = Pin::P1_11;

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 8;

type Chip = nrf52840dk_lib::Chip;
static mut CHIP: Option<&'static Chip> = None;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::StopWithDebugFaultPolicy =
    capsules_system::process_policies::StopWithDebugFaultPolicy {};

// How many credential verifying keys the kernel supports.
const NUM_CREDENTIAL_KEYS: usize = 1;
// Length of the key used for the ECDSA-P256 signature.
const SIGNATURE_KEY_LEN: usize = 64;
// Length of the hash used for the signature (SHA-256).
const SIGNATURE_HASH_LEN: usize = 32;
// Length of the ECDSA-P256 signature.
const SIGNATURE_SIG_LEN: usize = 64;

//------------------------------------------------------------------------------
// SYSCALL DRIVER TYPE DEFINITIONS
//------------------------------------------------------------------------------

/// Needed for process info capsule.
pub struct PMCapability;
unsafe impl capabilities::ProcessManagementCapability for PMCapability {}
unsafe impl capabilities::ProcessStartCapability for PMCapability {}

#[cfg(feature = "screen_ssd1306")]
type Screen = components::ssd1306::Ssd1306ComponentType<nrf52840::i2c::TWI<'static>>;
#[cfg(feature = "screen_sh1106")]
type Screen = components::sh1106::Sh1106ComponentType<nrf52840::i2c::TWI<'static>>;
type ScreenDriver = components::screen::ScreenSharedComponentType<Screen>;

type ProcessInfoDriver = capsules_extra::process_info_driver::ProcessInfo<PMCapability>;

type IsolatedNonvolatileStorageDriver =
    capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage<
        'static,
        {
            components::isolated_nonvolatile_storage::ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT
        },
    >;

type FlashUser =
    capsules_core::virtualizers::virtual_flash::FlashUser<'static, nrf52840::nvmc::Nvmc>;
type NonVolatilePages = components::dynamic_binary_storage::NVPages<FlashUser>;

type DynamicBinaryStorage<'a> = kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
    'static,
    'static,
    nrf52840::chip::NRF52<'a, Nrf52840DefaultPeripherals<'a>>,
    kernel::process::ProcessStandardDebugFull,
    NonVolatilePages,
>;
type AppLoaderDriver = capsules_extra::app_loader::AppLoader<
    DynamicBinaryStorage<'static>,
    DynamicBinaryStorage<'static>,
>;

type Verifier = ecdsa_sw::p256_verifier::EcdsaP256SignatureVerifier<'static>;
type SignatureVerifyInMemoryKeys =
    components::signature_verify_in_memory_keys::SignatureVerifyInMemoryKeysComponentType<
        Verifier,
        NUM_CREDENTIAL_KEYS,
        SIGNATURE_KEY_LEN,
        SIGNATURE_HASH_LEN,
        SIGNATURE_SIG_LEN,
    >;
type SignatureChecker = components::appid::checker_signature::AppCheckerSignatureComponentType<
    SignatureVerifyInMemoryKeys,
    capsules_extra::sha256::Sha256Software<'static>,
    SIGNATURE_HASH_LEN,
    SIGNATURE_SIG_LEN,
>;

//------------------------------------------------------------------------------
// SHORTID HELPER FUNCTION
//------------------------------------------------------------------------------

fn create_short_id_from_name(name: &str, metadata: u8) -> ShortId {
    let sum = kernel::utilities::helpers::crc32_posix(name.as_bytes());

    // Combine the metadata and CRC into the short id.
    let sid = ((metadata as u32) << 28) | (sum & 0xFFFFFFF);

    core::num::NonZeroU32::new(sid).into()
}

//------------------------------------------------------------------------------
// PLATFORM
//------------------------------------------------------------------------------

struct Platform {
    board_kernel: &'static kernel::Kernel,
    syscall_filter: &'static system_call_filter::DynamicPoliciesCustomFilter,
    base: nrf52840dk_lib::Platform,
    screen: &'static ScreenDriver,
    process_info: &'static ProcessInfoDriver,
    nonvolatile_storage: &'static IsolatedNonvolatileStorageDriver,
    dynamic_app_loader: &'static AppLoaderDriver,
}

// Expose system call interfaces to userspace.
impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::screen::DRIVER_NUM => f(Some(self.screen)),
            capsules_extra::process_info_driver::DRIVER_NUM => f(Some(self.process_info)),
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(self.nonvolatile_storage))
            }
            capsules_extra::app_loader::DRIVER_NUM => f(Some(self.dynamic_app_loader)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

// Configure the kernel.
impl KernelResources<Chip> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = system_call_filter::DynamicPoliciesCustomFilter;
    type ProcessFault = <nrf52840dk_lib::Platform as KernelResources<Chip>>::ProcessFault;
    type Scheduler = <nrf52840dk_lib::Platform as KernelResources<Chip>>::Scheduler;
    type SchedulerTimer = <nrf52840dk_lib::Platform as KernelResources<Chip>>::SchedulerTimer;
    type WatchDog = <nrf52840dk_lib::Platform as KernelResources<Chip>>::WatchDog;
    type ContextSwitchCallback =
        <nrf52840dk_lib::Platform as KernelResources<Chip>>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        self.syscall_filter
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        self.base.process_fault()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.base.scheduler()
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        self.base.scheduler_timer()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        self.base.watchdog()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        self.base.context_switch_callback()
    }
}

// Called by the process loader when the board boots.
impl kernel::process::ProcessLoadingAsyncClient for Platform {
    fn process_loaded(&self, _result: Result<(), kernel::process::ProcessLoadError>) {}

    fn process_loading_finished(&self) {
        kernel::debug!("Processes Loaded at Main:");

        for (i, p) in self
            .board_kernel
            .process_iter_capability(&create_capability!(
                capabilities::ProcessManagementCapability
            ))
            .enumerate()
        {
            kernel::debug!("[{}] {}", i, p.get_process_name());
            kernel::debug!("    ShortId: {}", p.short_app_id());
        }
    }
}

//------------------------------------------------------------------------------
// MAIN
//------------------------------------------------------------------------------

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // Create the base board:
    let (board_kernel, base_platform, chip, nrf52840_peripherals, _mux_alarm) =
        nrf52840dk_lib::start();

    CHIP = Some(chip);

    //--------------------------------------------------------------------------
    // SCREEN
    //--------------------------------------------------------------------------

    let i2c_bus = components::i2c::I2CMuxComponent::new(&nrf52840_peripherals.nrf52.twi1, None)
        .finalize(components::i2c_mux_component_static!(nrf52840::i2c::TWI));
    nrf52840_peripherals.nrf52.twi1.configure(
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SCL_PIN as u32),
        nrf52840::pinmux::Pinmux::new(SCREEN_I2C_SDA_PIN as u32),
    );
    nrf52840_peripherals
        .nrf52
        .twi1
        .set_speed(nrf52840::i2c::Speed::K400);

    // I2C address is b011110X, and on this board D/C̅ is GND.
    let ssd1306_sh1106_i2c = components::i2c::I2CComponent::new(i2c_bus, 0x3c)
        .finalize(components::i2c_component_static!(nrf52840::i2c::TWI));

    // Create the ssd1306 object for the actual screen driver.
    #[cfg(feature = "screen_ssd1306")]
    let ssd1306_sh1106 = components::ssd1306::Ssd1306Component::new(ssd1306_sh1106_i2c, true)
        .finalize(components::ssd1306_component_static!(nrf52840::i2c::TWI));

    #[cfg(feature = "screen_sh1106")]
    let ssd1306_sh1106 = components::sh1106::Sh1106Component::new(ssd1306_sh1106_i2c, true)
        .finalize(components::sh1106_component_static!(nrf52840::i2c::TWI));

    let apps_regions = kernel::static_init!(
        [capsules_extra::screen_shared::AppScreenRegion; 3],
        [
            capsules_extra::screen_shared::AppScreenRegion::new(
                create_short_id_from_name("process_manager", 0x0),
                0,      // x
                0,      // y
                16 * 8, // width
                7 * 8   // height
            ),
            capsules_extra::screen_shared::AppScreenRegion::new(
                create_short_id_from_name("counter", 0x0),
                0,     // x
                7 * 8, // y
                8 * 8, // width
                1 * 8  // height
            ),
            capsules_extra::screen_shared::AppScreenRegion::new(
                create_short_id_from_name("temperature", 0x0),
                8 * 8, // x
                7 * 8, // y
                8 * 8, // width
                1 * 8  // height
            )
        ]
    );

    let screen = components::screen::ScreenSharedComponent::new(
        board_kernel,
        capsules_extra::screen::DRIVER_NUM,
        ssd1306_sh1106,
        apps_regions,
    )
    .finalize(components::screen_shared_component_static!(1032, Screen));

    ssd1306_sh1106.init_screen();

    //--------------------------------------------------------------------------
    // VIRTUAL FLASH
    //--------------------------------------------------------------------------

    let mux_flash = components::flash::FlashMuxComponent::new(&nrf52840_peripherals.nrf52.nvmc)
        .finalize(components::flash_mux_component_static!(
            nrf52840::nvmc::Nvmc
        ));

    // Create a virtual flash user for dynamic binary storage
    let virtual_flash_dbs = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(nrf52840::nvmc::Nvmc),
    );

    // Create a virtual flash user for nonvolatile
    let virtual_flash_nvm = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(nrf52840::nvmc::Nvmc),
    );

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    // 32kB of userspace-accessible storage, page aligned:
    kernel::storage_volume!(APP_STORAGE, 32);

    let nonvolatile_storage = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM,
        virtual_flash_nvm,
        core::ptr::addr_of!(APP_STORAGE) as usize,
        APP_STORAGE.len()
    )
    .finalize(components::isolated_nonvolatile_storage_component_static!(
        capsules_core::virtualizers::virtual_flash::FlashUser<'static, nrf52840::nvmc::Nvmc>,
        { components::isolated_nonvolatile_storage::ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT }
    ));

    //--------------------------------------------------------------------------
    // PROCESS INFO FOR USERSPACE
    //--------------------------------------------------------------------------

    let process_info = components::process_info_driver::ProcessInfoComponent::new(
        board_kernel,
        capsules_extra::process_info_driver::DRIVER_NUM,
        PMCapability,
    )
    .finalize(components::process_info_component_static!(PMCapability));

    //--------------------------------------------------------------------------
    // SYSTEM CALL FILTERING
    //--------------------------------------------------------------------------

    let syscall_filter = static_init!(
        system_call_filter::DynamicPoliciesCustomFilter,
        system_call_filter::DynamicPoliciesCustomFilter {}
    );

    //--------------------------------------------------------------------------
    // CREDENTIAL CHECKING
    //--------------------------------------------------------------------------

    // Create the software-based SHA engine.
    let sha = components::sha::ShaSoftware256Component::new()
        .finalize(components::sha_software_256_component_static!());

    // Create the credential checker.
    //
    // Setup an example key.
    //
    // - `ec-secp256r1-priv-key.pem`:
    //   ```
    //   -----BEGIN EC PRIVATE KEY-----
    //   MHcCAQEEIGU0zCXHLqxDmrHHAWEQP5zNfWRQrAiIpH9YwxHlqysmoAoGCCqGSM49
    //   AwEHoUQDQgAE4BM6kKdKNWFRjuFECfFpwc9q239+Uvi3QXniTVdBI1IuthIDs4UQ
    //   5fMlB2KPVJWCV0VQvaPiF+g0MIkmTCNisQ==
    //   -----END EC PRIVATE KEY-----
    //   ```
    //
    // - `ec-secp256r1-pub-key.pem`:
    //   ```
    //   -----BEGIN PUBLIC KEY-----
    //   MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAE4BM6kKdKNWFRjuFECfFpwc9q239+
    //   Uvi3QXniTVdBI1IuthIDs4UQ5fMlB2KPVJWCV0VQvaPiF+g0MIkmTCNisQ==
    //   -----END PUBLIC KEY-----
    //   ```
    //
    // You can add the correct signature to a TBF by saving the private key to
    // a file and then running:
    //
    //     tockloader tbf credential add ecdsap256 --private-key ec-secp256r1-priv-key.pem
    //
    let verifying_key0 = kernel::static_init!(
        [u8; SIGNATURE_KEY_LEN],
        [
            0xe0, 0x13, 0x3a, 0x90, 0xa7, 0x4a, 0x35, 0x61, 0x51, 0x8e, 0xe1, 0x44, 0x09, 0xf1,
            0x69, 0xc1, 0xcf, 0x6a, 0xdb, 0x7f, 0x7e, 0x52, 0xf8, 0xb7, 0x41, 0x79, 0xe2, 0x4d,
            0x57, 0x41, 0x23, 0x52, 0x2e, 0xb6, 0x12, 0x03, 0xb3, 0x85, 0x10, 0xe5, 0xf3, 0x25,
            0x07, 0x62, 0x8f, 0x54, 0x95, 0x82, 0x57, 0x45, 0x50, 0xbd, 0xa3, 0xe2, 0x17, 0xe8,
            0x34, 0x30, 0x89, 0x26, 0x4c, 0x23, 0x62, 0xb1
        ]
    );
    let verifying_keys = kernel::static_init!(
        [&'static mut [u8; SIGNATURE_KEY_LEN]; NUM_CREDENTIAL_KEYS],
        [verifying_key0]
    );
    // Setup the ECDSA-P256 verifier.
    let ecdsa_p256_verifying_key =
        kernel::static_init!([u8; SIGNATURE_KEY_LEN], [0; SIGNATURE_KEY_LEN]);
    let ecdsa_p256_verifier = kernel::static_init!(
        ecdsa_sw::p256_verifier::EcdsaP256SignatureVerifier<'static>,
        ecdsa_sw::p256_verifier::EcdsaP256SignatureVerifier::new(ecdsa_p256_verifying_key)
    );
    ecdsa_p256_verifier.register();

    // Setup the in-memory key selector.
    let verifier_multiple_keys =
        components::signature_verify_in_memory_keys::SignatureVerifyInMemoryKeysComponent::new(
            ecdsa_p256_verifier,
            verifying_keys,
        )
        .finalize(
            components::signature_verify_in_memory_keys_component_static!(
                Verifier,
                NUM_CREDENTIAL_KEYS,
                SIGNATURE_KEY_LEN,
                SIGNATURE_HASH_LEN,
                SIGNATURE_SIG_LEN,
            ),
        );

    // Policy checks for a valid EcdsaNistP256 signature.
    let checking_policy_signature =
        components::appid::checker_signature::AppCheckerSignatureComponent::new(
            sha,
            verifier_multiple_keys,
            tock_tbf::types::TbfFooterV2CredentialsType::EcdsaNistP256,
        )
        .finalize(components::app_checker_signature_component_static!(
            SignatureVerifyInMemoryKeys,
            capsules_extra::sha256::Sha256Software<'static>,
            SIGNATURE_HASH_LEN,
            SIGNATURE_SIG_LEN,
        ));

    // Wrap the policy checker with a custom version that does not require valid
    // credentials to load the app. We are ok with this for this tutorial
    // because the verifying key (or lack thereof) is encoded in the AppId so
    // we can still check if an app is signed or not.
    let checking_policy = static_init!(
        checker_credentials_not_required::AppCheckerCredentialsNotRequired<
            SignatureChecker,
        >,
        checker_credentials_not_required::AppCheckerCredentialsNotRequired::new(
            checking_policy_signature
        ),
    );

    // Create the AppID assigner.
    let assigner = static_init!(
        app_id_assigner_name_metadata::AppIdAssignerNameMetadata,
        app_id_assigner_name_metadata::AppIdAssignerNameMetadata::new()
    );

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    let storage_permissions_policy =
        components::storage_permissions::null::StoragePermissionsNullComponent::new().finalize(
            components::storage_permissions_null_component_static!(
                nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
                kernel::process::ProcessStandardDebugFull,
            ),
        );

    //--------------------------------------------------------------------------
    // PROCESS LOADING
    //--------------------------------------------------------------------------

    // These symbols are defined in the standard Tock linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let app_flash = core::slice::from_raw_parts(
        core::ptr::addr_of!(_sapps),
        core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
    );
    let memory_bank = core::slice::from_raw_parts(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );
    let app_memory = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );

    // Create and start the asynchronous process loader.
    let loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        memory_bank,
        app_memory,
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // DYNAMIC PROCESS LOADING
    //--------------------------------------------------------------------------

    // Create the dynamic binary flasher.
    let dynamic_binary_storage =
        components::dynamic_binary_storage::SequentialBinaryStorageComponent::new(
            virtual_flash_dbs,
            loader,
        )
        .finalize(components::sequential_binary_storage_component_static!(
            FlashUser,
            nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
            kernel::process::ProcessStandardDebugFull,
        ));

    // Create the dynamic app loader capsule.
    let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
        board_kernel,
        capsules_extra::app_loader::DRIVER_NUM,
        dynamic_binary_storage,
        dynamic_binary_storage,
    )
    .finalize(components::app_loader_component_static!(
        DynamicBinaryStorage<'static>,
        DynamicBinaryStorage<'static>,
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let platform = static_init!(
        Platform,
        Platform {
            board_kernel,
            syscall_filter,
            base: base_platform,
            screen,
            process_info,
            nonvolatile_storage,
            dynamic_app_loader,
        }
    );
    loader.set_client(platform);

    board_kernel.kernel_loop(
        platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
