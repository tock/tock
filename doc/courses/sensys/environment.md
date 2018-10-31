# Tock OS Course Part 1: Getting your environment set up

> While we're getting set up and started, please make sure you have
> completed all of the [tutorial pre-requisites](README.md#preparation).
> If you prefer, you can download a
> [virtual machine image](README.md#virtual-machine) with all the pre-requisites
> already installed.

- [Intro](README.md)
- Getting started with Tock
- [TODO](application.md)
- [TODO](capsule.md)

The goal of this part of the course is to make sure you have a working
development environment for Tock.

During this portion of the course you will:

- Get a high-level overview of how Tock works.
- Learn how to compile and flash the kernel onto a Imix board.

## 1. Presentation: Tock's goals, architecture and components

A key contribution of Tock is that it uses Rust's borrow checker as a
language sandbox for isolation and a cooperative scheduling model for
concurrency in the kernel.  As a result, isolation is (more or less) free in
terms of resource consumption at the expense of preemptive scheduling (so a
malicious component could block the system by, e.g., spinning in an infinite
loop). This is accomplished by the following architecture:



<img src="architecture.png" width="50%">

Tock includes three architectural components:

  - A small trusted _kernel_, written in Rust, that implements a hardware
    abstraction layer (HAL), scheduler, and platform-specific configuration.
  - _Capsules_, which are compiled with the kernel and use Rust's type and
    module systems for safety.
  - _Processes_, which use the memory protection unit (MPU) for protection at runtime.

Read the Tock documentation for more details on its
[design](https://github.com/tock/tock/blob/master/doc/Design.md).

[_Presentation slides are available here._](presentation/presentation.pdf)

## 2. Check your understanding

1. What kinds of binaries exist on a Tock board? Hint: There are three, and
   only two can be programmed using `tockloader`.

2. What are the differences between capsules and processes? What performance
   and memory overhead does each entail? Why would you choose to write
   something as a process instead of a capsule and vice versa?

3. Clearly, the kernel should never enter an infinite loop. But is it
   acceptable for a process to spin? What about a capsule?

## 3. Compile and program the kernel

### Make sure your Tock repository is up to date

    $ git pull

### Build the kernel

To build the kernel, just type make in `boards/imix/`.

    $ cd boards/imix/
    $ make

If this is the first time you are trying to make the kernel, the build system
will use cargo and rustup to install various Tock dependencies.

Kernels must be compiled from within the desired board's folder. For example, to
compile for Hail instead you must first run `cd boards/hail/`.

### Connect to a imix board

> On Linux, you might need to give your user access to the imix's serial port.
> If you are using the VM, this is already done for you.
> You can do this by adding a udev rule:
>
>     $ sudo bash -c "echo 'ATTRS{idVendor}==\"0403\", ATTRS{idProduct}==\"6015\", MODE=\"0666\"' > /etc/udev/rules.d/99-imix"
>
> Afterwards, detach and re-attach the imix to reload the rule.

> With the virtual machine, you might need to attach the USB device to the
> VM. To do so, after plugging in imix, select in the VirtualBox/VMWare menu bar:
>
>     Devices -> USB -> "??????" TODO
>
> If this generates an error, often unplugging/replugging fixes it. You can also
> create a rule in the VM USB settings which will auto-attach the imix to the VM.

To connect your development machine to the imix, connect them with a micro-USB
cable. Any cable will do. imix should come with ???? installed... TODO

The imix board should appear as a regular serial device (e.g.
`/dev/tty.usbserial-c098e5130006` on my Mac and `/dev/ttyUSB0` on my Linux box).
While you can connect with any standard serial program (set to 115200 baud),
tockloader makes this easier. Tockloader can read attributes from connected
serial devices, and will automatically find your connected imix. Simply run:

    $ tockloader listen
    No device name specified. Using default "tock"
    Using "/dev/ttyUSB0 - ???????????"

    Listening for serial output.

    ??????????????
    ...

### Flash the kernel

Now that the imix board is connected and you have verified that the kernel
compiles, we can flash the imix board with the latest Tock kernel:

    $ cd boards/imix/
    $ make program

This command will compile the kernel if needed, and then use `tockloader` to
flash it onto the imix. When the flash command succeeds, the imix test app
should still be running and the blue LED will be blinking TODO IS THIS TRUE?????.
You now have the bleeding-edge Tock kernel running on your imix board!

### Clear out the applications and re-flash the test app.

Lets check what's on the board right now:

    $ tockloader list
    ...
    [App 0]
      Name:                  imix
      Enabled:               True
      Sticky:                False
      Total Size in Flash:   65536 bytes
    ...

As you can see, the old imix test app is still installed on the board. This
also nicely demonstrates that user applications are nicely isolated from the
kernel: it is possible to update one independently of the other. Remove it with
the following command:

    $ tockloader uninstall

The blue LED ??????? should no longer blink, and another `tockloader list` should show
nothing installed. Compile and re-flash the imix test app, using the app in
the `libtock-c` repository you cloned:

    $ cd libtock-c/examples/tests/imix/
    $ make program

## 4. (Optional) Familiarize yourself with `tockloader` commands
The `tockloader` tool is a useful and versatile tool for managing and installing
applications on Tock. It supports a number of commands, and a more complete
list can be found in the tockloader repository, located at
https://github.com/tock/tockloader. Below is a list of the more useful
and important commands for programming and querying a board.

### `tockloader install`
This is the main tockloader command, used to load Tock applications onto a
board.
By default, `tockloader install` adds the new application, but does not erase
any others, replacing any already existing application with the same name.
Use the `--no-replace` flag to install multiple copies of the same app.
In order to install an app, navigate to the correct directory, make the program,
then issue the install command:

    $ cd libtock-c/examples/blink
    $ make
    $ tockloader install

> *Tip:* You can add the `--make` flag to have tockloader automatically
> run make before installing, i.e. `tockloader install --make`

> *Tip:* You can add the `--erase` flag to have tockloader automatically
> remove other applications when installing a new one.

### `tockloader uninstall [application name(s)]`
Removes one or more applications from the board by name.

### `tockloader erase-apps`
Removes all applications from the board.

### `tockloader list`
Prints basic information about the apps currently loaded onto the board.

### `tockloader info`
Shows all properties of the board, including information about currently
loaded applications, their sizes and versions, and any set attributes.

### `tockloader listen`
This command prints output from Tock apps to the terminal. It listens via UART,
and will print out anything written to stdout/stderr from a board.

> *Tip:* As a long-running command, `listen` interacts with other tockloader
> sessions. You can leave a terminal window open and listening. If another
> tockloader process needs access to the board (e.g. to install an app update),
> tockloader will automatically pause and resume listening.

### `tockloader flash`
Loads binaries onto hardware platforms that are running a compatible bootloader.
This is used by the Tock Make system when kernel binaries are programmed to the
board with `make program`.

## 5. (Optional) Explore other Tock example applications

Other applications can be found in the `libtock-c/examples/` directory. Try
loading them on your imix and then try modifying them. By default, `tockloader
install` adds the new application, but does not erase any others. Be aware, not
all applications will work well together if they need the same resources (Tock
is in active development to add virtualization to all resources to remove this
issue!).

