# Hardware Continuous Integration with Treadmill

This guide provides a walkthrough on how to perform hardware continuous integration (CI) using Treadmill. By leveraging the [`tock-hardware-ci`](https://github.com/tock/tock-hardware-ci) repository, you can automatically test your code on real hardware within your CI pipelines. This guide covers:

- An overview of the `tock-hardware-ci` repository.
- How to integrate hardware tests into your GitHub Actions workflows using Treadmill.
- Steps to create and customize your own hardware CI setup.

## Prerequisites

Before proceeding, ensure you are familiar with the following concepts:

- [Treadmill Terminology](https://book.treadmill.ci/introduction/terminology.html), especially terms like [Host](https://book.treadmill.ci/introduction/terminology.html#host), [Supervisor](https://book.treadmill.ci/introduction/terminology.html#supervisor), and [Device Under Test (DUT)](../introduction/terminology.html#dut).

- The [Integrating Treadmill with GitHub Actions](https://book.treadmill.ci/user-guide/github-actions-integration.html) guide, which explains how to set up Treadmill jobs and GitHub Actions runners.

## Overview of `tock-hardware-ci`

The `tock-hardware-ci` repository is designed to facilitate hardware testing within the Tock ecosystem using Treadmill. It provides:

- **Board Harnesses**: Abstractions for different hardware boards to standardize interactions like flashing and serial communication.
- **Test Harnesses**: Reusable test scripts that can be applied across various boards.
- **Utilities**: Tools for serial communication, GPIO control, and other hardware interactions.

By using `tock-hardware-ci`, you can automate testing on real hardware, ensuring that changes to your codebase are validated against actual devices.

## Repository Structure

The repository is organized as follows:

- **`hwci/`**: Contains the core Python modules for hardware CI.
  - **`boards/`**: Definitions and implementations of different board harnesses.
  - **`tests/`**: Test scripts that can be executed on the boards.
  - **`utils/`**: Utility modules for serial communication and test helpers.
  - **`core/`**: Core classes like `BoardHarness` and `TestHarness`.
- **`README.md`**: Provides an overview of the repository.
- **`requirements.txt`**: Python dependencies required for the CI scripts.
- **`select_tests.py`**: Script to select tests based on code changes (placeholder for future enhancements).

## Setting Up Hardware CI with Treadmill

### 1. Fork or Clone the `tock-hardware-ci` Repository

Begin by cloning the repository to include it in your project:

```bash
git clone https://github.com/tock/tock-hardware-ci.git
```

### 2. Configure Your Treadmill Environment

Ensure you have access to a Treadmill deployment. You may refer to the [treadmill.ci Deployment](https://book.treadmill.ci/treadmillci-deployment.html) page for details on available deployments and hardware resources.

### 3. Update Treadmill Images

Make sure the Treadmill image you plan to use includes all necessary dependencies and configurations to run your tests and the self-hosted runner. You can find available images and their details on the [treadmill.ci Public Images](https://book.treadmill.ci/treadmillci-deployment/images.html) page.

### 4. Set Up Repository Secrets and Variables

In your GitHub repository:

- **Secrets**:
  - `TREADMILL_API_TOKEN`: API token for authenticating with Treadmill.
- **Variables**:
  - Any variables required for your tests or environment configurations.

Ensure these are securely stored in your repository's settings under **Settings** → **Secrets and variables** → **Actions**.

### 5. Define GitHub Actions Workflows

Create a workflow file (e.g., `.github/workflows/hardware-ci.yml`) that integrates with Treadmill and runs your hardware tests.

#### Example Workflow Structure

```yaml
name: Hardware CI

on:
  pull_request:
  push:

jobs:
  test-prepare:
    runs-on: ubuntu-latest
    outputs:
      runner-id: ${{ steps.treadmill-job-launch.outputs.runner-id }}
      tml-job-id: ${{ steps.treadmill-job-launch.outputs.tml-job-id }}
    steps:
      # Checkout the repositories, set up the environment, and compile necessary tools
      # Enqueue the Treadmill job and obtain the runner ID
      # Refer to the 'Integrating Treadmill with GitHub Actions' guide for detailed steps

  test-execute:
    needs: test-prepare
    runs-on: ${{ needs.test-prepare.outputs.runner-id }}
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Set Up Python Environment
        run: |
          python -m venv venv
          source venv/bin/activate
          pip install -r hwci/requirements.txt

      - name: Run Hardware Tests
        run: |
          python hwci/core/main.py --board hwci/boards/nrf52dk.py --test hwci/tests/c_hello.py
```

**Note**: The `test-prepare` job should follow the steps outlined in the [Integrating Treadmill with GitHub Actions](https://book.treadmill.ci/user-guide/github-actions-integration.html) guide, including creating a just-in-time GitHub Actions runner and launching a Treadmill job.

### 6. Customize Board and Tests

#### Board Harness

Implement or modify a board harness in `hwci/boards/` to match your hardware. For example, to create a new board harness:

```python
# hwci/boards/my_custom_board.py

from core.board_harness import BoardHarness
from utils.serial_port import SerialPort

class MyCustomBoard(BoardHarness):
    def __init__(self):
        super().__init__()
        # Set up board-specific configurations
        self.uart_port = "/dev/ttyUSB0"
        self.uart_baudrate = 115200
        self.serial = SerialPort(self.uart_port, self.uart_baudrate)

    # Implement required methods like flash_kernel, erase_board, etc.

board = MyCustomBoard()
```

#### Test Scripts

Write test scripts in `hwci/tests/` that perform the desired testing logic. Here's an example test script:

```python
# hwci/tests/my_test.py

from utils.test_helpers import OneshotTest

class MyTest(OneshotTest):
    def __init__(self):
        super().__init__(apps=["my_app"])

    def oneshot_test(self, board):
        # Implement test logic
        output = board.serial.expect("Expected Output", timeout=10)
        if output:
            print("Test passed")
        else:
            raise Exception("Test failed")

test = MyTest()
```

### 7. Run and Monitor the Workflow

Push your changes to trigger the GitHub Actions workflow. Monitor the workflow runs under the **Actions** tab in your repository to ensure everything executes as expected.

#### Alternative: Launch a Treadmill Job Manually and use tock-hardware-ci interactively via ssh

Coming soon

#### Alternative: Schedule a Job via Treadmill Web Interface

Coming soon

## Additional Resources

- [Operator's Guide](https://book.treadmill.ci/operator-guide.html): For operators managing Treadmill deployments.
- [Internals](https://book.treadmill.ci/internals.html): Documentation of Treadmill components.
- [Terminology](https://book.treadmill.ci/introduction/terminology.html): Definitions of key terms used in Treadmill.
