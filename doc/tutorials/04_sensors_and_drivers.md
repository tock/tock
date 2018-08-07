Reading Sensors From Scratch
============================

In this tutorial we will cover how to use the syscall interface from
applications to kernel drivers, and guide things based on reading the
[ISL29035](http://www.intersil.com/en/products/optoelectronics/ambient-light-sensors/light-to-digital-sensors/ISL29035.html)
digital light sensor and printing the readings over UART.

OK, lets get started.

1. **Setup a generic app for handling asynchronous events**. As with most
sensors, the ISL29035 is read asynchronously, and a callback is generated
from the kernel to userspace when the reading is ready. Therefore, to use
this sensor, our application needs to do two things: 1) setup a callback
the kernel driver can call when the reading is ready, and 2) instruct the
kernel driver to start the measurement. Lets first sketch this out:


    ```c
    #include <tock.h>

    #define DRIVER_NUM 0x60002

    // Callback when the ISL29035 has a light intensity measurement ready.
    static void isl29035_callback(int intensity, int unused1, int unused2, void* ud) {

    }

    int main() {
        // Tell the kernel about the callback.

        // Instruct the ISL29035 driver to begin a reading.

        // Wait until the reading is complete.

        // Print the resulting value.

        return 0;
    }
    ```

2. **Fill in the application with syscalls**. The standard
[Tock syscalls](../Syscalls.md) can be used to actually implement the sketch we
made above. We first use the `subscribe` syscall to inform the kernel about
the callback in our application. We then use the `command` syscall to start
the measurement. To wait, we use the `yield` call to wait for the callback to
actually fire. We do not need to use `allow` for this application, and typically
it is not required for reading sensors.

    For all syscalls that interact with drivers, the major number is set by
    the platform. In the case of the ISL29035, it is `0x60002`. The minor numbers
    are set by the driver and are specific to the particular driver.

    To save the value from the callback to use in the print statement, we will
    store it in a global variable.

    ```c
    #include <stdio.h>

    #include <tock.h>

    #define DRIVER_NUM 0x60002

    static int isl29035_reading;

    // Callback when the ISL29035 has a light intensity measurement ready.
    static void isl29035_callback(int intensity, int unused1, int unused2, void* ud) {
        // Save the reading when the callback fires.
        isl29035_reading = intensity;
    }

    int main() {
        // Tell the kernel about the callback.
        subscribe(DRIVER_NUM, 0, isl29035_callback, NULL);

        // Instruct the ISL29035 driver to begin a reading.
        command(DRIVER_NUM, 1, 0);

        // Wait until the reading is complete.
        yield();

        // Print the resulting value.
        printf("Light intensity reading: %d\n", isl29035_reading);

        return 0;
    }
    ```

3. **Be smarter about waiting for the callback**. While the above application
works, it's really relying on the fact that we are only sampling a single sensor.
In the current setup, if instead we had two sensors with outstanding commands,
the first callback that fired would trigger the `yield()` call to return
and then the `printf()` would execute. If, for example, sampling the ISL29035
takes 100 ms, and the new sensor only needs 10 ms, the new sensor's callback
would fire first and the `printf()` would execute with an incorrect value.

    To handle this, we can instead use the `yield_for()` call, which takes
    a flag and only returns when that flag has been set. We can then set this
    flag in the callback to make sure that our `printf()` only occurs when
    the light reading has completed.

    ```c
    #include <stdio.h>
    #include <stdbool.h>

    #include <tock.h>

    #define DRIVER_NUM 0x60002

    static int isl29035_reading;
    static bool isl29035_done = false;

    // Callback when the ISL29035 has a light intensity measurement ready.
    static void isl29035_callback(int intensity, int unused1, int unused2, void* ud) {
        // Save the reading when the callback fires.
        isl29035_reading = intensity;

        // Mark our flag true so that the `yield_for()` returns.
        isl29035_done = true;
    }

    int main() {
        // Tell the kernel about the callback.
        subscribe(DRIVER_NUM, 0, isl29035_callback, NULL);

        // Instruct the ISL29035 driver to begin a reading.
        command(DRIVER_NUM, 1, 0);

        // Wait until the reading is complete.
        yield_for(&isl29035_done);

        // Print the resulting value.
        printf("Light intensity reading: %d\n", isl29035_reading);

        return 0;
    }
    ```

4. **Use the `libtock` library functions**. Normally, applications don't
use the bare `command` and `subscribe` syscalls. Typically, these are wrapped
together into helpful commands inside of `libtock` and come with a function
that hides the `yield_for()` to a make a synchronous function which is useful
for developing applications quickly. Lets port the ISL29035 sensing app to use
the Tock Standard Library:

    ```c
    #include <stdio.h>

    #include <isl29035.h>

    int main() {
        // Take the ISL29035 measurement synchronously.
        int isl29035_reading = isl29035_read_light_intensity();

        // Print the resulting value.
        printf("Light intensity reading: %d\n", isl29035_reading);

        return 0;
    }
    ```

5. **Explore more sensors**. This tutorial highlights only one sensor. See the
   [sensors](https://github.com/tock/libtock-c/tree/master/examples/sensors) app
   for a more complete sensing application.
