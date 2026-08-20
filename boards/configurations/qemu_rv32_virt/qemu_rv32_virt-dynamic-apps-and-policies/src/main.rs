// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! Board file for qemu-system-riscv32 "virt" machine with dynamic app policies.
//!
//! This configuration extends the qemu_rv32_virt-tutorial board with:
//!
//! - `process_info_driver`: exposes the process list and ShortIds to userspace.
//! - A custom `SyscallFilter` (`DynamicPoliciesCustomFilter`) that can enforce
//!   per-app policies based on the signing key encoded in each process's ShortId.
//! - `AppIdAssignerNameMetadata`: assigns ShortIds whose upper four bits record
//!   which credential key signed the app, and whose lower 28 bits are a CRC of
//!   the app name.

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::process::ProcessLoadingAsync;
use kernel::{create_capability, debug, static_init};

mod app_id_assigner_name_metadata;
mod checker_credentials_not_required;
mod system_call_filter;

//------------------------------------------------------------------------------
// BOARD CONSTANTS
//------------------------------------------------------------------------------

pub const NUM_PROCS: usize = 4;

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

// How many credential verifying keys the kernel supports.
const NUM_CREDENTIAL_KEYS: usize = 1;
// Length of the key used for the ECDSA-P256 signature.
const SIGNATURE_KEY_LEN: usize = 64;
// Length of the hash used for the signature (SHA-256).
const SIGNATURE_HASH_LEN: usize = 32;
// Length of the ECDSA-P256 signature.
const SIGNATURE_SIG_LEN: usize = 64;

//------------------------------------------------------------------------------
// TYPE DEFINITIONS
//------------------------------------------------------------------------------

type ScreenDriver = capsules_extra::screen::screen::Screen<'static>;
type ScreenAdapter = capsules_extra::screen::screen_adapters::ScreenARGB8888ToMono8BitPage<
    'static,
    qemu_rv32_virt_lib::ScreenHw,
>;
type ScreenSplitUser = components::screen::ScreenSplitUserComponentType<ScreenAdapter>;
type ScreenOnLed = components::screen_on::ScreenOnLedComponentType<ScreenSplitUser, 4, 128, 64>;
type ScreenOnLedSingle =
    capsules_extra::screen::screen_on_led::ScreenOnLedSingle<'static, ScreenOnLed>;

type LedDriver = capsules_core::led::LedDriver<'static, ScreenOnLedSingle, 4>;

type ButtonDriver = capsules_extra::button_keyboard::ButtonKeyboard<'static>;

/// Needed for the process info capsule.
pub struct PMCapability;
unsafe impl capabilities::ProcessManagementCapability for PMCapability {}
unsafe impl capabilities::ProcessStartCapability for PMCapability {}

type ProcessInfoDriver = capsules_extra::process_info_driver::ProcessInfo<PMCapability>;

type IsolatedNonvolatileStorageDriver =
    capsules_extra::isolated_nonvolatile_storage_driver::IsolatedNonvolatileStorage<
        'static,
        {
            components::isolated_nonvolatile_storage::ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT
        },
    >;

type FlashUser = capsules_core::virtualizers::virtual_flash::FlashUser<
    'static,
    qemu_rv32_virt_chip::pflash::Pflash<'static>,
>;
type NonVolatilePages = components::dynamic_binary_storage::NVPages<FlashUser>;

type DynamicBinaryStorage<'a> = kernel::dynamic_binary_storage::SequentialDynamicBinaryStorage<
    'static,
    'static,
    qemu_rv32_virt_lib::ChipHw,
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
// PLATFORM AND SYSCALL HANDLING
//------------------------------------------------------------------------------

struct Platform {
    board_kernel: &'static kernel::Kernel,
    syscall_filter: &'static system_call_filter::DynamicPoliciesCustomFilter,
    base: qemu_rv32_virt_lib::QemuRv32VirtPlatform,
    screen: Option<&'static ScreenDriver>,
    led: Option<&'static LedDriver>,
    buttons: Option<&'static ButtonDriver>,
    process_info: &'static ProcessInfoDriver,
    nonvolatile_storage: &'static IsolatedNonvolatileStorageDriver,
    dynamic_app_loader: &'static AppLoaderDriver,
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_extra::screen::screen::DRIVER_NUM => {
                if let Some(screen_driver) = self.screen {
                    f(Some(screen_driver))
                } else {
                    f(None)
                }
            }
            capsules_core::led::DRIVER_NUM => {
                if let Some(led_driver) = self.led {
                    f(Some(led_driver))
                } else {
                    f(None)
                }
            }
            capsules_core::button::DRIVER_NUM => {
                if let Some(button_driver) = self.buttons {
                    f(Some(button_driver))
                } else {
                    f(None)
                }
            }
            capsules_extra::process_info_driver::DRIVER_NUM => f(Some(self.process_info)),
            capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM => {
                f(Some(self.nonvolatile_storage))
            }
            capsules_extra::app_loader::DRIVER_NUM => f(Some(self.dynamic_app_loader)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<qemu_rv32_virt_lib::ChipHw> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = system_call_filter::DynamicPoliciesCustomFilter;
    type ProcessFault = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::ProcessFault;
    type Scheduler = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::Scheduler;
    type SchedulerTimer = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::SchedulerTimer;
    type WatchDog = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::WatchDog;
    type ContextSwitchCallback = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::ChipHw,
    >>::ContextSwitchCallback;

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

// Print loaded processes when the loader finishes.
impl kernel::process::ProcessLoadingAsyncClient for Platform {
    fn process_loaded(&self, _result: Result<(), kernel::process::ProcessLoadError>) {}

    fn process_loading_finished(&self) {
        kernel::debug!("Processes loaded:");
        for (i, p) in self
            .board_kernel
            .process_iter_capability(&create_capability!(
                capabilities::ProcessManagementCapability
            ))
            .enumerate()
        {
            kernel::debug!("[{}] {} ShortId={}", i, p.get_process_name(), p.short_app_id());
        }
    }
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, base_platform, chip) = qemu_rv32_virt_lib::start();

    //--------------------------------------------------------------------------
    // SCREEN
    //--------------------------------------------------------------------------

    let (screen, led) = base_platform
        .virtio_gpu_screen
        .map_or((None, None), |screen| {
            let screen_split = components::screen::ScreenSplitMuxComponent::new(screen).finalize(
                components::screen_split_mux_component_static!(ScreenAdapter),
            );

            let screen_split_userspace =
                components::screen::ScreenSplitUserComponent::new(screen_split, 0, 0, 128, 64)
                    .finalize(components::screen_split_user_component_static!(
                        ScreenAdapter
                    ));

            let screen_split_kernel =
                components::screen::ScreenSplitUserComponent::new(screen_split, 0, 64, 128, 64)
                    .finalize(components::screen_split_user_component_static!(
                        ScreenAdapter
                    ));

            let screen = components::screen::ScreenComponent::new(
                board_kernel,
                capsules_extra::screen::screen::DRIVER_NUM,
                screen_split_userspace,
                None,
                create_capability!(capabilities::MemoryAllocationCapability),
            )
            .finalize(components::screen_component_static!(1032));

            let screen_on_leds =
                components::screen_on::ScreenOnLedComponent::new(screen_split_kernel).finalize(
                    components::screen_on_led_component_static!(ScreenSplitUser, 4, 128, 64),
                );

            let led =
                components::led::LedsComponent::new().finalize(components::led_component_static!(
                    ScreenOnLedSingle,
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        0
                    ),
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        1
                    ),
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        2
                    ),
                    capsules_extra::screen::screen_on_led::ScreenOnLedSingle::new(
                        screen_on_leds,
                        3
                    ),
                ));

            (Some(screen), Some(led))
        });

    //--------------------------------------------------------------------------
    // SIMULATED BUTTONS USING KEYBOARD
    //--------------------------------------------------------------------------

    let buttons = base_platform.virtio_input_keyboard.map(|keyboard| {
        let key_mappings = static_init!(
            [u16; 4],
            [
                103, // UP
                14,  // BACKSPACE
                108, // DOWN
                28,  // ENTER
            ]
        );

        components::button_keyboard::KeyboardButtonComponent::new(
            board_kernel,
            capsules_extra::button_keyboard::DRIVER_NUM,
            keyboard,
            key_mappings,
            create_capability!(capabilities::MemoryAllocationCapability),
        )
        .finalize(components::keyboard_button_component_static!())
    });

    //--------------------------------------------------------------------------
    // PROCESS INFO FOR USERSPACE
    //--------------------------------------------------------------------------

    let process_info = components::process_info_driver::ProcessInfoComponent::new(
        board_kernel,
        capsules_extra::process_info_driver::DRIVER_NUM,
        PMCapability,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::process_info_component_static!(PMCapability));

    //--------------------------------------------------------------------------
    // VIRTUAL FLASH
    //--------------------------------------------------------------------------

    let mux_flash = components::flash::FlashMuxComponent::new(&chip.peripherals().pflash)
        .finalize(components::flash_mux_component_static!(
            qemu_rv32_virt_chip::pflash::Pflash<'static>
        ));

    // Create a virtual flash user for dynamic binary storage.
    let virtual_flash_dbs = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(qemu_rv32_virt_chip::pflash::Pflash<'static>),
    );

    // Create a virtual flash user for (isolated) nonvolatile storage.
    let virtual_flash_nvm = components::flash::FlashUserComponent::new(mux_flash).finalize(
        components::flash_user_component_static!(qemu_rv32_virt_chip::pflash::Pflash<'static>),
    );

    //--------------------------------------------------------------------------
    // NONVOLATILE STORAGE
    //--------------------------------------------------------------------------

    // Reserve the last sector of pflash for userspace-accessible isolated
    // nonvolatile storage, leaving the rest of the device for (dynamically
    // loaded) app images.
    const ISOLATED_STORAGE_SIZE: usize = qemu_rv32_virt_chip::pflash::PFLASH_SECTOR_SIZE;
    const ISOLATED_STORAGE_START: usize =
        qemu_rv32_virt_chip::pflash::PFLASH_BASE + qemu_rv32_virt_chip::pflash::PFLASH_SIZE
            - ISOLATED_STORAGE_SIZE;

    let nonvolatile_storage = components::isolated_nonvolatile_storage::IsolatedNonvolatileStorageComponent::new(
        board_kernel,
        capsules_extra::isolated_nonvolatile_storage_driver::DRIVER_NUM,
        virtual_flash_nvm,
        ISOLATED_STORAGE_START,
        ISOLATED_STORAGE_SIZE,
        create_capability!(capabilities::MemoryAllocationCapability),
    )
    .finalize(components::isolated_nonvolatile_storage_component_static!(
        FlashUser,
        { components::isolated_nonvolatile_storage::ISOLATED_NONVOLATILE_STORAGE_APP_REGION_SIZE_DEFAULT }
    ));

    //--------------------------------------------------------------------------
    // SYSTEM CALL FILTERING
    //--------------------------------------------------------------------------

    let syscall_filter = static_init!(
        system_call_filter::DynamicPoliciesCustomFilter,
        system_call_filter::DynamicPoliciesCustomFilter {}
    );

    // Start the process console:
    let _ = base_platform.process_console_start();

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
    // credentials to load the app. We are ok with this because the verifying
    // key (or lack thereof) is encoded in the AppId so we can still check if
    // an app is signed or not.
    let checking_policy = static_init!(
        checker_credentials_not_required::AppCheckerCredentialsNotRequired<SignatureChecker>,
        checker_credentials_not_required::AppCheckerCredentialsNotRequired::new(
            checking_policy_signature
        ),
    );

    // Create the AppID assigner — encodes credential metadata in the high bits
    // of the ShortId so the syscall filter can read which key signed each app.
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
                qemu_rv32_virt_lib::ChipHw,
                kernel::process::ProcessStandardDebugFull,
            ),
        );

    //--------------------------------------------------------------------------
    // PROCESS LOADING
    //--------------------------------------------------------------------------

    // These symbols are defined in the standard Tock linker script.
    extern "C" {
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    // App images live in pflash (the same writable device used for dynamic
    // binary storage below), excluding the sector reserved for isolated
    // nonvolatile storage at its tail.
    let app_flash = core::slice::from_raw_parts(
        qemu_rv32_virt_chip::pflash::PFLASH_BASE as *const u8,
        qemu_rv32_virt_chip::pflash::PFLASH_SIZE - ISOLATED_STORAGE_SIZE,
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
        app_memory,
        create_capability!(capabilities::ProcessManagementCapability),
    )
    .finalize(components::process_loader_sequential_component_static!(
        qemu_rv32_virt_lib::ChipHw,
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
            qemu_rv32_virt_lib::ChipHw,
            kernel::process::ProcessStandardDebugFull,
        ));

    // Create the dynamic app loader capsule.
    let dynamic_app_loader = components::app_loader::AppLoaderComponent::new(
        board_kernel,
        capsules_extra::app_loader::DRIVER_NUM,
        dynamic_binary_storage,
        dynamic_binary_storage,
        create_capability!(capabilities::MemoryAllocationCapability),
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
            led,
            buttons,
            process_info,
            nonvolatile_storage,
            dynamic_app_loader,
        }
    );
    loader.set_client(platform);

    debug!("Starting main kernel loop.");

    board_kernel.kernel_loop(
        platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
