QEMU RV32 VIRT Tutorial Board
=============================

This QEMU board supports Tock tutorials.

Using the Board
---------------

First, initialize the local binary board file the kernel and apps will be stored
in:

```
make init
```

Then, install an app:

```
cd libtock-rs/demos/embedded_graphics/buttons
make
tockloader install
```

Then, build and install the kernel and run QEMU:

```
make run
```
