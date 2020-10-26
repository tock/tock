use rexpect::errors::Error;
use rexpect::spawn_stream;
use serialport::prelude::*;
use serialport::SerialPortSettings;
use std::env;
use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::time::Duration;
use std::{thread, time};

fn opentitan_flash(
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
    let port_name = &serialport::available_ports().expect("No serial port")[0].port_name;
    println!("Connecting to OpenTitan port: {:?}", port_name);
    let port = serialport::open_with_settings(port_name, &s).expect("Failed to open serial port");

    // Clone the port
    let port_clone = port.try_clone().expect("Failed to clone");

    // Create the Rexpect instance
    let mut p = spawn_stream(port, port_clone, Some(2000));

    // Flash the Tock kernel and app
    let mut build = Command::new("make")
        .arg("-C")
        .arg("../../boards/opentitan")
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
    p.exp_string("Processing frame #155, expecting #155")?;
    p.exp_string("Processing frame #183, expecting #183")?;
    p.exp_string("Processing frame #200, expecting #200")?;

    p.exp_string("Boot ROM initialisation has completed, jump into flash")?;

    Ok(p)
}

fn opentitan_c_hello() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/c_hello/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.exp_string("Hello World!")?;

    Ok(())
}

fn opentitan_blink() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/blink/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let _p = opentitan_flash(&app).unwrap();

    println!("Make sure the LEDs are blinking");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

fn opentitan_c_hello_and_printf_long() -> Result<(), Error> {
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
            "examples/tests/printf_long/build/rv32imc/rv32imc.0x20031080.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = opentitan_flash("../../tools/board-runner/app").unwrap();

    p.exp_string("Hello World!")?;
    p.exp_string("Hi welcome to Tock. This test makes sure that a greater than 64 byte message can be printed.")?;
    p.exp_string("And a short message.")?;

    Ok(())
}

fn opentitan_recv_short_and_recv_long() -> Result<(), Error> {
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
            "examples/tests/console_recv_long/build/rv32imc/rv32imc.0x20040080.0x10008000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = opentitan_flash("../../tools/board-runner/app").unwrap();

    p.exp_string("Error doing UART receive: -2")?;

    Ok(())
}

fn opentitan_blink_and_c_hello_and_buttons() -> Result<(), Error> {
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
            "examples/c_hello/build/rv32imc/rv32imc.0x20032080.0x10008000.tbf"
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
            "examples/buttons/build/rv32imc/rv32imc.0x20033080.0x1000B000.tbf"
        ))
        .stdout(app)
        .spawn()
        .expect("failed to spawn build");
    assert!(build.wait().unwrap().success());

    let mut p = opentitan_flash("../../tools/board-runner/app").unwrap();

    p.exp_string("Hello World!")?;

    println!("Make sure the LEDs are flashing");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

fn opentitan_console_recv_short() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/console_recv_short/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.send_line("Short recv")?;

    // Check the message
    p.exp_string("console_recv_short: Short recv")?;

    Ok(())
}

fn opentitan_console_timeout() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/console_timeout/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    // Send message
    p.send_line("Test message")?;

    // Wait 25 seconds
    let timeout = time::Duration::from_secs(25);
    thread::sleep(timeout);

    // Send enter
    p.send_line("")?;

    // Check the message
    p.exp_string("Userspace call to read console returned: Test message")?;

    Ok(())
}

#[allow(dead_code)]
fn opentitan_malloc_test1() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/malloc_test01/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.exp_string("malloc01: success")?;

    Ok(())
}

#[allow(dead_code)]
fn opentitan_stack_size_test1() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/stack_size_test01/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.exp_string("Stack Test App")?;
    p.exp_string("Current stack pointer: 0x100")?;

    Ok(())
}

#[allow(dead_code)]
fn opentitan_stack_size_test2() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/stack_size_test02/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.exp_string("Stack Test App")?;
    p.exp_string("Current stack pointer: 0x100")?;

    Ok(())
}

fn opentitan_mpu_stack_growth() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/mpu_stack_growth/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.exp_string("This test should recursively add stack frames until exceeding")?;
    p.exp_string("panicked at 'Process mpu_stack_growth had a fault'")?;
    p.exp_string("Store/AMO access fault")?;

    Ok(())
}

#[allow(dead_code)]
fn opentitan_mpu_walk_region() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/mpu_walk_region/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let mut p = opentitan_flash(&app).unwrap();

    p.exp_string("MPU Walk Regions")?;
    p.exp_string("Walking flash")?;
    p.exp_string("Will overrun")?;
    p.exp_string("panicked at 'Process mpu_walk_region had a fault'")?;

    Ok(())
}

fn opentitan_multi_alarm_test() -> Result<(), Error> {
    let app = format!(
        "{}/{}",
        env::var("LIBTOCK_C_TREE").unwrap(),
        "examples/tests/multi_alarm_test/build/rv32imc/rv32imc.0x20030080.0x10005000.tbf"
    );
    let _p = opentitan_flash(&app).unwrap();

    println!("Make sure the LEDs are blinking");

    let timeout = time::Duration::from_secs(10);
    thread::sleep(timeout);

    Ok(())
}

pub fn all_opentitan_tests() {
    println!("Tock board-runner starting...");
    println!();
    println!("Running opentitan tests...");
    opentitan_c_hello().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    opentitan_blink().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    opentitan_c_hello_and_printf_long()
        .unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    opentitan_recv_short_and_recv_long()
        .unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    opentitan_blink_and_c_hello_and_buttons()
        .unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    opentitan_console_recv_short().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));
    opentitan_console_timeout().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    // Disabled by default.
    // Requires:
    //    STACK_SIZE       = 2048
    //    APP_HEAP_SIZE    = 4096
    //    KERNEL_HEAP_SIZE = 2048
    // opentitan_malloc_test1().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    // Disabled by default.
    // Requires:
    //    STACK_SIZE       = 2048
    //    APP_HEAP_SIZE    = 4096
    //    KERNEL_HEAP_SIZE = 2048
    // opentitan_stack_size_test1().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    // Disabled by default.
    // Requires:
    //    STACK_SIZE       = 2048
    //    APP_HEAP_SIZE    = 4096
    //    KERNEL_HEAP_SIZE = 2048
    // opentitan_stack_size_test2().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    opentitan_mpu_stack_growth().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    // Disabled by default.
    // Requires:
    //    STACK_SIZE       = 2048
    //    APP_HEAP_SIZE    = 4096
    //    KERNEL_HEAP_SIZE = 2048
    // opentitan_mpu_walk_region().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    opentitan_multi_alarm_test().unwrap_or_else(|e| panic!("opentitan job failed with {}", e));

    println!("opentitan SUCCESS.");
}
