Application Persistent Data Storage Permissions
===============================================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell<br/>
**Draft-Created:** 2024/06/06<br/>
**Draft-Modified:** 2024/06/06<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>


Abstract
-------------------------------

Tock supports storing persistent state for applications, and all persistent
state in Tock is identified based on the application that stored it. Tock
supports permissions for persistent state, allowing for the kernel to restrict
which applications can store state and which applications can read stored state.
This TRD describes the permissions architecture for persistent state in Tock.
This document is in full compliance with [TRD1][TRD1].


1 Introduction
-------------------------------

Tock applications need to be able to store persistent state. Additionally,
applications need to be able to keep data private from other applications. The
kernel should also be able to allow specific applications to read and modify
state from other applications.

This requires a method for assigning applications persistent identifiers,
a mechanism for granting storage permissions to specific applications,
and kernel abstractions for implementing storage capsules that respect
the storage permissions.


2 Scope
-------------------------------

This document only describes the permission architecture in Tock for supporting
application persistent storage. This document does not prescribe specific types of
persistent storage (e.g., flash, FRAM, etc.), storage access abstractions
(e.g., block-access, byte-access, etc.),
or storage interfaces (e.g., key-value, filesystems, logs, etc.).


3 Stored State Identifiers
-------------------------------

All shared persistent storage implementations must store a 32 bit identifier
with each stored object to mark the application that created the stored object.

When applications write data, their [ShortId](trd-appid.md) must be used as the
identifier. When the kernel writes data, the identifier must be 0.

The security, uniqueness, mapping policy, and other properties of ShortIds are
allowed to vary based on board configuration. For storage use cases which have
specific concerns or constraints around the policies for storage identifiers,
users should consult the [properties of ShortIds afforded by AppId policy](trd-appid.md#8-short-ids-and-the-compress-trait).

4 Permissions
-------------------------------

All persistent application data is labeled based on the application which wrote
the data.
Applications can read and modify data with suitable
permissions.

There are three types of permissions:

1. **Write**: The application can write data.
1. **Read**: The application can read data.
1. **Modify**: The application can modify existing data.

Each permission type is independent. For example, an application can be given
read permission for specific data but not be able to write new data itself.

Write is a boolean permission. An application either has permission to write or
it does not.

Read and Modify permissions are tuples of `(the permission type, stored state
identifier)`. These permissions only exist as associated with a particular
stored state identifier. That is, a Read permission gives an application
permission to read only stored state marked with the associated stored state
identifier, and a Modify permission gives an application permission to modify
only stored state marked with the associated stored state identifier.


5 Requirements
-------------------------------

The Tock storage model imposes the following requirements:

1. Applications are given separate write, read, and modify permissions.
2. The label stored with the persistent data when the data are written is the
   application's short AppID.
3. Applications without a `ShortId::Fixed` cannot access (i.e.,
   read/write/modify) any persistent storage.
4. How permissions are mapped to applications must be customizable for different
   Tock kernels.

Additionally, the kernel itself can be given permission to store state.

### 5.1 ShortId Implications

As all persistent state written by applications is marked with the writing
application's ShortId, the assignment mechanism for ShortIds is tightly coupled
with the access policies for persistent state. This coupling is intentional as
AppIDs are unique to specific applications. However, as ShortIds are only 32
bits, it is not possible to assign a globally unique ShortId to all
applications. Therefore, board authors should be intentional with how ShortIds
are assigned when persistent storage is accessible to userspace.

In particular, two potentially problematic cases can arise:

1. A ShortId is re-used for different applications. This might happen if one
   application is discontinued and a new application is assigned the same
   ShortId. The new application would then have unconditional access to any
   state the old application stored.
2. A new ShortId is used for the same application. This might happen if the
   ShortId assignment algorithm changes. The same application then would lose
   access to data it previously stored.


6 Kernel Enforcement
-------------------------------

It is not feasible to implement all persistent storage APIs through the core
kernel (i.e., in trusted code). Instead, the kernel provides an API to retrieve
the storage permissions for a specific process. Capsules then use these
permissions to enforce restrictions on storage access.
The API consists of these functions:

```rust
/// Check if these storage permissions grant read access to the stored state
/// marked with identifier `stored_id`.
pub fn check_read_permission(&self, stored_id: u32) -> bool;

/// Check if these storage permissions grant modify access to the stored
/// state marked with identifier `stored_id`.
pub fn check_modify_permission(&self, stored_id: u32) -> bool;

/// Retrieve the identifier to use when storing state, if the application
/// has permission to write. Returns `None` if the application cannot write.
pub fn get_write_id(&self) -> Option<u32>;
```

This API is implemented for the `StoragePermissions` object (which is an
`enum`). The `StoragePermissions` type can be stored per-process and passed in
storage APIs to express the storage permissions of the caller of any storage
operations.

### 6.1 Using Permissions in Capsules

When writing storage capsules, capsule authors should include APIs which include
`StoragePermissions` as an argument, and should check for permission before
performing any storage operation.

For example, a filing cabinet abstraction that identifies stored state based on
a record name might have an (asynchronous) API like this:

```rust
pub trait FilingCabinet {
    fn read(&self, record: &str, permissions: &dyn StoragePermissions) -> Result<(), ErrorCode>;
    fn write(&self, record: &str, data: &[u8], permissions: &dyn StoragePermissions) -> Result<(), ErrorCode>;
}
```

Inside the implementation for any storage abstraction, the implementation must
consider three operations and check for permissions:

1. The operation is a **read**. If there is no stored state that matches the
   read request, the capsule should return `ErrorCode::NOSUPPORT`. If there is
   stored state that matches the request, the capsule must call
   `StoragePermissions::check_read_permission(stored_id)` with the identifier
   associated with the stored record. If `check_read_permission()` returns
   false, the capsule should return `ErrorCode::NOSUPPORT`. If
   `check_read_permission()` returns true, the capsule should return the read
   data.
2. The operation is a **write**, and the write would store **new data**. The
   capsule must call `StoragePermissions::get_write_id()`. If `get_write_id()`
   returns `None`, the capsule should return `ErrorCode::NOSUPPORT`. If
   `get_write_id()` returns `Some()`, the capsule should save the new data and
   must use the returned `u32` identifier. It should then return `Ok(())`.
3. The operation is a **write**, and the write would overwrite **existing
   data**. The capsule must first retrieve the storage identifier for the
   existing state. The the capsule must call
   `StoragePermissions::check_modify_permission(stored_id)`. If
   `check_modify_permission()` returns false, the capsule should return
   `ErrorCode::NOSUPPORT`. If `check_modify_permission()` returns true, the
   capsule should overwrite the data while not changing this stored identifier.
   The capsule should then return `Ok(())`.

For example, with the filing cabinet example:

```rust
pub trait FilingCabinet {
    fn read(&self, record: &str, permissions: &dyn StoragePermissions) -> Result<[u8], ErrorCode> {
        let obj = self.cabinet.read(record);
        match obj {
            Some(r) => {
                if permissions.check_read_permission(r.id) {
                    Ok(r.data)
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            None => Err(ErrorCode::NOSUPPORT),
        }
    }

    fn write(&self, record: &str, data: &[u8], permissions: &dyn StoragePermissions) -> Result<(), ErrorCode> {
        let obj = self.cabinet.read(record);
        match obj {
            Some(r) => {
                if permissions.check_modify_permission(r.id) {
                    self.cabinet.write(record, r.id, data);
                    Ok(())
                } else {
                    Err(ErrorCode::NOSUPPORT)
                }
            }
            None => {
                match permissions.get_write_id() {
                    Some(id) => {
                        self.cabinet.write(record, id, data);
                        Ok(())
                    }
                    None => Err(ErrorCode::NOSUPPORT),
                }
            }
        }
    }
}
```

### 6.2 `StoragePermissions` Type

The kernel defines a `StoragePermissions` type which expresses the storage
permissions of an application. This is implemented as a definite type rather
than a trait interface so permissions can be passed in storage APIs without
requiring a static object for every process in the system.

The `StoragePermissions` type is capable of holding storage permissions in
different formats. In general, the type looks like:

```rust
pub enum StoragePermissions {
    SelfOnly(core::num::NonZeroU32),
    FixedSize(FixedSizePermissions),
    Listed(ListedPermissions),
    Kernel,
    Null,
}
```

Each variant is a different method for representing and storing storage
permissions. For example, `FixedSize` contains fixed size lists of permissions,
where as `Null` grants no storage permissions.

The `StoragePermissions` type includes multiple constructors for instantiating
storage permissions.


7 Specifying Permissions
-------------------------------

Different users and different kernels will use different methods for determining
the persistent storage access permissions for different applications (and by
extensions the running process for that application). The following are some
examples of how storage permissions may be specified.

1. In TBF headers. The `StoragePermissions` TBF header allows a developer to
   specify storage permissions when the app is compiled. Using this method
   assumes the kernel can trust the application's headers, perhaps because the
   kernel only runs apps signed by a trusted party that has verified the TBF
   headers.
2. Within the kernel. The kernel can maintain a data structure of permissions
   for known applications. This should be coupled with the AppID mechanism to
   consistently assign storage permissions to applications based on their
   persistent identifier.
3. With a generic policy. The kernel may permit all applications with a fixed
   ShortId to use persistent storage. This method can isolate applications by
   only permitting read and modify access to state stored by the same
   application.

### 7.1 Assigning Permissions to Processes

The core kernel allows individual boards to configure how permissions are
assigned to applications. At runtime, the kernel needs to know what permissions
each executing process has. To facilitate this, Tock uses the
`ProcessStandardStoragePermissionsPolicy` process policy. Each process, when created,
will store a `StoragePermissions` object that specifies the storage permissions for
that process.

```rust
/// Generic trait for implementing a policy on how applications should be
/// assigned storage permissions.
pub trait ProcessStandardStoragePermissionsPolicy<C: Chip> {
    /// Return the storage permissions for the specified `process`.
    fn get_permissions(&self, process: &ProcessStandard<C>) -> StoragePermissions;
}
```

This trait is specific to the `ProcessStandard` implementation of `Process` to
enable policies to use TBF headers when assigning permissions.

Several examples of policies are in the `capsules/system` crate.


8 Storage Examples
-------------------------------

The permissions architecture is generic for storage in Tock, but this section
describes some examples of how this architecture may be used for several storage
abstractions. Note, these are just examples and not descriptions of actual Tock
implementations nor requirements for how various storage abstractions must be
implemented.

1. Key-Value storage. Each key-value pair is stored as a triple: (key, value,
   storage identifier). On `get()`, the storage identifier for the key-value
   pair is checked. On `set()`, if the key already exists the modify permission
   is used, and if the key does not exist the write permission is used.
2. Logging. Loggers append to a shared log. Loggers can only append to the log
   if the have the write permission. Each log entry includes the storage
   identifier of the writing logger. Loggers do not have any read permission.
   Log analyzers only have read permissions. The analyzers have multiple read
   permissions to the log entries they need to analyze. The modify permission is
   not used.
3. Per-application nonvolatile storage. Each application is given a region of
   nonvolatile storage. Applications only access their own storage region. The
   storage implementation still checks and enforces read, write, and modify
   permissions, but the expectation is that applications that have the write
   permission also have modify and read permissions for their own stored state.
   There is no API for accessing other application state, so maintaining lists
   of read/modify permissions is not necessary.
4. Global persistent configuration. The storage abstraction maintains a
   persistent data store that multiple applications use. Only one application is
   expected to have the write permission to initialize the configuration. Other
   applications that use the configuration have read permission for the
   initializing application's storage identifier, and may have modify permission
   if they need to update the configuration.


8 Authors' Addresses
===============================
```
Brad Campbell <bradjc@virginia.edu>
```

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
