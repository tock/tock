# Tock OS Integration Test Harness

This tool is a single, unified integration test harness that can execute a
variety of Tock OS integration tests on both emulated and physical targets.

## Usage

Prerequisites

- A libtock-c checkout
- `tockloader`
- QEMU (For QEMU-emulated boards like `qemu_rv64_virt` or `hifive1`)
- OpenOCD (for boards such as `nrf52840dk` or `nrf52dk`)

### Three-stage mode

1. Compute a plan for the selected tests:

   ```
   $ cargo run -- plan \
       --test hello_world@nrf52840dk \
       --test hello_world@qemu_rv64_virt \
       --out plan.json
   ```

   This writes `plan.json` and prints a summary of the plan:

   ```
   [INFO ] 2 tests, using 2 kernels and 2 apps
             nrf52840dk (1, up to 38s): hello_world
             qemu_rv64_virt (1, up to 33s): hello_world
   ```

   The plan lists each test with the board it runs on, and artifacts required to
   run the test (e.g. kernel images or application binaries). Artifacts are
   deduplicated if multiple tests require identical ones:

   ```json
   {
     "artifacts": {
       "libtock-c-app-c_hello-2d0f2be8": {
         "type": "libtock-c-app",
         "name": "c_hello",
         "tock_targets": ["cortex-m4"]
       },
       "tock-kernel-nrf52840dk-2b4ec908": {
         "type": "tock-kernel",
         "board": "nrf52840dk"
       },
       ...
     },
     "tests": [
       {
         "test": "hello_world",
         "board": "nrf52840dk",
         "kernel": "tock-kernel-nrf52840dk-2b4ec908",
         "apps": ["libtock-c-app-c_hello-2d0f2be8"]
       },
       ...
     ]
   }
   ```

2. Build the artifacts of the plan:

   ```
   $ cargo run -- prebuild \
       --plan plan.json \
       --tock ../../../ \
       --libtock-c ../../../../libtock-c \
       --artifacts-dir ./artifacts
   ```

   This builds kernel binaries using `make` in their board directory, and apps
   using `make` in the respective libtock-c `example/<app>` directory. Apps are
   only builts for the `TOCK_TARGETS` of the boards using them. Each artifact is
   then written to `<--artifacts-dir>/<label>.bin` following the labels of the
   plan document.

3. Run the tests of the plan:

   ```
   $ cargo run -- run \
       --plan ./plan.json \
       --artifacts-dir ./artifacts \
       --tock ../../../ \
       --dut nrf52840dk,console=/dev/ttyACM0 --out-dir out
   ```

   This runs all tests of the plan that can be run on the boards attached to
   this host, as given via `--dut <board>[,probe=<serial>][,console=<device>]`.
   The `--dut` option can be given multiple times, when multiple boards are
   attached to the current host.

   `probe` selects a board by the serial number of its debugger (e.g., J-Link).
   Virtual boards require the `--tock` argument, as QEMU is started by running
   `make` from their board directory. Tests of the plan for which no DUT is
   given are skipped. `--test` can filter the plan to only run a subset of the
   tests.

### One-shot mode

Without `--plan`, `run` computes the plan from `--test`. Without
`--artifacts-dir`, it also builds the artifacts, into `<out-dir>/artifacts`:

```
$ cargo run -- run \
    --test hello_world@qemu_rv64_virt \
    --tock ../../../ \
    --libtock-c ../../../../libtock-c \
    --out-dir out
```

```
$ cargo run -- run \
    --test hello_world@allboards \
    --tock ../../../ \
    --libtock-c ../../../../libtock-c \
    --dut nrf52840dk,console=/dev/ttyACM0 \
    --dut nrf52dk,console=/dev/ttyACM1 \
    --out-dir out
```

### Output

`run` writes to its `--out-dir`:

- `results.json`: the outcome of each test, by `<test>@<board>`. One of
  `Passed`, `Failed` or `InfraFailed` (the board or harness failed, e.g.,
  flashing exceeded its bound), or `Skipped`, each with a reason except
  `Passed`.
- `logs/<test>@<board>.log`: the console transcript of each test.
- `artifacts/`: the artifacts, if `run` built them.

`run` refuses to write to a directory which already contains `results.json` or
`logs/`. `-v` and `-vv` show more detailed logs, such as the console output
received while waiting for each step.


## Design

The tool is designed to be board- and test-agnostic. It supports multiple
different boards (located in the `src/boards/` directory) using the `Dut` Rust
trait. This trait defines how the test harness can interact with a given board,
such as flashing a combined kernel and application image, resetting it, or
interacting with its UART console or GPIO pins.

Each test is defined in `src/test_cases/`. For example, for the `c_hello` test
case, a developer might define the following `TestCase` instance.

```
#[linkme::distributed_slice(TESTS)]
static TEST: TestCase = TestCase {
    id: "hello_world",
    requires: &[Requirement::Uart],
    apps: &["c_hello"],
    steps: &[Expect("Hello World!")],
};
```

Each test case specifies a unique name, the particular features of a board it
requires (such as a connection to the board's UART console), and the `libtock-c`
apps that should run on that board. It then defines a sequence of steps that
execute in sequence, defined in `src/test_cases/steps.rs`.

The integration test harness works using three stages:
1. A *plan* stage (`plan` subcommand), which produces a plan document containing
   the permutations of boards and the tests to run on them, as well as the
   various artifacts that those tests require (e.g., kernel images, application
   binaries, etc.).

2. A *prebuild* stage (`prebuild` subcommand), which consumes a plan document
   and produces an artifacts directory, containing all built artifacts that the
   plan document mandates.

3. An *execution* stage (`run` subcommand) that executes one or more tests of a
   plan document, depending on the boards attached to the host that the tool is
   running on, using the pre-built artifacts. Alternatively, the tool supports
   an ad-hoc mode where `run` can execute a set of tests without a prior plan or
   pre-build phase useful for local testing or debugging.

This decision reflects how the tool will later integrate into the Tock OS
development flow:
1. When a pull request arrives, or has new changes, or gets enqueued for merge,
   or upon an explicit comment by a maintainer requesting that one or more tests
   be run, the plan stage computes a `(test, board)` matrix from those events.

   Currently, the plan stage generates a plan document purely from command line
   parameters. Later revisions can extend it to determine the set of `(test,
   board)` permutations from the changes made in a given pull request, or
   requested through a GitHub comment.

2. Boards available for hardware CI are a scarce resource, multiplexed across
   all CI jobs in the Tock ecosystem. A significant fraction of CI time is spent
   on building application or kernel images, or fetching dependencies for such
   steps. For this reason, the integration test harness tool separates the
   phases of building such artifacts from actual test execution, which minimizes
   the time each test needs to occupy a given board.

   In a later GitHub integration, this pre-build step should run on a hosted
   GitHub runner instead of our own infrastructure. It will then upload the
   artifacts bundle to an object storage service, from which the actual test
   runners can fetch and use it.

3. A plan may span multiple boards, or tests that require different
   "configurations" of the same board (e.g., one with GPIO connected to the test
   host and another with target USB connected). We don't necessarily feature
   test hosts that have all boards or all configurations of a board connected
   concurrently. As such, the execution stage only executes the subset of the
   plan document that can be executed on the current host.

   A later central coordination service can dispatch a plan document onto
   multiple test hosts, check their status, and progressively run all tests of
   the plan document's test matrix. This service will also be responsible for
   reacting to GitHub events, computing the test plan, kicking off the prebuild
   stage in a GitHub actions workflow, and reporting test results back to
   GitHub.
