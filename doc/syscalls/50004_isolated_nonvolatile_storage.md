---
driver number: 0x50004
---

# Isolated Nonvolatile Storage

This Driver provides access to a contiguous, fixed-size region of nonvolatile
storage the application can use. This interface supports both 32-bit and 64-bit
address spaces for the nonvolatile storage.

Note: use of this interface is protected by `StoragePermissions`, so
applications will need storage permissions to use this interface.

## Command

- ### Command number: `0`

  Does the driver exist?

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if it exists, otherwise `NODEVICE`.

- ### Command number: `1`

  **Get Size**. Query the size of the nonvolatile storage region the application
  has access to in bytes.

  Calling this command will allocate a storage region if one was not previously
  allocated to the application.

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  ##### Success

  The get size command was accepted and the response will be issued via an
  upcall.

  ##### Failure

  If the command does not succeed then no upcall will be issued and the command
  returns type `SyscallReturn::Failure` with one of these error codes:

  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `BUSY`: A prior request is pending.


- ### Command number: `2`

  **Read**. Read a region of the app's nonvolatile storage.

  The read is asynchronous as the nonvolatile storage may not be attached to
  main memory bus or mapped to the main address space. The data will be copied
  into the buffer shared with the kernel via read-write allow 0.

  The read is specified by the offset (in bytes) from the beginning of the app's
  allocated nonvolatile storage region and the length (in bytes) of the read.
  The length is equal to the size of the allowed read buffer.

  Calling this command will allocate a storage region if one was not previously
  allocated to the application.

  The application must have permissions to access nonvolatile storage for the
  read to succeed. The permission check may be asynchronous, and a permissions
  error may be returned via the upcall.

  #### Arguments

  - **1**: read offset, in bytes (lower 32 bits)
  - **2**: read offset, in bytes (upper 32 bits)

  #### Returns

  ##### Success

  The read command was accepted and a response will be issued via the upcall.

  ##### Failure

  If the command does not succeed then no upcall will be issued and the command
  returns type `SyscallReturn::Failure` with one of these error codes:

  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `BUSY`: A prior request is pending.


- ### Command number: `3`

  **Write**. Write a buffer to a region of the app's nonvolatile storage.

  The write is asynchronous as the write may require an erase first or the
  nonvolatile storage may not be attached to main memory bus or mapped to the
  main address space. The data to be written will be copied from the buffer
  shared with the kernel via read-only allow 0.

  The write is specified by the offset (in bytes) from the beginning of the
  app's allocated nonvolatile storage region and the length (in bytes) of the
  write. The length is equal to the size of the allowed buffer.

  Calling this command will allocate a storage region if one was not previously
  allocated to the application.

  The application must have permissions to access nonvolatile storage for the
  write to succeed. The permission check may be asynchronous, and a permissions
  error may be returned via the upcall.

  #### Arguments

  - **1**: write offset, in bytes (lower 32 bits)
  - **2**: write offset, in bytes (upper 32 bits)

  #### Returns

  ##### Success

  The write command was accepted and a response will be issued via the upcall.

  ##### Failure

  If the command does not succeed then no upcall will be issued and the command
  returns type `SyscallReturn::Failure` with one of these error codes:

  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `BUSY`: A prior request is pending.



## Subscribe

- ### Subscribe number: `0`

  Subscribe to get size upcalls. This upcall provides the size of the app's
  nonvolatile storage region in bytes.

  #### Upcall Signature

  The upcall signature looks like:

  ```rust
  fn upcall(s: Statuscode, length: usize);
  ```

  Upcall arguments:
  - 0: A `Statuscode` returning the success or failure of the operation.
  - 1: The size of the region in bytes.
  - 2: unused

  The `length` argument is only valid if the status code is `SUCCESS`.

  ##### `Statuscode` Values

  - `SUCCESS`: The get size command succeeded and `length` is set to the number
    of bytes in the app's nonvolatile storage region.
  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `NOMEM`: There is no space remaining.

- ### Subscribe number: `1`

  Subscribe to read done upcalls. This upcall fires after a read command
  completes or encounters an error.

  #### Upcall Signature

  The upcall signature looks like:

  ```rust
  fn upcall(s: Statuscode);
  ```

  Upcall arguments:
  - 0: A `Statuscode` returning the success or failure of the operation.
  - 1: unused
  - 2: unused

  The `length` argument is only valid if the status code is `SUCCESS`.

  ##### `Statuscode` Values

  - `SUCCESS`: The read command succeeded and `length` is set to the number
    of bytes read into the allowed buffer.
  - `RESERVE`: No buffer was allowed for read-write allow 0 or the allowed
    buffer has a length of 0.
  - `NOMEM`: The app has no nonvolatile storage region.
  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `INVAL`: The read was not within the app's storage region.
  - `FAIL`: There was an error accessing the underlying storage.

- ### Subscribe number: `2`

  Subscribe to write done upcalls. This upcall fires after a write command
  completes or encounters an error.

  #### Upcall Signature

  The upcall signature looks like:

  ```rust
  fn upcall(s: Statuscode);
  ```

  Upcall arguments:
  - 0: A `Statuscode` returning the success or failure of the operation.
  - 1: unused
  - 2: unused

  The `length` argument is only valid if the status code is `SUCCESS`.

  ##### `Statuscode` Values

  - `SUCCESS`: The read command succeeded and `length` is set to the number
    of bytes written from the allowed buffer.
  - `RESERVE`: No buffer was allowed for read-only allow 0 or the allowed
    buffer has a length of 0.
  - `NOMEM`: The app has no nonvolatile storage region.
  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `INVAL`: The write was not within the app's storage region.
  - `FAIL`: There was an error accessing the underlying storage.



## Read-Only Allow

- ### RO Allow number: `0`

  This buffer to use for writes to the nonvolatile storage.

## Read-Write Allow

- ### RW Allow number: `0`

  This buffer to use for reads from the nonvolatile storage.
