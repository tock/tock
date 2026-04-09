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
| **`make binary`** | Merges the kernel with a userspace app (`.tbf`) into a single unified ELF. Requires `APP_PATH`. |
| **`make binary_flash`** | Compiles, merges, and **flashes both the kernel and app** in one step. |

### Usage Examples

**1. Flash only the kernel:**
```bash
make flash
```

**2. Flash kernel merged with a specific app:**
```bash
# Provide the path to your compiled Tock Binary Format (.tbf) file
make binary_flash APP_PATH=/path/to/your/app.tbf
```

## Flashing Notes

This board is flashed using **`probe-rs`**. Due to the specific memory layout 
and metadata sections of the STM32U5, the Makefile surgically extracts 
executable sections (stripping metadata like `.ARM.attributes`) before flashing 
to prevent errors when writing to protected system memory addresses.

## Console

The kernel console is available on USART1 via the ST-LINK USB connection at 
115,200 baud.
