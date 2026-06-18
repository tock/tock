// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use serde_json::Value;
use sha2::{Digest, Sha256};

const BOARD_DIR: &str = "../../../boards/configurations/qemu_rv64_virt/qemu_rv64_virt-test-ci";
const LIBTOCK_C_EXAMPLES: &str = "../../../../libtock-c/examples";

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
fn run_with_apps<F>(app_names: &[&str], test_fn: F) -> Result<(), String>
where
    F: FnOnce(&mut QmpConnection, &mut BufReader<TcpStream>) -> Result<(), String>,
{
    println!("Installing apps: {:?}", app_names);

    // Install each libtock-c example app with tockloader.
    for app in app_names {
        let app_path = PathBuf::from(LIBTOCK_C_EXAMPLES).join(app);
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

    let result = test_fn(&mut qmp, &mut serial);

    qemu.kill();
    result
}

/// Read serial output until all strings in `needles` have appeared (in any
/// order) or `timeout` elapses, whichever comes first.
///
/// Each needle is checked off as soon as it is seen; the function returns
/// `Ok(())` only once every needle has been matched.  On timeout the error
/// message lists whichever needles were still outstanding.
fn expect_serial(
    serial: &mut BufReader<TcpStream>,
    needles: &[&str],
    timeout: Duration,
) -> Result<(), String> {
    println!(
        "Waiting for serial output: {:?} (timeout: {:?})",
        needles, timeout
    );

    // Track which needles are still outstanding.
    let mut remaining: Vec<&str> = needles.to_vec();
    let deadline = Instant::now() + timeout;
    let mut buf = String::new();

    // Apply the caller-supplied timeout to the underlying socket so that
    // read_line() unblocks when the deadline is reached.
    serial
        .get_ref()
        .set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;

    loop {
        // Check wall-clock deadline before every read attempt.
        if Instant::now() >= deadline {
            return Err(format!(
                "timeout after {:?} waiting for serial output; still expecting: {:?}",
                timeout, remaining
            ));
        }

        let mut line = String::new();
        match serial.read_line(&mut line) {
            Ok(0) => {
                return Err(format!(
                    "serial connection closed; still expecting: {:?}",
                    remaining
                ))
            }
            Ok(_) => {
                print!("[serial] {}", line);
                buf.push_str(&line);
                // Remove every needle that now appears anywhere in the
                // accumulated output.
                remaining.retain(|needle| !buf.contains(needle));
                if remaining.is_empty() {
                    return Ok(());
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                return Err(format!(
                    "timeout after {:?} waiting for serial output; still expecting: {:?}",
                    timeout, remaining
                ));
            }
            Err(e) => return Err(format!("serial read error: {}", e)),
        }
    }
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

struct TestCase {
    name: &'static str,
    apps: &'static [&'static str],
    /// Every string in this list must appear in the serial output (in any
    /// order) before `serial_timeout` elapses.  `None` skips the serial check.
    expected_serial: Option<&'static [&'static str]>,
    /// How long to wait for all expected serial strings before failing.
    serial_timeout: Duration,
    /// How long to wait after serial output (or after boot if no serial check)
    /// before capturing the screenshot.  Gives the display time to settle.
    screenshot_delay: Duration,
    /// Optional known-good screendump hash. `None` means skip the check.
    expected_screen_hash: Option<&'static str>,
}

const TESTS: &[TestCase] = &[
    TestCase {
        name: "c_hello",
        apps: &["c_hello"],
        expected_serial: Some(&["Hello World!"]),
        serial_timeout: Duration::from_secs(30),
        screenshot_delay: Duration::from_millis(0),
        // Set to Some("known_hash_here") once a baseline is established.
        expected_screen_hash: None,
    },
    TestCase {
        name: "led-odd",
        apps: &["tests/led/led-odd"],
        expected_serial: Some(&["Entering main loop."]),
        serial_timeout: Duration::from_secs(30),
        screenshot_delay: Duration::from_millis(500),
        expected_screen_hash: Some(
            "20867e9c50573728461a70c4421b86ea08e09b4693d172c54123937f8a2a455e",
        ),
    },
];

// ---------------------------------------------------------------------------

fn run_test(tc: &TestCase) -> Result<(), String> {
    println!();
    println!("{}", "=".repeat(70));
    println!("TEST: {}", tc.name);
    println!("{}", "=".repeat(70));

    let expected_serial = tc.expected_serial;
    let serial_timeout = tc.serial_timeout;
    let screenshot_delay = tc.screenshot_delay;
    let expected_hash = tc.expected_screen_hash;

    run_with_apps(tc.apps, |qmp, serial| {
        // Check serial output if expected strings were specified.
        if let Some(needles) = expected_serial {
            expect_serial(serial, needles, serial_timeout)?;
            println!("Serial check passed: {:?}", needles);
        } else {
            println!("Serial check skipped (no expected output configured).");
        }

        // Wait for the display to settle before capturing.
        if !screenshot_delay.is_zero() {
            println!("Waiting {:?} before screenshot...", screenshot_delay);
            std::thread::sleep(screenshot_delay);
        }

        // Optionally verify the screen.
        if let Some(known_hash) = expected_hash {
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
fn cmd_screenshot(test_name: &str, dest: &std::path::Path) -> Result<(), String> {
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

    let serial_timeout = tc.serial_timeout;
    let expected_serial = tc.expected_serial;
    let screenshot_delay = tc.screenshot_delay;

    run_with_apps(tc.apps, |qmp, serial| {
        // Wait for serial output first so the board has reached a stable
        // state before we capture the screen.
        if let Some(needles) = expected_serial {
            expect_serial(serial, needles, serial_timeout)?;
            println!("Serial output matched; board is ready.");
        } else {
            println!("No serial expectation configured; capturing screen immediately.");
        }

        // Wait for the display to settle before capturing.
        if !screenshot_delay.is_zero() {
            println!("Waiting {:?} before screenshot...", screenshot_delay);
            std::thread::sleep(screenshot_delay);
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

fn cmd_run_all() -> Result<(), String> {
    println!("qemu-virt CI runner starting...");

    for tc in TESTS {
        run_test(tc)?;
        println!("TEST PASSED: {}", tc.name);
        println!("{}", "-".repeat(70));
    }

    Ok(())
}

fn usage(argv0: &str) {
    eprintln!("Usage:");
    eprintln!("  {}                               Run all tests", argv0);
    eprintln!(
        "  {} --screenshot <test> <file>     Boot <test>, save screenshot to <file>, print hash",
        argv0
    );
    eprintln!();
    eprintln!("Available tests:");
    for tc in TESTS {
        eprintln!("  {}", tc.name);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let argv0 = args[0].as_str();

    let result = match args.as_slice() {
        // Normal mode: run all tests.
        [_] => cmd_run_all(),

        // Screenshot mode: boot one test and save a screendump.
        [_, flag, test_name, dest] if flag == "--screenshot" => {
            cmd_screenshot(test_name, std::path::Path::new(dest))
        }

        _ => {
            usage(argv0);
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
