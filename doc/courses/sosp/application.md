# Write an environment sensing Bluetooth Low Energy application

## 1. Presentation: Process overview, relocation model and system call API

In this section, we're going to learn about processes (a.k.a applications) in
Tock, and build our own applications in Rust.

## 2. Check your understanding

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. What is a Grant? How do processes interact with grants? Hint: Think about
   memory exhaustion.

## 3. Get a C application running on Hail

You'll find the outline of a C application in the directory
`userland/examples/sosp`.

Take a look at the code in `main.c`.

So far, this application merely prints "Hello, World!"  It could have
accomplished that by making Tock system calls directly, but just like in other
systems, a user library (in `userland/libtock/`) provides a more convenient
interface for this and many other purposes.  Let's look at the interface for
console I/O.

#### Console

You'll notice that this program includes the header file `console.h`.  You can
find that file in `userland/libtock/console.h`.

The console interface contains the function `putstr` and a couple variants.
On your development board, this function can be used to send messages over the
USB connection to your PC.  What's actually happening on the board is that the
UART transceiver on the microcontroller sends serial data to a chip (FTDI) that
then converts the data to USB messages.

The `putstr` function itself is "synchronous", meaning that it doesn't return
until the I/O operation has completed.  But your example program instead calls
`putnstr_async`, which is more fundamental in that it sends the message to print
and then waits for a "callback" to signal that the operation has been completed.
(The `putstr` function is implemented by the tock library in terms of
`putnstr_async`.)

The callback in this program presently does nothing, but you may find it useful
later.

#### Timer

You'll find the interface for timers in `userland/libtock/timer.h`.  The
function you'll find useful today is:

    void delay_ms(uint32_t ms);

This function sleeps until the specified number of milliseconds have passed, and
then returns.  So we call this function "synchronous": no further code will run
until the delay is complete.

### Loading a Rust application

Okay, let's build and load this simple program.

1. Erase all other applications from the development board

         tockloader erase-apps

2. Build this application

         make

3. Load the application.  (Note: `tockloader install` automatically searches the
   current working directory and its subdirectories for Tock binaries.)

         tockloader install

4. Check that it worked.

         tockloader listen

The expected output should look something like:

```
$ tockloader listen
No device name specified. Using default "tock"
Using "/dev/cu.usbserial-c098e5130012 - Hail IoT Module - TockOS"

Listening for serial output.
Hello, World!
```

### Creating your own application

Now that you've got a basic app working, modify it so that it continuously
prints out `Hello World` twice per second.  Remember the function `delay_ms` as
explained above

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

