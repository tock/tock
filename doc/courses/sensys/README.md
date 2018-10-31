---
location: Shenzhen, China
date: November 4, 2018
---

# Tock OS Training @ SenSys 2018

This course introduces you to Tock, a secure embedded operating system for
sensor networks and the Internet of Things. Tock is the first operating system
to allow multiple untrusted applications to run concurrently on a
microcontroller-based computer. The Tock kernel is written in Rust, a
memory-safe systems language that does not rely on a garbage collector.
Userspace applications are run in single-threaded processes that can be written
in any language. A paper describing Tock's goals, design, and implementation was
published at the SOSP'17 conference and is available
[here](https://www.amitlevy.com/papers/tock-sosp2017.pdf).

In this course, you will learn...TODO


<!-- the basic Tock system architecture, how to write
a userspace process in C, Tock's system call interface, and fill in code for a
small kernel extension written in Rust. The course assumes experience
programming embedded devices and fluency in C. It assumes no knowledge of Rust,
although knowing Rust will allow you to be more creative in the Rust programming
part of the course. -->

## Preparation

We will go over setting up a development environment during the course and help
out with possible problems you run into. However, because the WiFi is likely to
be slow, we **strongly urge you to set up the following dependencies ahead of
time, preferably by downloading the provided VM image**.

First, you will need a laptop running Linux or OS X. Linux in a VM will work
just fine, see below for a pre-made image with all the dependencies. We strongly
recommend you use the pre-made image unless you have set up and tested your
installation before the course.

### Virtual Machine

If you're comfortable working inside a Debian virtual machine, you can download
an image with all of the dependencies already installed
[here](http://www.scs.stanford.edu/~alevy/Tock.ova)

 * VirtualBox users: [File → Import Appliance...](https://docs.oracle.com/cd/E26217_01/E26796/html/qs-import-vm.html),
 * VMWare users: [File → Open...](https://pubs.vmware.com/workstation-9/index.jsp?topic=%2Fcom.vmware.ws.using.doc%2FGUID-DDCBE9C0-0EC9-4D09-8042-18436DA62F7A.html)

The VM account is "user" with password "user". Feel free to customize it with
whichever editors, window managers, etc. you like before the training starts.

> If the Host OS is Linux, you may need to add your user to the `vboxusers`
> group on your machine in order to connect the hardware boards to the virtual
> machine.

### Manual Installation

If you choose to install manually, you will need the following software:

1. Command line utilities: curl, make, git

1. Python 3 and pip3

1. A local clone of the Tock repository

        $ git clone https://github.com/tock/tock.git

1. A local clone of the Tock applications repository (for apps written in C)

        $ git clone https://github.com/tock/libtock-c.git

1. [rustup](http://rustup.rs/). This tool helps manage installations of the
   Rust compiler and related tools.

        $ curl https://sh.rustup.rs -sSf | sh

1. [arm-none-eabi toolchain](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads) (version >= 5.2)

   OS-specific installation instructions can be found
   [here](https://github.com/tock/tock/blob/master/doc/Getting_Started.md#arm-none-eabi-toolchain)

1. [tockloader](https://github.com/tock/tockloader)

        $ pip3 install -U --user tockloader

    > Note: On MacOS, you may need to add `tockloader` to your path. If you
    > cannot run it after installation, run the following:

        $ export PATH=$HOME/Library/Python/3.6/bin/:$PATH

    > Similarly, on Linux distributions, this will typically install to
    > `$HOME/.local/bin`, and you may need to add that to your `$PATH` if not
    > already present:

        $ PATH=$HOME/.local/bin:$PATH

### Testing

To test if your environment is working, go to the `tock/boards/imix` directory
and type `make program`. This should compile the kernel for the default board,
Imix, and try to program it over a USB serial connection. It may need to compile
several supporting libraries first (so may take 30 seconds or so the first
time). You should see output like this:

```
$ make
   Compiling tock-registers v0.2.0 (file:///Users/bradjc/git/tock/libraries/tock-register-interface)
   Compiling tock-cells v0.1.0 (file:///Users/bradjc/git/tock/libraries/tock-cells)
   Compiling enum_primitive v0.1.0 (file:///Users/bradjc/git/tock/libraries/enum_primitive)
   Compiling imix v0.1.0 (file:///Users/bradjc/git/tock/boards/imix)
   Compiling kernel v0.1.0 (file:///Users/bradjc/git/tock/kernel)
   Compiling cortexm v0.1.0 (file:///Users/bradjc/git/tock/arch/cortex-m)
   Compiling capsules v0.1.0 (file:///Users/bradjc/git/tock/capsules)
   Compiling cortexm4 v0.1.0 (file:///Users/bradjc/git/tock/arch/cortex-m4)
   Compiling sam4l v0.1.0 (file:///Users/bradjc/git/tock/chips/sam4l)
    Finished release [optimized + debuginfo] target(s) in 23.89s
   text    data     bss     dec     hex filename
 148192    5988   34968  189148   2e2dc target/thumbv7em-none-eabi/release/imix
tockloader  flash --address 0x10000 target/thumbv7em-none-eabi/release/imix.bin
No device name specified. Using default "tock"
No serial ports found. Is the board connected?

make: *** [program] Error 1
```

That is, since you don't yet have a board plugged in it can't program it. But
the above output indicates that it can compile correctly and invoke `tockloader`
to program a board.

## Agenda

The training is divided into three sections, each starting with a short
presentation to introduce some concepts, followed by a practical exercise.

1. [Environment Setup](environment.md): Get familiar with the Tock tools
   and getting a board setup.
2. [Deliver for the Client](client.md): Help an important client get a
   new board setup.



## Older Version of this Course

If you are looking for the 2017 version of this course (held at SenSys'17),
please use the following commit:

```bash
$ git checkout 1203437bff05667bb0636dc9ab69e1daca13c2a2
```

