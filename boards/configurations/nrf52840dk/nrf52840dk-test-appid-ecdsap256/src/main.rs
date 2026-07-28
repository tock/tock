// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::deferred_call::DeferredCallClient;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

type Verifier = ecdsa_sw::p256_verifier::EcdsaP256SignatureVerifier<'static>;
type SignatureVerifyInMemoryKeys =
    components::signature_verify_in_memory_keys::SignatureVerifyInMemoryKeysComponentType<
        Verifier,
        2,
        64,
        32,
        64,
    >;

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let (board_kernel, platform, chip, _default_peripherals, _mux_uart, _mux_alarm) =
        nrf52840dk_test_base_lib::start();

    //--------------------------------------------------------------------------
    // Credential Checking
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
        [u8; 64],
        [
            0xe0, 0x13, 0x3a, 0x90, 0xa7, 0x4a, 0x35, 0x61, 0x51, 0x8e, 0xe1, 0x44, 0x09, 0xf1,
            0x69, 0xc1, 0xcf, 0x6a, 0xdb, 0x7f, 0x7e, 0x52, 0xf8, 0xb7, 0x41, 0x79, 0xe2, 0x4d,
            0x57, 0x41, 0x23, 0x52, 0x2e, 0xb6, 0x12, 0x03, 0xb3, 0x85, 0x10, 0xe5, 0xf3, 0x25,
            0x07, 0x62, 0x8f, 0x54, 0x95, 0x82, 0x57, 0x45, 0x50, 0xbd, 0xa3, 0xe2, 0x17, 0xe8,
            0x34, 0x30, 0x89, 0x26, 0x4c, 0x23, 0x62, 0xb1
        ]
    );
    // - `ec-secp256r1-priv-key2.pem`:
    //   ```
    //   -----BEGIN EC PRIVATE KEY-----
    //   MHcCAQEEIMlpHXMiwjFiTRH015zyxsur59JVKzBUzM9jQTUSjcC9oAoGCCqGSM49
    //   AwEHoUQDQgAEyT04ecALSi9cv8r8AyQUe++on+X1K3ec2fNR/bw35wwp5u7DxO1X
    //   bZWNw8Bzh031jaY+je/40/CnCCKt9/ejqg==
    //   -----END EC PRIVATE KEY-----
    //   ```
    //
    // - `ec-secp256r1-pub-key2.pem`:
    //   ```
    //   -----BEGIN PUBLIC KEY-----
    //   MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEyT04ecALSi9cv8r8AyQUe++on+X1
    //   K3ec2fNR/bw35wwp5u7DxO1XbZWNw8Bzh031jaY+je/40/CnCCKt9/ejqg==
    //   -----END PUBLIC KEY-----
    //   ```
    let verifying_key1 = kernel::static_init!(
        [u8; 64],
        [
            0xc9, 0x3d, 0x38, 0x79, 0xc0, 0x0b, 0x4a, 0x2f, 0x5c, 0xbf, 0xca, 0xfc, 0x03, 0x24,
            0x14, 0x7b, 0xef, 0xa8, 0x9f, 0xe5, 0xf5, 0x2b, 0x77, 0x9c, 0xd9, 0xf3, 0x51, 0xfd,
            0xbc, 0x37, 0xe7, 0x0c, 0x29, 0xe6, 0xee, 0xc3, 0xc4, 0xed, 0x57, 0x6d, 0x95, 0x8d,
            0xc3, 0xc0, 0x73, 0x87, 0x4d, 0xf5, 0x8d, 0xa6, 0x3e, 0x8d, 0xef, 0xf8, 0xd3, 0xf0,
            0xa7, 0x08, 0x22, 0xad, 0xf7, 0xf7, 0xa3, 0xaa,
        ]
    );
    let verifying_keys =
        kernel::static_init!([&'static mut [u8; 64]; 2], [verifying_key0, verifying_key1]);
    // kernel::static_init!([&'static mut [u8; 64]; 1], [verifying_key0]);
    // Setup the ECDSA-P256 verifier.
    let ecdsa_p256_verifying_key = kernel::static_init!([u8; 64], [0; 64]);
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
            components::signature_verify_in_memory_keys_component_static!(Verifier, 2, 64, 32, 64,),
        );

    // Policy checks for a valid EcdsaNistP256 signature.
    let checking_policy = components::appid::checker_signature::AppCheckerSignatureComponent::new(
        sha,
        verifier_multiple_keys,
        tock_tbf::types::TbfFooterV2CredentialsType::EcdsaNistP256,
    )
    .finalize(components::app_checker_signature_component_static!(
        SignatureVerifyInMemoryKeys,
        capsules_extra::sha256::Sha256Software<'static>,
        32,
        64,
    ));

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
    let app_memory = core::slice::from_raw_parts_mut(
        core::ptr::addr_of_mut!(_sappmem),
        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
    );

    // Create and start the asynchronous process loader.
    let _loader = components::loader::sequential::ProcessLoaderSequentialComponent::new(
        checker,
        board_kernel,
        chip,
        &nrf52840dk_test_base_lib::FAULT_RESPONSE,
        assigner,
        storage_permissions_policy,
        app_flash,
        app_memory,
        create_capability!(capabilities::ProcessManagementCapability),
    )
    .finalize(components::process_loader_sequential_component_static!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        kernel::process::ProcessStandardDebugFull,
        nrf52840dk_test_base_lib::NUM_PROCS
    ));

    //--------------------------------------------------------------------------
    // PLATFORM SETUP, SCHEDULER, AND START KERNEL LOOP
    //--------------------------------------------------------------------------

    let main_loop_capability = create_capability!(kernel::capabilities::MainLoopCapability);
    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}
