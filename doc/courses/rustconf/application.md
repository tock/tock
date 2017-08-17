# Write an environment sensing Bluetooth Low Energy application

## 1. Presentation: Process overview, relocation model and system call API (10 min)

## 2. Check your understanding (5 min)

1. How does a process perform a blocking operation? Can you draw the flow of
   operations when a process calls `delay_ms(1000)`?

2. What is a Grant? How do processes interact with grants? Hint: Think about
   memory exhaustion.

## 3. Get a Rust application running on Hail (10 min)

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

`Console` is used to send messages over the USB connection on a Hail
(technically it sends serial data through a UART to and FTDI UART-to-USB chip,
but same difference). Its functions are:

         pub fn new() -> Console

   Creates and returns a new Console struct.

         pub fn write(&mut self, string: String)

   Writes a string object to the Console.

`Console` also implements `fmt::write`, which enables the `write!` macro to
work. We recommend using
[`write!`](https://doc.rust-lang.org/1.5.0/std/macro.write!.html) for this
tutorial, as it allows you to use [format
options](https://doc.rust-lang.org/1.5.0/std/fmt/) while printing.

#### Timer

`Timer` is used to trigger events at a specific number of seconds in the
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


## 4. Write an app that periodically samples the on-board sensors (20 min)

% here we need to explain which sensors exist (they probably already know)
% how to initialize the sensors
% and what the possible calls are to each of them

## 5. Extend your app to report through the `ble-env-sense` service (15 min)

% here we need to explain the ble-ess application and load it
% and then adjust the layout.ld file (sigh)

% we then need to explain what the possible IPC calls are
% and what the possible BLE IPC calls are
% showing how those work on the C side might not be a bad idea...

