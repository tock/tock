# Write an environment sensing Bluetooth Low Energy application

## 1. Presentation: Process overview, relocation model and system call API

In this section, we're going to learn about processes (a.k.a applications) in
Tock, and build our own applications in Rust.

## 2. Check your understanding

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. What is a Grant? How do processes interact with grants? Hint: Think about
   memory exhaustion.

## 3. Get a Rust application running on Hail

First, clone the tock-rust-template repository.

         $ git clone https://github.com/helena-project/tock-rust-template.git

This is the base for Tock applications written in Rust. Your code goes in the
`src` folder in `main.rs`. The `Cargo` and `Xargo` files are Rust build
configurations. The `thumbv7em-tock-eabi.json` and `layout.ld` files are code
compilation configurations. The Makefile uses `xargo` to create ELF files, and
several scripts in `tools/` to build Tock binaries, with all built output going
in the directory `target/thumb7em-tock-eabi/release/`.

First, lets look at the application code. `main()` is the function called when
the app is started. The base functionality of it creates a Tock console object
and then prints a message through it via the `write!` macro. The
[`alloc`](https://doc.rust-lang.org/beta/alloc/) crate is used to make the
`write_fmt` function inside of `write!` work. Note that `write!` returns a
`Result`, which we call unwrap on to handle.

We also use the [Tock crate](https://github.com/helena-project/libtock-rs)
which contains the Rust library for interacting with a Tock kernel. Two pieces
of Tock functionality which we will explain here are the Console and Timer
modules that the Tock crate exports.

#### Console

`tock::console::Console` is used to send messages over the USB connection on a
Hail (technically it sends serial data through a UART to an FTDI UART-to-USB
chip, but same difference). Its functions are:

         pub fn new() -> Console

   Creates, initializes, and returns a new Console struct.

         pub fn write(&mut self, string: String)

   Writes a string object to the Console.

`Console` also implements `fmt::write`, which enables the `write!` macro to
work. We recommend using
[`write!`](https://doc.rust-lang.org/1.5.0/std/macro.write!.html) for this
tutorial, as it allows you to use [format
options](https://doc.rust-lang.org/1.5.0/std/fmt/) while printing.

#### Timer

`tock::timer` is used to trigger events at a specific number of seconds in the
future. It has several functions, only one of which will be used today:

         pub fn delay_ms(ms: u32)

   Sleeps until the specified number of milliseconds have passed, at which
   point this function will return. Note that this is synchronous, and no
   further code will run until the delay is complete.

### Loading a Rust application

Now, lets build and load the base template application in `src/main.rs`.

1. Erase all other applications from the Hail.

         tockloader erase-apps

2. Build this Rust application.

         make

3. Load the Rust application. (note: `tockloader install` automatically
   searches subdirectories for Tock binaries)

         tockloader install

4. Check that it worked.

         tockloader listen

The expected output should look like:

```
$ tockloader listen
No device name specified. Using default "tock"
Using "/dev/cu.usbserial-c098e5130012 - Hail IoT Module - TockOS"

Listening for serial output.
Tock App
```

### Creating your own Rust application

Now that you've got a basic Rust app working, modify it so that it continuously
prints out `Hello World` twice per second. Note the Tock function `delay_ms` as
explained above, as well as the Rust
[loop](https://doc.rust-lang.org/1.6.0/book/loops.html) instruction.


## 4. Write an app that periodically samples the on-board sensors

Now that we have the ability to write Tock applications in Rust, lets do
something a little more complex. The Hail board you are using has several
sensors on it [as shown here](https://github.com/helena-project/tock/blob/master/boards/hail/media/hail_reva_noheaders_labeled.png).
These sensors include a light sensor, a humidity and temperature sensor, and an
acceleration and magnetic field sensor (marked as accelerometer in the
picture). Each sensing medium can be accessed separately through the Tock
crate, each within the `sensors` module.

#### Light

`tock::sensors::AmbientLightSensor` is used to measure ambient light conditions
in [lux](https://en.wikipedia.org/wiki/Lux). Specifically, it uses the sensor
[ISL29035](https://www.intersil.com/en/products/optoelectronics/ambient-light-sensors/light-to-digital-sensors/ISL29035.html).
It has the function:

         pub fn read(&mut self) -> Reading

   Where a Reading in this case is implemented as the type `AmbientLight`,
   which is capable of being cast into an `i32` or printed in a message.

#### Temperature

`tock::sensors::TemperatureSensor` is used to measure ambient temperature in degrees
Celsius. It uses the [SI7021](https://www.silabs.com/products/sensors/humidity-sensors/Pages/si7013-20-21.aspx)
sensor. It has the function:

         pub fn read(&mut self) -> Reading

   Where a Reading in this case is implemented as the type `Temperature`, which
   is capable of being cast into an `i32` or printed in a message.

#### Humidity

`tock::sensors::HumiditySensor` is used to measure the ambient
[relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in
percent. It has the function:

         pub fn read(&mut self) -> Reading

   Where a Reading in this case is implemented as the type `Humdity`, which
   is capable of being cast into an `i32` or printed in a message.

#### Nindedof

`tock::sensors::Ninedof` is used to read acceleration or magnetic field
strength from the
[FXOS8700CQ](http://www.nxp.com/products/sensors/6-axis-sensors/digital-sensor-3d-accelerometer-2g-4g-8g-plus-3d-magnetometer:FXOS8700CQ).
Note that Hail's hardware implementation of the Ninedof does not include the
traditional rotational sensor. It has the functions:

         pub unsafe fn new() -> Ninedof

   Which creates a new Ninedof struct on which the following functions may be
   called. Note that this function is `unsafe` and must be called within an
   unsafe block.

         pub fn read_acceleration(&mut self) -> NinedofReading

   Which reads acceleration in [g's](https://en.wikipedia.org/wiki/G-force) in
   the x, y, and z orientations.

         pub fn read_magnetometer(&mut self) -> NinedofReading

   Which reads magnetic field strength in
   [microTeslas](https://en.wikipedia.org/wiki/Tesla_(unit)) in the x, y, and z
   orientations. 

It also has the NinedofReading struct:

         pub struct NinedofReading {
            pub x: i32,
            pub y: i32,
            pub z: i32
         }

   Which has `fmt:Display` implemented for it and thus can be directly printed.


### Read sensors in a Tock application

Using the tock-rust-template, write an application that reads all of the
sensors on Hail and reports their readings over serial. As a bonus, experiment
with turning on/or off an LED when readings are above or below a certain
threshold.

#### LED

`tock::led` is used to control lights on Tock boards. On the Hail board, there
are three LEDs: Red, Blue, and Green which can be controlled. The functions in
the LED module are:

         pub fn count() -> isize

   Which returns the number of LEDs available on a board.

         pub fn on(led_num: u32)

   Which turns an LED on, accessed by its number.

         pub fn off(led_num: u32)

   Which turns an LED off, accessed by its number.

         pub fn toggle(led_num: u32)

   Which changes the state of an LED, accessed by its number.

[Sample Solution](https://gist.github.com/alevy/73d0a1e5c8784df066c86dc5da9d3107).


## 5. Extend your app to report through the `ble-env-sense` service

Finally, lets explore accessing the Bluetooth Low-Energy (BLE) capabilities of
the hardware. The Hail board has an
[nRF51822](https://www.nordicsemi.com/eng/Products/Bluetooth-low-energy/nRF51822)
radio which provides BLE communications. Given that and the sensors available,
we can use Tock to provide the BLE
[Environmental Sensing Service](https://www.bluetooth.com/specifications/assigned-numbers/environmental-sensing-service-characteristics)
(ESS).

Currently, the Tock libraries for Rust do not support directly
interacting with the BLE radio. However, we can still access BLE by loading an
additional process on the board as a service and sending it commands over
Tock's inter-process communication (IPC) method.

### Loading the BLE ESS Service

The BLE ESS service can be found in the main Tock repository under
`userland/examples/services/ble-env-sense`. It accepts commands over IPC and
updates the BLE ESS service, which is broadcasts through the BLE radio.

Before we load the service though, you should chose modify its name so that
you'll be able to tell your Hail apart from everyone else's (be sure to pick
something short but reasonably unique). On Line 32, change the adv_name to a
string of your choice. For example:

```
   .adv_name          = "AmitHail",
```

Once you've changed the name, we can load the service onto the Hail.

```
$ tockloader erase-apps
$ cd userland/examples/services/ble-env-sense/
$ make program
$ tockloader listen
...
[BLE] Environmental Sensing IPC Service
...
```

### Using the BLE ESS Service from a Rust application

Now that we've got the service loaded, we can build our own application to use
it in the tock-rust-template repository.

**IMPORTANT**

For this section only, the `layout.ld` file needs to be modified. On Line 18,
the FLASH ORIGIN needs to be changed to `0x00040038`. This places the Rust
application after the BLE ESS service in memory (and will not be necessary soon
when we get Rust applications compiling as position independent code). It
should look like this:

```
    FLASH (rx) : ORIGIN = 0x00040038, LENGTH = PROG_LENGTH
```

#### IPC to the BLE ESS Service

`tock::ipc::ble_ess` allows for data to be sent to the BLE ESS service via
Tock's inter-process communication mechanism. It has one function:

         pub fn connect() -> Result<BleEss, ()>

   This connects to the BLE ESS service over IPC, returning a
   [Result](https://doc.rust-lang.org/std/result/) with a BleEss struct.

The BleEss struct itself has one function:

         pub fn set_reading<I>(&mut self, sensor: ReadingType, data: I) -> Result<(), ()>

   Which takes a ReadingType and a measurement, and updates it in the
   Environmental Sensing Service.

The `tock::ipc::ble_ess` also has the ReadingType enum:

         pub enum ReadingType {
             Temperature = 0,
             Humidity = 1,
             Light = 2
         }

   Note that the ESS does not accept acceleration or magnetic field strength
   measurements.


Now that you've got the IPC library, you should be able to write an app that
sends data over BLE. To get you started, here are what the first couple lines
will probably look like:

```
#![feature(alloc)]
#![no_std]

extern crate alloc;
extern crate tock;

use alloc::fmt::Write;
use tock::console::Console;
use tock::ipc::ble_ess::{self, ReadingType};
use tock::sensors::*;

#[inline(never)]
fn main() {
    let mut console = Console::new();
    write!(&mut console, "Starting BLE ESS\n").unwrap();

    let mut ess = match ble_ess::connect() {
        Ok(ess) => ess,
        _ => {
            write!(&mut console, "BLE IPC Service not installed\n").unwrap();
            return
        }
    };
    write!(&mut console, "Found BLE IPC Service\n").unwrap();
    ...
```

To test that everything is working, you can connect to your Hail with a
smartphone. We recommend the nRF Connect app
[[Android](https://play.google.com/store/apps/details?id=no.nordicsemi.android.mcp&hl=en)
 | [iOS](https://itunes.apple.com/us/app/nrf-connect/id1054362403?mt=8)].
The BLE address of the Hail is labeled on its bottom, however iOS devices
cannot access the address of a BLE device. However, you should be able to see
the unique name that you chose earlier.

[Sample Solution](https://gist.github.com/alevy/a274981a29ffc00230aa16101ee0b89f).

