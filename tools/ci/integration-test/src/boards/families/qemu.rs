// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Common support code for boards emulated by QEMU.
//!
//! Each board is started through the `make run` target of its Tock board
//! directory, with `FLASH_FILE` pointing to the harness's flash image (or with
//! `KERNEL` and `LOCAL_BOARD_BIN` pointing to the kernel and an image of the
//! apps, see [`Boot`]). This allows us to avoid re-defining the QEMU command
//! line in this harness.
//!
//! The harness writes the kernel and apps to its own flash file using
//! tockloader. It communicates with QEMU over two Unix sockets which it adds to
//! the command line, for QEMU's QMP monitor and the board's console.

use crate::boards::helpers::console_buffer::ConsoleBuffer;
use crate::boards::helpers::tockloader;
use crate::boards::{Board, Dut, Timing, Uart};
use anyhow::{Context, Result, anyhow, bail};
use fs_err as fs;
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Default timing bounds for QEMU boards.
pub const TIMING: Timing = Timing {
    flash: Duration::from_secs(20),
    boot: Duration::from_secs(10),
    expect: Duration::from_secs(3),
};

/// Flash locations at which tockloader writes a board's kernel and apps.
pub struct Layout {
    /// Name of the board in tockloader.
    pub board: &'static str,
    /// Flash address of the kernel.
    pub kernel: u64,
    /// Flash address of the first app.
    pub apps: u64,
    /// How the board's `make run` boots the kernel and apps.
    pub boot: Boot,
}

/// How the `make run` target of a board boots the kernel and apps.
pub enum Boot {
    /// `make run FLASH_FILE=<image>` boots an image of the board's flash, which
    /// holds the kernel and apps.
    FlashFile,
    /// `make run KERNEL=<kernel> LOCAL_BOARD_BIN=<apps>` loads the kernel (as
    /// built) and an image of the board's apps region separately.
    ///
    /// QEMU cannot load raw images larger than the RAM of some machines (e.g.,
    /// `opentitan`), which an image of the whole flash would exceed.
    KernelAndApps {
        /// Address at which tockloader's flash file for the board starts.
        flash: u64,
    },
}

/// Return the apps region of the tockloader flash file `flash_file`, which
/// starts at `offset` in the file.
///
/// The region ends with the last app, or is a single erased page if there are
/// no apps (tockloader then does not create the file).
fn apps_region(flash_file: &Path, offset: u64) -> Result<Vec<u8>> {
    let mut region = match fs::read(flash_file) {
        Ok(image) => image.get(offset as usize..).unwrap_or_default().to_vec(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e.into()),
    };
    if region.is_empty() {
        region = vec![0xff; ERASED_PAGE];
    }
    Ok(region)
}

/// Size of the erased region written in place of an empty apps region.
const ERASED_PAGE: usize = 512;

/// Maximum time QEMU may take to answer a QMP command.
const QMP_TIMEOUT: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub struct Qemu {
    board: &'static Board,
    /// Tock repository, which holds the board's directory.
    tock: PathBuf,
    layout: Layout,
    /// Temporary directory holding the flash file and QEMU's sockets, which is
    /// created when the board is flashed.
    dir: Option<tempfile::TempDir>,
    /// The `make` process, which leads the process group that QEMU runs in.
    make: Option<Child>,
    qmp: Option<BufReader<UnixStream>>,
    console: Option<UnixStream>,
    output: ConsoleBuffer,
}

impl Qemu {
    pub fn new(board: &'static Board, tock: &Path, layout: Layout) -> Self {
        Qemu {
            board,
            tock: tock.to_path_buf(),
            layout,
            dir: None,
            make: None,
            qmp: None,
            console: None,
            output: ConsoleBuffer::new(),
        }
    }

    /// Connect to the Unix sockets that QEMU listens on at `paths`. Connecting
    /// is retried until `deadline`.
    ///
    /// QEMU only creates each socket once the previous one is connected.
    fn connect(&mut self, paths: [&Path; 2], deadline: Instant) -> Result<[UnixStream; 2]> {
        let mut streams = [None, None];
        loop {
            for (stream, path) in streams.iter_mut().zip(paths) {
                if stream.is_none() {
                    *stream = UnixStream::connect(path).ok();
                }
            }
            if streams.iter().all(Option::is_some) {
                return Ok(streams.map(Option::unwrap));
            }
            if let Some(Ok(Some(status))) = self.make.as_mut().map(Child::try_wait) {
                bail!("make run exited ({status})");
            }
            if Instant::now() > deadline {
                bail!("QEMU did not listen on its sockets in time");
            }
            std::thread::sleep(POLL_INTERVAL);
        }
    }

    /// Execute a QMP command and wait for its response. Events which QEMU
    /// reports in the meantime are skipped.
    fn qmp(&mut self, command: &str, arguments: Value) -> Result<()> {
        let qmp = self.qmp.as_mut().context("QEMU is not running")?;
        let request = json!({ "execute": command, "arguments": arguments });
        let context = || format!("QMP {command} failed");
        writeln!(qmp.get_mut(), "{request}").with_context(context)?;
        loop {
            let mut line = String::new();
            if qmp.read_line(&mut line).with_context(context)? == 0 {
                return Err(anyhow!("QEMU closed the connection").context(context()));
            }
            let response: Value = serde_json::from_str(&line).with_context(context)?;
            if let Some(error) = response.get("error") {
                bail!("QMP {command} failed: {error}");
            }
            if response.get("return").is_some() {
                return Ok(());
            }
        }
    }
}

impl Dut for Qemu {
    fn board(&self) -> &'static Board {
        self.board
    }

    fn flash(&mut self, kernel: &Path, apps: &[&Path]) -> Result<()> {
        let dir = self.dir.insert(tempfile::tempdir()?).path().to_path_buf();
        let flash_file = dir.join("flash.bin");
        let layout = &self.layout;
        let images = match layout.boot {
            Boot::FlashFile => {
                tockloader::write_flash_file(
                    &flash_file,
                    layout.board,
                    layout.kernel,
                    Some(layout.apps),
                    kernel,
                    apps,
                )?;
                vec![format!("FLASH_FILE={}", flash_file.display())]
            }
            Boot::KernelAndApps { flash } => {
                tockloader::install_apps(&flash_file, layout.board, Some(layout.apps), apps)?;
                let apps_file = dir.join("apps.bin");
                fs::write(&apps_file, apps_region(&flash_file, layout.apps - flash)?)?;
                // `make` runs in the board's directory:
                let kernel = std::path::absolute(kernel)?;
                vec![
                    format!("KERNEL={}", kernel.display()),
                    format!("LOCAL_BOARD_BIN={}", apps_file.display()),
                ]
            }
        };

        // QEMU only starts once both sockets are connected. `-S` then keeps the
        // CPU paused until the `cont` command below:
        let qmp = dir.join("qmp");
        let console = dir.join("console");
        let extra = format!(
            "-qmp unix:{},server=on -chardev socket,id=console,path={},server=on \
             -serial chardev:console -S",
            qmp.display(),
            console.display()
        );
        let make = Command::new("make")
            .arg("-C")
            .arg(self.tock.join(self.board.dir))
            .arg("run")
            .args(images)
            .arg(format!("QEMU_CMDLINE_EXTRA={extra}"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .process_group(0)
            .spawn()
            .context("failed to run make")?;
        self.make = Some(make);

        let deadline = Instant::now() + self.board.timing.flash;
        let [qmp, console] = self.connect([&qmp, &console], deadline)?;
        console.set_nonblocking(true)?;
        self.console = Some(console);
        qmp.set_read_timeout(Some(QMP_TIMEOUT))?;
        let mut qmp = BufReader::new(qmp);
        let mut greeting = String::new();
        qmp.read_line(&mut greeting).context("no QMP greeting")?;
        self.qmp = Some(qmp);
        self.qmp("qmp_capabilities", json!({}))?;
        self.qmp("cont", json!({}))
    }

    fn reset(&mut self) -> Result<()> {
        self.qmp = None;
        self.console = None;
        if let Some(mut make) = self.make.take() {
            let _ = killpg(Pid::from_raw(make.id() as i32), Signal::SIGTERM);
            let _ = make.wait();
        }
        Ok(())
    }

    fn uart(&mut self) -> Option<&mut dyn Uart> {
        Some(self)
    }
}

impl Drop for Qemu {
    fn drop(&mut self) {
        let _ = self.reset();
    }
}

impl Uart for Qemu {
    fn receive_until(&mut self, needle: &str, timeout: Duration) -> Result<String> {
        let deadline = Instant::now() + timeout;
        let mut chunk = [0u8; 256];
        while Instant::now() < deadline {
            let console = self.console.as_mut().context("QEMU is not running")?;
            match console.read(&mut chunk) {
                Ok(0) => bail!("QEMU closed the console"),
                Ok(n) => self.output.push(&chunk[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(e) => return Err(e.into()),
            }
            if let Some(out) = self.output.take_through(needle) {
                return Ok(out);
            }
        }
        Err(self.output.timeout_error(needle))
    }

    fn transcript(&mut self) -> &[u8] {
        self.output.transcript()
    }
}
