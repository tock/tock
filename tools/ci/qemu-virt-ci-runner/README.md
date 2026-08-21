# qemu-virt-ci-runner

A CI test runner for the `qemu_rv64_virt` Tock board.  It installs
[libtock-c] apps via tockloader, runs the board inside
`qemu-system-riscv64`, and verifies behavior via the QEMU Machine Protocol
(QMP) and a TCP serial connection.

## Prerequisites

- `qemu-system-riscv64` in `PATH`
- `make` in `PATH`
- [`tockloader`] installed (`pip install tockloader`)
- The [libtock-c] repository.  By default the runner looks for it as a sibling
  of the Tock repository (i.e. `../libtock-c` relative to the Tock root).
  Override with `--libtock-c`:
  ```sh
  cargo run -- --libtock-c /path/to/libtock-c
  ```
- The target board built and initialized.  For `qemu_rv64_virt`:
  ```sh
  cd boards/configurations/qemu_rv64_virt/qemu_rv64_virt-test-ci
  make
  make init
  make install
  ```

## Usage

### List all available boards

```sh
cargo run -- --boards
```

### Run all tests

```sh
cargo run -- [--board <board-name>]
```

If only one board is registered, `--board` can be omitted.

### Run a single test

```sh
cargo run -- [--board <board-name>] --test <test-name>
```

### List all tests with descriptions

```sh
cargo run -- [--board <board-name>] --tests
```

### Capture a screenshot to establish a baseline hash

When adding a new test that checks the screen, capture a reference screenshot
first so you can record its hash in `expected_screen_hash`.  The runner executes
all of the test's steps before capturing, so the board is in the same state it
would be during a normal run.

```sh
cargo run -- --screenshot <test-name> <output-file.ppm>
```

Example:

```sh
cargo run -- --screenshot led-odd /tmp/led-odd.ppm
```

Output:

```
Screenshot saved to:  /tmp/led-odd.ppm
SHA-256:              20867e9c50573728461a70c4421b86ea08e09b4693d172c54123937f8a2a455e

To verify from the command line:
  shasum -a 256 /tmp/led-odd.ppm
```

Paste the hash into `expected_screen_hash` in the board's test file:

```rust
expected_screen_hash: Some("20867e9c50573728461a70c4421b86ea08e09b4693d172c54123937f8a2a455e"),
```

## Adding a new board

Create `src/boards/<board_name>.rs` defining a `pub static BOARD: super::Board`
with the board directory and a `static TESTS: &[TestCase]` slice.  Then add the
new module to `src/boards/mod.rs` (`pub mod <board_name>;`) and append
`&<board_name>::BOARD` to the `BOARDS` slice.

## Adding a new test

Add a `TestCase` entry to the `TESTS` slice in the appropriate board file
(e.g. `src/boards/qemu_rv64_virt.rs`).  Each test has a name, a
human-readable description, a list of apps to install, an ordered sequence of
steps, an optional settle delay before the screenshot, and an optional expected
screenshot hash.

```rust
TestCase {
    // Short identifier used with --test, --screenshot, and in log output.
    name: "my-app",

    // One sentence describing what the test checks.
    description: "Run my-app and verify it prints the expected output.",

    // libtock-c example paths relative to libtock-c/examples/.
    apps: &["my-app"],

    // Ordered sequence of actions to perform after QEMU starts.
    steps: &[
        TestStep::WaitSerialInOrder {
            needles: &["App is ready"],
            timeout: Duration::from_secs(30),
        },
    ],

    // How long to wait after all steps before capturing the screenshot.
    screenshot_delay: Duration::from_millis(500),

    // SHA-256 hash of the expected screenshot (PPM file).
    // Set to None to skip the screen check (hash is still printed as a
    // baseline).  Use `--screenshot` to capture the reference image.
    expected_screen_hash: None,
},
```

### Test steps

Steps are executed in order after QEMU is running.  Serial waits, key presses,
serial writes, and sleeps can be freely interleaved.

| Step | Description |
|------|-------------|
| `WaitSerialInOrder { needles, timeout }` | Wait until every string in `needles` appears in the serial output **in the given order**. |
| `WaitSerialAnyOrder { needles, timeout }` | Wait until every string in `needles` appears in the serial output **in any order**. Use this when the Tock scheduler may interleave output from multiple apps. |
| `Sleep(duration)` | Pause for `duration` without reading serial or interacting with QEMU. |
| `SendKey(qcode)` | Send a single keystroke via QMP `input-send-event` (down then up). `qcode` is a QEMU key name such as `"ret"`, `"space"`, `"a"`, or `"up"`. See the [QEMU key documentation] for the full list. |
| `SendSerial(text)` | Write raw bytes to the serial port as if typed at a terminal. Include `"\r\n"` or `"\n"` if the board expects a line ending. |

Example — wait for a prompt, send a command, then verify the response:

```rust
steps: &[
    TestStep::WaitSerialInOrder {
        needles: &["tock$ "],
        timeout: Duration::from_secs(30),
    },
    TestStep::SendSerial("help\r\n"),
    TestStep::WaitSerialInOrder {
        needles: &["Valid commands are:"],
        timeout: Duration::from_secs(10),
    },
],
```

Example — press a hardware button and verify the output appears:

```rust
steps: &[
    TestStep::Sleep(Duration::from_millis(500)),
    TestStep::SendKey("up"),
    TestStep::WaitSerialInOrder {
        needles: &["Button Press!"],
        timeout: Duration::from_secs(30),
    },
],
```

Example — two apps whose output order depends on the scheduler:

```rust
steps: &[
    TestStep::WaitSerialAnyOrder {
        needles: &["App A ready", "App B ready"],
        timeout: Duration::from_secs(30),
    },
],
```

## How QEMU is controlled

The runner passes the following extra flags to QEMU via the `QEMU_CMDLINE_EXTRA`
make variable:

```
-qmp tcp:localhost:44444,server
-chardev socket,id=serial0,host=localhost,port=44445,server=on
-serial chardev:serial0
-S
```

| Flag | Purpose |
|------|---------|
| `-qmp tcp:localhost:44444,server` | Opens a [QMP] JSON control socket |
| `-chardev socket,...,port=44445,...` | Exposes the UART as a TCP socket for both reading and writing |
| `-serial chardev:serial0` | Routes the first serial port to that socket |
| `-S` | Starts the CPU paused; the runner sends `cont` when ready |

**Important:** QEMU will not send the QMP greeting until *all* chardev
clients are connected.  The runner therefore connects to both port 44444 and
port 44445 before attempting the QMP handshake.

[libtock-c]: https://github.com/tock/libtock-c
[tockloader]: https://github.com/tock/tockloader
[QMP]: https://wiki.qemu.org/Documentation/QMP
[QEMU key documentation]: https://qemu-project.gitlab.io/qemu/system/keys.html
