---
location: Portland, OR, USA
date: August 19th
---

# Tock OS Training @ RustConf 2017

Put Rust to practice in low-level embedded systems. This training will introduce
cover programming for Tock, a secure embedded operating system for sensor
networks and the Internet of Things, written in Rust. You will learn to write
kernel extensions, the basics of porting Tock to a new platform, and how to
write power- and memory-efficient applications. We will also give an overview of
the system architecture.

This tutorial assumes basic knowledge of Rust, including ownership, borrowing,
traits, and lifetimes. While not required, it is most appropriate for people who
are familiar with the material covered in the Advanced Rust training, and
attending the morning Intermediate Rust training is highly encouraged.

## Pre-requisites

We will go over setting up a development environment during the training.
However, because the WiFi might not be provide fastest Internet connection in
the world, it would be useful to set up the following dependencies ahead of
time:

1. A laptop running Linux or OS X. Linux in a VM will work just fine, see below
   for a pre-made image with all the dependencies.

2. Command line utilities: wget, sed, make, cmake, git

4. Python 3 and pip

5. A local clone of the Tock repository
     
        $ git clone https://github.com/helena-project/tock.git

6. [rustup](http://rustup.rs/).
     
        $ curl https://sh.rustup.rs -sSf | sh
        $ rustup install nightly-2017-06-20

7. [Xargo](https://github.com/japaric/xargo)
     
        $ cargo install xargo

8. [arm-none-eabi toolchain](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads) (version >= 5.2)

    > Note that you can install the version packaged by your Linux distribution,
    > but make sure you install the newlib port as well. For instance, on Debian or
    > Ubuntu, install both gcc-arm-none-eabi and libnewlib-arm-none-eabi.

9. [tockloader](https://github.com/helena-project/tockloader)
     
        $ pip3 install -U --user tockloader

    > Note: On MacOS, you may need to add `tockloader` to your path. If you
    > cannot run it after installation, run the following:

        $ export PATH=$HOME/Library/Python/3.6/bin/:$PATH

    > Similarly, on Linux distributions, this will typically install to
    > `$HOME/.local/bin`, and you may need to add that to your `$PATH` if not
    > already present:

        $ PATH=$HOME/.local/bin:$PATH

### Virtual Machine

If you're comfortable working inside a Debian virtual machine, you can download
an image with all of the dependencies already installed
[here](https://www.dropbox.com/s/5km04herxa9h05w/Tock.ova?dl=0)
(VirtualBox users:
[File → Import Appliance...](https://docs.oracle.com/cd/E26217_01/E26796/html/qs-import-vm.html),
VMWare users:
[File → Open...](https://pubs.vmware.com/workstation-9/index.jsp?topic=%2Fcom.vmware.ws.using.doc%2FGUID-DDCBE9C0-0EC9-4D09-8042-18436DA62F7A.html)
).
The VM account is "user" with password "user".
Feel free to customize it with whichever editors, window managers, etc you like
before the training starts.

> #### Heads Up!
> There's a small error in the VM configuration. You'll need to manually
> `source ~/.profile` when you open a new terminal to set up all the needed
> paths.

## Agenda

The training is divided into three sections, each starting with a short
presentation to introduce some concepts, followed by a practical exercise.

1. [Getting your environment set up](environment.md) (~1 hour)

2. [Add a new capsule to the kernel](capsule.md) (~2 hours)

3. [Write an environment sensing Bluetooth Low Energy
   application](application.md) (~1 hour)

