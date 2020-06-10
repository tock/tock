use std::process::Command;

use rexpect::errors::Error;
use rexpect::session::PtySession;
use rexpect::spawn;

fn kill_qemu(p: &mut PtySession) -> Result<(), Error> {
    p.send_control('a')?;
    p.send("x")?;
    p.flush()?;

    Ok(())
}

fn hifive1() -> Result<(), Error> {
    // First, build the board if needed
    // n.b. rexpect's `exp_eof` does not actually block main thread, so use
    // the standard Rust process library mechanism instead.
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/hifive1")
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = spawn("make qemu -C ../../boards/hifive1", Some(3_000))?;

    p.exp_string("HiFive1 initialization complete.")?;
    p.exp_string("Entering main loop.")?;

    // Test completed, kill QEMU
    kill_qemu(&mut p)?;

    p.exp_eof()?;
    Ok(())
}

fn opentitan() -> Result<(), Error> {
    // First, build the board if needed
    // n.b. rexpect's `exp_eof` does not actually block main thread, so use
    // the standard Rust process library mechanism instead.
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/opentitan")
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    // Get canonicalized path to opentitan rom
    let mut rom_path = std::env::current_exe().unwrap();
    rom_path.pop(); // strip exe file
    rom_path.pop(); // strip /debug
    rom_path.pop(); // strip /target
    rom_path.push("opentitan-boot-rom.elf");

    let mut p = spawn(
        &format!(
            "make OPENTITAN_BOOT_ROM={} qemu -C ../../boards/opentitan",
            rom_path.to_str().unwrap()
        ),
        Some(10_000),
    )?;

    p.exp_string("Boot ROM initialisation has completed, jump into flash")?;
    p.exp_string("OpenTitan initialisation complete.")?;
    p.exp_string("Entering main loop")?;

    // Test completed, kill QEMU
    kill_qemu(&mut p)?;

    p.exp_eof()?;
    Ok(())
}

fn main() {
    println!("Tock qemu-runner starting...");
    println!("");
    println!("Running hifive1 tests...");
    hifive1().unwrap_or_else(|e| panic!("hifive1 job failed with {}", e));
    println!("hifive1 SUCCESS.");
    println!("");
    println!("Running opentitan tests...");
    opentitan().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    println!("opentitan SUCCESS.");
}
