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
application persistent storage. This document does not include specific types of
persistent storage (e.g., flash, FRAM, etc.) or storage access abstractions
(e.g., block-access, byte-access, etc.).


3 Stored State Identifiers
-------------------------------

All shared persistent storage implementations must store a 32 bit identifier
with each stored object to mark the application that created the stored object.

When applications write data, their [ShortId](../trd-appid.md) must be used as the identifier. When
the kernel writes data, the identifier must be 0.


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

Storage permissions are objects that implement the `StoragePermissions` trait.
The trait allows for storage implementations (e.g. capsules) to check
per-application permissions for read and modify operations on stored state, as
well as if the application has permission to write new state.

```rust
/// Interface for checking permissions to access persistent storage.
trait StoragePermissions {
    /// Check if these storage permissions grant read access to the stored state
    /// marked with identifier `stored_id`.
    fn check_read_permission(&self, stored_id: u32) -> bool;

    /// Check if these storage permissions grant modify access to the stored
    /// state marked with identifier `stored_id`.
    fn check_modify_permission(&self, stored_id: u32) -> bool;

    /// Retrieve the identifier to use when storing state, if the application
    /// has permission to write. Returns `None` if the application cannot write.
    fn get_write_id(&self) -> Option<u32>;
}
```


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


8 Authors' Addresses
===============================
```
Brad Campbell <bradjc@virginia.edu>
```

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
