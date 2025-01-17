---
driver number: 0x50004
---

# Isolated Nonvolatile Storage

This Driver provides access to a contiguous, fixed-size region of nonvolatile
storage the application can use.

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
  The driver will copy up to the number of bytes specified by length. The copy
  length is the smallest of the length requested, the size of the specified read
  range that is withing the app's allocated nonvolatile region, and the size of
  the allowed buffer.

  Calling this command will allocate a storage region if one was not previously
  allocated to the application.

  #### Arguments

  - **1**: read offset, in bytes
  - **2**: read length, in bytes

  #### Returns

  ##### Success

  The read command was accepted and a response will be issued via the upcall.

  ##### Failure

  If the command does not succeed then no upcall will be issued and the command
  returns type `SyscallReturn::Failure` with one of these error codes:

  - `NOSUPPORT`: The application does not have permissions to access the
    nonvolatile storage.
  - `BUSY`: A prior request is pending.



## Subscribe

- ### Subscribe number: `0`

  Subscribe to operation completion upcalls. All K-V operations will trigger
  this upcall when complete.

  #### Upcall Signature

  The upcall signature looks like:

  ```rust
  fn upcall(s: Statuscode, value_length: usize, unused: usize);
  ```

  If the requested operation was set/add/update/delete, the other fields are
  always 0.

  If the requested operation was a GET, `value_length` will be set to the length
  of the value in bytes. If the value was longer than what fit in the RW allowed
  buffer, `s` will be a `SIZE` error. If a different error occurred
  `value_length` will be set to 0.

  The third argument `unused` is always 0.

  ##### `Statuscode` Values

  If the operation succeeded `s` will be `SUCCESS`.

  On failure, the following errors will be returned:

  - For GET:
    - `SIZE`: The value is longer than the provided buffer.
    - `NOSUPPORT`: The key could not be found or the app does not have
      permission to read this key.
    - `FAIL`: An internal error occurred.
  - For SET:
    - `NOSUPPORT`: The app does not have permission to store this key.
  - For ADD:
    - `NOSUPPORT`: The key already exists and cannot be added or the app does
      not have permission to add this key.
  - For UPDATE:
    - `NOSUPPORT`: The key does not already exist and cannot be modified or the
      app does not have permission to modify this key.
  - For SET/ADD/UPDATE:
    - `NOMEM`: The key could not be updated because the KV store is full.
    - `SIZE`: The key or value is too many bytes.
    - `FAIL`: An internal error occurred.
  - For DELETE:
    - `NOSUPPORT`: The key does not exist or the app does not have permission to
      delete this key.
    - `FAIL`: An internal error occurred.

## Read-Only Allow

- ### RO Allow number: `0`

  The key to use for the intended operation. The length of the allowed buffer
  must match the length of the key.


- ### RO Allow number: `1`

  The value to use for the intended write operation. The length of the allowed
  buffer must match the length of the value.

  This is only used for set/add/update operations.

## Read-Write Allow

- ### RW Allow number: `0`

  Storage for the value after a GET operation. The kernel will write the value
  read from the database here.

  If the read value is longer than the size of the allowed buffer the driver
  will provide the portion of the value that does fit. The `value_length` in the
  callback will still be set to the full size of the original value.

  As the kernel must be able to write the buffer to provide userspace the value
  this must be a read-write allow, and separate from the RO allow for setting
  the value.
