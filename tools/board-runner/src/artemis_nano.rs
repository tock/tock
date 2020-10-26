use rexpect::errors::Error;
use rexpect::spawn_stream;
use serialport::prelude::*;
use serialport::SerialPortSettings;
use std::env;
use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{thread, time};

fn artemis_nano_flash(
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

    // Flash the app
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/redboard_artemis_nano")
        .arg(format!("APP={}", app_name))
        .arg("flash-app")
        .stdout(Stdio::null())
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    // Open the first serialport available.
    let port_name = &serialport::available_ports().expect("No serial port")[0].port_name;
    println!("Connecting to redboard_artemis_nano port: {:?}", port_name);
    let port = serialport::open_with_settings(port_name, &s).expect("Failed to open serial port");

    // Clone the port
    let port_clone = port.try_clone().expect("Failed to clone");

    // Create the Rexpect instance
    let mut p = spawn_stream(port, port_clone, Some(2000));

    // Make sure the image is flashed
    p.exp_string("Apollo3 chip revision: B")?;

    Ok(p)
}

fn artemis_nano_c_hello() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/c_hello/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("Hello World!")?;

    Ok(())
}

fn artemis_nano_blink() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/blink/build/cortex-m4/cortex-m4.tbf"
    );
    let _p = artemis_nano_flash(&app).unwrap();

    println!("Make sure the LEDs are blinking");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_sensors() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/sensors/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("Hello World!")?;

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_c_hello_and_printf_long() -> Result<(), Error> {
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
            "examples/c_hello/build/cortex-m4/cortex-m4.tbf"
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
            "examples/tests/printf_long/build/cortex-m4/cortex-m4.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = artemis_nano_flash("../../tools/board-runner/app").unwrap();

    p.exp_string("Hello World!")?;
    p.exp_string("Hi welcome to Tock. This test makes sure that a greater than 64 byte message can be printed.")?;
    p.exp_string("And a short message.")?;

    Ok(())
}

fn artemis_nano_blink_and_c_hello_and_buttons() -> Result<(), Error> {
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
            "examples/blink/build/cortex-m4/cortex-m4.tbf"
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
            "examples/c_hello/build/cortex-m4/cortex-m4.tbf"
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
            "examples/buttons/build/cortex-m4/cortex-m4.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = artemis_nano_flash("../../tools/board-runner/app").unwrap();

    p.exp_string("Hello World!")?;

    println!("Make sure the LEDs are flashing");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

fn artemis_nano_malloc_test1() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/malloc_test01/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("malloc01: success")?;

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_stack_size_test1() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/stack_size_test01/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("Stack Test App")?;
    p.exp_string("Current stack pointer: 0x100")?;

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_stack_size_test2() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/stack_size_test02/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("Stack Test App")?;
    p.exp_string("Current stack pointer: 0x100")?;

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_mpu_stack_growth() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/mpu_stack_growth/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("This test should recursively add stack frames until exceeding")?;
    p.exp_string("panicked at 'Process mpu_stack_growth had a fault'")?;
    p.exp_string("Store/AMO access fault")?;

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_mpu_walk_region() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/mpu_walk_region/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_string("MPU Walk Regions")?;
    p.exp_string("Walking flash")?;
    p.exp_string("Will overrun")?;
    p.exp_string("panicked at 'Process mpu_walk_region had a fault'")?;

    Ok(())
}

fn artemis_nano_multi_alarm_test() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/multi_alarm_test/build/cortex-m4/cortex-m4.tbf"
    );
    let _p = artemis_nano_flash(&app).unwrap();

    println!("Make sure the LEDs are blinking");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

#[allow(dead_code)]
fn artemis_nano_lua() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/lua-hello/build/cortex-m4/cortex-m4.tbf"
    );
    let _p = artemis_nano_flash(&app).unwrap();

    Ok(())
}

fn artemis_nano_whileone() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/whileone/build/cortex-m4/cortex-m4.tbf"
    );
    let mut p = artemis_nano_flash(&app).unwrap();

    p.exp_eof()?;

    Ok(())
}

pub fn all_artemis_nano_tests() {
    println!("Tock board-runner starting...");
    println!();
    println!("Running artemis_nano tests...");
    artemis_nano_c_hello().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    artemis_nano_blink().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    // Doesn't work
    // artemis_nano_sensors().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    // Doesn't work
    // artemis_nano_c_hello_and_printf_long().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    artemis_nano_blink_and_c_hello_and_buttons()
        .unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));

    artemis_nano_malloc_test1().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    // Doesn't work
    // artemis_nano_stack_size_test1().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    // Doesn't work
    // artemis_nano_stack_size_test2().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    // Doesn't work
    // artemis_nano_mpu_stack_growth().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    artemis_nano_mpu_walk_region().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    artemis_nano_multi_alarm_test()
        .unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    // Doesn't work
    // artemis_nano_lua().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));
    artemis_nano_whileone().unwrap_or_else(|e| panic!("artemis_nano job failed with {}", e));

    println!("artemis_nano SUCCESS.");
}
