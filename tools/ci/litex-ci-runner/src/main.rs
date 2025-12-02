// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use rexpect::errors::Error;
use rexpect::process::signal::Signal;
use rexpect::session::PtySession;
use rexpect::spawn;

use uuid::Uuid;

use litex_simctrl::gpio::GpioCtrl;
use litex_simctrl::sim::SimCtrl;

// Semantic GPIO assignments
// const LED0: u64 = 0;
// const LED1: u64 = 1;
// const LED2: u64 = 2;
// const LED3: u64 = 3;
// const LED4: u64 = 4;
// const LED5: u64 = 5;
// const LED6: u64 = 6;
// const LED7: u64 = 7;
const BTN0: u64 = 8;
// const BTN1: u64 = 9;
// const BTN2: u64 = 10;
// const BTN3: u64 = 11;
// const BTN4: u64 = 12;
// const BTN5: u64 = 13;
// const BTN6: u64 = 14;
// const BTN7: u64 = 15;

fn run_with_apps<F, R>(kernel_binary: &str, app_paths: &[&str], pty_fn: F) -> Result<R, ()>
where
    F: FnOnce(&mut PtySession) -> R,
{
    // First, get a path to a new temporary file representing the board's flash
    // region.
    let mut flash_file = std::env::temp_dir();
    flash_file.push(format!("litex_ci_runner-{}", Uuid::new_v4().to_string()));
    // Allocates the file in the filesystem, which is required by tockloader
    {
        std::fs::File::create(flash_file.clone())
            .expect("failed to create board flash file")
            .sync_all()
            .unwrap();
    }

    // Now, use tockloader to flash the kernel to the file
    println!("Flashing kernel \"{}\"...", kernel_binary);
    let mut tockloader_flash = std::process::Command::new("tockloader")
        .arg("flash")
        .arg("--flash-file")
        .arg(&flash_file.clone().into_os_string())
        .arg("--board")
        .arg("litex_sim")
        .arg("-a")
        .arg("0x0")
        .arg(kernel_binary)
        .spawn()
        .expect("failed to flash the board kernel");
    assert!(tockloader_flash.wait().unwrap().success());

    // Use tockloader to install the apps on the board. We expect to have
    // sufficient builds of the non-relocatable RISC-V apps such that tockloader
    // can always find a valid combination. If this doesn't work, TOCK_TARGETS
    // might need some adjustment.
    for app_path in app_paths.iter() {
        println!("Installing app \"{}\"...", app_path);
        let mut tockloader_app_install = std::process::Command::new("tockloader")
            .current_dir(app_path)
            .arg("install")
            .arg("--flash-file")
            .arg(&flash_file.clone().into_os_string())
            .arg("--board")
            .arg("litex_sim")
            .spawn()
            .expect("failed to install app in board flash");
        assert!(tockloader_app_install.wait().unwrap().success());
    }

    // With the kernel and all apps prepared, run the simulation
    //
    // --with-ethernet will cause the simulation to run under a
    // privileged (root) user. This might change in the future, but
    // for now will use sudo to get elevated privileges. This will
    // thus hang if sudo prompts for a password.
    let mut p = spawn(
        &format!(
            "litex_sim \
            --csr-data-width=32 \
            --integrated-rom-size=0x100000 \
            --integrated-main-ram-size=0x10000000 \
            --cpu-variant=tock+secure+imc \
            --with-ethernet \
            --timer-uptime \
            --with-gpio \
            --rom-init {} \
            --non-interactive \
            --with-simctrl \
            --no-compile-software",
            flash_file.to_string_lossy(),
        ),
        // The initial build might take a while, which is why we're
        // allowing it to run for 2min between exp_* commands.
        Some(120_000),
    )
    .expect("failed to run the simulation");

    // Execute user-defined test
    let res = pty_fn(&mut p);

    // Test completed, kill the simulation
    p.process
        .kill(Signal::SIGINT)
        .expect("failed to kill the simulation");

    Ok(res)
}

fn run() -> Result<(), Error> {
    println!("litex-sim CI runner starting...");
    println!("");

    // Shortcut function, with the kernel binary path and the
    // libtock-c path prefix fixed and prepended to the
    // application. Always expects the kernel message first.
    fn libtock_c_examples_test(
        example_apps: &[&str],
        pty_fn: impl FnOnce(&mut PtySession, &GpioCtrl) -> Result<(), Error>,
    ) -> Result<(), Error> {
        println!("Testing with apps: {:?}", example_apps);

        // Prepend the libtock-c path to the example apps
        let app_paths: Vec<String> = example_apps
            .iter()
            .map(|example_path| format!("../../../libtock-c/examples/{}", example_path))
            .collect();
        let app_paths_str: Vec<&str> = app_paths.iter().map(|s| &**s).collect();

        let res = run_with_apps(
            "../../../target/riscv32imc-unknown-none-elf/release/litex_sim.bin",
            &app_paths_str,
            |p: &mut PtySession| -> Result<(), Error> {
                println!("Starting test. This will compile a Verilated LiteX simulation and thus might take a bit...");

                // Always expect the kernel greeting
                p.exp_string(
                    "Verilated LiteX+VexRiscv: initialization complete, entering main loop.",
                )?;

                println!("We're up! Got the kernel greeting. Connecting to ZeroMQ simulation control socket...");
                let simctrl = SimCtrl::new("tcp://localhost:7173").unwrap();
                // We expect to not have multiple GPIO module instances in the
                // simulation, so just pick the first session found.
                let gpio_session = GpioCtrl::list_sessions(&simctrl).unwrap().next().unwrap();
                let gpioctrl = GpioCtrl::new(&simctrl, gpio_session.session_id).unwrap();

                // Execute custom user code
                pty_fn(p, &gpioctrl)?;

                println!("Test succeeded!");
                Ok(())
            },
        ).unwrap();

        // Print a few dashes after each test, this makes it
        // easier to tell when a new test starts
        for _ in 0..80 {
            print!("-");
        }
        println!();

        // Return the test result
        res
    }

    // Test: c_hello
    //
    // Tests basic functionality and whether apps can run at all.
    libtock_c_examples_test(&["c_hello"], |p, _| {
        p.exp_string("Hello World!")?;
        Ok(())
    })?;

    // Test: console_timeout
    //
    // After receiving the kernel greeting, write some characters to
    // the console. After a short period of time, the application
    // should return that.
    //
    // Tests parts of the UART stack and the LiteX UART driver.
    libtock_c_examples_test(&["tests/console/console_timeout"], |p, _| {
        p.send_line("Tock is awesome!")?;
        p.exp_string("Userspace call to read console returned: Tock is awesome!")?;
        Ok(())
    })?;

    // Test: mpu_walk_region, no. 1
    //
    // This tests two things:
    //
    // - it tests that both the entire flash region and RAM region of a
    //   process are accessible and can be read.
    //
    // - it tests whether an overrun of the flash region will trigger a process fault
    //   by the MPU because of a load page fault.
    //
    // It does not make sense to split them up, as we must do a successful pass
    // of both the flash and RAM region anyways before we can overrun the flash
    // region. This is because we need a reliably detectable moment in time to
    // "press" the overrun button. Once we receive the message that the
    // application will walk a region, it's already too late for that. Thus the
    // button is pressed immediately after the start of the prior run.
    libtock_c_examples_test(&["tests/mpu/mpu_walk_region"], |p, gpio| {
        p.exp_string("[TEST] MPU Walk Regions")?;

        // Start walking the entire flash region, should not panic
        p.exp_string("Walking flash")?;

        // If we receive this string, walking the flash region worked. Start
        // walking the entire RAM region, also should not panic.
        p.exp_string("Walking memory")?;
        println!("Walking flash worked.");

        // As soon as we see the first " incr " print, we know the device
        // started walking RAM. Now its safe to "press" the button, so that we
        // will overrun flash in the next iteration.
        p.exp_string(" incr ")?;
        gpio.set_input(BTN0, true).unwrap();
        let button_state = gpio.get_state(BTN0).unwrap();
        println!(
            "Set button input to {}, currently driven by {}.",
            button_state.state, button_state.driven_by
        );
        println!("Expecting to overrun flash region.");

        // If we receive this string, walking the flash region worked. Start
        // walking the entire memory region, this should now detect the button
        // is pushed.
        p.exp_regex("Walking flash[[[:space:]]!]*Will overrun")?;
        println!("Walking RAM worked, walking flash now, will overrun!");

        p.exp_string("mpu_walk_region had a fault")?;
        println!("Process faulted.");

        p.exp_string("mpu_walk_region   -   [Faulted]")?;
        p.exp_string("mcause: 0x00000005 (Load access fault)")?;
        println!("Process had a load access fault, as expected.");

        Ok(())
    })?;

    // Test: mpu_walk_region, no 2
    //
    // Tests whether an overrun of the RAM region will trigger a process fault
    // by the MPU because of a load page fault.
    libtock_c_examples_test(&["tests/mpu/mpu_walk_region"], |p, gpio| {
        p.exp_string("[TEST] MPU Walk Regions")?;

        // Start walking the entire flash region.
        p.exp_string("Walking flash")?;

        // As soon as we see the first " incr " print, we know the device
        // started walking flash. Now its safe to "press" the button, so that we
        // will overrun the next walk RAM run.
        p.exp_string(" incr ")?;
        gpio.set_input(BTN0, true).unwrap();
        let button_state = gpio.get_state(BTN0).unwrap();
        println!(
            "Set button input to {}, currently driven by {}.",
            button_state.state, button_state.driven_by
        );
        println!("Expecting to overrun RAM region.");

        // If we receive this string, walking the flash region worked. Start
        // walking the entire memory region, this should now detect the button
        // is pushed.
        p.exp_regex("Walking memory[[[:space:]]!]*Will overrun")?;
        println!("Walking flash worked, walking RAM now, will overrun!");

        p.exp_string("mpu_walk_region had a fault")?;
        println!("Process faulted.");

        p.exp_string("mpu_walk_region   -   [Faulted]")?;
        p.exp_string("mcause: 0x00000005 (Load access fault)")?;
        println!("Process had a load access fault, as expected.");

        Ok(())
    })?;

    // Test: c_hello and printf_long
    //
    // Most importantly, tests whether multiple apps can run on the
    // board. Furthermore tests whether the console (TX) is properly
    // muxed between applications.
    libtock_c_examples_test(&["c_hello", "tests/printf_long"], |p, _| {
        // Output may arrive out of order, but verify that all three
        // messages arrive fully, and the second message of
        // printf_long arrives after the first one.
        let (_, matched) = p.exp_regex("(Hi welcome to Tock\\. |Hello World!)")?;
        if matched == "Hi welcome to Tock. " {
            let (_, matched) = p.exp_regex("(Hello World!|This test makes sure )")?;
            if matched == "Hello World!" {
                p.exp_string(
                    "This test makes sure that a greater than 64 byte message can be printed.",
                )?;
            } else {
                p.exp_string("that a greater than 64 byte message can be printed.")?;
            }
        } else {
            p.exp_string("Hi welcome to Tock. This test makes sure that a greater than 64 byte message can be printed.")?;
            p.exp_string("And a short message.")?;
        }

        Ok(())
    })?;

    // Test: rot13_client and rot13_service
    //
    // Tests IPC and IPC service discovery.
    libtock_c_examples_test(&["rot13_client", "rot13_service"], |p, _| {
        // Let run for a few cycles to make sure it doesn't crash
        // after the first few messages
        for _ in 0..10 {
            p.exp_string("12: Uryyb Jbeyq!")?;
            p.exp_string("12: Hello World!")?;
        }

        Ok(())
    })?;

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Experienced errors during the test: {}", e);
        std::process::exit(1);
    } else {
        println!("Tests successful!");
    }
}
