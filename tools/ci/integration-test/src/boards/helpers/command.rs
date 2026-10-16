// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

use anyhow::{Context, bail};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// Number of trailing output lines of a failed tool to include in its error.
const FAILURE_CONTEXT_LINES: usize = 15;

/// Run an external tool (e.g., tockloader or OpenOCD) to completion.
///
/// Each line of the tool's output is logged at debug level as it arrives, along
/// with its stream. If the tool fails, the returned error contains the last
/// [`FAILURE_CONTEXT_LINES`] lines of its output.
pub fn run(cmd: &mut Command) -> anyhow::Result<()> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    run_labeled(cmd, &program)
}

/// Run an external tool like [`run`], and label its log lines with `label`
/// (e.g., `kernel:nrf52840dk`) instead of the tool's name.
pub fn run_labeled(cmd: &mut Command, label: &str) -> anyhow::Result<()> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    log::debug!("  running {cmd:?}");

    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run {program}"))?;
    let (stdout, stderr) = (child.stdout.take().unwrap(), child.stderr.take().unwrap());

    // Each thread ends when the tool closes its stream, i.e., when it exits:
    let lines = Mutex::new(Vec::new());
    let forward = |stream: &str, from: Box<dyn Read + Send>| {
        for line in BufReader::new(from).split(b'\n').map_while(Result::ok) {
            let line = String::from_utf8_lossy(&line).into_owned();
            log::debug!("  {label} {stream} | {line}");
            lines.lock().unwrap().push(line);
        }
    };
    std::thread::scope(|scope| {
        scope.spawn(|| forward("stdout", Box::new(stdout)));
        scope.spawn(|| forward("stderr", Box::new(stderr)));
    });

    let status = child
        .wait()
        .with_context(|| format!("failed to wait for {program}"))?;
    if status.success() {
        return Ok(());
    }
    let lines = lines.into_inner().unwrap();
    let tail = &lines[lines.len().saturating_sub(FAILURE_CONTEXT_LINES)..];

    bail!("{program} failed ({status}):\n{}", tail.join("\n"))
}
