// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use rexpect::errors::Error;
use rexpect::spawn_stream;
use serialport::prelude::*;
use serialport::SerialPortSettings;
use std::env;
use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{thread, time};

fn earlgrey_cw310_flash(
    app_name: &str,
) -> Result<rexpect::session::StreamSession<std::boxed::Box<dyn serialport::SerialPort>>, Error> {
    let s = SerialPortSettings {
        baud_rate: 115200,
        data_bits: DataBits::Eight,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Duration::from_millis(1000),
    };

    // Open the first serialport available.
    let port_name = &serialport::available_ports().expect("No serial port")[1].port_name;
    println!("Connecting to OpenTitan port: {:?}", port_name);
    let port = serialport::open_with_settings(port_name, &s).expect("Failed to open serial port");

    // Clone the port
    let port_clone = port.try_clone().expect("Failed to clone");

    // Create the Rexpect instance
    let mut p = spawn_stream(port, port_clone, Some(2000));

    // Flash the Tock kernel and app
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/opentitan/earlgrey-cw310")
        .arg(format!(
            "OPENTITAN_TREE={}",
            env::var("OPENTITAN_TREE").unwrap()
        ))
        .arg(format!("APP={}", app_name))
        .arg("flash-app")
        .stdout(Stdio::null())
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    // Make sure the image is flashed
    p.exp_string("Processing frame #13, expecting #13")?;
    p.exp_string("Processing frame #67, expecting #67")?;

    p.exp_string("Test ROM complete, jumping to flash!")?;

    Ok(p)
}

fn earlgrey_cw310_c_hello() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/c_hello/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.exp_string("Hello World!")?;

    Ok(())
}

fn earlgrey_cw310_blink() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/blink/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let _p = earlgrey_cw310_flash(&app).unwrap();

    println!("Make sure the LEDs are blinking");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

fn earlgrey_cw310_c_hello_and_printf_long() -> Result<(), Error> {
    let app = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/c_hello/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let app = OpenOptions::new()
        .append(true)
        .create(false)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/tests/printf_long/build/rv32imc/rv32imc.0x20030880.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = earlgrey_cw310_flash("../../../tools/board-runner/app").unwrap();

    p.exp_string("Hello World!")?;
    p.exp_string("Hi welcome to Tock. This test makes sure that a greater than 64 byte message can be printed.")?;
    p.exp_string("And a short message.")?;

    Ok(())
}

fn earlgrey_cw310_recv_short_and_recv_long() -> Result<(), Error> {
    let app = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/tests/console_recv_short/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let app = OpenOptions::new()
        .append(true)
        .create(false)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/tests/console_recv_long/build/rv32imc/rv32imc.0x20034080.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = earlgrey_cw310_flash("../../../tools/board-runner/app").unwrap();

    p.exp_string("Error doing UART receive: -2")?;

    Ok(())
}

fn earlgrey_cw310_blink_and_c_hello_and_buttons() -> Result<(), Error> {
    let app = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/blink/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let app = OpenOptions::new()
        .append(true)
        .create(false)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/c_hello/build/rv32imc/rv32imc.0x20030880.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let app = OpenOptions::new()
        .append(true)
        .create(false)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/buttons/build/rv32imc/rv32imc.0x20034080.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = earlgrey_cw310_flash("../../../tools/board-runner/app").unwrap();

    p.exp_string("Hello World!")?;

    println!("Make sure the LEDs are flashing");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

fn earlgrey_cw310_console_recv_short() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/console_recv_short/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.send_line("Short recv")?;

    // Check the message
    p.exp_string("console_recv_short: Short recv")?;

    Ok(())
}

fn earlgrey_cw310_console_timeout() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/console_timeout/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    // Wait 5 seconds
    let timeout = time::Duration::from_secs(5);
    thread::sleep(timeout);

    // Send a 60 charecter message
    p.send_line("This is a test message that we are sending. Look at us go...")?;

    // Check the message
    p.exp_string("Userspace call to read console returned: This is a test message that we are sending. Look at us go...")?;

    Ok(())
}

#[allow(dead_code)]
fn earlgrey_cw310_malloc_test1() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/malloc_test01/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.exp_string("malloc01: success")?;

    Ok(())
}

#[allow(dead_code)]
fn earlgrey_cw310_stack_size_test1() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/stack_size_test01/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.exp_string("Stack Test App")?;
    p.exp_string("Current stack pointer: 0x100")?;

    Ok(())
}

#[allow(dead_code)]
fn earlgrey_cw310_stack_size_test2() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/stack_size_test02/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.exp_string("Stack Test App")?;
    p.exp_string("Current stack pointer: 0x100")?;

    Ok(())
}

fn earlgrey_cw310_mpu_stack_growth() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/mpu_stack_growth/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.exp_string("This test should recursively add stack frames until exceeding")?;
    p.exp_string("panicked at 'Process mpu_stack_growth had a fault'")?;
    p.exp_string("Store/AMO access fault")?;

    Ok(())
}

#[allow(dead_code)]
fn earlgrey_cw310_mpu_walk_region() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/mpu_walk_region/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = earlgrey_cw310_flash(&app).unwrap();

    p.exp_string("MPU Walk Regions")?;
    p.exp_string("Walking flash")?;
    p.exp_string("Will overrun")?;
    p.exp_string("0x2003ba00")?;
    p.exp_string("panicked at 'Process mpu_walk_region had a fault'")?;

    Ok(())
}

fn earlgrey_cw310_multi_alarm_test() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/multi_alarm_test/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let _p = earlgrey_cw310_flash(&app).unwrap();

    println!("Make sure the LEDs are blinking");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

fn earlgrey_cw310_sha_hmac_test() -> Result<(), Error> {
    let app = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/tests/hmac/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let app = OpenOptions::new()
        .append(true)
        .create(false)
        .open("app")
        .unwrap();

    let mut build = Command::new("cat")
        .arg(format!(
            "{}/{}",
            env::var("LIBTOCK_C_TREE").unwrap(),
            "examples/tests/sha/build/rv32imc/rv32imc.0x20034080.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = earlgrey_cw310_flash("../../../tools/board-runner/app").unwrap();

    p.exp_string("HMAC Example Test")?;
    p.exp_string("SHA Example Test")?;

    p.exp_string("Running HMAC...")?;
    p.exp_string("0: 0xeb")?;
    p.exp_string("10: 0xde")?;

    p.exp_string("Running SHA...")?;
    p.exp_string("0: 0x68")?;
    p.exp_string("10: 0x8f")?;
    p.exp_string("31: 0x15")?;

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

pub fn all_earlgrey_cw310_tests() {
    println!("Tock board-runner starting...");
    println!();
    println!("Running earlgrey_cw310 tests...");

    earlgrey_cw310_c_hello()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_c_hello job failed with {}", e));
    earlgrey_cw310_blink().unwrap_or_else(|e| panic!("earlgrey_cw310_blink job failed with {}", e));
    earlgrey_cw310_c_hello_and_printf_long().unwrap_or_else(|e| {
        panic!(
            "earlgrey_cw310_c_hello_and_printf_long job failed with {}",
            e
        )
    });
    earlgrey_cw310_recv_short_and_recv_long().unwrap_or_else(|e| {
        panic!(
            "earlgrey_cw310_recv_short_and_recv_long job failed with {}",
            e
        )
    });
    earlgrey_cw310_blink_and_c_hello_and_buttons().unwrap_or_else(|e| {
        panic!(
            "earlgrey_cw310_blink_and_c_hello_and_buttons job failed with {}",
            e
        )
    });
    earlgrey_cw310_console_recv_short()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_console_recv_short job failed with {}", e));
    earlgrey_cw310_console_timeout()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_console_timeout job failed with {}", e));

    earlgrey_cw310_malloc_test1()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_malloc_test1 job failed with {}", e));

    earlgrey_cw310_stack_size_test1()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_stack_size_test1 job failed with {}", e));

    earlgrey_cw310_stack_size_test2()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_stack_size_test2 job failed with {}", e));

    earlgrey_cw310_mpu_stack_growth()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_mpu_stack_growth job failed with {}", e));

    // earlgrey_cw310_mpu_walk_region()
    //     .unwrap_or_else(|e| panic!("earlgrey_cw310_mpu_walk_region job failed with {}", e));

    earlgrey_cw310_multi_alarm_test()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_multi_alarm_test job failed with {}", e));

    earlgrey_cw310_sha_hmac_test()
        .unwrap_or_else(|e| panic!("earlgrey_cw310_sha_hmac_test job failed with {}", e));

    println!("earlgrey_cw310 SUCCESS.");
}
