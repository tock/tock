---
location: Shenzhen, China
date: November 4, 2018
---

**Tock is an active project and details may have changes since this tutorial was authored.
If you are interested in doing this tutorial yourself, please
`git checkout courses/2018-sensys` in both this repository and the libtock repository.**

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

In this course, we will look at some of the high-level services provided by Tock.
We will start with an understanding of the OS and its programming environment.
Then we'll look at how a process management application can help afford remote
debugging, diagnosing and fixing a resource-intensive app over the network.
The last part of the tutorial is a bit more free-form, inviting attendees to
further explore the networking and application features of Tock or to dig into
the kernel a bit and explore how to enhance and extend the kernel.

This course assumes some experience programming embedded devices and fluency in C.
It assumes no knowledge of Rust, although knowing Rust will allow you to be
more creative during the kernel exploration at the end.

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

If you would prefer not to use the virtual machine,
[there are directions for manual installation of dependencies](manual_installation.md).

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

2. [Userland programming](application.md): write a basic sensing application in C.

3. [Deliver for the Client](client.md): Help an important client get a
   new board setup.

4. [Free-form Experimentation](freeform.md): Open-ended exploration with
   support from the Tock team.

