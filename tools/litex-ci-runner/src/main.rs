use rexpect::errors::Error;
use rexpect::process::signal::Signal;
use rexpect::session::PtySession;
use rexpect::spawn;

use uuid::Uuid;

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
            --cpu-variant=secure \
            --with-ethernet \
            --timer-uptime \
            --rom-init {} \
            --non-interactive",
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
        pty_fn: impl FnOnce(&mut PtySession) -> Result<(), Error>,
    ) -> Result<(), Error> {
        println!("Testing with apps: {:?}", example_apps);

        // Prepend the libtock-c path to the example apps
        let app_paths: Vec<String> = example_apps
            .iter()
            .map(|example_path| format!("../../libtock-c/examples/{}", example_path))
            .collect();
        let app_paths_str: Vec<&str> = app_paths.iter().map(|s| &**s).collect();

        let res = run_with_apps(
            "../../target/riscv32i-unknown-none-elf/release/litex_sim.bin",
            &app_paths_str,
            |p: &mut PtySession| -> Result<(), Error> {
                println!("Starting test. This will compile a Verilated LiteX simulation and thus might take a bit...");

                // Always expect the kernel greeting
                p.exp_string(
                    "Verilated LiteX+VexRiscv: initialization complete, entering main loop.",
                )?;
                println!("We're up! Got the kernel greeting. Running custom tests...");

                // Execute custom user code
                pty_fn(p)?;

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
    libtock_c_examples_test(&["c_hello"], |p| {
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
    libtock_c_examples_test(&["tests/console_timeout"], |p| {
        p.send_line("Tock is awesome!")?;
        p.exp_string("Userspace call to read console returned: Tock is awesome!")?;
        Ok(())
    })?;

    // Test: mpu_walk_region, no 1
    //
    // Tests whether the MPU regions are entirely accessible by the
    // application.
    libtock_c_examples_test(&["tests/mpu_walk_region"], |p| {
        p.exp_string("[TEST] MPU Walk Regions")?;

        // Start walking the entire flash region, should not panic
        p.exp_string("Walking flash")?;

        // If we receive this string, walking the flash region
        // worked. Start walking the entire memory region, also should
        // not panic.
        p.exp_string("Walking memory")?;
        println!("Walking flash worked.");

        // One final time, expect "Walking flash", which indicates
        // that walking the memory region worked.
        p.exp_string("Walking flash")?;
        println!("Walking memory worked.");

        Ok(())
    })?;

    // TODO: simulate a button press to overrun both the memory and
    // flash sections to verify that the MPU protections indeed work

    // Test: c_hello and printf_long
    //
    // Most importantly, tests whether multiple apps can run on the
    // board. Furthermore tests whether the console (TX) is properly
    // muxed between applications.
    libtock_c_examples_test(&["c_hello", "tests/printf_long"], |p| {
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
    libtock_c_examples_test(&["rot13_service", "rot13_client"], |p| {
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
