---
driver number: 0x50003
---

# Key-Value Storage

This Driver provides access to a shared key-value store.

Note: use of this interface is protected by `StoragePermissions`, so
applications will need permissions in the TBF headers to use this interface.

## Command

- ### Command number: `0`

  Does the driver exist?

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if it exists, otherwise `NODEVICE`.

- ### Command number: `1`

  **GET**. Retrieve a value from the store based on the provided key.

  Use RO allow 0 to set the key.

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if the get command was accepted. On error, returns:

  - `NOMEM`: Already a pending request for this application.
  - `RESERVE`: Error in the driver, requesting process not set.
  - `SIZE`: Key too long.
  - `INVAL`: Incorrect permissions for the app.

- ### Command number: `2`

  **SET**. Set a key-value pair in the storage. If the key already exists the
  existing value will be overwritten.

  Use RO allow 0 to set the key and RO allow 1 to set the value.

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if the set command was accepted. On error, returns:

  - `NOMEM`: Already a pending request for this application or no room
    in internal buffers to store value.
  - `RESERVE`: Error in the driver, requesting process not set or value allow
    buffer not set.
  - `SIZE`: Key too long or value too long.
  - `INVAL`: Incorrect permissions for the app.

- ### Command number: `3`

  **DELETE**. Delete a key-value pair from the database.

  Use RO allow 0 to set the key.

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if the delete command was accepted. On error, returns:

  - `NOMEM`: Already a pending request for this application.
  - `RESERVE`: Error in the driver, requesting process not set.
  - `SIZE`: Key too long.
  - `INVAL`: Incorrect permissions for the app.

- ### Command number: `4`

  **ADD**. Add a key-value pair to the storage. The key must not already exist
  in the database. If the key already exists the operation will ultimately fail
  with an error in the upcall.

  Use RO allow 0 to set the key and RO allow 1 to set the value.

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if the add command was accepted. On error, returns:

  - `NOMEM`: Already a pending request for this application or no room
    in internal buffers to store value.
  - `RESERVE`: Error in the driver, requesting process not set or value allow
    buffer not set.
  - `SIZE`: Key too long or value too long.
  - `INVAL`: Incorrect permissions for the app.

- ### Command number: `5`

  **UPDATE**. Modify a value belonging to the specified key in the storage. The
  key must already exist in the database. If the key does not already exist the
  operation will ultimately fail with an error in the upcall.

  Use RO allow 0 to set the key and RO allow 1 to set the value.

  #### Arguments

  - **1**: unused
  - **2**: unused

  #### Returns

  `SUCCESS` if the update command was accepted. On error, returns:

  - `NOMEM`: Already a pending request for this application or no room
    in internal buffers to store value.
  - `RESERVE`: Error in the driver, requesting process not set or value allow
    buffer not set.
  - `SIZE`: Key too long or value too long.
  - `INVAL`: Incorrect permissions for the app.

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
