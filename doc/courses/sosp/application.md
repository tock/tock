# Write an environment sensing Bluetooth Low Energy application

## 1. Presentation: Process overview, relocation model and system call API

In this section, we're going to learn about processes (a.k.a applications) in
Tock, and build our own applications in Rust.

## 2. Check your understanding

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. What is a Grant? How do processes interact with grants? Hint: Think about
   memory exhaustion.

## 3. Get a C application running on your board

You'll find the outline of a C application in the directory
`userland/examples/sosp`.

Take a look at the code in `main.c`.  So far, this application merely prints
"Hello, World!"

What's actually happening under the hood is that `printf` uses the interface in
`userland/libtock/console.h` to send your message to the microcontroller's
asynchronous serial transceiver, called the UART.  The UART sends data to
another chip that translates it to USB messages, which can be received by your
PC.

### Loading a Rust application

Okay, let's build and load this simple program.

1. Erase all other applications from the development board:

         tockloader erase-apps

2. Build this application:

         make

3. Load the application (Note: `tockloader install` automatically searches the
   current working directory and its subdirectories for Tock binaries.)

         tockloader install

4. Check that it worked:

         tockloader listen

The output should look something like:

```
$ tockloader listen
No device name specified. Using default "tock"
Using "/dev/cu.usbserial-c098e5130012 - Hail IoT Module - TockOS"

Listening for serial output.
Hello, World!
```

### Creating your own application

Now that you've got a basic app working, modify it so that it continuously
prints out `Hello World` twice per second.  You'll want to use the user
library's timer facilities to manage this:

#### Timer

You'll find the interface for timers in `userland/libtock/timer.h`.  The
function you'll find useful today is:

    void delay_ms(uint32_t ms);

This function sleeps until the specified number of milliseconds have passed, and
then returns.

## 4. Write an app that periodically samples the on-board sensors

Now that we have the ability to write applications, let's do
something a little more complex. The development board you are using has several
sensors on it, [as shown here](https://github.com/helena-project/tock/blob/master/boards/hail/media/hail_reva_noheaders_labeled.png)
for the Hail board.
These sensors include a light sensor, a humidity and temperature sensor, and an
acceleration and magnetic field sensor (marked as accelerometer in the
picture). Each sensing medium can be accessed separately via the Tock user
library.

#### Light

The interface in `ambient_light.h` is used to measure ambient light conditions
in [lux](https://en.wikipedia.org/wiki/Lux). Specifically, it uses the sensor
[ISL29035](https://www.intersil.com/en/products/optoelectronics/ambient-light-sensors/light-to-digital-sensors/ISL29035.html).
It contains the function:

    int ambient_light_read_intensity(void);

#### Temperature

The interface in `temperature.h` is used to measure ambient temperature in degrees
Celsius, times 100. It uses the [SI7021](https://www.silabs.com/products/sensors/humidity-sensors/Pages/si7013-20-21.aspx)
sensor. It contains the function:

    int temperature_read_sync(int* temperature);

Note that the temperature reading is written to the location passed as an
argument, and the function returns non-zero in the case of an error.

#### Humidity

The interface in `humidity.h` is used to measure the ambient
[relative humidity](https://en.wikipedia.org/wiki/Relative_humidity) in
percent, times 100. It contains the function:

    int humidity_read_sync (unsigned* humi);

Again, this function returns non-zero in the case of an error.

#### Nindedof

The interface in `ninedof.h` is used to read acceleration or magnetic field
strength from the
[FXOS8700CQ](http://www.nxp.com/products/sensors/6-axis-sensors/digital-sensor-3d-accelerometer-2g-4g-8g-plus-3d-magnetometer:FXOS8700CQ).
(Note that Hail's hardware implementation of the Ninedof does not include the
traditional rotational sensor.)  It contains these functions:

    int ninedof_read_acceleration_sync(int* x, int* y, int* z);

The above reads acceleration in [g's](https://en.wikipedia.org/wiki/G-force) in
the x, y, and z orientations.

    int ninedof_read_magenetometer_sync(int* x, int* y, int* z);

The above reads magnetic field strength in
[microTeslas](https://en.wikipedia.org/wiki/Tesla_(unit)) in the x, y, and z
orientations.

### Read sensors in a Tock application

Using the example program you're working on, write an application that reads
all of the sensors on your development board and reports their readings over
the serial port. As a bonus, experiment with toggling an LED when readings are
above or below a certain threshold:

#### LED

The interface in `led.h` is used to control lights on Tock boards. On the Hail
board, there are three LEDs which can be controlled: Red, Blue, and Green. The
functions in the LED module are:

    int led_count(void);

Which returns the number of LEDs available on the board.

    int led_on(int led_num);

Which turns an LED on, accessed by its number.

    int led_off(int led_num);

Which turns an LED off, accessed by its number.

    int led_toggle(int led_num);

Which toggles the state of an LED, accessed by its number.


## 5. Extend your app to report through the `ble-env-sense` service

Finally, let's explore accessing the Bluetooth Low-Energy (BLE) capabilities of
the hardware. The Hail board has an
[nRF51822](https://www.nordicsemi.com/eng/Products/Bluetooth-low-energy/nRF51822)
radio which provides BLE communications. With that and the available sensors,
we can use Tock to provide the BLE
[Environmental Sensing Service](https://www.bluetooth.com/specifications/assigned-numbers/environmental-sensing-service-characteristics)
(ESS).

Currently, the Tock libraries for Rust do not support directly
interacting with the BLE radio. However, we can still access BLE by loading an
additional process on the board as a service and sending it commands over
Tock's inter-process communication (IPC) mechanism.

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

### Using the BLE ESS Service from a Rust application

Now that we've got the service loaded, we can extend the application
we've been working on to send environmental measurements over BLE.

#### IPC to the BLE ESS Service

The `ipc.h` interface can be used to send data to the BLE ESS service via
Tock's inter-process communication mechanism.  Details about how to do this
are [here](../../../userspace/examples/services/ble-env-sense/README.md), and example
code for sending BLE ESS updates is
[here](../../../userspace/examples/services/ble-env-sense/test/main.c).

Now you should be able to write an app that sends data over BLE.  You can load
your app alongside the service that's already loaded on the board, and they
will communicate via IPC.

To test that everything is working, you can connect to your development board
with a smartphone. We recommend the nRF Connect app
[[Android](https://play.google.com/store/apps/details?id=no.nordicsemi.android.mcp&hl=en)
 | [iOS](https://itunes.apple.com/us/app/nrf-connect/id1054362403?mt=8)].
The BLE address of the Hail is labeled on its bottom, however iOS devices
cannot access the address of a BLE device. However, you should be able to see
the unique name that you chose earlier.

