Friendly Apps Share Data
========================

This tutorial covers how to use Tock's IPC mechanism to allow applications
to communicate and share memory.


Tock IPC Basics
---------------

IPC in Tock uses a client-server model. Applications can provide a service by
telling the Tock kernel that they provide a service. Each application can only
provide a single service, and that service's name is set to the name of the
application. Other applications can then discover that service and explicitly
share a buffer with the server. Once a client shares a buffer, it can then
notify the server to instruct the server to somehow interact with the shared
buffer. The protocol for what the server should do with the buffer is service
specific and not specified by Tock. Servers can also notify clients, but when
and why servers notify clients is service specific.

Example Application
-------------------

To provide an overview of IPC, we will build an example system consisting of
three apps: a random number service, a LED control service, and a main
application that uses the two services. While simple, this example both
demonstrates the core aspects of the IPC mechanism and should run on any
hardware platform.

### LED Service

Lets start with the LED service. The goal of this service is to allow other
applications to use the shared buffer as a command message to instruct the
LED service on how to turn on or off the system's LEDs.

1. We must tell the kernel that our app wishes to provide a service. All that
we have to do is call `ipc_register_svc()`.

    ```c
    #include "ipc.h"

    int main(void) {
      ipc_register_svc(ipc_callback, NULL);
      return 0;
    }
    ```

2. We also need that callback (`ipc_callback`) to handle IPC requests from
other applications. This callback will be called when the client app notifies
the service.

    ```c
    static void ipc_callback(int pid, int len, int buf, void* ud) {
      // pid: An identifier for the app that notified us.
      // len: How long the buffer is that the client shared with us.
      // buf: Pointer to the shared buffer.
    }
    ```

3. Now lets fill in the callback for the LED application. This is a simplified
version for illustration. The full example can be found in the
`examples/tutorials` folder.

    ```c
    #include "led.h"

    static void ipc_callback(int pid, int len, int buf, void* ud) {
      uint8_t* buffer = (uint8_t*) buf;

      // First byte is the command, second byte is the LED index to set,
      // and the third byte is whether the LED should be on or off.
      uint8_t command = buffer[0];
      if (command == 1) {
          uint8_t led_id = buffer[1];
          uint8_t led_state = buffer[2] > 0;

          if (led_state == 0) {
            led_off(led_id);
          } else {
            led_on(led_id);
          }

          // Tell the client that we have finished setting the specified LED.
          ipc_notify_client(pid);
          break;
      }
    }
    ```


### RNG Service

The RNG service returns the requested number of random bytes in the shared
folder.

1. Again, register that this service exists.

    ```c
    int main(void) {
      ipc_register_svc(ipc_callback, NULL);
      return 0;
    }
    ```

2. Also need a callback function for when the client signals the service.
The client specifies how many random bytes it wants by setting the first byte
of the shared buffer before calling notify.

    ```c
    #include <rng.h>

    static void ipc_callback(int pid, int len, int buf, void* ud) {
      uint8_t* buffer = (uint8_t*) buf;
      uint8_t rng[len];

      uint8_t number_of_bytes = buffer[0];

      // Fill the buffer with random bytes.
      int number_of_bytes_received = rng_sync(rng, len, number_of_bytes);
      memcpy(buffer, rng, number_of_bytes_received);

      // Signal the client that we have the number of random bytes requested.
      ipc_notify_client(pid);
    }
    ```

    This is again not a complete example but illustrates the key aspects.


### Main Logic Client Application

The third application uses the two services to randomly control the LEDs on
the board. This application is not a server but instead is a client of the
two service applications.

1. When using an IPC service, the first step is to discover the service and
record its identifier. This will allow the application to share memory with it
and notify it. Services are discovered by the name of the application that
provides them. Currently these are set in the application Makefile or by default
based on the folder name of the application. The examples in Tock commonly
use a Java style naming format.

    ```c
    int main(void) {
      int led_service = ipc_discover("org.tockos.tutorials.ipc.led");
      int rng_service = ipc_discover("org.tockos.tutorials.ipc.rng");

      return 0;
    }
    ```

    If the services requested are valid and exist the return value from
    ` ipc_discover` is the identifier of the found service. If the service
    cannot be found an error is returned.

2. Next we must share a buffer with each service (the buffer is the only way to
share between processes), and setup a callback that is called when the server
notifies us as a client. Once shared, the kernel will permit both applications
to read/modify that memory.

    ```c
    char led_buf[64] __attribute__((aligned(64)));
    char rng_buf[64] __attribute__((aligned(64)));

    int main(void) {
      int led_service = ipc_discover("org.tockos.tutorials.ipc.led");
      int rng_service = ipc_discover("org.tockos.tutorials.ipc.rng");

      // Setup IPC for LED service
      ipc_register_client_cb(led_service, ipc_callback, NULL);
      ipc_share(led_service, led_buf, 64);

      // Setup IPC for RNG service
      ipc_register_client_cb(rng_service, ipc_callback, NULL);
      ipc_share(rng_service, rng_buf, 64);

      return 0;
    }
    ```

3. We of course need the callback too. For this app we use the `yield_for`
function to implement the logical synchronously, so all the callback needs
to do is set a flag to signal the end of the `yield_for`.

    ```c
    bool done = false;

    static void ipc_callback(int pid, int len, int arg2, void* ud) {
      done = true;
    }
    ```

3. Now we use the two services to implement our application.

    ```c
    #include <timer.h>

    void app() {
      while (1) {
        // Get two random bytes from the RNG service
        done = false;
        rng_buf[0] = 2; // Request two bytes.
        ipc_notify_svc(rng_service);
        yield_for(&done);

        // Control the LEDs based on those two bytes.
        done = false;
        led_buf[0] = 1;                     // Control LED command.
        led_buf[1] = rng_buf[0] % NUM_LEDS; // Choose the LED index.
        led_buf[2] = rng_buf[1] & 0x01;     // On or off.
        ipc_notify_svc(led_service);        // Notify to signal LED service.
        yield_for(&done);

        delay_ms(500);
      }
    }
    ```

Try It Out
----------

To test this out, see the complete apps in the [IPC tutorial
example](https://github.com/tock/libtock-c/tree/master/examples/tutorials/05_ipc)
folder.

To install all of the apps on a board:

    $ cd examples/tutorials/05_ipc
    $ tockloader erase-apps
    $ pushd led && make && tockloader install && popd
    $ pushd rng && make && tockloader install && popd
    $ pushd logic && make && tockloader install && popd

You should see the LEDs randomly turning on and off!
