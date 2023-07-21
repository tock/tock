# Tock Binary Format

<!-- npm i -g markdown-toc; markdown-toc -i Compilation.md -->

<!-- toc -->

- [App Storage](#app-storage)
- [Empty Tock Apps](#empty-tock-apps)
- [TBF Header Section](#tbf-header-section)
  * [TBF Header Base](#tbf-header-base)
  * [TLV Elements](#tlv-elements)
  * [TLV Types](#tlv-types)
    + [`1` Main](#1-main)
    + [`2` Writeable Flash Region](#2-writeable-flash-region)
    + [`3` Package Name](#3-package-name)
    + [`5` Fixed Addresses](#5-fixed-addresses)
    + [`6` Permissions](#6-permissions)
    + [`7` Storage Permissions](#7-storage-permissions)
    + [`8` Kernel Version](#8-kernel-version)
    + [`9` Program](#9-program)
    + [`128` Credentials Footer](#128-credentials-footer)
- [Code](#code)

<!-- tocstop -->

Tock userspace applications must follow the Tock Binary Format
(TBF). A TBF Object has four parts: a Header section, which encodes
meta-data about the TBF Object, the actual Userspace Binary, an
optional Footer section which encodes metadata about the TBF Object,
and finally optional padding. 

TBF Headers and Footers differ in how they are handled for TBF Object
integrity. Integrity values (e.g., hashes) for a TBF Object are
computed over the Header section and Userspace Binary but not the
Footer Region or padding after footers. TBF Headers are covered by
integrity, while TBF Footers are not covered by integrity.

```
Tock App Binary:

Start of app -> +-------------------+---
              ^ | TBF Header        | ^
              | +-------------------+ | Protected region
              | | (Optional)        | |
              | | protected trailer | V
 Covered by   | +-------------------+---
 integrity    | | Userspace Binary  |
              | |                   |
              | |                   |
              V |                   |
                +-------------------+
                | Optional footers  |
                | (only if Program  |
                |  header present)  |
                +-------------------+
                | Optional padding  |
                +-------------------+
```

The header is interpreted by the kernel (and other tools, like tockloader) to
understand important aspects of the app. In particular, the kernel must know
where in the application binary is the entry point that it should start
executing when running the app for the first time.

After the header the app is free to include whatever Userspace Binary
it wants, and the format is completely up to the app. All support for
relocations must be handled by the app itself, for example.

If the TBF Object has a Program Header in the Header section, the
Userspace Binary can be followed by optional TBF Footers.

Finally, the TBF Object can be padded to a specific length. This is
useful when a memory protection unit (MPU) restricts the length and
offset of protection regions to powers of two. In such cases, padding
allows a TBF Object to be padded to a power of two in size, so the
next TBF Object is at a valid alignment.

Both TBF Footers and Headers follow the same TLV (type-length-value)
format, to simplify parsing.

## App Storage

TBF Objects in Tock are stored sequentially.  The start of TBF Object
N+1 is immediately at the end of TBF Object N. Therefore, the TBF
header specifies the length of the TBF Object so that the kernel can
find the start of the next one.

If there is a gap between TBF Objects an "empty object" can be
inserted to keep the structure intact.

Tock apps are typically stored in sorted order, from longest to
shortest. This is to help match MPU rules about alignment.

## Empty Tock Apps

A TBF Object can contain no code. A TBF Object can be marked as
disabled to act as padding between other objects.

## TBF Header Section

The TBF Header section contains all of a TBF Object's headers. All TBF
Objects have a Base Header and the Base Header is always first.  All
headers are a multiple of 4 bytes long; the TBF Header section is
multiple of 4 bytes long.

These are the Rust structures the kernel uses, defined in the
`tock-tbf` crate, to represent headers.  Their in-flash
representations are described below.

```rust
struct TbfHeaderV2Base {
    version: u16,            // Version of the Tock Binary Format (currently 2)
    header_size: u16,        // Number of bytes in the TBF header section
    total_size: u32,         // Total padded size of the program image in bytes, including header
    flags: u32,              // Various flags associated with the application
    checksum: u32,           // XOR of all 4 byte words in the header, including existing optional structs
}
```

After the Base Header come optional headers. Optional headers are
structured as TLVs (type-length-values). Footers are encoded in the
same way. Footers are also called headers for historical reasons:
originally TBFs only had headers, and since footers follow the same
format TBFs keep these types without changing their names.

```rust
// Identifiers for the optional header structs.
enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderPicOption1 = 4,
    TbfHeaderFixedAddresses = 5,
    TbfHeaderPermissions = 6,
    TbfHeaderPersistent = 7,
    TbfHeaderKernelVersion = 8,
    TbfHeaderProgram = 9,
    TbfFooterCredentials = 128,
}
// Type-length-value header to identify each struct.
struct TbfHeaderTlv {
    tipe: TbfHeaderTypes,    // 16 bit specifier of which struct follows
                             // When highest bit of the 16 bit specifier is set
                             // it indicates out-of-tree (private) TLV entry
    length: u16,             // Number of bytes of the following struct
}

// All apps must have a Main Header or a Program Header; it may
// have both. Without either, the "app" is considered padding and used 
// to insert an empty linked-list element into the app flash space. If 
// an app has both, it is the kernel's decision which to use. Older kernels
// use Main Headers, while newer (>= 2.1) kernels use Program Headers.
struct TbfHeaderMain {
    base: TbfHeaderTlv,
    init_fn_offset: u32,         // The function to call to start the application
    protected_trailer_size: u32, // The number of app-immutable bytes after the header
    minimum_ram_size: u32,       // How much RAM the application is requesting
}

// A Program Header specifies the end of the application binary within the 
// TBF, such that the application binary can be followed by footers. It also
// has a version number, such that multiple versions of the same application
// can be installed.
pub struct TbfHeaderV2Program {
    init_fn_offset: u32,
    protected_trailer_size: u32,
    minimum_ram_size: u32,
    binary_end_offset: u32,
    version: u32,
}

// Optional package name for the app.
struct TbfHeaderPackageName {
    base: TbfHeaderTlv,
    package_name: [u8],      // UTF-8 string of the application name
}

// A defined flash region inside of the app's flash space.
struct TbfHeaderWriteableFlashRegion {
    writeable_flash_region_offset: u32,
    writeable_flash_region_size: u32,
}

// One or more specially identified flash regions the app intends to write.
struct TbfHeaderWriteableFlashRegions {
    base: TbfHeaderTlv,
    writeable_flash_regions: [TbfHeaderWriteableFlashRegion],
}

// Fixed and required addresses for process RAM and/or process flash.
struct TbfHeaderV2FixedAddresses {
    base: TbfHeaderTlv,
    start_process_ram: u32,
    start_process_flash: u32,
}

struct TbfHeaderDriverPermission {
    driver_number: u32,
    offset: u32,
    allowed_commands: u64,
}

// A list of permissions for this app
struct TbfHeaderV2Permissions {
    base: TbfHeaderTlv,
    length: u16,
    perms: [TbfHeaderDriverPermission],
}

// A list of storage permissions for accessing persistent storage
struct TbfHeaderV2StoragePermissions {
    base: TbfHeaderTlv,
    write_id: u32,
    read_length: u16,
    read_ids: [u32],
    modify_length: u16,
    modify_ids: [u32],
}

// Kernel Version
struct TbfHeaderV2KernelVersion {
    base: TbfHeaderTlv,
    major: u16,
    minor: u16
}

// Types of credentials footers
pub enum TbfFooterV2CredentialsType {
    Reserved = 0,
    Rsa3072Key = 1,
    Rsa4096Key = 2,
    SHA256 = 3,
    SHA384 = 4,
    SHA512 = 5,
}

// Credentials footer. The length field of the TLV determines
// the size of the data slice.
pub struct TbfFooterV2Credentials {
    format: TbfFooterV2CredentialsType,
    data: &'static [u8],
}
```

### TBF Header Base

The TBF Header section contains a Base Header, followed by a sequence of
type-length-value encoded elements. All fields in both the base header and TLV
elements are little-endian. The base header is 16 bytes, and has 5 fields:

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Version     | Header Size | Total Size                |
+-------------+-------------+---------------------------+
| Flags                     | Checksum                  |
+---------------------------+---------------------------+
```

  * `Version` a 16-bit unsigned integer specifying the TBF header version.
    Always `2`.
  * `Header Size` a 16-bit unsigned integer specifying the length of the
    entire TBF header in bytes (including the base header and all TLV
    elements).
  * `Total Size` a 32-bit unsigned integer specifying the total size of the
    TBF in bytes (including the header).
  * `Flags` specifies properties of the process.
    ```
       3                   2                   1                   0
     1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0 9 8 7 6 5 4 3 2 1 0
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    | Reserved                                                  |S|E|
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    ```

    - Bit 0 marks the process enabled. A `1` indicates the process is
      enabled. Disabled processes will not be launched at startup.
    - Bit 1 marks the process as sticky. A `1` indicates the process is
      sticky. Sticky processes require additional confirmation to be erased.
      For example, `tockloader` requires the `--force` flag erase them.  This
      is useful for services running as processes that should always be
      available.
    - Bits 2-31 are reserved and should be set to 0.
  * `Checksum` the result of XORing each 4-byte word in the header, excluding
    the word containing the checksum field itself.

### TLV Elements

The header is followed immediately by a sequence of TLV elements. TLV
elements are aligned to 4 bytes. If a TLV element size is not 4-byte aligned, it
will be padded with up to 3 bytes. Each element begins with a 16-bit type and
16-bit length followed by the element data:

```
0             2             4
+-------------+-------------+-----...---+
| Type        | Length      | Data      |
+-------------+-------------+-----...---+
```

  * `Type` is a 16-bit unsigned integer specifying the element type.
  * `Length` is a 16-bit unsigned integer specifying the size of the data field
    in bytes.
  * `Data` is the element specific data. The format for the `data` field is
    determined by its `type`.

### TLV Types

TBF may contain arbitrary element types. To avoid type ID collisions
between elements defined by the Tock project and elements defined
out-of-tree, the ID space is partitioned into two segments. Type IDs
defined by the Tock project will have their high bit (bit 15) unset,
and type IDs defined out-of-tree should have their high bit set.

#### `1` Main

The `Main` element has three 32-bit fields:

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (1)    | Length (12) | init_offset               |
+-------------+-------------+---------------------------+
| protected_trailer_size    | min_ram_size              |
+---------------------------+---------------------------+
```

  * `init_offset` the offset in bytes from the beginning of binary payload
    (i.e. the actual application binary) that contains the first instruction to
    execute (typically the `_start` symbol).
  * `protected_trailer_size` the size of the protected region _after_ the TBF
    headers. Processes do not have write access to the protected region. TBF
    headers are contained in the protected region, but are not counted towards
    `protected_trailer_size`. The protected region thus starts at the first byte
    of the TBF base header, and is `header_size + protected_trailer_size` bytes
    in size.
  * `minimum_ram_size` the minimum amount of memory, in bytes, the process
    needs.

If the Main TLV header is not present, these values all default to `0`.

The Main Header and Program Header have overlapping functionality. If
a TBF Object has both, the kernel decides which to use.  Tock is
transitioning to having the Program Header as the standard one to use,
but older kernels (2.0 and earlier) do not recognize it and use the
Main Header.

#### `2` Writeable Flash Region

`Writeable flash regions` indicate portions of the binary that the process
intends to mutate in flash.

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (2)    | Length (8)  | offset                    |
+-------------+-------------+-------------+-------------+
| size                      |
+---------------------------+
```

  * `offset` the offset from the beginning of the binary of the writeable
    region.
  * `size` the size of the writeable region.


#### `3` Package Name

The `Package name` specifies a unique name for the binary. Its only field is
an UTF-8 encoded package name.

```
0             2             4
+-------------+-------------+----------...-+
| Type (3)    |   Length    | package_name |
+-------------+-------------+----------...-+
```

  * `package_name` is an UTF-8 encoded package name

#### `5` Fixed Addresses

`Fixed Addresses` allows processes to specify specific addresses they need for
flash and RAM. Tock supports position-independent apps, but not all apps are
position-independent. This allows the kernel (and other tools) to avoid loading
a non-position-independent binary at an incorrect location.

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (5)    | Length (8)  | ram_address               |
+-------------+-------------+-------------+-------------+
| flash_address             |
+---------------------------+
```

  * `ram_address` the address in memory the process's memory address must start
    at. If a fixed address is not required this should be set to `0xFFFFFFFF`.
  * `flash_address` the address in flash that the process binary (not the
    header) must be located at. This would match the value provided for flash to
    the linker. If a fixed address is not required this should be set to
    `0xFFFFFFFF`.

#### `6` Permissions

The `Permissions` section allows an app to specify driver permissions that it is
allowed to use. All driver syscalls that an app will use must be listed. The
list should not include drivers that are not being used by the app.

The data is stored in the optional `TbfHeaderV2Permissions` field. This includes
an array of all the `perms`.

```
0             2             4             6
+-------------+-------------+-------------+---------...--+
| Type (6)    | Length      | # perms     | perms        |
+-------------+-------------+-------------+---------...--+
```

The `perms` array is made up of a number of elements of
`TbfHeaderDriverPermission`. The first 16-bit field in the TLV is the number of
driver permission structures included in the `perms` array. The elements in
`TbfHeaderDriverPermission` are described below:

```text
Driver Permission Structure:
0             2             4             6             8
+-------------+-------------+---------------------------+
| driver_number             | offset                    |
+-------------+-------------+-------------+-------------+
| allowed_commands                                      |
+-------------------------------------------------------+
```

* `driver_number` is the number of the driver that is allowed. This for example
  could be `0x00000` to indicate that the `Alarm` syscalls are allowed.
* `allowed_commands` is a bit mask of the allowed commands. For example a value
  of `0b0001` indicates that only command 0 is allowed. `0b0111` would indicate
  that commands 2, 1 and 0 are all allowed. Note that this assumes `offset` is
  0, for more details on `offset` see below.
* The `offset` field in `TbfHeaderDriverPermission` indicates the offset of the
  `allowed_commands` bitmask. All of the examples described in the paragraph
  above assume an `offset` of 0. The `offset` field indicates the start of the
  `allowed_commands` bitmask. The `offset` is multiple by 64 (the size of the
  `allowed_commands` bitmask). For example an `offset` of 1 and a
  `allowed_commands` value of `0b0001` indicates that command 64 is allowed.

Subscribe and allow commands are always allowed as long as the specific
`driver_number` has been specified. If a `driver_number` has not been specified
for the capsule driver then `allow` and `subscribe` will be blocked.

Multiple `TbfHeaderDriverPermission` with the same `driver_numer` can be
included, so long as no `offset` is repeated for a single driver. When
multiple `offset`s and `allowed_commands`s are used they are ORed together,
so that they all apply.

#### `7` Storage Permissions

The `Storage Permissions` section is used to identify what access the app has to
persistent storage.

The data is stored in the `TbfHeaderV2StoragePermissions` field, which includes
a `write_id`, a number of `read_id`s, and a number of `modify_id`s.

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (7)    | Length      | write_id                  |
+-------------+-------------+---------------------------+
| # Read IDs  | read_ids (4 bytes each)                 |
+-------------+------------------------------------...--+
| # Modify IDs| modify_ids (4 bytes each)               |
+--------------------------------------------------...--+
```

- `write_id` indicates the id that all new persistent data is written with. All
  new data created will be stored with permissions from the `write_id` field.
  For existing data see the `modify_ids` section below. `write_id` does not need
  to be unique, that is multiple apps can have the same id. A `write_id` of
  `0x00` indicates that the app can not perform write operations.
- `read_ids` list all of the ids that this app has permission to read. The
  `read_length` specifies the length of the `read_ids` in elements (not bytes).
  `read_length` can be `0` indicating that there are no `read_ids`.
- `modify_ids` list all of the ids that this app has permission to modify or
  remove. `modify_ids` are different from `write_id` in that `write_id` applies
  to new data while `modify_ids` allows modification of existing data. The
  `modify_length` specifies the length of the `modify_ids` in elements (not
  bytes). `modify_length` can be `0` indicating that there are no `modify_ids`
  and the app cannot modify existing stored data (even data that it itself
  wrote).

For example, consider an app that has a `write_id` of `1`, `read_ids` of `2, 3`
and `modify_ids` of `3, 4`. If the app was to write new data, it would be stored
with id `1`. The app is able to read data stored with id `2` or `3`, note that
it cannot read the data that it writes. The app is also able to overwrite
existing data that was stored with id `3` or `4`.

An example of when `modify_ids` would be useful is on a system where each app
logs errors in its own write_region. An error-reporting app reports these errors
over the network, and once the reported errors are acked erases them from the
log. In this case, `modify_ids` allow an app to erase multiple different
regions.

#### `8` Kernel Version

The `compatibility` header is designed to prevent the kernel
from running applications that are not compatible with it.

It defines the following two items:
* `Kernel major` or `V` is the kernel major number (for Tock 2.0, it is 2)
* `Kernel minor` or `v` is the kernel minor number (for Tock 2.0, it is 0)

Apps defining this header are compatible with kernel version ^V.v (>= V.v and < (V+1).0)

The kernel version header refers only to the ABI and API exposed by the kernel 
itself, it does not cover API changes within drivers. 

A kernel major and minor version guarantees the ABI for exchanging 
data between kernel and userspace and the the system call numbers.

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (8)    | Length (4)  | Kernel major| Kernel minor|
+-------------+-------------+---------------------------+
```

#### `9` Program

A Program Header is an extended form of the Main Header. It adds two
fields, `binary_end_offset` and `version`. The `binary_end_offset` field
allows the kernel to identify where in the TBF object the application
binary ends. The gap between the end of the application binary and
the end of the TBF object can contain footers. 

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (9)    | Length (20) | init_offset               |
+-------------+-------------+---------------------------+
| protected_trailer_size    | min_ram_size              |
+---------------------------+---------------------------+
| binary_end_offset         | version                   |
+---------------------------+---------------------------+
```

  * `init_offset` the offset in bytes from the beginning of binary payload
    (i.e. the actual application binary) that contains the first instruction to
    execute (typically the `_start` symbol).
  * `protected_trailer_size` the size of the protected region _after_ the TBF
    headers. Processes do not have write access to the protected region. TBF
    headers are contained in the protected region, but are not counted towards
    `protected_trailer_size`. The protected region thus starts at the first byte
    of the TBF base header, and is `header_size + protected_trailer_size` bytes
    in size.
  * `minimum_ram_size` the minimum amount of memory, in bytes, the process
    needs.
  * `binary_end_offset` specifies the offset from the beginning of the TBF
    Object at which the Userspace Binary ends and optional footers begin.
  * `version` specifies a version number for the application implemented by
    the Userspace Binary. This allows a kernel to distinguish different 
    versions of a given application.

If a Program header is not present, `binary_end_offset` can be
considered to be `total_size` of the Base Header and `version` is 0.

The Main Header and Program Header have overlapping functionality. If
a TBF Object has both, the kernel decides which to use.  Tock is
transitioning to having the Program Header as the standard one to use,
but older kernels (2.0 and earlier) do not recognize it and use the
Main Header.

#### `128` Credentials Footer

A Credentials Footer contains cryptographic credentials for the integrity
and possibly identity of a Userspace Binary. A Credentials Footer has
the following format:

```
0             2             4             6             8
+-------------+-------------+---------------------------+
| Type (128)  | Length (4+n)| format                    |
+-------------+-------------+---------------------------+
| data...
+---------------------------+---------------------------+
```

The length of the data field is defined by the `Length` field. If
the data field is `n` bytes long, the `Length` field is 4+n. The
`format` field defines the format of the data field:

```rust
pub enum TbfFooterV2CredentialsType {
    Reserved = 0,
    Rsa3072Key = 1,
    Rsa4096Key = 2,
    SHA256 = 3,
    SHA384 = 4,
    SHA512 = 5,
}
```
[TRD-appid](reference/trd-appid.md) provides further details on 
TBF Credentials Footers, their format, and processing.

## Code

The process code itself has no particular format. It will reside in flash,
but the specific address is determined by the platform. Code in the binary
should be able to execute successfully at any address, e.g. using position
independent code.
