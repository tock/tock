---
driver number: 0x00001
---

# Console

## Overview

The console driver allows the process to write buffers to serial device. To
write a buffer, a process must share the buffer using `allow` then initiate the
write using a `command` call. It may also using `subscribe` to receive a
callback when the write has completed.

## Command

  * ### Command number: `0`

    **Description**: Does the driver exist?

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Ok(()) if it exists, otherwise NODEVICE

  * ### Command number: `1`

    **Description**: Initiate a write transaction of a buffer shared using `allow`.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed`.

    **Argument 1**: The maximum number of bytes to write. If this argument is
    greater than or equal to the buffer's size, the entire buffer will be
    written. Otherwise, the first N bytes of the buffer will be written, where N
    is the value of this argument.

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if no buffer was
    shared, or NOMEM if the driver failed to allocate memory for the
    transaction.

    **Additional notes:** A process may call this command with a write size of
    `0` to cancel a write transaction, if one is ongoing. Unless an error
    occurs, this will generate a write transaction completed event, regardless
    of whether or not a write transaction was already in progress.

  * ### Command number: `2`

    **Description**: Initiate a read transaction into a buffer shared using `allow`.
    At the end of the transaction, a callback will be delivered if the process
    has `subscribed` to read events using `subscribe number` 2.

    **Argument 1**: The maximum number of bytes to read.

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if no buffer was
    shared, or NOMEM if the driver failed to allocate memory for the
    transaction.

  * ### Command number: `3`

    **Description**: Abort any ongoing read transactions.
    Any received bytes will be delivered via callback if the process
    has `subscribed` to read events using `subscribe number` 2.

    **Argument 1**: unused

    **Argument 2**: unused

    **Returns**: Ok(()) if the command was successful, BUSY if no buffer was
    shared, or NOMEM if the driver failed to allocate memory for the
    transaction.

## Subscribe

  * ### Subscribe number: `1`

    **Description**: Subscribe to write transaction completion event. The
    callback will be called whenever a write transaction completes.

    **Callback signature**: The callback receives a single argument, the number
    of bytes written in the transaction. The value of the remaining arguments
    is undefined.

    **Returns**: Ok(()) if the subscribe was successful or NOMEM if the
    driver failed to allocate memory for the transaction.

  * ### Subscribe number: `2`

    **Description**: Subscribe to read transaction completion event. The
    callback will be called whenever a read transaction completes.

    **Callback signature**: The callback receives two arguments. The first
    is a statuscode, containing any error if one occurred. The second is the
    number of bytes read in the transaction. The value of the remaining arguments
    is undefined.

    **Returns**: Ok(()) if the subscribe was successful or NOMEM if the
    driver failed to allocate memory for the transaction.

## Read-Only Allow

  * ### Allow number: `1`

    **Description**: Sets a shared buffer to be used as a source of data for
    the next write transaction. A shared buffer is released if it is replaced
    by a subsequent call and after a write transaction is completed. Replacing
    the buffer after beginning a write transaction but before receiving a
    completion callback is undefined.

    **Returns**: Ok(()) if the subscribe was successful or NOMEM if the
    driver failed to allocate memory for the transaction.

## Read-Write Allow
  * ### Allow number: `1`

    **Description**: Sets a shared buffer to be read into by the next read
    transaction. A shared buffer is released in two cases: if it is replaced by
    a subsequent call or after a read transaction is completed. Replacing the
    buffer after beginning a read transaction but before receiving a completion
    callback is undefined.

    **Returns**: Ok(()) if the subscribe was successful or NOMEM if the
    driver failed to allocate memory for the transaction.

