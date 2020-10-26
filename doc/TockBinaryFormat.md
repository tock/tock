# Tock Binary Format

<!-- npm i -g markdown-toc; markdown-toc -i Compilation.md -->

<!-- toc -->

- [App Linked List](#app-linked-list)
- [Empty Tock Apps](#empty-tock-apps)
- [TBF Header](#tbf-header)
  * [TBF Header Base](#tbf-header-base)
  * [TLV Elements](#tlv-elements)
  * [TLV Types](#tlv-types)
    + [`1` Main](#1-main)
    + [`2` Writeable Flash Region](#2-writeable-flash-region)
    + [`3` Package Name](#3-package-name)
    + [`5` Fixed Addresses](#5-fixed-addresses)
    + [`6` Permissions](#6-permissions)
    + [`7` Persistent ACL](#7-persistent-acl)
- [Code](#code)

<!-- tocstop -->

Tock process binaries are must be in the Tock Binary Format (TBF). A TBF
includes a header portion, which encodes meta-data about the process, followed
by a binary blob which is executed directly, followed by optional padding.

```
Tock App Binary:

Start of app -> +-------------------+
                | TBF Header        |
                +-------------------+
                | Compiled app      |
                | binary            |
                |                   |
                |                   |
                +-------------------+
                | Optional padding  |
                +-------------------+
```

The header is interpreted by the kernel (and other tools, like tockloader) to
understand important aspects of the app. In particular, the kernel must know
where in the application binary is the entry point that it should start
executing when running the app for the first time.

After the header the app is free to include whatever binary data it wants, and
the format is completely up to the app. All support for relocations must be
handled by the app itself, for example.

Finally, the app binary can be padded to a specific length. This is necessary
for MPU restrictions where length and starting points must be at powers of two.

## App Linked List

Apps in Tock create an effective linked list structure in flash. That is, the
start of the next app is immediately at the end of the previous app. Therefore,
the TBF header must specify the length of the app so that the kernel can find
the start of the next app.

If there is a gap between apps an "empty app" can be inserted to keep the linked
list structure intact.

Also, functionally Tock apps are sorted by size from longest to shortest. This
is to match MPU rules about alignment.

## Empty Tock Apps

An "app" need not contain any code. An app can be marked as disabled and
effectively act as padding between apps.

## TBF Header

The fields of the TBF header are as shown below. All fields in the header are
little-endian.

```rust
struct TbfHeader {
    version: u16,            // Version of the Tock Binary Format (currently 2)
    header_size: u16,        // Number of bytes in the complete TBF header
    total_size: u32,         // Total padded size of the program image in bytes, including header
    flags: u32,              // Various flags associated with the application
    checksum: u32,           // XOR of all 4 byte words in the header, including existing optional structs

    // Optional structs. All optional structs start on a 4-byte boundary.
    main: Option<TbfHeaderMain>,
    pic_options: Option<TbfHeaderPicOption1Fields>,
    name: Option<TbfHeaderPackageName>,
    flash_regions: Option<TbfHeaderWriteableFlashRegions>,
    fixed_address: Option<TbfHeaderV2FixedAddresses>,
    permissions: Option<TbfHeaderV2Permissions>,
    persistent_acl: Option<TbfHeaderV2PersistentAcl>,
}

// Identifiers for the optional header structs.
enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderPicOption1 = 4,
    TbfHeaderFixedAddresses = 5,
    TbfHeaderPermissions = 6,
    TbfHeaderPersistent = 7,
}

// Type-length-value header to identify each struct.
struct TbfHeaderTlv {
    tipe: TbfHeaderTypes,    // 16 bit specifier of which struct follows
                             // When highest bit of the 16 bit specifier is set
                             // it indicates out-of-tree (private) TLV entry
    length: u16,             // Number of bytes of the following struct
}

// Main settings required for all apps. If this does not exist, the "app" is
// considered padding and used to insert an empty linked-list element into the
// app flash space.
struct TbfHeaderMain {
    base: TbfHeaderTlv,
    init_fn_offset: u32,     // The function to call to start the application
    protected_size: u32,     // The number of bytes the application cannot write
    minimum_ram_size: u32,   // How much RAM the application is requesting
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

// A list of persistent access permissions
struct TbfHeaderV2PersistentAcl {
    base: TbfHeaderTlv,
    write_id: u32,
    read_length: u16,
    read_ids: [u32],
    access_length: u16,
    access_ids: [u32],
}
```

Since all headers are a multiple of four bytes, and all TLV structures must be a
multiple of four bytes, the entire TBF header will always be a multiple of four
bytes.


### TBF Header Base

The TBF header contains a base header, followed by a sequence of
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
| protected_size            | min_ram_size              |
+---------------------------+---------------------------+
```

  * `init_offset` the offset in bytes from the beginning of binary payload
    (i.e. the actual application binary) that contains the first instruction to
    execute (typically the `_start` symbol).
  * `protected_size` the amount of flash, in bytes, after the header, to
    prevent the process from writing to.
  * `minimum_ram_size` the minimum amount of memory, in bytes, the process
    needs.

If the Main TLV header is not present, these values all default to `0`.

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

The `Permissions` section allows an app to specify driver permissions that it
is allowed to use. All driver syscalls that an app will use must be listed. The
list should not include drivers that are not being used by the app.

The data is stored in the optional `TbfHeaderV2Permissions` field. This
includes an array of all the `perms`.

```
0             2             4
+-------------+-------------+---------...--+
| Type (6)    | Length      | perms        |
+-------------+-------------+---------...--+
```

The `perms` array is made up of a number of elements of
`TbfHeaderDriverPermission`. The length of the TLV can be used to determine
the number of array elements. The elements in `TbfHeaderDriverPermission` are
described below:

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

#### `7` Persistent ACL

The `Persistent ACL` section is used to identify what access the app has to
persistent storage.

The data is stored in the `TbfHeaderV2PersistentAcl` field, which includes a
`write_id` and a number of `read_ids`.

```
0             2             4             6             8              x            x+2
+-------------+---------------------------+-------------+---------...--+-------------+---------...--+
| Type (6)    | write_id                  | read_length | read_ids     |access_ids|  access_ids  |
+-------------+-------------+-------------+-------------+---------...--+-------------+---------...--+
```

`write_id` indicates the id that all new persistent data is written with.
All new data created will be stored with permissions from the `write_id`
field. For existing data see the `access_ids` section below.
Only apps with the same id listed in the `read_ids` can read the data.
Apps with the same `access_ids` or `write_id` can overwrite the data.
`write_id` does not need to be unique, that is multiple apps can have the
same id.
A `write_id` of `0x00` indicates that the app can not perform write operations.

`read_ids` list all of the ids that this app has permission to read. The
`read_length` specifiies the length of the `read_ids` in elements (not bytes).
`read_length` can be `0` indicating that there are no `read_ids`.

`access_ids` list all of the ids that this app has permission to write.
`access_ids` are different to `write_id` in that `write_id` applies to new data
while `access_ids` allows modification of existing data.
The `access_length` specifiies the length of the `access_ids` in elements (not bytes).
`access_length` can be `0` indicating that there are no `access_ids`.

For example an app has a `write_id` of `1`, `read_ids` of `2, 3` and
`access_ids` of `3, 4`. If the app was to write new data, it would be stored
with id `1`. The app is able to read data stored with id `2` or `3`, note that
it can not read the data that it writes. The app is also able to overwrite
existing data that was stored with id `3` or `4`.

An example of when `access_ids` would be useful is on a system where each app
logs errors in its own write_region. An error-reporting app reports these
errors over the network, and once the reported errors are acked erases them
from the log. In this case `access_ids` allow an app to erase multiple
different regions.

## Code

The process code itself has no particular format. It will reside in flash,
but the specific address is determined by the platform. Code in the binary
should be able to execute successfully at any address, e.g. using position
independent code.

