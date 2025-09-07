// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Board file for qemu-system-riscv32 "virt" machine type

#![no_std]
#![no_main]

use kernel::capabilities;
use kernel::component::Component;
use kernel::deferred_call::DeferredCallClient;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::{create_capability, debug, static_init};

mod checker_credentials_not_required;

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
    base: qemu_rv32_virt_lib::QemuRv32VirtPlatform,
    screen: Option<&'static ScreenDriver>,
    led: Option<&'static LedDriver>,
    buttons: Option<&'static ButtonDriver>,
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

            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<qemu_rv32_virt_lib::Chip> for Platform {
    type SyscallDriverLookup = Self;
    type SyscallFilter = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::Chip,
    >>::SyscallFilter;
    type ProcessFault = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::Chip,
    >>::ProcessFault;
    type Scheduler = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::Chip,
    >>::Scheduler;
    type SchedulerTimer = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::Chip,
    >>::SchedulerTimer;
    type WatchDog = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::Chip,
    >>::WatchDog;
    type ContextSwitchCallback = <qemu_rv32_virt_lib::QemuRv32VirtPlatform as KernelResources<
        qemu_rv32_virt_lib::Chip,
    >>::ContextSwitchCallback;

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        self.base.syscall_filter()
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
        )
        .finalize(components::keyboard_button_component_static!())
    });

    let platform = Platform {
        base: base_platform,
        screen,
        led,
        buttons,
    };

    // Start the process console:
    let _ = platform.base.pconsole.start();

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
    let assigner = components::appid::assigner_name::AppIdAssignerNamesComponent::new()
        .finalize(components::appid_assigner_names_component_static!());

    // Create the process checking machine.
    let checker = components::appid::checker::ProcessCheckerMachineComponent::new(checking_policy)
        .finalize(components::process_checker_machine_component_static!());

    //--------------------------------------------------------------------------
    // STORAGE PERMISSIONS
    //--------------------------------------------------------------------------

    let storage_permissions_policy =
        components::storage_permissions::null::StoragePermissionsNullComponent::new().finalize(
            components::storage_permissions_null_component_static!(
                qemu_rv32_virt_lib::Chip,
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
    let app_memory = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );

    // Create and start the asynchronous process loader.
    let _loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
    )
    .finalize(components::process_loader_sequential_component_static!(
        qemu_rv32_virt_lib::Chip,
        kernel::process::ProcessStandardDebugFull,
        NUM_PROCS
    ));

    debug!("Starting main kernel loop.");

    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
