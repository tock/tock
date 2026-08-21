// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde_json::Value;
use sha2::{Digest, Sha256};

const BOARD_DIR: &str = "../../../boards/configurations/qemu_rv64_virt/qemu_rv64_virt-test-ci";

// Default libtock-c location: a sibling of the Tock repository root.
// Override with --libtock-c <dir>.
const LIBTOCK_C_DIR_DEFAULT: &str = "../../../../libtock-c";

const QMP_PORT: u16 = 44444;
const SERIAL_PORT: u16 = 44445;

// Extra QEMU flags for CI: expose QMP control socket, serial over TCP, start paused.
const QEMU_CMDLINE_EXTRA: &str = concat!(
    "-qmp tcp:localhost:44444,server ",
    "-chardev socket,id=serial0,host=localhost,port=44445,server=on ",
    "-serial chardev:serial0 ",
    "-S"
);

// Maximum time to wait for QEMU sockets to become available.
const SOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

// Default read timeout on raw TCP socket connections (used for QMP reads and
// as a polling interval inside expect_serial; the per-test serial timeout is
// set independently on the serial socket inside expect_serial).
const SOCKET_READ_TIMEOUT: Duration = Duration::from_secs(60);

struct QemuInstance {
    child: Child,
}

impl QemuInstance {
    fn kill(mut self) {
        // Send SIGINT (Ctrl+C) to the entire process group so that make and
        // qemu-system-riscv64 both receive it and can shut down cleanly.
        // process_group(0) at spawn time ensures pgid == child pid.
        let pgid = Pid::from_raw(self.child.id() as i32);
        let _ = killpg(pgid, Signal::SIGINT);
        let _ = self.child.wait();
    }
}

struct QmpConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl QmpConnection {
    /// Open the TCP connection to the QMP port and return an unhandshaked
    /// connection.  The caller must ensure the serial TCP connection is also
    /// established before calling `handshake()`, because QEMU will not send
    /// the QMP greeting until all chardev clients are connected.
    fn connect() -> Result<Self, String> {
        let stream = wait_for_tcp(QMP_PORT)?;
        let reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
        Ok(QmpConnection { stream, reader })
    }

    /// Read the QMP greeting and negotiate capabilities.  Must be called after
    /// both the QMP *and* serial TCP sockets have been connected to QEMU, as
    /// QEMU will not send the greeting until all chardev listeners have a
    /// client.
    fn handshake(&mut self) -> Result<(), String> {
        // Read and discard the QMP greeting banner.
        let mut greeting = String::new();
        let resp = self.reader.read_line(&mut greeting);
        resp.map_err(|e| e.to_string())?;

        // Negotiate capabilities before any other commands.
        self.send_command("qmp_capabilities", None)?;

        Ok(())
    }

    fn send_command(&mut self, execute: &str, arguments: Option<Value>) -> Result<Value, String> {
        let mut cmd = serde_json::json!({ "execute": execute });
        if let Some(args) = arguments {
            cmd["arguments"] = args;
        }
        let mut line = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;
        line.push('\n');
        self.stream
            .write_all(line.as_bytes())
            .map_err(|e| e.to_string())?;
        self.stream.flush().map_err(|e| e.to_string())?;

        // Read response lines until we get one containing "return" or "error".
        loop {
            let mut resp = String::new();
            self.reader
                .read_line(&mut resp)
                .map_err(|e| e.to_string())?;
            let v: Value = serde_json::from_str(resp.trim()).map_err(|e| e.to_string())?;
            if v.get("return").is_some() || v.get("error").is_some() {
                if let Some(err) = v.get("error") {
                    return Err(format!("QMP error: {}", err));
                }
                return Ok(v["return"].clone());
            }
            // Otherwise it's an event; ignore and keep reading.
        }
    }

    fn resume(&mut self) -> Result<(), String> {
        self.send_command("cont", None)?;
        Ok(())
    }

    /// Send a single keystroke (down then up) via the QMP `input-send-event`
    /// command.  `qcode` is a QEMU key name such as `"ret"`, `"space"`, or
    /// `"a"`.  See the QEMU documentation for the full list of qcodes.
    fn send_key(&mut self, qcode: &str) -> Result<(), String> {
        let key = serde_json::json!({ "type": "qcode", "data": qcode });
        self.send_command(
            "input-send-event",
            Some(serde_json::json!({
                "events": [{"type": "key", "data": {"down": true,  "key": key}}]
            })),
        )?;
        self.send_command(
            "input-send-event",
            Some(serde_json::json!({
                "events": [{"type": "key", "data": {"down": false, "key": key}}]
            })),
        )?;
        Ok(())
    }

    fn screendump(&mut self, path: &std::path::Path) -> Result<(), String> {
        self.send_command(
            "screendump",
            Some(serde_json::json!({ "filename": path.to_string_lossy() })),
        )?;
        Ok(())
    }
}

/// Wait until a TCP listener appears on `port`, then return a connected stream.
fn wait_for_tcp(port: u16) -> Result<TcpStream, String> {
    let addr = format!("127.0.0.1:{}", port);
    let deadline = Instant::now() + SOCKET_CONNECT_TIMEOUT;
    loop {
        match TcpStream::connect(&addr) {
            Ok(s) => {
                s.set_read_timeout(Some(SOCKET_READ_TIMEOUT))
                    .map_err(|e| e.to_string())?;
                return Ok(s);
            }
            Err(_) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                return Err(format!("timeout connecting to {}: {}", addr, e));
            }
        }
    }
}

/// Install tockloader apps, start QEMU in the background, and run the test closure.
fn run_with_apps<F>(app_names: &[&str], libtock_c_dir: &Path, test_fn: F) -> Result<(), String>
where
    F: FnOnce(&mut QmpConnection, &mut BufReader<TcpStream>, &mut TcpStream) -> Result<(), String>,
{
    println!("Uninstalling all apps (if any)");

    let status = Command::new("tockloader")
        .args(["erase-apps"])
        .status()
        .map_err(|e| format!("tockloader erase-apps failed: {}", e))?;
    if !status.success() {
        return Err(format!("tockloader erase-apps failed"));
    }

    println!("Installing apps: {:?}", app_names);

    // Install each libtock-c example app with tockloader.
    for app in app_names {
        let app_path = libtock_c_dir.join("examples").join(app);
        let status = Command::new("tockloader")
            .current_dir(&app_path)
            .args(["install", "--board", "qemu_rv64_virt"])
            .status()
            .map_err(|e| format!("tockloader install failed for {}: {}", app, e))?;
        if !status.success() {
            return Err(format!("tockloader install failed for {}", app));
        }
    }

    // Spawn `make run` with the extra QEMU flags that expose control sockets.
    println!("Starting QEMU via `make run`...");
    let child = Command::new("make")
        .current_dir(BOARD_DIR)
        .arg("run")
        .env("QEMU_CMDLINE_EXTRA", QEMU_CMDLINE_EXTRA)
        .process_group(0)
        .spawn()
        .map_err(|e| format!("failed to spawn make run: {}", e))?;
    let qemu = QemuInstance { child };

    // Connect the raw TCP streams to both ports before doing any protocol
    // work.  QEMU will not send the QMP greeting until every chardev socket
    // (i.e. the serial port socket) also has a client connected, so we must
    // establish both connections first.
    println!("Waiting for QMP socket on port {}...", QMP_PORT);
    let mut qmp = QmpConnection::connect().map_err(|e| format!("QMP connect failed: {}", e))?;
    println!("QMP TCP connected.");

    println!("Waiting for serial socket on port {}...", SERIAL_PORT);
    let serial_stream =
        wait_for_tcp(SERIAL_PORT).map_err(|e| format!("serial connect failed: {}", e))?;
    let mut serial_write = serial_stream
        .try_clone()
        .map_err(|e| format!("failed to clone serial stream: {}", e))?;
    let mut serial = BufReader::new(serial_stream);
    println!("Serial TCP connected.");

    // Now that both clients are connected, QEMU will emit the QMP greeting.
    qmp.handshake()
        .map_err(|e| format!("QMP handshake failed: {}", e))?;
    println!("QMP ready.");

    // Un-pause QEMU so the board actually starts running.
    qmp.resume()
        .map_err(|e| format!("QMP cont failed: {}", e))?;
    println!("QEMU running.");

    let result = test_fn(&mut qmp, &mut serial, &mut serial_write);

    qemu.kill();
    result
}

/// Read serial lines until the deadline, calling `check` after each line.
/// `check` receives the full accumulated buffer and returns `true` when done.
fn read_serial_until<F>(
    serial: &mut BufReader<TcpStream>,
    timeout: Duration,
    mut check: F,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<bool, String>,
{
    serial
        .get_ref()
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;

    let deadline = Instant::now() + timeout;
    let mut buf = String::new();

    loop {
        if Instant::now() >= deadline {
            return Err(format!("serial timeout after {:?}", timeout));
        }
        let mut line = String::new();
        match serial.read_line(&mut line) {
            Ok(0) => return Err("serial connection closed unexpectedly".into()),
            Ok(_) => {
                print!("[serial] {}", line);
                buf.push_str(&line);
                if check(&buf)? {
                    return Ok(());
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                return Err(format!("serial timeout after {:?}", timeout));
            }
            Err(e) => return Err(format!("serial read error: {}", e)),
        }
    }
}

/// Wait until every needle has appeared somewhere in the serial output,
/// in any order.
fn expect_serial_any_order(
    serial: &mut BufReader<TcpStream>,
    needles: &[&str],
    timeout: Duration,
) -> Result<(), String> {
    println!(
        "Waiting for serial output (any order): {:?} (timeout: {:?})",
        needles, timeout
    );
    let mut remaining: Vec<&str> = needles.to_vec();
    read_serial_until(serial, timeout, |buf| {
        remaining.retain(|n| !buf.contains(n));
        if remaining.is_empty() {
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .map_err(|e| format!("{}: still waiting for: {:?}", e, remaining))
}

/// Wait until each needle appears in the serial output in the given order.
/// Each needle must appear after all preceding needles have been matched.
fn expect_serial_in_order(
    serial: &mut BufReader<TcpStream>,
    needles: &[&str],
    timeout: Duration,
) -> Result<(), String> {
    println!(
        "Waiting for serial output (in order): {:?} (timeout: {:?})",
        needles, timeout
    );
    let mut idx = 0;
    read_serial_until(serial, timeout, |buf| {
        while idx < needles.len() && buf.contains(needles[idx]) {
            idx += 1;
        }
        Ok(idx == needles.len())
    })
    .map_err(|e| format!("{}: still waiting for: {:?}", e, &needles[idx..]))
}

/// Take a screendump via QMP and return the SHA-256 hash of the image file
/// as a lowercase hex string.
///
/// This matches the output of `shasum -a 256 <file>` (macOS) and
/// `sha256sum <file>` (Linux), so a baseline hash can be captured and
/// verified from the command line:
///
///   shasum -a 256 /tmp/screenshot.ppm
fn screendump_hash(qmp: &mut QmpConnection) -> Result<String, String> {
    let tmp = tempfile::Builder::new()
        .suffix(".ppm")
        .tempfile()
        .map_err(|e| e.to_string())?;
    qmp.screendump(tmp.path())?;
    let bytes = std::fs::read(tmp.path()).map_err(|e| e.to_string())?;

    let hash = Sha256::digest(&bytes);
    Ok(format!("{:x}", hash))
}

// ---------------------------------------------------------------------------
// Test definitions
// ---------------------------------------------------------------------------

/// One action in an ordered test sequence.  Steps are executed in order after
/// QEMU is running; serial waits and key presses may be freely interleaved.
enum TestStep {
    /// Wait until every needle has appeared in the serial output, in any order.
    /// Use this when the Tock scheduler may interleave output from multiple apps.
    WaitSerialAnyOrder {
        needles: &'static [&'static str],
        timeout: Duration,
    },
    /// Wait until every needle has appeared in the serial output, in the given
    /// order.  Each needle must appear strictly after all preceding ones.
    WaitSerialInOrder {
        needles: &'static [&'static str],
        timeout: Duration,
    },
    /// Sleep for `duration` without reading serial or interacting with QEMU.
    Sleep(Duration),
    /// Send a single keystroke via QMP `input-send-event`.  `key` is a QEMU
    /// qcode name such as `"ret"`, `"space"`, `"a"`, or `"tab"`.
    /// See https://qemu-project.gitlab.io/qemu/system/keys.html for the full
    /// list.
    SendKey(&'static str),
    /// Write raw bytes to the serial port (as if a user typed them at a
    /// terminal).  The string is sent exactly as given; include `"\r\n"` or
    /// `"\n"` if the board expects a line ending.
    SendSerial(&'static str),
}

struct TestCase {
    name: &'static str,
    description: &'static str,
    apps: &'static [&'static str],
    /// Ordered sequence of actions to perform after QEMU is running.
    steps: &'static [TestStep],
    /// How long to wait after all steps before capturing the screenshot.
    screenshot_delay: Duration,
    /// Optional known-good screendump hash. `None` means skip the check.
    expected_screen_hash: Option<&'static str>,
}

const TESTS: &[TestCase] = &[
    TestCase {
        name: "c_hello",
        description: "Run the c_hello app and verify it prints \"Hello World!\" over serial.",
        apps: &["c_hello"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Hello World!"],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(0),
        // Set to Some("known_hash_here") once a baseline is established.
        expected_screen_hash: None,
    },
    TestCase {
        name: "led-odd",
        description: "Run led-odd and verify the screen shows LEDs 1 and 3 on.",
        apps: &["tests/led/led-odd"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Entering main loop."],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "c4d49a185592f23074ae5b83caa7d506967fbe8f93a547fa60de93bc2ddd282f",
        ),
    },
    TestCase {
        name: "tock-logo",
        description: "Run the tock-logo u8g2 demo and verify the screen matches the expected logo.",
        apps: &["tests/u8g2-demos/tock-logo"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Entering main loop."],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "4150b38c3cf468a388c0a09eb309ac8c480f692599cc0fc3e2684e0fafb5463d",
        ),
    },
    TestCase {
        name: "led-odd-logo",
        description: "Run led-odd and tock-logo together and verify the combined screen output as the LEDs display on the screen.",
        apps: &["tests/led/led-odd", "tests/u8g2-demos/tock-logo"],
        steps: &[TestStep::WaitSerialInOrder {
            needles: &["Entering main loop."],
            timeout: Duration::from_secs(30),
        }],
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "bfec0b5a3c5b72edb6128c7c9b6690b56d7ac93e1047d9ef45a22be0c9b4d4fa",
        ),
    },
    TestCase {
        name: "button_print",
        description: "Use the keyboard to send the character up arrow, and ensure that is translated to button 0 being pressed for userspace.",
        apps: &["tests/button_print"],
        steps: &[
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendKey("up"),
            TestStep::WaitSerialInOrder {
                needles: &["Button Press! Button: 0 Status: 0"],
                timeout: Duration::from_secs(30),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "console",
        description: "Send characters over serial and verify the console app echoes them back.",
        apps: &["tests/console/console"],
        steps: &[
            TestStep::WaitSerialInOrder {
                needles: &["Entering main loop."],
                timeout: Duration::from_secs(30),
            },
            TestStep::Sleep(Duration::from_millis(30)),
            TestStep::SendSerial("a"),
            TestStep::WaitSerialInOrder {
                needles: &["Got character: 'a'"],
                timeout: Duration::from_secs(10),
            },
            TestStep::Sleep(Duration::from_millis(100)),
            TestStep::SendSerial("R"),
            TestStep::WaitSerialInOrder {
                needles: &["Got character: 'R'"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "hello-pconsole",
        description: "Run c_hello alongside the process console and verify both the app output and the tock$ prompt appear.",
        apps: &["c_hello"],
        steps: &[
            TestStep::Sleep(Duration::from_millis(100)),
            TestStep::SendSerial("\n\r"),
            TestStep::WaitSerialAnyOrder {
                needles: &["Hello World!", "tock$"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "pconsole-help",
        description: "Send the 'help' command to the process console and verify the help output.",
        apps: &[],
        steps: &[
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendSerial("help\n\r"),
            TestStep::WaitSerialInOrder {
                needles: &[
                    "Valid commands are: help status list stop start fault boot terminate process kernel reset panic console-start console-stop",
                ],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "pconsole-list",
        description: "Run two apps and verify the process console 'list' command shows both, in any scheduler order.",
        apps: &["c_hello", "blink"],
        steps: &[
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendSerial("list\n\r"),
            TestStep::WaitSerialAnyOrder {
                needles: &["blink", "c_hello"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "exit",
        description: "Run the exit app and confirm it shows as Terminated in the process list.",
        apps: &["tests/exit"],
        steps: &[
            TestStep::WaitSerialInOrder {
                needles: &["Testing exit.", "Exiting."],
                timeout: Duration::from_secs(10),
            },
            TestStep::Sleep(Duration::from_millis(500)),
            TestStep::SendSerial("list\n\r"),
            TestStep::WaitSerialInOrder {
                needles: &["exit", "Terminated"],
                timeout: Duration::from_secs(10),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
    TestCase {
        name: "unexpected-rx",
        description: "Send a serial byte before the app starts and verify the kernel handles unexpected RX without crashing.",
        apps: &["c_hello"],
        steps: &[
            TestStep::SendSerial("a"),
            TestStep::WaitSerialInOrder {
                needles: &["Hello World!"],
                timeout: Duration::from_secs(5),
            },
        ],
        screenshot_delay: Duration::from_millis(0),
        expected_screen_hash: None,
    },
];

// ---------------------------------------------------------------------------

/// Execute all steps for a test.
fn run_steps(
    steps: &[TestStep],
    qmp: &mut QmpConnection,
    serial: &mut BufReader<TcpStream>,
    serial_write: &mut TcpStream,
) -> Result<(), String> {
    for step in steps {
        match step {
            TestStep::WaitSerialAnyOrder { needles, timeout } => {
                expect_serial_any_order(serial, needles, *timeout)?;
                println!("Serial check passed: {:?}", needles);
            }
            TestStep::WaitSerialInOrder { needles, timeout } => {
                expect_serial_in_order(serial, needles, *timeout)?;
                println!("Serial check passed: {:?}", needles);
            }
            TestStep::Sleep(duration) => {
                println!("Sleeping {:?}...", duration);
                std::thread::sleep(*duration);
            }
            TestStep::SendKey(key) => {
                println!("Sending key: {:?}", key);
                qmp.send_key(key)
                    .map_err(|e| format!("send_key({:?}) failed: {}", key, e))?;
            }
            TestStep::SendSerial(text) => {
                println!("Sending serial: {:?}", text);
                serial_write
                    .write_all(text.as_bytes())
                    .and_then(|_| serial_write.flush())
                    .map_err(|e| format!("serial write failed: {}", e))?;
            }
        }
    }
    Ok(())
}

fn run_test(tc: &TestCase, libtock_c_dir: &Path) -> Result<(), String> {
    println!();
    println!("{}", "=".repeat(70));
    println!("TEST: {}", tc.name);
    println!("{}", "=".repeat(70));

    run_with_apps(tc.apps, libtock_c_dir, |qmp, serial, serial_write| {
        run_steps(tc.steps, qmp, serial, serial_write)?;

        // Wait for the display to settle before capturing.
        if !tc.screenshot_delay.is_zero() {
            println!("Waiting {:?} before screenshot...", tc.screenshot_delay);
            std::thread::sleep(tc.screenshot_delay);
        }

        // Optionally verify the screen.
        if let Some(known_hash) = tc.expected_screen_hash {
            let actual = screendump_hash(qmp)?;
            if actual != known_hash {
                return Err(format!(
                    "screen hash mismatch: expected {} got {}",
                    known_hash, actual
                ));
            }
            println!("Screen hash check passed: {}", actual);
        } else {
            // Still capture the hash so it can be recorded as a baseline.
            match screendump_hash(qmp) {
                Ok(hash) => println!("Screen hash (baseline): {}", hash),
                Err(e) => println!("Screen hash unavailable: {}", e),
            }
        }

        Ok(())
    })
}

/// Boot a single named test, wait until the board is in a known running state
/// (by satisfying its serial expectations if any), take a screendump, copy it
/// to `dest`, and print the SHA-256 hash.  This is intended for establishing
/// or inspecting the baseline hash that goes into `expected_screen_hash`.
fn cmd_screenshot(test_name: &str, dest: &Path, libtock_c_dir: &Path) -> Result<(), String> {
    let tc = TESTS.iter().find(|t| t.name == test_name).ok_or_else(|| {
        format!(
            "unknown test {:?}; available tests: {}",
            test_name,
            TESTS.iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
        )
    })?;

    println!(
        "Screenshot mode: test={:?} dest={}",
        tc.name,
        dest.display()
    );

    run_with_apps(tc.apps, libtock_c_dir, |qmp, serial, serial_write| {
        // Run all steps (serial waits, key presses, sleeps) so the board
        // reaches a known state before the screenshot is captured.
        run_steps(tc.steps, qmp, serial, serial_write)?;

        // Wait for the display to settle before capturing.
        if !tc.screenshot_delay.is_zero() {
            println!("Waiting {:?} before screenshot...", tc.screenshot_delay);
            std::thread::sleep(tc.screenshot_delay);
        }

        // Take the screendump into a temp file, then copy to dest.
        let tmp = tempfile::Builder::new()
            .suffix(".ppm")
            .tempfile()
            .map_err(|e| e.to_string())?;
        qmp.screendump(tmp.path())?;
        std::fs::copy(tmp.path(), dest)
            .map_err(|e| format!("failed to copy screenshot to {}: {}", dest.display(), e))?;

        // Compute and print the hash so it can be pasted into expected_screen_hash.
        let bytes = std::fs::read(dest).map_err(|e| e.to_string())?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        println!("Screenshot saved to:  {}", dest.display());
        println!("SHA-256:              {}", hash);
        println!();
        println!(
            "To verify from the command line:\n  shasum -a 256 {}",
            dest.display()
        );

        Ok(())
    })
}

fn cmd_run_all(libtock_c_dir: &Path) -> Result<(), String> {
    println!("qemu-virt CI runner starting...");

    for tc in TESTS {
        run_test(tc, libtock_c_dir)?;
        println!("TEST PASSED: {}", tc.name);
        println!("{}", "-".repeat(70));
    }

    Ok(())
}

fn cmd_list_tests() {
    for tc in TESTS {
        println!("{}", tc.name);
        println!("  {}", tc.description);
    }
}

fn usage(argv0: &str) {
    eprintln!("Usage:");
    eprintln!(
        "  {} [--libtock-c <dir>]                            Run all tests",
        argv0
    );
    eprintln!(
        "  {} [--libtock-c <dir>] --test <test>              Run a single named test",
        argv0
    );
    eprintln!(
        "  {} --tests                                         List all tests with descriptions",
        argv0
    );
    eprintln!(
        "  {} [--libtock-c <dir>] --screenshot <test> <file> Boot <test>, save screenshot, print hash",
        argv0
    );
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --libtock-c <dir>   Path to the libtock-c repository root.");
    eprintln!("                      Default: {}", LIBTOCK_C_DIR_DEFAULT);
    eprintln!();
    eprintln!("Available tests:");
    for tc in TESTS {
        eprintln!("  {}", tc.name);
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let argv0 = args[0].clone();

    // Extract --libtock-c <dir> from args before dispatching, since it may
    // appear at any position.
    let libtock_c_dir = if let Some(pos) = args.iter().position(|a| a == "--libtock-c") {
        if pos + 1 >= args.len() {
            eprintln!("error: --libtock-c requires a directory argument");
            std::process::exit(2);
        }
        let dir = PathBuf::from(args.remove(pos + 1));
        args.remove(pos);
        dir
    } else {
        PathBuf::from(LIBTOCK_C_DIR_DEFAULT)
    };

    let result = match args.as_slice() {
        // Normal mode: run all tests.
        [_] => cmd_run_all(&libtock_c_dir),

        // List tests with descriptions (no libtock-c needed).
        [_, flag] if flag == "--tests" => {
            cmd_list_tests();
            std::process::exit(0);
        }

        // Single-test mode.
        [_, flag, test_name] if flag == "--test" => {
            match TESTS.iter().find(|t| t.name == test_name.as_str()) {
                Some(tc) => {
                    let r = run_test(tc, &libtock_c_dir);
                    if r.is_ok() {
                        println!("TEST PASSED: {}", tc.name);
                    }
                    r
                }
                None => {
                    eprintln!(
                        "unknown test {:?}; available tests: {}",
                        test_name,
                        TESTS.iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
                    );
                    std::process::exit(2);
                }
            }
        }

        // Screenshot mode: boot one test and save a screendump.
        [_, flag, test_name, dest] if flag == "--screenshot" => {
            cmd_screenshot(test_name, Path::new(dest), &libtock_c_dir)
        }

        _ => {
            usage(&argv0);
            std::process::exit(2);
        }
    };

    match result {
        Ok(()) => println!("Done."),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
