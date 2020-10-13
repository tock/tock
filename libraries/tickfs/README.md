# TickFS

TickFS (Tiny Circular Key Value File System) is a small file system allowing
key value pairs to be stored in Flash Memory.

TickFS was written to allow the Tock OS kernel to persistently store app data
on flash. It was written to be generic though, so other Rust applications can
use it if they want.

TickFS is based on similar concepts as
[Yaffs1](https://yaffs.net/documents/how-yaffs-works]).

TickFS provides ACID properties.

## Goals of TickFS

TickFS is designed with these main goals (in order)

 * Fully implemented in no_std Rust
 * Power loss resilient
 * Maintain data integrity and detect media errors
 * Wear leveling
 * Low memory usage
 * Low storage overhead
 * No external crates in use (not including unit tests)

TickFS is also designed with some assumptions

 * Most operations will be retrieving keys
 * Some operations will be storing keys
 * Keys will rarely be deleted
 * Key values will rarely need to be modified

## Using TickFS

See the generated Rust documentation for details on using this in your project.

## How TickFS works

Unlike a regular File System (FS) TickFS is only designed to store Key/Value (KV)
pairs in flash. It does not support writing actual files, directories or other
complex objects. Although a traditional file system layer could be added on top
to add such features.

TickFS allows writing new key/value pairs (by appending them) and removing
old key/value pairs.

TickFS has two important types, regions and objects.

A TickFS region is the smallest region of the flash memory that can be erased
in a single command.

TickFS saves and restores objects from flash. TickFS objects contain the value
the user wanted to store as well as extra header data. Objects are internal to
TickFS and users don't need to understand them in detail to use it.

For more details on the technical implementation see the [SPEC.md](./spec.md) file.

### Collisions

TickFS will prevent a new key/value pair with a colliding hash of the key to be
added. The collision will be reported to the user with
`ErrorCode::KeyAlreadyExists`.

### Power loss protection

TickFS ensures that in the event of a power loss, all commited data remains
commited. This is the durability guarantee as part of the ACID semantics.
The only data that can be lost in the event of a power loss is
the data which hasn't been write to flash yet.

If a power loss occurs after calling `append_key()` or `invalidate_key()`
before it has completed then the operation probably did not complete and
that data is lost.

### Security

TickFS uses checksums to check data integrity. TickFS does not have any measures
to prevent malicious manipulation or privacy. An attacker with access to the
flash can change the values without being detected. An attacked with access
to flash can also read all of the information. Any privacy, security or
authentication measures need to be layered on top of TickFS.

## Versions

TickFS stores the version when adding objects to the flash storage.

TickFS is currently version 0.

 * Version 0
   * Version 0 is a draft version. It should NOT be used for important data!
     Version 0 maintains no backwards compatible support and could change at
     any time.
