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

The Makefile for this board is simplified to focus on OpenOCD-based deployment.

### Makefile Targets

| Target | Description |
| :--- | :--- |
| **`make`** | Compiles the Tock kernel for the Nucleo-U545RE-Q. |
| **`make flash`** | Flashes the release kernel using `openocd`. |
| **`make flash-debug`** | Flashes the debug kernel using `openocd`. |
| **`make program`** | Installs apps using `tockloader` via `openocd`. Requires `APP`. |
| **`make install`** | Alias for `make flash`. |

### Usage Examples

**1. Flash the release kernel:**
```bash
make flash
```

**2. Flash the debug kernel:**
```bash
make flash-debug
```

**3. Install an application:**
Note: This requires a `.tab` (Tock Application Bundle) file.
```bash
make program APP=/path/to/your/app.tab
```

## Flashing Notes

This board is flashed using **`openocd`**. The Makefile handles the necessary 
initialization and mass erase before programming.

### OpenOCD Requirements

The STM32U545 is a newer chip and is **not supported** by OpenOCD 0.12.0 or 
earlier. To use the OpenOCD targets, you must use **OpenOCD 0.12.0+dev** built 
from source.

#### Compiling OpenOCD from Source

If your system's OpenOCD is too old, follow these steps to compile a 
compatible version:

1.  **Install dependencies**:

    **Ubuntu-based distros**
    ```bash
    sudo apt update
    sudo apt install build-essential libusb-1.0-0-dev libftdi1-dev \
        libtool autoconf automake texinfo pkg-config
    ```
    **Fedora-based distros**
    ```bash
    sudo dnf update
    sudo dnf install build-essential libusb1-devel jimtcl libftd-devel \
        libtool autoconf automake texinfo pkgconf
2.  **Clone and build**:

    **Ubuntu-based distro**
    ```bash
    git clone https://github.com/openocd-org/openocd.git
    cd openocd
    git submodule update --init --recursive
    ./bootstrap
    ./configure --enable-stlink
    make -j$(nproc)
    sudo make install
    ```
    **Fedora-based distro**
    ```bash
    git clone https://github.com/openocd-org/openocd.git
    cd openocd
    git submodule update --init --recursive
    ./bootstrap
    ./configure --enable-stlink --enable internal jimtcl
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
