# qemu-virt-ci-runner

A CI test runner for the `qemu_rv64_virt` Tock board.  It builds the board,
installs [libtock-c] apps via tockloader, runs the board inside
`qemu-system-riscv64`, and verifies the output via the QEMU Machine Protocol
(QMP) and a TCP serial connection.

## Prerequisites

- `qemu-system-riscv64` in `PATH`
- `make` in `PATH`
- [`tockloader`] installed (`pip install tockloader`)
- The [libtock-c] repository checked out as a sibling of the Tock repository
  (i.e. `../libtock-c` relative to the Tock root)
- The `qemu_rv64_virt-test-ci` board built at least once so that the kernel
  binary exists (`make` in
  `boards/configurations/qemu_rv64_virt/qemu_rv64_virt-test-ci`)

## Running all tests

```sh
cargo run
```

## Capturing a screenshot to establish a baseline hash

When adding a new test that checks the screen, you first need to capture a
reference screenshot so you can record its hash in `expected_screen_hash`.

Use the `--screenshot` subcommand:

```sh
cargo run -- --screenshot <test-name> <output-file.ppm>
```

For example:

```sh
cargo run -- --screenshot led-odd /tmp/led-odd.ppm
```

The runner will:

1. Boot the test exactly as it would during a normal run.
2. Wait for any configured serial output (so the board is in a known state).
3. Wait the test's `screenshot_delay` (so the display has time to settle).
4. Save the screenshot to `<output-file.ppm>`.
5. Print the SHA-256 hash.

Example output:

```
Screenshot saved to:  /tmp/led-odd.ppm
SHA-256:              20867e9c50573728461a70c4421b86ea08e09b4693d172c54123937f8a2a455e

To verify from the command line:
  shasum -a 256 /tmp/led-odd.ppm
```

You can verify the hash independently:

```sh
sha256sum /tmp/led-odd.ppm
```

Once you are happy with the screenshot, paste the hash into the
`expected_screen_hash` field of the test case in `src/main.rs`:

```rust
expected_screen_hash: Some("20867e9c50573728461a70c4421b86ea08e09b4693d172c54123937f8a2a455e"),
```

## Adding a new test

Add a `TestCase` entry to the `TESTS` slice in `src/main.rs`:

```rust
TestCase {
    // Short identifier used with --screenshot and in log output.
    name: "my-app",

    // libtock-c example paths relative to libtock-c/examples/.
    apps: &["my-app"],

    // Strings that must all appear in serial output before serial_timeout.
    // Set to None to skip the serial check entirely.
    expected_serial: Some(&["Expected output line"]),

    // How long to wait for the serial strings before failing.
    serial_timeout: Duration::from_secs(30),

    // How long to wait after serial output before capturing the screenshot.
    // Use a non-zero value if the display needs time to settle after the
    // serial event.
    screenshot_delay: Duration::from_millis(500),

    // SHA-256 hash of the expected screenshot (PPM file).
    // Set to None to skip the screen check (hash is still printed as a
    // baseline when running normally).  Use `--screenshot` to capture the
    // reference image and obtain the hash.
    expected_screen_hash: None,
},
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
| `-chardev socket,...,port=44445,...` | Exposes the UART as a TCP socket |
| `-serial chardev:serial0` | Routes the first serial port to that socket |
| `-S` | Starts the CPU paused; the runner sends `cont` when ready |

**Important:** QEMU will not send the QMP greeting until *all* chardev
clients are connected.  The runner therefore connects to both port 44444 and
port 44445 before attempting the QMP handshake.

[libtock-c]: https://github.com/tock/libtock-c
[tockloader]: https://github.com/tock/tockloader
[QMP]: https://wiki.qemu.org/Documentation/QMP
