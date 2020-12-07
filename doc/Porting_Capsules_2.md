Porting Tock Capsules to Tock 2.0
============

This guide covers how to port Tock capsules from the 1.x system call API
to the 2.x system call API. It outlines how the API has changed and
gives code examples.

<!-- npm i -g markdown-toc; markdown-toc -i Porting_Capsules_2.md -->

<!-- toc -->

<!-- tocstop -->

Overview
--------

Version 2 of the Tock operating system changes the system call API and
ABI in several ways. This document describes the changes and their
implications to capsule implementations. It gives guidance on how to
port a capsule from Tock 1.x to 2.0.

Tock 2.0 System Call API
-------------

The Tock system call API is implemented in the `Driver` trait. Tock
2.0 updates this trait to be more precise and correctly support Rust's
memory semantics. 

### `LegacyDriver`

The old version of the `Driver` trait has been renamed `LegacyDriver.`
When the scheduler dispatches system calls, it first checks if a given
capsule has an implementation of `Driver` or `LegacyDriver` and
dispatches accordingly. This means that all 1.x capsules are
supported with the old system call API. This document focuses on how
to update them to use the new `Driver` trait.

Whether a given capsule implements `Driver` or `Legacy` driver is
determined by the implementation of `with_driver` in a board's
`main.rs` file. In 1.x, this method returns an `Option` containing a
reference to `Driver`. It now returns an `Option` containing a
`Result<&dyn kernel::Driver, &dyn kernel::LegacyDriver>`. If the
method returns `Ok`, there is a reference to a 2.0 driver and the
scheduler invokes it. If it returns `Err`, there is a reference to a
1.x driver and the scheduler invokes it. Here is a snippet of an
implementation of this method on the `imix` board that shows what
this looks like:

```rust
impl kernel::Platform for Imix {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<Result<&dyn kernel::Driver, &dyn kernel::LegacyDriver>>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(Ok(self.console))),
            capsules::gpio::DRIVER_NUM => f(Some(Err(self.gpio)))
```

You can see that in this code, `self.console` implements the 2.0 `Driver` and
`self.gpio` implements the 1.x `LegacyDriver`.

### `Driver`

This is the signature for the 2.0 `Driver` trait:

```rust
pub trait Driver {
    fn subscribe(
        &self,
        which: usize,
        callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        Err((callback, ErrorCode::NOSUPPORT))
    }

    fn command(&self, which: usize, r2: usize, r3: usize, caller_id: AppId) -> CommandResult {
        CommandResult::failure(ErrorCode::NOSUPPORT)
    }

    fn allow_readwrite(
        &self,
        app: AppId,
        which: usize,
        slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        Err((slice, ErrorCode::NOSUPPORT))
    }

    fn allow_readonly(
        &self,
        app: AppId,
        which: usize,
        slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        Err((slice, ErrorCode::NOSUPPORT))
    }
}
```


The first thing to note is that there are now two versions of the old `allow`
method: one for a read/write buffer and one for a read-only buffer. They
pass different types of slices.

The second thing to note is that the three methods that pass pointers,
`allow_readwrite`, `allow_readonly`, and `subscribe`, return a `Result`.
The success case (`Ok`) returns a pointer back in the form of a `Callback`
or application slice. The failure case (`Err`) returns the same structure
back but also has an `ErrorCode`.

These three methods follow a swapping calling convention: you pass in
a pointer and get one back. If the call fails, you get back the one
you passed in. If the call succeeds, you get back the one the capsule
previously had. That is, you call `allow_readwrite` with an
application slice A and it succeeds, then the next successful call to
`allow_readwrite` will return A.

These swapping semantics allow the kernel to maintain an invariant
that there is only one instance of a particular application slice at
any time.  Since an application slice represents a region of
application memory, having two objects representing the same region of
memory violates Rust's memory guarantees. When the scheduler calls
`allow_readwrite`, `allow_readonly` or `subscribe`, it moves the
application slice or callback into the capsule. The capsule, in turn,
moves the previous one out.

The `command` method behaves differently, because commands only
operate on values, not pointers. Each command has its own arguments
number of return types. This is encapsulated within `CommandResult`.


Porting Capsules and Example Code
-------------------

The major change you'll see in porting your code is that capsule logic
becomes simpler: `Options` have been replaced by structures, and
there's a basic structure to swapping callbacks or application slices.


### Examples of command and `CommandResult`

The LED capsule implements only commands, so it provides a very simple
example of what commands look like.

```rust
 fn command(&self, command_num: usize, data: usize, _: usize, _: AppId) -> CommandResult {
        self.leds
            .map(|leds| {
                match command_num {
...				
                    // on
                    1 => {
                        if data >= leds.len() {
                            CommandResult::failure(ErrorCode::INVAL) /* led out of range */
                        } else {
                            leds[data].on();
                            CommandResult::success()
                        }
                    },

```

The capsule dispatches on the command number. It uses the first
argument, `data`, as which LED to turn activate. It then returns
either a `CommandResult::Success` (generated with
`CommandResult::success()`) or a `CommandResult::Failure` (generated
with `CommandResult::failure()`).

A `CommandResult` is a wrapper around a `GenericSyscallReturnValue`,
constraining it to the versions of `GenericSyscallReturnValue` that
can be returned by a command.

Here is a slightly more complex implementation of `command`, from the
`console` capsule.

```rust
    fn command(&self, cmd_num: usize, arg1: usize, _: usize, appid: AppId) -> CommandResult{
        let res = match cmd_num {
            0 => Ok(ReturnCode::SUCCESS),
            1 => { // putstr
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    self.send_new(appid, app, len)
                }).map_err(ErrorCode::from)
            },
            2 => { // getnstr
                let len = arg1;
                self.apps.enter(appid, |app, _| {
                    self.receive_new(appid, app, len)
                }).map_err(ErrorCode::from)
            },
            3 => { // Abort RX
                self.uart.receive_abort();
                Ok(ReturnCode::SUCCESS)
            }
            _ => Err(ErrorCode::NOSUPPORT)
        };
        match res {
            Ok(r) => {
                let res = ErrorCode::try_from(r);
                match res {
                    Err(_) =>  CommandResult::success(),
                    Ok(e) => CommandResult::failure(e)
                }
            },
            Err(e) => CommandResult::failure(e)
        }
    }
```

This implementation is more complex because it uses a grant region
that stores per-process state. `Grant::enter` returns a
`Result<ReturnCode, grant::Error>`. An `Err` return type means the
grant could not be entered successfully and the closure was not invoked:
this returns what grant error occured. An `Ok` return type means the
closure was executed, but it is possible that an error occured during
its execution. So there are three cases:

  - Ok(ReturnCode::Success) | Ok(ReturnCode::SuccessWithValue)
  - Ok(ReturnCode:: error cases)
  - Err(grant::Error)
  
The bottom `match` statement separates these two. In the `Ok()` case,
it checks whether the `ReturnCode` can be turned into an `ErrorCode`.
If not (`Err`), this means it was a success, and the result was a
success, so it returns a `CommandResult::Success`. If it can be convered
into an error code, or if the grant produced an error, it returns a
`CommandResult::Failure`. 

#### ReturnCode versus ErrorCode 

Because the new system call ABI explicitly distinguishes failures and
successes, it replaces `ReturnCode` with `ErrorCode` to denote which
error in failure cases. `ErrorCode` is simply `ReturnCode` without any
success cases, and with names that remove the leading E since it's
obvious they are an error: `ErrorCode::FAIL` is the equivalent of
`ReturnCode::EFAIL`.

### Examples of `allow_readwrite` and `allow_readonly`

Because `ReadWriteAppSlice` and `ReadOnlyAppSlice` represent access to
userspace memory, the kernel tightly constrains how these objects
are constructed and passed. They do not implement `Copy` or `Clone`, 
so only one instance of these objects exists in the kernel at any 
time. 

Note that `console` has one `ReadOnlyAppSlice` for printing/`putnstr`
and and one `ReadWriteAppSlice` for reading/`getnstr`.  Here is a
sample implementation of `allow_readwrite` for the `console` capsule:

```rust
pub struct App {
    write_buffer: ReadOnlyAppSlice,
...
	fn allow_readonly(
        &self,
        appid: AppId,
        allow_num: usize,
        mut slice: ReadOnlyAppSlice,
    ) -> Result<ReadOnlyAppSlice, (ReadOnlyAppSlice, ErrorCode)> {
        let res = match allow_num {
            1 => self
                .apps
                .enter(appid, |app, _| {
                    mem::swap(&mut app.write_buffer, &mut slice);
                })
                .map_err(ErrorCode::from),
            _ => Err(ErrorCode::NOSUPPORT),
        };

        if let Err(e) = res {
            Err((slice, e))
        } else {
            Ok(slice)
        }
    }
```


The implemention is quite simple: if there is a valid grant region, the
method swaps the passed `ReadOnlyAppSlice` and the one in the `App` region,
returning the one that was in the app region. It then returns `slice`,
which is either the passed slice or the swapped out one.

### Example of `subscribe`

A call to `subscribe` has a similar structure to `allow`. Here is
an example from `console`:

```rust
    fn subscribe(
        &self,
        subscribe_num: usize, 
        mut callback: Callback,
        app_id: AppId,
    ) -> Result<Callback, (Callback, ErrorCode)> {
        let res = match subscribe_num {
            1 => { // putstr/write done
                self
                .apps
                .enter(app_id, |app, _| {
                    mem::swap(&mut app.write_callback, &mut callback);
                })
               .map_err(ErrorCode::from)
            },
            2 => { // getnstr/read done
                self
                .apps
                .enter(app_id, |app, _| {
                     mem::swap(&mut app.read_callback, &mut callback);
                }).map_err(ErrorCode::from)
            },
            _ => Err(ErrorCode::NOSUPPORT)
        };

        if let Err(e) = res {
            Err((callback, e))
        } else {
            Ok(callback)
        }
    }
```

Note that `subscribe` now takes a `Callback` instead of an `Option<Callback>`.
The `Callback` structure is a wrapper around an `Option<ProcessCallback>`,
which holds actual callback information. The Null Callback is represented
as a `Callback` where the `Option` is `None`. This is then encapsulated
within the call to `Callback::schedule`, where the Null Callback does
nothing.

### Using `ReadOnlyAppSlice` and `ReadWriteAppSlice`

One key change in the Tock 2.0 API is explicitly acknowledging that
application slices may disappear at any time. For example, if a process
passes a slice into the kernel, it can later swap it out with a later
allow call. Similarly, application grants may disappear at any time.

This means that `ReadWriteAppSlice` and `ReadOnlyAppSlice` now
do not allow you to obtain their pointers and lengths. Instead,
they provide a `map_or` method. This is how `console` uses this,
for example, to copy process data into its write buffer and
call the underlying `transmit_buffer`:

```rust
  fn send(&self, app_id: AppId, app: &mut App) {
        if self.tx_in_progress.is_none() {
            self.tx_in_progress.set(app_id);
            self.tx_buffer.take().map(|buffer| {
                let len = app.write_buffer.map_or(0, |data| data.len());
                if app.write_remaining > len {
                    // A slice has changed under us and is now smaller than
                    // what we need to write -- just write what we can.
                    app.write_remaining = len;
                }
				let transaction_len = app.write_buffer.map_or(0, |data| {
                    for (i, c) in data[data.len() - app.write_remaining..data.len()]
                        .iter()
                        .enumerate()
                    {
                        if buffer.len() <= i {
                            return i;
                        }
                        buffer[i] = *c;
                    }
                    app.write_remaining
                });

                app.write_remaining -= transaction_len;
                let (_err, _opt) = self.uart.transmit_buffer(buffer, transaction_len);
            });
        } else {
            app.pending_write = true;
        }
    }
```

Note that the implementation looks at the length of the slice: it doesn't copy
it out into grant state. If a slice was suddenly truncated, it checks and
adjust the amount it has written.

