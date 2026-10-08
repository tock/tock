// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde_json::Value;
use sha2::{Digest, Sha256};

mod boards;

// Default libtock-c location: a sibling of the Tock repository root.
// Override with --libtock-c <dir>.
const LIBTOCK_C_DIR_DEFAULT: &str = "../../../../libtock-c";

const QMP_PORT: u16 = 44444;
const SERIAL_PORT: u16 = 44445;
// Second serial port, only wired up for tests whose `TestCase::needs_serial1`
// is set (see `qemu_cmdline_extra`).
const SERIAL1_PORT: u16 = 44446;

/// Extra QEMU flags for CI: expose a QMP control socket, the first serial
/// port over TCP, and (when `needs_serial1` is set) a second serial port
/// over TCP too, then start paused.  `-serial` arguments are assigned to the
/// machine's UARTs in command-line order, so the second `-chardev`/`-serial`
/// pair here always lands on the board's second UART.
fn qemu_cmdline_extra(needs_serial1: bool) -> String {
    let mut cmdline = format!(
        "-qmp tcp:localhost:{qmp},server \
         -chardev socket,id=serial0,host=localhost,port={serial0},server=on \
         -serial chardev:serial0 ",
        qmp = QMP_PORT,
        serial0 = SERIAL_PORT,
    );
    if needs_serial1 {
        cmdline.push_str(&format!(
            "-chardev socket,id=serial1,host=localhost,port={serial1},server=on \
             -serial chardev:serial1 ",
            serial1 = SERIAL1_PORT,
        ));
    }
    cmdline.push_str("-S");
    cmdline
}

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

/// Reads lines from `stream` on a background thread for the life of the
/// test, printing each one as soon as it arrives and forwarding it over the
/// returned channel.
///
/// This exists so the primary serial port's output (including any kernel
/// debug prints) is visible in real time even while another step -- e.g.
/// `SendFileYmodem` on the secondary port -- blocks the main thread for a
/// while: without a dedicated reader thread, nothing would drain the
/// primary port during that time, so its output would just sit unread
/// (and unprinted) until the next step happened to read it.
fn spawn_serial_reader(stream: TcpStream) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF: QEMU exited.
                Ok(_) => {
                    print!("[serial] {}", line);
                    if tx.send(line).is_err() {
                        // Nothing left to receive it (test already ended).
                        break;
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    continue;
                }
                Err(_) => break,
            }
        }
    });
    rx
}

/// The serial connection(s) to the board.  Every board has a primary serial
/// port; `secondary` is only populated when the test's `needs_serial1` is
/// set, for boards that wire up a second UART (e.g. for YMODEM app loading).
struct SerialPorts {
    /// Lines read from the primary port, continuously drained and printed
    /// by a background thread (see `spawn_serial_reader`) so output is
    /// visible live rather than only after some other step finishes.
    primary: mpsc::Receiver<String>,
    primary_write: TcpStream,
    secondary: Option<(BufReader<TcpStream>, TcpStream)>,
}

/// Install tockloader apps, start QEMU in the background, and run the test closure.
fn run_with_apps<F>(
    apps: &[App],
    libtock_c_dir: &Path,
    board_dir: &str,
    needs_serial1: bool,
    test_fn: F,
) -> Result<(), String>
where
    F: FnOnce(&mut QmpConnection, &mut SerialPorts) -> Result<(), String>,
{
    println!("Uninstalling all apps (if any)");

    let status = Command::new("tockloader")
        .args(["erase-apps"])
        .status()
        .map_err(|e| format!("tockloader erase-apps failed: {}", e))?;
    if !status.success() {
        return Err(format!("tockloader erase-apps failed"));
    }

    let app_labels: Vec<&str> = apps
        .iter()
        .map(|a| match a {
            App::LibtockC(p) => *p,
        })
        .collect();
    println!("Installing apps: {:?}", app_labels);

    for app in apps {
        match app {
            App::LibtockC(path) => {
                let app_path = libtock_c_dir.join("examples").join(path);
                let status = Command::new("tockloader")
                    .current_dir(&app_path)
                    .args(["install"])
                    .status()
                    .map_err(|e| format!("tockloader install failed for {}: {}", path, e))?;
                if !status.success() {
                    return Err(format!("tockloader install failed for {}", path));
                }
            }
        }
    }

    // Spawn `make run` with the extra QEMU flags that expose control sockets.
    println!("Starting QEMU via `make run`...");
    let child = Command::new("make")
        .current_dir(board_dir)
        .arg("run")
        .env("QEMU_CMDLINE_EXTRA", qemu_cmdline_extra(needs_serial1))
        .process_group(0)
        .spawn()
        .map_err(|e| format!("failed to spawn make run: {}", e))?;
    let qemu = QemuInstance { child };

    // Connect the raw TCP streams to every socket before doing any protocol
    // work.  QEMU will not send the QMP greeting until every chardev socket
    // (i.e. every serial port socket) also has a client connected, so we
    // must establish all connections first.
    println!("Waiting for QMP socket on port {}...", QMP_PORT);
    let mut qmp = QmpConnection::connect().map_err(|e| format!("QMP connect failed: {}", e))?;

    println!("Waiting for serial socket on port {}...", SERIAL_PORT);
    let serial_stream =
        wait_for_tcp(SERIAL_PORT).map_err(|e| format!("serial connect failed: {}", e))?;
    println!("Connected to serial socket on port {}...", SERIAL_PORT);
    let serial_write = serial_stream
        .try_clone()
        .map_err(|e| format!("failed to clone serial stream: {}", e))?;
    let primary = spawn_serial_reader(serial_stream);

    let secondary = if needs_serial1 {
        println!("Waiting for serial socket on port {}...", SERIAL1_PORT);
        let serial1_stream =
            wait_for_tcp(SERIAL1_PORT).map_err(|e| format!("serial1 connect failed: {}", e))?;
        println!("Connected to serial socket on port {}...", SERIAL1_PORT);
        let serial1_write = serial1_stream
            .try_clone()
            .map_err(|e| format!("failed to clone serial1 stream: {}", e))?;
        Some((BufReader::new(serial1_stream), serial1_write))
    } else {
        None
    };

    // Now every TCP connection exists; negotiate the QMP protocol.
    qmp.handshake()
        .map_err(|e| format!("QMP handshake failed: {}", e))?;
    println!("QMP handshake complete.");

    // Resume the CPU (QEMU starts paused with -S).
    qmp.resume()
        .map_err(|e| format!("QMP resume failed: {}", e))?;
    println!("QEMU resumed.");

    let mut serial = SerialPorts {
        primary,
        primary_write: serial_write,
        secondary,
    };

    let result = test_fn(&mut qmp, &mut serial);

    // Always stop QEMU after the test, whether it passed or failed.
    qemu.kill();

    result
}

/// Read serial output until `done` returns `true` or `timeout` elapses.
///
/// `serial` is fed by the background thread started in
/// `spawn_serial_reader`, which has already printed each line as it
/// arrived -- this just accumulates them to match `done` against.
fn read_serial_until<F>(
    serial: &mpsc::Receiver<String>,
    timeout: Duration,
    mut done: F,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<bool, String>,
{
    let deadline = Instant::now() + timeout;
    let mut buf = String::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match serial.recv_timeout(remaining) {
            Ok(line) => {
                buf.push_str(&line);
                if done(&buf)? {
                    return Ok(());
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(format!("timeout after {:?}", timeout));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(format!(
                    "serial reader thread exited (QEMU exited?) after {:?}",
                    timeout
                ));
            }
        }
    }
}

/// Wait until every needle has appeared in the serial output, in any order.
fn expect_serial_any_order(
    serial: &mpsc::Receiver<String>,
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
/// Each needle must appear strictly after the end of the previous needle's
/// match, so a single occurrence in the output cannot satisfy two needles.
fn expect_serial_in_order(
    serial: &mpsc::Receiver<String>,
    needles: &[&str],
    timeout: Duration,
) -> Result<(), String> {
    println!(
        "Waiting for serial output (in order): {:?} (timeout: {:?})",
        needles, timeout
    );
    let mut idx = 0;
    // Byte offset into the accumulated buffer; needle[idx] is only searched
    // in buf[search_from..] so each match must follow the previous one.
    let mut search_from = 0usize;
    read_serial_until(serial, timeout, |buf| {
        while idx < needles.len() {
            match buf[search_from..].find(needles[idx]) {
                Some(pos) => {
                    search_from += pos + needles[idx].len();
                    idx += 1;
                }
                None => break,
            }
        }
        Ok(idx == needles.len())
    })
    .map_err(|e| format!("{}: still waiting for: {:?}", e, &needles[idx..]))
}

/// Adapts a buffered serial reader so it also implements `AsFd`, as required
/// by `rzsz`'s poll-based `ModemReader`.  `read()` delegates straight to the
/// `BufReader`, which drains any already-buffered bytes before touching the
/// socket, so this is only safe to use on a port nothing else has read from
/// yet (true for `SerialPort::Secondary`, which is dedicated to YMODEM) --
/// otherwise bytes already sitting in the `BufReader`'s buffer from an
/// earlier step wouldn't make the underlying socket's `poll()` ready, and a
/// `ModemReader::read_byte` could spuriously time out despite there being
/// buffered data available.
struct SerialReader<'a> {
    inner: &'a mut BufReader<TcpStream>,
}

impl Read for SerialReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl AsFd for SerialReader<'_> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.get_ref().as_fd()
    }
}

/// Send `path` to the device over the serial connection using YMODEM.
///
/// The device is expected to be running a YMODEM receiver on its console
/// (this mirrors what the host-side `lsb -vv <file>` command does when
/// pointed at a real serial port).
fn send_file_ymodem(
    serial: &mut BufReader<TcpStream>,
    serial_write: &mut TcpStream,
    path: &Path,
) -> Result<(), String> {
    // Matches the buffer size `rzsz`'s own rz/sz/zz binaries use.
    println!("creating reader");
    let mut reader = rzsz::serial::reader::ModemReader::new(SerialReader { inner: serial }, 16384);

    println!("got reader");
    rzsz::ymodem::ymodem_send(&mut reader, serial_write, &[path])
        .map(|_bytes_sent| {
            println!("sent {}", _bytes_sent);
        })
        .map_err(|e| format!("YMODEM transfer of {} failed: {}", path.display(), e))
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
// Shared test types (referenced by board modules via `crate::TestCase`)
// ---------------------------------------------------------------------------

/// Which serial port a step should act on, for boards that expose more than
/// one (see `TestCase::needs_serial1`).  Steps that don't take a `port`
/// field always use the primary port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SerialPort {
    #[allow(unused)]
    Primary,
    Secondary,
}

/// One action in an ordered test sequence.  Steps are executed in order after
/// QEMU is running; serial waits and key presses may be freely interleaved.
pub(crate) enum TestStep {
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
    /// Send a file to the device over a serial port using the YMODEM
    /// protocol, as e.g. the host-side `lsb`/`sb --ymodem` tools do.  `path`
    /// is resolved relative to the runner's working directory.  `port` must
    /// be `SerialPort::Secondary` (with `TestCase::needs_serial1` set) --
    /// the primary port is read as text lines so its output can be streamed
    /// live, which doesn't work for YMODEM's raw binary framing.  The
    /// board's console on that port is expected to be running a YMODEM
    /// receiver.
    SendFileYmodem {
        port: SerialPort,
        path: &'static str,
    },
}

/// An app to install before running a test.
#[derive(Clone, Copy)]
pub(crate) enum App {
    /// A libtock-c app identified by its path relative to `libtock-c/examples/`.
    LibtockC(&'static str),
}

pub(crate) struct TestCase {
    pub name: &'static str,
    pub description: &'static str,
    pub apps: &'static [App],
    /// Ordered sequence of actions to perform after QEMU is running.
    pub steps: &'static [TestStep],
    /// How long to wait after all steps before capturing the screenshot.
    pub screenshot_delay: Duration,
    /// Optional known-good screendump hash. `None` means skip the check.
    pub expected_screen_hash: Option<&'static str>,
    /// Whether this test needs a second serial port, exposed as a TCP
    /// socket on `SERIAL1_PORT` in addition to the primary port on
    /// `SERIAL_PORT`.  Set this when a step uses `SerialPort::Secondary`
    /// (e.g. a board that wires a second UART to a YMODEM receiver).
    pub needs_serial1: bool,
}

// ---------------------------------------------------------------------------

/// Execute all steps for a test.
fn run_steps(
    steps: &[TestStep],
    qmp: &mut QmpConnection,
    serial: &mut SerialPorts,
) -> Result<(), String> {
    for step in steps {
        match step {
            TestStep::WaitSerialAnyOrder { needles, timeout } => {
                expect_serial_any_order(&serial.primary, needles, *timeout)?;
                println!("Serial check passed: {:?}", needles);
            }
            TestStep::WaitSerialInOrder { needles, timeout } => {
                expect_serial_in_order(&serial.primary, needles, *timeout)?;
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
                serial
                    .primary_write
                    .write_all(text.as_bytes())
                    .and_then(|_| serial.primary_write.flush())
                    .map_err(|e| format!("serial write failed: {}", e))?;
            }
            TestStep::SendFileYmodem { port, path } => {
                println!("Sending file via YMODEM ({:?}): {}", port, path);
                match port {
                    // The primary port is read as decoded text lines (see
                    // `spawn_serial_reader`) so its output can be streamed
                    // live; that's incompatible with YMODEM's raw binary
                    // framing, which is why boards that need YMODEM wire up
                    // a dedicated secondary port for it instead.
                    SerialPort::Primary => Err(format!(
                        "SendFileYmodem does not support SerialPort::Primary: \
                         that port is read as text lines, not a raw byte \
                         stream; use SerialPort::Secondary (needs_serial1) \
                         instead (path: {:?})",
                        path
                    )),
                    SerialPort::Secondary => {
                        let (reader, writer) = serial.secondary.as_mut().ok_or_else(|| {
                            format!(
                                "step requests the secondary serial port to send {:?}, but \
                                 this test's TestCase does not set needs_serial1",
                                path
                            )
                        })?;
                        send_file_ymodem(reader, writer, Path::new(path))
                    }
                }?;
                println!("YMODEM transfer complete: {}", path);
            }
        }
    }
    Ok(())
}

fn run_test(tc: &TestCase, libtock_c_dir: &Path, board_dir: &str) -> Result<(), String> {
    println!();
    println!("{}", "=".repeat(70));
    println!("TEST: {}", tc.name);
    println!("{}", "=".repeat(70));

    run_with_apps(
        tc.apps,
        libtock_c_dir,
        board_dir,
        tc.needs_serial1,
        |qmp, serial| {
            run_steps(tc.steps, qmp, serial)?;

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
        },
    )
}

/// Boot a single named test, wait until the board is in a known running state
/// (by satisfying its serial expectations if any), take a screendump, copy it
/// to `dest`, and print the SHA-256 hash.  This is intended for establishing
/// or inspecting the baseline hash that goes into `expected_screen_hash`.
fn cmd_screenshot(
    board: &boards::Board,
    test_name: &str,
    dest: &Path,
    libtock_c_dir: &Path,
) -> Result<(), String> {
    let tc = board
        .tests
        .iter()
        .find(|t| t.name == test_name)
        .ok_or_else(|| {
            format!(
                "unknown test {:?}; available tests: {}",
                test_name,
                board
                    .tests
                    .iter()
                    .map(|t| t.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    println!(
        "Screenshot mode: test={:?} dest={}",
        tc.name,
        dest.display()
    );

    build_apps(tc.apps, libtock_c_dir, board.tock_targets)?;

    run_with_apps(
        tc.apps,
        libtock_c_dir,
        board.board_dir,
        tc.needs_serial1,
        |qmp, serial| {
            // Run all steps (serial waits, key presses, sleeps) so the board
            // reaches a known state before the screenshot is captured.
            run_steps(tc.steps, qmp, serial)?;

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
        },
    )
}

fn build_apps(apps: &[App], libtock_c_dir: &Path, tock_targets: &str) -> Result<(), String> {
    let mut libtock_c_paths: Vec<&str> = apps
        .iter()
        .filter_map(|a| match a {
            App::LibtockC(path) => Some(*path),
        })
        .collect();
    libtock_c_paths.sort();
    libtock_c_paths.dedup();
    for path in libtock_c_paths {
        let app_path = libtock_c_dir.join("examples").join(path);
        println!("Building libtock-c app: {}", path);
        let status = Command::new("make")
            .current_dir(&app_path)
            .env("TOCK_TARGETS", tock_targets)
            .status()
            .map_err(|e| format!("`make` failed for {}: {}", path, e))?;
        if !status.success() {
            return Err(format!("`make` failed for {}", path));
        }
    }
    Ok(())
}

fn cmd_run_all(board: &boards::Board, libtock_c_dir: &Path) -> Result<(), String> {
    println!("qemu-virt CI runner starting (board: {})...", board.name);

    println!("Running `make init` in {}...", board.board_dir);
    let status = Command::new("make")
        .current_dir(board.board_dir)
        .arg("init")
        .status()
        .map_err(|e| format!("failed to run `make init` in {}: {}", board.board_dir, e))?;
    if !status.success() {
        return Err(format!("`make init` failed in {}", board.board_dir));
    }

    let all_apps: Vec<App> = board
        .tests
        .iter()
        .flat_map(|tc| tc.apps.iter().copied())
        .collect();
    build_apps(&all_apps, libtock_c_dir, board.tock_targets)?;

    for tc in board.tests {
        run_test(tc, libtock_c_dir, board.board_dir)?;
        println!("TEST PASSED: {}", tc.name);
        println!("{}", "-".repeat(70));
    }

    Ok(())
}

fn cmd_run_one(board: &boards::Board, test_name: &str, libtock_c_dir: &Path) -> Result<(), String> {
    println!(
        "qemu-virt CI runner single test (board: {}, test: {})...",
        board.name, test_name
    );

    println!("Running `make init` in {}...", board.board_dir);
    let status = Command::new("make")
        .current_dir(board.board_dir)
        .arg("init")
        .status()
        .map_err(|e| format!("failed to run `make init` in {}: {}", board.board_dir, e))?;
    if !status.success() {
        return Err(format!("`make init` failed in {}", board.board_dir));
    }

    let tc = board
        .tests
        .iter()
        .find(|t| t.name == test_name)
        .ok_or_else(|| {
            format!(
                "unknown test {:?}; available tests: {}",
                test_name,
                board
                    .tests
                    .iter()
                    .map(|t| t.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    build_apps(tc.apps, libtock_c_dir, board.tock_targets)?;
    let r = run_test(tc, libtock_c_dir, board.board_dir);
    if r.is_ok() {
        println!("TEST PASSED: {}", tc.name);
    }
    r
}

fn cmd_list_tests(board: &boards::Board) {
    println!("Board: {}", board.name);
    println!();
    for tc in board.tests {
        println!("{}", tc.name);
        println!("  {}", tc.description);
    }
}

fn usage(argv0: &str) {
    eprintln!("Usage:");
    eprintln!(
        "  {} [--board <board>] [--libtock-c <dir>]                            Run all tests",
        argv0
    );
    eprintln!(
        "  {} [--board <board>] [--libtock-c <dir>] --test <test>              Run a single named test",
        argv0
    );
    eprintln!(
        "  {} [--board <board>] --tests                                         List all tests with descriptions",
        argv0
    );
    eprintln!(
        "  {} [--board <board>] [--libtock-c <dir>] --screenshot <test> <file> Boot <test>, save screenshot, print hash",
        argv0
    );
    eprintln!(
        "  {} --boards                                                           List all available boards",
        argv0
    );
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --board <name>      Select the target board.");
    eprintln!(
        "                      Default: {} (only board available)",
        boards::BOARDS[0].name
    );
    eprintln!("  --libtock-c <dir>   Path to the libtock-c repository root.");
    eprintln!("                      Default: {}", LIBTOCK_C_DIR_DEFAULT);
    eprintln!();
    eprintln!("Available boards:");
    for b in boards::BOARDS {
        eprintln!("  {}", b.name);
    }
}

fn select_board(name: &str) -> Option<&'static boards::Board> {
    boards::BOARDS.iter().copied().find(|b| b.name == name)
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let argv0 = args[0].clone();

    // --boards: list available boards and exit.
    if args.iter().any(|a| a == "--boards") {
        for b in boards::BOARDS {
            println!("{}", b.name);
        }
        std::process::exit(0);
    }

    // Extract --board <name> from args before dispatching.
    let board: &boards::Board = if let Some(pos) = args.iter().position(|a| a == "--board") {
        if pos + 1 >= args.len() {
            eprintln!("error: --board requires a board name argument");
            std::process::exit(2);
        }
        let name = args.remove(pos + 1);
        args.remove(pos);
        match select_board(&name) {
            Some(b) => b,
            None => {
                eprintln!(
                    "error: unknown board {:?}; available boards: {}",
                    name,
                    boards::BOARDS
                        .iter()
                        .map(|b| b.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                std::process::exit(2);
            }
        }
    } else if boards::BOARDS.len() == 1 {
        boards::BOARDS[0]
    } else {
        eprintln!(
            "error: multiple boards available; specify one with --board <name>: {}",
            boards::BOARDS
                .iter()
                .map(|b| b.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        std::process::exit(2);
    };

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
        [_] => cmd_run_all(board, &libtock_c_dir),

        // List tests with descriptions (no libtock-c needed).
        [_, flag] if flag == "--tests" => {
            cmd_list_tests(board);
            std::process::exit(0);
        }

        // Single-test mode.
        [_, flag, test_name] if flag == "--test" => cmd_run_one(board, test_name, &libtock_c_dir),

        // Screenshot mode: boot one test and save a screendump.
        [_, flag, test_name, dest] if flag == "--screenshot" => {
            cmd_screenshot(board, test_name, Path::new(dest), &libtock_c_dir)
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
