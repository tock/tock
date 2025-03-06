Dynamic Process Loading
=======================

**TRD:** <br/>
**Working Group:** Kernel<br/>
**Type:** Documentary<br/>
**Status:** Draft <br/>
**Author:** Brad Campbell, Viswajith Govinda Rajan<br/>
**Draft-Created:** 2025/03/05<br/>
**Draft-Modified:** 2025/03/05<br/>
**Draft-Version:** 1<br/>
**Draft-Discuss:** devel@lists.tockos.org<br/>


Abstract
=================================

Tock supports dynamically storing and loading processes at runtime (i.e., after
the main kernel loop has started). The primary use case for dynamic process
loading is adding functionality (i.e., installing a new application) or updating
functionality (i.e., installing a new version of an existing application). This
TRD documents the design and implementation of the dynamic process loading
framework in Tock. This document is in full compliance with [TRD1][TRD1].


1 Introduction
=================================

Typically Tock applications are stored in some persistent storage and loaded
when the Tock kernel starts. Because Tock applications are stored separately
from the kernel, they can be updated or added while the Tock kernel continues
normal operation. Tock provides an interface for supporting application installs
at runtime.

Dynamically adding applications is comprised of two operations:

1. Storing the new process binary.
2. Loading the new process binary into a Tock process.

This TRD documents both interfaces. The first operation (i.e., storing the
process binary) is particularly important as that functionality is introduced by
this TRD. In contrast, loading the process binary largely re-uses the same
functionality as described in [TRD-AppID](../trd-appid.md).

The design of the dynamic process loading infrastructure attempts to make
minimal assumptions about the underlying format of process binaries, how process
binaries are organized and discovered on a particular board, and how process
binaries are stored on a particular board. For example, while conventionally
Tock process binaries use a TBF header and are stored sequentially in
executable, address-space mapped flash, the general interfaces are designed to
not require that particular implementation. Boards that choose to use a
different data structure for process binaries and need dynamic process loading
should be able to use the described interfaces. However, as the conventional
TBF-based sequential process storage is common, this TRD also describes a
reference implementation that assumes that process storage model.

The Tock dynamic process loading architecture supports a privileged userspace
application that is responsible for obtaining the new process binary to be
loaded. However, this is not required as the described interfaces could be used
without such a userspace application.


2 Architecture Overview
=================================

The dynamic process loading architecture is modular to meet the requirements of
the Tock Threat Model and to enable different implementations based on the
process binary storage design used by a particular board.

The general architecture is shown here:

```text                                                                         
┌────────────────────────────────────────────────────┐                         
│                                                    │                         
│             Userspace Application                  │                         
│                                                    │                         
└────────────────────────────────────────────────────┘                         
─────────────────Syscall 0x10001──────────────────────                         
┌────────────────────────────────────────────────────┐                         
│                                                    │ Conventional    
│               AppLoader Capsule                    │ Capsule            
│                                                    │                         
└────────────────────────────────────────────────────┘                         
 trait DynamicBinaryStore    trait DynamicProcessLoad                         
┌────────────────────────┐  ┌────────────────────────┐                         
│                        │  │                        │ Kernel          
│     DynamicStore       │  │      DynamicLoad       │ Capsules                  
│                        │  │                        │                         
└────────────────────────┘  └────────────────────────┘                         
```

The layers provide differing levels of trust for each component.

- **Userspace Application**: The userspace application is generally untrusted.
  It is responsible for obtaining the process binary. Generally, this
  application is expected to work correctly, but it may not be able to verify a
  process binary it receives.
- **AppLoader Capsule**: The `AppLoader` capsule has normal capsule trust and
  provides a system call interface for dynamic process loading. Because it is a
  conventional capsule, we do not trust it to enforce the Tock Threat Model
  requirements.
- **Kernel Capsules**: The kernel capsules are the minimal functionality that
  _must_ be trusted to ensure the Tock security guarantees are met. This
  includes verifying process credentials and ensuring the process storage format
  remains valid.


3 System Call API
=================================

The `0x10001` system call interface provides four operations:

1. `setup(process_binary_size_bytes: usize)`: This initiates the process of
   loading a new process binary. The `AppLoader` capsule will attempt to allocate resources
   to be able to store and load a process of the specified size.
   Each process may only setup one process binary to load at a time.
   Success or
   failure is indicated via an upcall.
2. `write(process_binary: &[u8], length_bytes: usize, offset_bytes: usize)`:
   This stores a portion of the process binary.
   `setup()` must have completed successfully before `write()` can be called.
   Write is expected to be called
   multiple times to store the entire process binary and there is no assumption
   about the order the process binary is written in. Success or failure is
   indicated via an upcall.
3. `load()`: This indicates the entire process binary has been written and the
   new process binary should be loaded into a process. Success or failure is
   indicated via an upcall.
4. `abort()`: This operation cancels the setup/write operation and frees 
   allocated resources so that they are available for a different process. 
   Success or failure is indicated via an upcall.

These operations are implemented using conventional allow, command, and
subscribe system calls.


4 Kernel Capsule Traits
=================================

The trusted capsules use the following traits to provide functionality to the
app loading capsule.

4.1 Dynamic Process Binary Storing
---------------------------------

The kernel provides an interface for storing a process binary loaded at runtime.
The kernel must expose this interface to ensure that the implementation
correctly upholds the Tock Threat Model guarantees. Specifically, storing a new
process binary must not cause existing process binaries to be "lost" or not
loaded on future reboots of the board.

```rust
pub trait DynamicBinaryStore {
    /// Initiate storing a new process binary with the specified length.
    fn setup(&self, length: usize) -> Result<usize, ErrorCode>;

    /// Store a portion of the process binary.
    fn write(&self, buffer: SubSliceMut<'static, u8>, offset: usize) -> Result<(), ErrorCode>;

    /// Writing the process binary has finished.
    fn finalize(&self) -> Result<(), ErrorCode>;

    /// Call to abort the setup/writing process.
    fn abort(&self) -> Result<(), ErrorCode>;

    fn set_storage_client(&self, client: &'static dyn DynamicBinaryStoreClient);
}

trait DynamicBinaryStoreClient {
    /// Any setup work is done and we are ready to write the process binary.
    fn setup_done(&self);

    /// The provided process binary buffer has been stored.
    fn write_done(&self, result: Result<(), ErrorCode>, buffer: &'static mut [u8], length: usize);

    /// Operations to finish up writing the new process binary are completed.
    fn finalize_done(&self, result: Result<(), ErrorCode>);

    /// Canceled any setup or writing operation and freed up reserved space.
    fn abort_done(&self, result: Result<(), ErrorCode>);
}
```

There is a coupling between the `setup()`, `write()`, and `finalize()` calls. The `setup()`
call allows the implementation to allocate the needed resources to store the
process binary. Because the kernel requires that it reserves the resources
required for the new binary before storing it, this method MUST be called before
calling `write()`. An implementation is responsible for storing individual
chunks of the process binary.
Once the entire process binary has been written the `finalize()` method must be
called. This completes the store and the implementation can handle a new
`setup()` call.

The `abort()` call deallocates any stored resources and resets the
implementation to handle a new `setup()` call.
`abort()` can only be called after `setup()` and before `finalize()`.

Each operation is asynchronous and must generate a callback if the operation
returns `Ok(())`.

The interface is intentionally general to support different underlying storage
formats and storage media.

4.2 Dynamic Process Loading
---------------------------------

The kernel provides an interface for loading a stored process binary into an
active Tock process.

```rust
trait DynamicProcessLoad {
    /// Request kernel to load the newly stored process.
    fn load(&self) -> Result<(), ErrorCode>;

    fn set_load_client(&self, client: &'static dyn DynamicProcessLoadClient);
}

trait DynamicProcessLoadClient {
    /// The new process has been loaded.
    fn load_done(&self, result: Result<(), ErrorCode>);
}
```

The `load()` operation is asynchronous and must generate a callback if it
returns `Ok(())`.

This interface does not mandate that the kernel capsule creates a new process.
Implementations may include a policy for choosing whether to load a new process
binary. The implementation must also use the board's chosen credential checking
policy.


5 Sequential Process Loading Implementation
=================================

Tock includes a reference implementation of the dynamic process loading
architecture. This includes the `AppLoader` capsule and implementations of the
kernel capsule traits.

The implementations are structs with names that start with `Sequential`. The
"Sequential" designation indicates that this implementation assumes 1) process
binaries include TBF headers, 2) process binaries are stored back-to-back, and
3) process binaries are stored in executable flash that is mapped to the main
address space of the processor.

This implementation relies on the `SequentialProcessLoaderMachine` which loads
process binaries from flash, checks their credentials, and verifies uniqueness.

The implementation is structured like this:

```text
 trait DynamicBinaryStore
 trait DynamicProcessLoad
┌────────────────────────────────┐   ┌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┐
│                                │   ╎                                ╎
│                              ──┼──►╎                                ╎
│                                │   ╎                                ╎
│ SequentialDynamicBinaryStorage │   ╎ SequentialProcessLoaderMachine ╎
│                                │   ╎                                ╎
│                                │   ╎                                ╎
│                                │   ╎                                ╎
└────────────────────────────────┘   └╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┘
 hil::NonvolatileStorage
```

The existing `SequentialProcessLoaderMachine` object is responsible for managing
the stored process binaries, including discovering the existing process binaries
and identifying a location in flash where a new process binary can be stored.

The `SequentialDynamicBinaryStorage` is responsible for writing to the flash and
implementing the kernel capsule traits. On errors when writing to the flash, or
if the upper layer calls `abort()`, the implementation will write a padding
application over the incomplete new process binary. This method reduces
potential fragmentation due to problematic binaries.

5.1 Ensuring Application Availability
---------------------------------

In the [Tock Threat Model](threat), the `total_size` field in the TBF header is
trusted to ensure that processes stored sequentially in flash can be discovered
and that any individual process binary cannot prevent another stored process
from being discovered by the process loader.

The `SequentialDynamicBinaryStorage` implementation ensures this invariant holds
because of two implementation details. First, during the `setup` phase, the
available storage location for the new process binary may need a padding app
before, after, or both before and after the new process binary to ensure the
sequential process binary array remains discoverable. If a padding app will be
required after the new process binary, the padding app is written first before
any portion of the new process binary is stored in flash. If a padding app will be
required before the new process binary, the padding app is only written once
 the new process binary is determined to have a valid TBF header.

Second, `SequentialDynamicBinaryStorage` does not allow the calling capsule to
write a portion of the first eight bytes of the process binary (where the
`total_size` field is located). It must write the entire region, and
`SequentialDynamicBinaryStorage` ensures those first eight bytes are correct
(i.e., they match the size of the process binary and use a valid TBF header
version).


6 Future Extensions
===============================

There are certain additions to this architecture which may be desirable in the
future.

1. Modular structs for a particular process storage format. All mechanisms that
   operate on stored process binaries or store new process binaries must
   understand the particular process storage format in use (e.g., storing
   process binaries with TBF headers sequentially in flash). Currently that
   logic is merged with other logic (e.g., loading and credential checking). It
   may be useful to separate those functions, so that the process storage format
   can be decoupled from other mechanisms, enabling more code re-use. However,
   with only one process storage format it is currently unclear what abstraction
   would be required.
2. Interfaces for decryption and decompression. Process binaries may need to be
   compressed or encrypted when transferred to the board. Currently, any
   decompression or decryption must happen in the userspace application. It may
   be useful to include those operations within the kernel for easier code
   reuse, performance, and/or better buffer management.
3. Explicit process loading policy. Currently, the
   `SequentialDynamicBinaryStorage` implementation attempts to load any process
   that is newly stored. A kernel may wish to have a certain policy for
   determining whether to attempt to load a process, even if the credentials are
   valid.
4. Process version checking. If a process binary to be stored is an update, it
   may be useful to verify the new process binary is newer than the existing
   process binary before storing the new process binary.


7 Authors' Addresses
===============================
```
Brad Campbell <bradjc@virginia.edu>
Viswajith Govinda Rajan <vishgr@virginia.edu>
```

[TRD1]: trd1-trds.md "Tock Reference Document (TRD) Structure and Keywords"
[threat]: https://book.tockos.org/doc/threat_model/threat_model "Tock Threat Model"
