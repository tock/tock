Running Tock as a Linux Process
===============================

## Building the kernel

In `Makefile`, select your target CPU by setting the `CPU` variable. Allowed CPUs are:
  - x86_64 (AMD64)

The kernel can be compiled with or without apps. `cd` into `boards/linux_process`
directory and run:

```bash
$ make
```

> **Note:** This will compile only the kernel with no apps.

## Flashing app

Apps are built out-of-tree. Once an app is built, you can use
`objcopy` with `--update-section` to create an ELF image with the
apps included.

```bash
$ objcopy  \
    --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
```

For example, you can update `Makefile` as follows.

```
APP=../../../libtock-c/examples/c_hello/build/$(CPU)/$(CPU).tbf
KERNEL=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM).elf
KERNEL_WITH_APP=$(TOCK_ROOT_DIRECTORY)/target/$(TARGET)/debug/$(PLATFORM)-app.elf

.PHONY: program
program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/debug/$(PLATFORM).elf
	objcopy --update-section .apps=$(APP) $(KERNEL) $(KERNEL_WITH_APP)
	$(KERNEL_WITH_APP)
```

After setting `APP`, `KERNEL`, `KERNEL_WITH_APP`, and `program` target
dependency, you can do

```bash
$ make program
```

to build the kernel with apps and run it.

Running Tock is running a linux process.
