Say "Hello!" On Every Button Press
==================================

This tutorial will walk you through calling `printf()` in response to a
button press.

1. **Start a new application**. A Tock application in C looks like a typical
C application. Lets start with the basics:

    ```c
    #include <stdio.h>

    int main(void) {
      return 0;
    }
    ```

    You also need a makefile. Copying a makefile from an existing app is
    the easiest way to get started.

2. **Setup a button callback handler**. A button press in Tock is treated
as an interrupt, and in an application this translates to a function being
called, much like in any other event-driven system. To listen for button
presses, we first need to define a callback function, then tell the kernel
that the callback exists.

    ```c
    #include <stdio.h>
    #include <button.h>

    // Callback for button presses.
    //   btn_num: The index of the button associated with the callback
    //   val:     1 if pressed, 0 if depressed
    static void button_callback(int btn_num,
                                int val,
                                int arg2 __attribute__ ((unused)),
                                void *user_data __attribute__ ((unused)) ) {
    }

    int main(void) {
      button_subscribe(button_callback, NULL);

      return 0;
    }
    ```
    All callbacks from the kernel are passed four arguments, and the meaning of
    the four arguments depends on the driver. The first three are integers,
    and can be used to represent buffer lengths, pin numbers, button numbers,
    and other simple data. The fourth argument is a pointer to user defined
    object. This pointer is set in the subscribe call (in this example
    it is set to `NULL`), and returned when the callback fires.

3. **Enable the button interrupts**. By default, the interrupts for the
buttons are not enabled. To enable them, we make a syscall. Buttons, like
other drivers in Tock, follow the convention that applications can ask the
kernel how many there are. This is done by calling `button_count()`.

    ```c
    #include <stdio.h>
    #include <button.h>

    // Callback for button presses.
    //   btn_num: The index of the button associated with the callback
    //   val:     1 if pressed, 0 if depressed
    static void button_callback(int btn_num,
                                int val,
                                int arg2 __attribute__ ((unused)),
                                void *user_data __attribute__ ((unused)) ) {
    }

    int main(void) {
      button_subscribe(button_callback, NULL);

      // Ensure there is a button to use.
      int count = button_count();
      if (count < 1) {
        // There are no buttons on this platform.
        printf("Error! No buttons on this platform.");
      } else {
        // Enable an interrupt on the first button.
        button_enable_interrupt(0);
      }

      // Can just return here. The application will continue to execute.
      return 0;
    }
    ```

    The button count is checked, and the app only continues if there
    exists at least one button. To enable the button interrupt,
    `button_enable_interrupt()` is called with the index of the button
    to use. In this example we just use the first button.

4. **Call `printf()` on button press**. To print a message, we call
`printf()` in the callback.

    ```c
    #include <stdio.h>
    #include <button.h>

    // Callback for button presses.
    //   btn_num: The index of the button associated with the callback
    //   val:     1 if pressed, 0 if depressed
    static void button_callback(int btn_num,
                                int val,
                                int arg2 __attribute__ ((unused)),
                                void *user_data __attribute__ ((unused)) ) {
      // Only print on the down press.
      if (val == 1) {
        printf("Hello!\n");
      }
    }

    int main(void) {
      button_subscribe(button_callback, NULL);

      // Ensure there is a button to use.
      int count = button_count();
      if (count < 1) {
        // There are no buttons on this platform.
        printf("Error! No buttons on this platform.\n");
      } else {
        // Enable an interrupt on the first button.
        button_enable_interrupt(0);
      }

      // Can just return here. The application will continue to execute.
      return 0;
    }
    ```

5. **Run the application**. To try this tutorial application, you can find it in
   the [tutorials app
   folder](https://github.com/tock/libtock-c/tree/master/examples/tutorials/02_button_print).
   See the first tutorial for details on how to compile and install a C
   application.

    Once installed, when you press the button, you should see "Hello!" printed
    to the terminal!
