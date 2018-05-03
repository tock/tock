# Write an environment sensing Bluetooth Low Energy application

- [Intro](README.md)
- [Getting started with Tock](environment.md)
- Write an environment sensing BLE application
- [Add a new capsule to the kernel](capsule.md)

## 1. Presentation: Process overview, relocation model and system call API

In this section, we're going to learn about processes (a.k.a applications) in
Tock, and build our own applications in C.

## 2. Check your understanding

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. How would you write an IPC service to print to the console? Which functions
   would the client need to call?

## 3. Get a C application running on your board

You'll find the outline of a C application in the directory
`docs/courses/sosp/exercises/app`.

Take a look at the code in `main.c`.  So far, this application merely prints
"Hello, World!".

The code uses the standard C library routine `printf` to compose a message
using a format string and print it to the console. Let's break down what the
code layers are here:

1. `printf` is provided by the C standard library (implemented by
   [newlib](https://sourceware.org/newlib/)). It takes the format string and
   arguments, and generates an output string from them. To actually write the
   string to standard out, `printf` calls `_write`.

2. `_write` (in `userland/libtock/sys.c`) is a wrapper for actually writing to
   output streams (in this case, standard out a.k.a. the console). It calls
   the Tock-specific console writing function `putnstr`.

3. `putnstr`(in `userland/libtock/console.c`) buffers data to be written, calls
   `putnstr_async`, and acts as a synchronous wrapper, yielding until the
   operation is complete.

4. `putnstr_async` (in `userland/libtock/console.c`) finally performs the
   actual system calls, calling to `allow`, `subscribe`, and `command` to
   enable the kernel to access the buffer, request a callback when the write is
   complete, and begin the write operation respectively.


The application could accomplish all of this by invoking Tock system calls
directly, but using libraries makes for a much cleaner interface and allows
users to not need to know the inner workings of the OS.


### Loading an application

Okay, let's build and load this simple program.

1. Erase all other applications from the development board:

        $ tockloader erase-apps

2. Build this application:

        $ make

3. Load the application (Note: `tockloader install` automatically searches the
   current working directory and its subdirectories for Tock binaries.)

        $ tockloader install

4. Check that it worked:

        $ tockloader listen

The output should look something like:

```
$ tockloader listen
No device name specified. Using default "tock"
Using "/dev/cu.usbserial-c098e5130012 - Hail IoT Module - TockOS"

Listening for serial output.
Hello, World!
```

## 4. Creating your own application

Now that you've got a basic app working, modify it so that it continuously
prints out `Hello World` twice per second.  You'll want to use the user
library's timer facilities to manage this:

### Timer

You'll find the interface for timers in `userland/libtock/timer.h`. The
function you'll find useful today is:

```c
#include <timer.h>
void delay_ms(uint32_t ms);
```

This function sleeps until the specified number of milliseconds have passed, and
then returns.  So we call this function "synchronous": no further code will run
until the delay is complete.

## 5. Write an app that periodically samples the on-board sensors

Now that we have the ability to write applications, let's do
something a little more complex. The development board you are using has several
sensors on it, [as shown here](https://github.com/tock/tock/blob/master/boards/hail/media/hail_reva_noheaders_labeled.png)
for the Hail board.
These sensors include a light sensor, a humidity sensor, and a temperature sensor. Each sensing medium can be accessed separately via the Tock user
library. We'll just be using the light, temperature, and humidity measurements
for today's tutorial.

#### Light

The interface in `userland/libtock/ambient_light.h` is used to measure ambient light conditions
in [lux](https://en.wikipedia.org/wiki/Lux). Specifically, it uses the sensor
[ISL29035](https://www.intersil.com/en/products/optoelectronics/ambient-light-sensors/light-to-digital-sensors/ISL29035.html).
It contains the function:

```c
#include <ambient_light.h>
int ambient_light_read_intensity_sync(int* lux);
```

Note that the light reading is written to the location passed as an
argument, and the function returns non-zero in the case of an error.

#### Temperature

The interface in `userland/libtock/temperature.h` is used to measure ambient temperature in degrees
Celsius, times 100. It uses the [SI7021](https://www.silabs.com/products/sensors/humidity-sensors/Pages/si7013-20-21.aspx)
sensor. It contains the function:

```c
#include <temperature.h>
int temperature_read_sync(int* temperature);
```

Again, this function returns non-zero in the case of an error.

#### Humidity

The interface in `userland/libtock/humidity.h` is used to measure the ambient
[relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in
percent, times 100. It contains the function:

```c
#include <humidity.h>
int humidity_read_sync (unsigned* humi);
```

Again, this function returns non-zero in the case of an error.

### Read sensors in a Tock application

Using the example program you're working on, write an application that reads
all of the sensors on your development board and reports their readings over
the serial port.

As a bonus, experiment with toggling an LED when readings are above or below a
certain threshold:

#### LED

The interface in `userland/libtock/led.h` is used to control lights on Tock boards. On the Hail
board, there are three LEDs which can be controlled: Red, Blue, and Green. The
functions in the LED module are:

```c
#include <led.h>
int led_count(void);
```

Which returns the number of LEDs available on the board.

```c
int led_on(int led_num);
```

Which turns an LED on, accessed by its number.

```c
int led_off(int led_num);
```

Which turns an LED off, accessed by its number.

```c
int led_toggle(int led_num);
```

Which toggles the state of an LED, accessed by its number.


## 6. Extend your app to report through the `ble-env-sense` service

Finally, let's explore accessing the Bluetooth Low-Energy (BLE) capabilities of
the hardware. The Hail board has an
[nRF51822](https://www.nordicsemi.com/eng/Products/Bluetooth-low-energy/nRF51822)
radio which provides BLE communications. With that and the available sensors,
we can use Tock to provide the BLE
[Environmental Sensing Service](https://www.bluetooth.com/specifications/assigned-numbers/environmental-sensing-service-characteristics)
(ESS).

Tock exposes raw BLE functionality from kernelspace to userland via a syscall
interface. There is also a userland app that leverages the BLE syscall API to
implement an environment sensing service (ESS) as a separate process, instead
of in the kernel. Publishing ESS characteristics (eg.  temperature, ambient
light, etc.) is thus as simple as creating another process on the board that
reads the sensors and communicates with the ESS service over Tock's
inter-process communication (IPC) mechanism.

### Loading the BLE ESS Service

The BLE ESS service can be found in the main Tock repository under
`userland/examples/services/ble-env-sense`. It accepts commands over IPC and
updates the BLE ESS service, which is broadcasts through the BLE radio.

Before we load the service though, you should modify its name so that
you'll be able to tell your Hail apart from everyone else's.  Pick
something short but reasonably unique. On Line 32 of `main.c`, change the
`adv_name` to a string of your choice. For example:

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

### Using the BLE ESS Service from an application

Now that we've got the service loaded, we can extend the application
we've been working on to send environmental measurements over BLE.

#### IPC to the BLE ESS Service

The `ipc.h` interface can be used to send data to the BLE ESS service via
Tock's inter-process communication mechanism.  Details about how to do this
are [here](../../../userland/examples/services/ble-env-sense/README.md), and example
code for sending BLE ESS updates is
[here](../../../userland/examples/services/ble-env-sense/test/main.c).

Now you should be able to write an app that sends data over BLE.  You can load
your app alongside the service that's already loaded on the board, and they
will communicate via IPC.

To test that everything is working, you can connect to your development board
with a smartphone. We recommend the nRF Connect app
[[Android](https://play.google.com/store/apps/details?id=no.nordicsemi.android.mcp&hl=en)
 | [iOS](https://itunes.apple.com/us/app/nrf-connect/id1054362403?mt=8)].
The BLE address of the Hail is labeled on its bottom, but iOS devices
cannot access the address of a BLE device. However, you should be able to see
the unique name that you chose earlier.

