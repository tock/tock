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

First, lets look at the application code.

% we're probably going to have to have them clone libtock-rs too, so they can look at it
% the escape would be if we can document what the possibilities are well enough that they don't have to
% that would be good
% so here we should explain console, write!, and delay_ms

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
Starting 0
Starting 1
Starting 2
Starting 3
```

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

