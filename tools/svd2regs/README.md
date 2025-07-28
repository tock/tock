# SVD to Rust Register Generator(svd2regs.py)

This Python script, `svd2regs.py`, converts CMSIS-SVD (System View Description) files into Rust code for memory-mapped peripheral registers. It helps automate the creation of register interface code for embedded Rust projects, using the style of the Tock OS kernel.

## Requirements

* Python 3.10+
* [uv](https://docs.astral.sh/uv/) package manager

## Run

To install all dependencies use:
```bash
  uv sync
```

## Usage
Once the dependencies are installed, you can execute the script using `uv run`. The basic command structure is:
```bash
  uv run python svd2regs.py [OPTIONS] <PERIPHERAL>
```

## Command-Line Arguments
* **peripheral**: (Required) The name of the peripheral to generate code for.
* **--mcu VENDOR MCU**: Use a packaged SVD file from the cmsis-svd database.
* **--svd [SVD_FILE]**: Use a local SVD file. If no file path is given, it reads from standard input (stdin), which is useful for piping.
* **--group, -g**: Treat the peripheral as a group with multiple instances.
* **--save FILE**: Save the generated Rust code to a specific file. Defaults to printing to the console (stdout).
* **--fmt ['ARG ..']**: Format the output using rustfmt. You can pass optional arguments to rustfmt as a quoted string (e.g., '--force').
* **--path PATH**: Specify the path to the rustfmt executable if it's not in your system's PATH.
* **-h, --help**: Show the help message and exit.

## Examples

1. Generate from the `cmsis-svd` Database
```bash
  uv run python svd2regs.py SIM --mcu Freescale MK64F12
```

2. Generate from a Local SVD File and Format Output
```bash
  uv run python svd2regs.py SIM --svd mcu.svd --fmt
```

3. Generate, Format, and Save to a File
```bash
  uv run python svd2regs.py SIM --svd mcu.svd --fmt '--force' --save src/peripherals.rs
```
