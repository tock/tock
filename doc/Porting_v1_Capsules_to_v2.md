Porting Tock 1.x Capsules to Tock 2.0
============

This guide covers how to port Tock capsules from the 1.x system call API
to the 2.x system call API. It outlines how the API has changed and
gives code examples.

<!-- npm i -g markdown-toc; markdown-toc -i Porting_Capsules_2.md -->

<!-- toc -->

- [Overview](#overview)
- [Tock 2.0 System Call API](#tock-20-system-call-api)
  * [`Driver`](#driver)
- [Porting Capsules and Example Code](#porting-capsules-and-example-code)
  * [Examples of command and `CommandResult`](#examples-of-command-and-commandresult)
    + [ReturnCode versus ErrorCode](#returncode-versus-errorcode)
  * [Examples of `allow_readwrite` and `allow_readonly`](#examples-of-allow_readwrite-and-allow_readonly)
  * [The new subscription mechanism](#the-new-subscription-mechanism)
  * [Using `ReadOnlyAppSlice` and `ReadWriteAppSlice`: `console`](#using-readonlyappslice-and-readwriteappslice-console)
  * [Using `ReadOnlyAppSlice` and `ReadWriteAppSlice`: `spi_controller`](#using-readonlyappslice-and-readwriteappslice-spi_controller)

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

### `Driver`

This is the signature for the 2.0 `Driver` trait:

```rust
pub trait Driver {
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

    fn allocate_grant(&self, appid: ProcessId) -> Result<(), crate::process::Error>;
}
```


The first thing to note is that there are now two versions of the old `allow`
method: one for a read/write buffer and one for a read-only buffer. They
pass different types of slices.

The second thing to note is that the two methods that pass pointers,
`allow_readwrite` and `allow_readonly`, return a `Result`.
The success case (`Ok`) returns a pointer back in the form of
an application slice. The failure case (`Err`) returns the same structure
back but also has an `ErrorCode`.

These two methods follow a swapping calling convention: you pass in
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
operate on values, not pointers. Each command has its own arguments and
number of return types. This is encapsulated within `CommandResult`.

The third thing to note is that there is no longer a `subscribe()` method. This
has been removed and instead all upcalls are managed entirely by the kernel.
Scheduling an upcall is now done with a provided object from entering a grant.

The fourth thing to note is the new `allocate_grant()` method. This allows the
kernel to request that a capsule enters its grant region so that it is allocated
for the specific process. This should be implemented with a roughly boilerplate
implementation [described below](#the-new-subscription-mechanism).

Porting Capsules and Example Code
-------------------

The major change you'll see in porting your code is that capsule logic
becomes simpler: `Options` have been replaced by structures, and
there's a basic structure to swapping application slices.


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
            0 => Ok(Ok(())),
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
                Ok(Ok(()))
            }
            _ => Err(ErrorCode::NOSUPPORT)
        };
        match res {
            Ok(r) => {
                CommandResult::from(r),
            },
            Err(e) => CommandResult::failure(e)
        }
    }
```

This implementation is more complex because it uses a grant region
that stores per-process state. `Grant::enter` returns a
`Result<Result<(), ErrorCode>, grant::Error>`. An outer `Err` return type means the
grant could not be entered successfully and the closure was not invoked:
this returns what grant error occurred. An `Ok` return type means the
closure was executed, but it is possible that an error occurred during
its execution. So there are three cases:

  - Ok(Ok(()))
  - Ok(Err(ErrorCode:: error cases))
  - Err(grant::Error)

The bottom `match` statement separates these two. In the `Ok()` case,
it checks whether the inner Result contains an `Err(ErrorCode)`.
If not (`Err`), this means it was a success, and the result was a
success, so it returns a `CommandResult::Success`. If it can be converted
into an error code, or if the grant produced an error, it returns a
`CommandResult::Failure`.

One of the requirements of commands in 2.0 is that each individual
`command_num` have a single failure return type and a single success
return size. This means that for a given `command_num`, it is not allowed
for it to sometimes return `CommandResult::Success` and other times return
`Command::SuccessWithValue`, as these are different sizes. As part of easing
this transition, Tock 2.0 removed the `SuccessWithValue` variant of
`ReturnCode`, and then later in the transition removed `ReturnCode` entirely,
replacing all uses of `ReturnCode` with `Result<(), ErrorCode>`.

If, while porting, you encounter a construction of `ReturnCode::SuccessWithValue{v}`
in `command()` for an out-of-tree capsule, replace it with a construction of
`CommandResult::success_u32(v)`, and make sure that it is impossible for that
command_num to return `CommandResult::Success` in any other scenario.

#### ReturnCode versus ErrorCode 

Because the new system call ABI explicitly distinguishes failures and
successes, it replaces `ReturnCode` with `ErrorCode` to denote which
error in failure cases. `ErrorCode` is simply `ReturnCode` without any
success cases, and with names that remove the leading E since it's
obvious they are an error: `ErrorCode::FAIL` is the equivalent of
`ReturnCode::EFAIL`. `ReturnCode` is still used in the kernel,
but may be deprecated in time.

### Examples of `allow_readwrite` and `allow_readonly`

Because `ReadWriteAppSlice` and `ReadOnlyAppSlice` represent access to
userspace memory, the kernel tightly constrains how these objects
are constructed and passed. They do not implement `Copy` or `Clone`,
so only one instance of these objects exists in the kernel at any
time.

Note that `console` has one `ReadOnlyAppSlice` for printing/`putnstr`
and one `ReadWriteAppSlice` for reading/`getnstr`.  Here is a
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


The implementation is quite simple: if there is a valid grant region, the
method swaps the passed `ReadOnlyAppSlice` and the one in the `App` region,
returning the one that was in the app region. It then returns `slice`,
which is either the passed slice or the swapped out one.

### The new subscription mechanism

Tock 2.0 introduces a guarantee for the subscribe syscall that for every unique
subscribe (i.e. `(driver_num, subscribe_num)` tuple), userspace will be returned
the previously subscribe upcall (or null if this is the first subscription).
This guarantee means that once an upcall is returned, the kernel will never
schedule the upcall again (unless it is re-subscribed in the future), and
userspace can deallocate the upcall function if it so chooses.

Providing this guarantee necessitates changes to the capsule interface for
declaring and using upcalls. To declare upcalls, a capsule now provides the
number of upcalls as a templated value on `Grant`.

```rust
struct capsule {
    ...
    apps: Grant<T, NUM_UPCALLS>,
    ...
}
```

The second parameter tells the kernel how many upcalls to save. Capsules no
longer can store an object of type `Upcall` in their grant region.

To ensure that the kernel can store the upcalls, a capsule must implement the
`allocate_grant()` method. The typical implementation looks like:

```rust
fn allocate_grant(&self, processid: ProcessId) -> Result<(), kernel::procs::Error> {
   self.apps.enter(processid, |_, _| {})
}
```

Finally to schedule an upcall any calls to `app.upcall.schedule()` should be
replaced with code like:

```rust
self.apps.enter(appid, |app, upcalls| {
    upcalls.schedule_upcall(upcall_number, r0, r1, r2);
});
```

The parameter `upcall_number` matches the `subscribe_num` the process used with
the subscribe syscall.




### Using `ReadOnlyAppSlice` and `ReadWriteAppSlice`: `console`

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

### Using `ReadOnlyAppSlice` and `ReadWriteAppSlice`: `spi_controller`

This is a second example, taken from `spi_controller`. Because SPI transfers
are bidirectional, there is an RX buffer and a TX buffer. However, a client
can ignore what it receives, and only pass a TX buffer if it wants: the RX
buffer can be zero length. As with other bus transfers, the SPI driver
needs to handle the case when its buffers change in length under it.
For example, a client may make the following calls:

  1. `allow_readwrite(rx_buf, 200)`
  2. `allow_readonly(tx_buf, 200)`
  3. `command(SPI_TRANSFER, 200)`
  4. (after some time, while transfer is ongoing) `allow_readonly(tx_buf2, 100)`

Because the underlying SPI tranfer typically uses DMA, the buffer
passed to the peripheral driver is `static`. The `spi_controller` has
fixed-size static buffers. It performs a transfer by copying
application slice data into/from these buffers. A very long
application transfer may be broken into multiple low-level transfers.

If a transfer is smaller than the static buffer, it is simple:
`spi_controller` copies the application slice data into its static
transmit buffer and starts the transfer. If the process rescinds the
buffer, it doesn't matter, as the capsule has the data. Similarly, the
presence of a receive application slice only matters when the transfer
completes, and the capsule decides whether to copy what it received out.

The principal complexity is when the buffers change during a low-level
transfer and then the capsule needs to figure out whether to continue
with a subsequent low-level transfer or finish the operation. The code
needs to be careful to not access past the end of a slice and cause a
kernel panic.

The code looks like this:

```rust
    // Assumes checks for busy/etc. already done
    // Updates app.index to be index + length of op
    fn do_next_read_write(&self, app: &mut App) {
        let write_len = self.kernel_write.map_or(0, |kwbuf| {
            let mut start = app.index;
            let tmp_len = app.app_write.map_or(0, |src| {
                let len = cmp::min(app.len - start, self.kernel_len.get());
                let end = cmp::min(start + len, src.len());
                start = cmp::min(start, end);

                for (i, c) in src.as_ref()[start..end].iter().enumerate() {
                    kwbuf[i] = *c;
                }
                end - start
            });
            app.index = start + tmp_len;
            tmp_len
        });
        self.spi_master.read_write_bytes(
            self.kernel_write.take().unwrap(),
            self.kernel_read.take(),
            write_len,
        );
    }
```


The capsule keeps track of its current write position with `app.index`. This
points to the first byte of untransmitted data. When a transfer starts
in response to a system call, the capsule checks that the requested length
of the transfer is not longer than the length of the transmit buffer, and
also that the receive buffer is either zero or at least as long. The
total length of a transfer is stored in `app.len`.

But if the transmit buffer is swapped during a transfer, it may be
shorter than `app.index`. In the above code, the variable `len` stores
the desired length of the low-level transfer: it's the minimum of data
remaining in the transfer and the size of the low-level static buffer.
The variable `end` stores the index of the last byte that can be
safely transmitted: it is the minimum of the low-level transfer end
(`start` + `len`) and the length of the application slice
(`src.len()`). Note that `end` can be smaller than `start` if
the application slice is shorter than the current write position.
To handle this case, `start` is set to be the minimum of `start` and
`end`: the transfer will be of length zero.
