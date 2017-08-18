### Tock OS Course Part 2: Adding a New Capsule to the Kernel

The goal of this part of the course is to make you comfortable with the
Tock kernel and writing code for it. By the end of this part, you'll have
written a new capsule that reads a 9DOF (nine degrees of freedom, consisting
of a 3-axis accelerometer, magnetometer, and gyroscope) sensor and outputs
its readings over the serial port.

During this you will:

1. Learn how Tock uses Rust's memory safety to provide isolation for free
2. Read the Tock boot sequence, seeing how Tock uses static allocation
3. Learn about Tock's event-driven programming
4. Write a new capsule that reads a 9DOF sensor and prints it over serial

#### 1. Listen to presentation on Tock's kernel and capsules (20 min)

This part of the course will start with a member of the Tock development
team presenting its core software architecture. This will explain how a
Tock platform has a small amount of trusted (can use `unsafe`) code, but
the bulk of the kernel code is in *capsules*, which cannot violate Rust's
safety guarantees. It'll also explain how RAM constraints lead the Tock
kernel to rely on static allocation and use a purely event-driven execution
model.

This presentation will give you the intellectual framework to understand
why capsules work as they do, and understand what you'll be doing in the rest
of this part of the course.

#### 2. Check your understanding (10 min)

1. What is a `VolatileCell`? Can you find some uses of `VolatileCell`, and do you understand why they are needed? Hint: look inside `chips/sam4l/src`.
2. What is a `TakeCell`? When is a `TakeCell` preferable to a standard `Cell`?

#### 3. Read the Tock boot sequence (20m)

Open `boards/hail/src/main.rs` in your favorite editor. This file defines the
Hail platform: how it boots, what capsules it uses, and what system calls it
supports for userland applications.

Find `struct Hail`. This declares the structure representing the
platform: it has many fields, all of which are capsules, except `ipc`
for inter-process procedure calls (IPC). You'll notice two
things. First, every field is a reference to an object with a static
lifetime. Second, all of the capsules take a lifetime as a parameter
and this lifetime is `` `static``.  This reflects the fact that the
Tock kernel doesn't have a dynamic memory pool: all of its RAM state
is statically allocated. The implementations of these capsules,
however, do not rely on this assumption.

The method `reset_handler` is invoked when the chip resets (i.e., boots).
It's pretty long because Hail has a lot of drivers that need to be created
and initialized, and many of them depend on other, lower layer abstractions
that need to be created and initialized as well. Take a look at the first
few lines. You'll see that the boot sequence
initializes memory (copies initialized variables into RAM, clears the BSS),
sets up the system clocks, and configures the GPIO pins.

Next, it initializes the system console, which is what turns calls to `print!`
into bytes sent to the USB serial port:


```rust
let console = static_init!(
    Console<usart::USART>,
    Console::new(&usart::USART0,
                 115200,
                 &mut console::WRITE_BUF,
                 kernel::Container::create()));
hil::uart::UART::set_client(&usart::USART0, console);
```

> ##### A brief aside on `console::WRITE_BUF`
>
> It's a little weird that Console's `new` method takes in a reference to
> itself. This is an ergonomics tradeoff. The Console needs a mutable static
> buffer to use internally, which the Console capsule declares. However writing
> global statics is unsafe. To avoid the unsafe operation in the Console
> capsule itself, we make it the responsibility of the instantiator to give the
> Console a buffer to use, without burdening the instantiator with sizing the
> buffer.

You're going to use this capsule to output data from the 9DOF sensor,
so it's a useful example to see how you instantiate and initialize capsules.
The `static_init!` macro is simply an easy way to allocate a static
variable with a call to `new`. The first parameter is the type, the second
is the expression to produce an instance of the type. This call creates
a `Console` that uses serial port 0 (`USART0`) at 115200 bits per second.

Notice that you have to pass a write buffer to the console for it to use:
this buffer has to have a `` `static`` lifetime. This is because low-level
hardware drivers, especially those that use DMA, require `` `static`` buffers.
Since Tock doesn't promise when a DMA operation will complete, and you
need to be able to promise that the buffer outlives the operation, the
one lifetime that is assured to be alive at the end of an operation is
`` `static``. So that other code which has buffers
without a `` `static`` lifetime, such as userspace processes, can use the
`Console`, it copies them into its own internal `` `static`` buffer before
passing it to the serial port. So the buffer passing architecture looks like
this:

![Console/UART buffer lifetimes](console.png)

The final parameter, the `Container`, is for handling system calls:
you don't need to worry about it for now.

Next, jump to around line 360, where a `Hail` structure is allocated
(`let hail = Hail {`). Note
that its `console` field is initialized to the `console` capsule that
was allocated in the code above. 20-30 lines later, you'll see the console
is initialized,

```rust
hail.console.initialize();
```


which configures the serial port to be as text consoles
expect (8 data bits, 1 stop bit, no parity bit, no hardware flow control).
Next -- and this is the key part! -- the kernel's debug interface is
connected to the `Console`. Now, kernel debug messages will be printed
on the serial port! Userspace processes can also print messages to the
`Console`, which handles interleaving them correctly.

If you jump down just a few more lines, around line 400, you'll see the
code that loads userspace processes from flash, then starts the kernel
main loop:

```rust
kernel::process::load_processes(&_sapps as *const u8,
                                &mut APP_MEMORY,
                                &mut PROCESSES,
                                FAULT_RESPONSE);
kernel::main(&hail, &mut chip, &mut PROCESSES, &hail.ipc);
```

#### 4. Check your understanding (10 min)

Take a look at the implementation of the `debug!` macro in
`kernel/src/debug.rs`. Note that it has an output buffer of size
`BUF_SIZE` (`debug.rs:29`). When the kernel calls `debug!`, does
the macro return when the message has been written to the serial
port (synchronous), or does it return and asynchonrously write
out the debug message? Hint: the call to `command` on line 123
is what starts the write operation, resulting in the `callback` on
line 130.

#### 5. Create a "Hello World" capsule (20m)

Now that you've seen how Tock initializes and uses capsules, you're going to
write a new one. At the end of this section, your capsule will sample the
accelerometer from the 9dof sensor once a second and printing the results as
serial output. But you'll start with something simpler: printing "Hello World"
to the debug console once on boot.

To begin, because you're going to be modifying the boot sequence of Hail,
make a branch of the Tock repository. This will keep your master
branch clean.

```bash
$ git checkout -b rustconf
```

Next, create a new module in `boards/hail/src` and import it from
`boards/hail/src/main.rs`. In your new module, make a new `struct` for your
capsule (e.g. called `Acclerate`), a `new` function to construct it and a `start` method.

Eventually, the `start` method will kick off the state machine for periodic
accelerometer readings, but for now, you'll just print "Hello World" to the
debug console and return:

```rust
debug!("Hello World");
```

Finally, initialize this new capsule in the `main.rs` boot sequence. You'll
want to use the `static_init!` macro to makesure it's initialized in static
memory. `static_init!` is already imported, and has the following signature:

```rust
static_init!($T:ty, :expr -> $T) -> &'static T
```

That is, the first parameter is the type of the thing you want to allocate and
the second parameter is an expression constructing it. The result is a
reference to the constructed value with `'static` lifetime.

Compile and program your new kernel:

```bash
$ make program
$ tockloader listen
No device name specified. Using default "tock"                                                                         Using "/dev/ttyUSB0 - Hail IoT Module - TockOS"
Listening for serial output.
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:18: Hello World
```

[Sample Solution](https://gist.github.com/alevy/56b0566e2d1a6ba582b7d4c09968ddc9)

#### 6. Extend your capsule to print "Hello World" every second (35m)

In order for your capsule to keep track of time, it will need to depend on
another capsule that implements the Alarm interface. We'll have to do something
similar for reading the accelerometer, so this is good practice.

The Alarm HIL includes several traits, `Alarm`, `AlarmClient` and `Frequency`,
all in the `kernel::hil::time` module. You'll use the `set\_alarm` and `now`
methods from the `Alarm` trait to set an alarm for a particular value of the
clock. The `Alarm` trait also has an associated type that implements the
`Frequency` trait which lets us call its `frequency` method to get the clock
frequency.

Modify your capsule to have a field of the type `&'a Alarm` and to accept an
`&'a Alarm` in the `new` function.

Your capsule will also need to implement the `AlarmClient` trait so it can
recieve alarm events. The `AlarmClient` trait has a single method:

```rust
fn fired(&self)
```

Your capsule should now set an alarm in the `start` method, print the debug
message and set an alarm again when the alarm fires.

Finally, you'll need to modify the capsule initialization to pass in an alarm
implementation. Since lots of other capsules use the alarm, you should use a
virtual alarm. You can make a new one like this:

```rust
let my_virtual_alarm = static_init!(
    VirtualMuxAlarm<'static, sam4l::ast::Ast>,
    VirtualMuxAlarm::new(mux_alarm));
```

and you have to make sure to set your capsule as the client of the virtual alarm after initializing it:

```rust
my_virtual_alarm.set_client(my_capsule);
```

Compile and program your new kernel:

```bash
$ make program
$ tockloader listen
No device name specified. Using default "tock"                                                                         Using "/dev/ttyUSB0 - Hail IoT Module - TockOS"
Listening for serial output.
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:31: Hello World
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:31: Hello World
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:31: Hello World
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:31: Hello World
```

[Sample Solution](https://gist.github.com/alevy/73fca7b0dddcb5449088cebcbfc035f1)

#### 7. Extend your capsule to sample the accelerometer once a second (35m)

The steps for reading an accelerometer from your capsule are similar to using
the alarm. You'll use a capsule that implements the NineDof (nine degrees of
freedom) HIL, which includes the `NineDof` and `NineDofClient` traits, both in
`kernel::hil::sensors`.

The `NineDof` trait includes the method `read_accelerometer` which initiates an
accelerometer reading. The `NineDofClient` trait has a single method for receiving readings:

```rust
fn callback(&self, x: usize, y: usize, z: usize);
```

However, unlike the alarm, there is no virtualization layer for the `NineDof`
HIL (yet!) and there is already a driver used in Hail that exposes the 9dof
sensor to userland. Fortunately, we can just remove it for our purposes.

Remove the `ninedof` field from the `Hail` struct definition:

```rust
ninedof: &'static capsules::ninedof::NineDof<'static>,
```

and initialization:

```rust
ninedof: ninedof,
```

Remove `ninedof` from the system call lookup table in `with_driver`:

```rust
11 => f(Some(self.ninedof)), // Comment this out
```

And, finally, remove the initialization of `ninedof` from the boot sequence:

```rust
let ninedof = static_init!(
    capsules::ninedof::NineDof<'static>,
    capsules::ninedof::NineDof::new(fxos8700, kernel::Container::create()));
hil::ninedof::NineDof::set_client(fxos8700, ninedof);
```

Follow the same steps you did for adding an alarm to your capsule for the 9dof
sensor capsule:

  1. Add a `&'a NineDof` field.

  2. Accept one in the `new` function.

  3. Implement the `NineDofClient` trait.

Now, modify the Hail boot sequence passing in the `fxos8700` capsule, which
implements the `NineDof` trait, to your capsule (it should already be
initialized). Make sure to set your capsule as its client:

```rust
{
  use hil::ninedof::NineDof
  fxos8700.set_client(my_capsule);
}
```

Finally, implement logic to initiate a accelerometer reading every second and
report the results.

![Structure of `rustconf` capsule](rustconf.png)

Compile and program your kernel:

```bash
$ make program
$ tockloader listen
No device name specified. Using default "tock"                                                                         Using "/dev/ttyUSB0 - Hail IoT Module - TockOS"
Listening for serial output.
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:31: 982 33 166
TOCK_DEBUG(0): /home/alevy/hack/helena/rustconf/tock/boards/hail/src/accelerate.rs:31: 988 31 158
```

[Sample solution](https://gist.github.com/alevy/798d11dbfa5409e0aa56d870b4b7afcf)

#### 8. Extra credit! Virtualize the 9dof capsule (∞)

#### 9. Some further questions and directions to explore (20m)

Your `rustconf` capsule used the fxos8700 and virtual alarm. Take a look at the
code behind each of these services:

1. Is the 9DOF sensor on-chip or a separate chip connected over a bus?

2. What happens if you request two 9DOF sensors (e.g., acceleration and gyro)

   back-to-back?
3. Is there a limit on how many virtual alarms can be created?

4. How many virtual alarms does the Hail boot sequence create?

