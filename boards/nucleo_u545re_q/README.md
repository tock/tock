# NUCLEO-U545RE-Q

The [NUCLEO-U545RE-Q](https://www.st.com/en/evaluation-tools/nucleo-u545re-q.html) 
is a development board based on the STM32U545RE microcontroller.

## Building

To build the kernel for this board, run `make` in this directory:

```bash
cd boards/nucleo_u545re_q
make
```

## Programming and Deployment

The Makefile in this directory provides several helper targets for deployment:

| Target | Description |
| :--- | :--- |
| **`make`** | Compiles the Tock kernel for the Nucleo-U545RE-Q. |
| **`make flash`** | Flashes **only the kernel** to the board using `probe-rs`. |
| **`make flash_ocd`** | Flashes **only the kernel** to the board using `openocd`. |
| **`make app`** | Merges the kernel with a userspace app (`.tbf`) into a single unified ELF. Requires `APP_PATH`. |
| **`make app_flash`** | Compiles, merges, and **flashes both the kernel and app** in one step using `probe-rs`. |
| **`make app_flash_ocd`** | Compiles, merges, and **flashes both the kernel and app** in one step using `openocd`. |

### Usage Examples

**1. Flash only the kernel:**
```bash
make flash
# OR
make flash_ocd
```

**2. Flash kernel merged with a specific app:**
```bash
# Provide the path to your compiled Tock Binary Format (.tbf) file
make app_flash APP_PATH=/path/to/your/app.tbf
# OR
make app_flash_ocd APP_PATH=/path/to/your/app.tbf
```

## Flashing Notes

This board is flashed using **`probe-rs`** or **`openocd`**. Due to the specific 
memory layout and metadata sections of the STM32U5, the Makefile surgically 
extracts executable sections (stripping metadata like `.ARM.attributes`) before 
flashing to prevent errors when writing to protected system memory addresses.

### OpenOCD Requirements

The STM32U545 is a newer chip and is **not supported** by OpenOCD 0.12.0 or 
earlier. To use the OpenOCD targets, you must use **OpenOCD 0.12.0+dev** built 
from source.

#### Compiling OpenOCD from Source

If your system's OpenOCD is too old, follow these steps to compile a 
compatible version:

1.  **Install dependencies**:
    ```bash
    sudo apt update
    sudo apt install build-essential libusb-1.0-0-dev libftdi1-dev \
        libtool autoconf automake texinfo pkg-config
    ```

2.  **Clone and build**:
    ```bash
    git clone https://github.com/openocd-org/openocd.git
    cd openocd
    git submodule update --init --recursive
    ./bootstrap
    ./configure --enable-stlink
    make -j$(nproc)
    sudo make install
    ```

3.  **Verify version**:
    ```bash
    openocd --version
    # Should report 0.12.0+dev
    ```

## Console

The kernel console is available on USART1 via the ST-LINK USB connection at 
115200 baud.
