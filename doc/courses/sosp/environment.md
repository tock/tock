# Tock OS Course Part 1: Getting your environment set up

> While we're getting set up and started, please make sure you have
> completed all of the [tutorial pre-requisites](./#preparation).
> If you prefer, you can download a
> [virtual machine image](./#virtual-machine) with all the pre-requisites
> already installed.

- [Intro](README.md)
- Getting started with Tock
- [Write an environment sensing BLE application](application.md)
- [Add a new capsule to the kernel](capsule.md)

The goal of this part of the course is to make sure you have a working
development environment for Tock.

During this portion of the course you will:

- Get a high-level overview of how Tock works.
- Learn how to compile and flash the kernel onto a Hail board.

## 1. Presentation: Tock's goals, architecture and components

A key contribution of Tock is that it uses Rust's borrow checker as a
language sandbox for isolation and a cooperative scheduling model for
concurrency in the kernel.  As a result, isolation is (more or less) free in
terms of resource consumption at the expense of preemptive scheduling (so a
malicious component could block the system by, e.g., spinning in an infinite
loop). This is accomplished by the following architecture:

![Tock architecture](architecture.png)

Tock includes three architectural components:

  - A small trusted _kernel_, written in Rust, that implements a hardware
    abstraction layer (HAL), scheduler, and platform-specific configuration.
  - _Capsules_, which are compiled with the kernel and use Rust's type and
    module systems for safety.
  - _Processes_, which use the MPU for protection at runtime.

Read the Tock documentation for more details on its
[design](https://github.com/helena-project/tock/blob/master/doc/Design.md).

[_Presentation slides are availble here._](presentation/presentation.pdf)

## 2. Check your understanding

1. What kinds of binaries exist on a Tock board? Hint: There are three, and
   only two can be programmed using `tockloader`.

2. What are the differences between capsules and processes? What performance
   and memory overhead does each entail? Why would you choose to write
   something as a process instead of a capsule and vice versa?

3. Clearly, the kernel should never enter an infinite loop. But is it
   acceptable for a process to spin? What about a capsule?

## 3. Compile and flash the kernel

### Make sure your Tock repository is up to date

    $ git pull

### Build the kernel

To build the kernel, just type make in `boards/hail/`.

    $ cd boards/hail/
    $ make

If this is the first time you are trying to make the kernel, the build system
will use cargo and rustup to install various Tock dependencies.

Kernels must be compiled from within the desired board's folder. For example, to
compile for imix instead you must first run `cd boards/imix/`.

### Connect to a Hail board

> On Linux, you might need to give your user access to the Hail's serial port.
> If you are using the VM, this is already done for you.
> You can do this by adding a udev rule:
>
>     $ sudo bash -c "echo 'ATTRS{idVendor}==\"0403\", ATTRS{idProduct}==\"6015\", MODE=\"0666\"' > /etc/udev/rules.d/99-hail"
>
> Afterwards, detach and re-attach the Hail to reload the rule.

> With the virtual machine, you might need to attach the USB device to the
> VM. To do so, after plugging in Hail, select in the VirtualBox/VMWare menu bar:
>
>     Devices -> USB -> "Lab11 Hail IoT Module - TockOS"
>
> If this generates an error, often unplugging/replugging fixes it. You can also
> create a rule in the VM USB settings which will auto-attach the Hail to the VM.

To connect your development machine to the Hail, connect them with a micro-USB
cable. Any cable will do. Hail should come with the Tock kernel and the Hail
test app pre-loaded. When you plug in Hail, the blue LED should blink slowly
(about once per second). Pressing the User Button—just to the right of the USB
plug—should turn on the green LED (if the blue LED turned off, the other button
is the Reset button, make sure you hit the right one!).

The Hail board should appear as a regular serial device (e.g.
`/dev/tty.usbserial-c098e5130006` on my Mac and `/dev/ttyUSB0` on my Linux box).
While you can connect with any standard serial program (set to 115200 baud),
tockloader makes this easier. Tockloader can read attributes from connected
serial devices, and will automatically find your connected Hail. Simply run:

    $ tockloader listen
    No device name specified. Using default "tock"
    Using "/dev/ttyUSB0 - Hail IoT Module - TockOS"

    Listening for serial output.

    [Hail] Test App!
    [Hail] Samples all sensors.
    [Hail] Transmits name over BLE.
    [Hail] Button controls LED.
    [Hail Sensor Reading]
      Temperature:  3174 1/100 degrees C
      Humidity:     3915 0.01%
      Light:        15
      Acceleration: 987
    ...

### Flash the kernel

Now that the Hail board is connected and you have verified that the kernel
compiles, we can flash the Hail board with the latest Tock kernel:

    $ cd boards/hail/
    $ make program

This command will compile the kernel if needed, and then use `tockloader` to
flash it onto the Hail. When the flash command succeeds, the Hail test app
should still be running and the blue LED will be blinking.
You now have the bleeding-edge Tock kernel running on your Hail board!

### Clear out the applications and re-flash the test app.

Lets check what's on the board right now:

    $ tockloader list
    ...
    [App 0]
      Name:                  hail
      Enabled:               True
      Sticky:                False
      Total Size in Flash:   65536 bytes
    ...

As you can see, the old Hail test app is still installed on the board. This
also nicely demonstrates that user applications are nicely isolated from the
kernel: it is possible to update one independently of the other. Remove it with
the following command:

    $ tockloader uninstall

The blue LED should no longer blink, and another `tockloader list` should show
nothing installed. Compile and re-flash the Hail test app:

    $ cd userland/examples/tests/hail/
    $ make program

## 4. (Optional) Familiarize yourself with `tockloader` commands
The `tockloader` tool is a useful and versatile tool for managing and installing
applications on Tock. It supports a number of commands, and a more complete
list can be found in the tockloader repository, located at
https://github.com/helena-project/tockloader. Below is a list of the more useful
and important commands for programming and querying a board.

### `tockloader install`
This is the main tockloader command, used to load Tock applications onto a
board.
By default, `tockloader install` adds the new application, but does not erase
any others, replacing any already existing application with the same name.
Use the `--no-replace` flag to install multiple copies of the same app.
In order to install an app, navigate to the correct directory, make the program,
then issue the install command:

    $ cd tock/userland/examples/blink
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

Other applications can be found in the `userland/examples/` directory. Try
loading them on your Hail and then try modifying them. By default, `tockloader
install` adds the new application, but does not erase any others. Be aware, not
all applications will work well together if they need the same resources (Tock
is in active development to add virtualization to all resources to remove this
issue!).

