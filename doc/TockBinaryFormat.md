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
}

// Identifiers for the optional header structs.
enum TbfHeaderTypes {
    TbfHeaderMain = 1,
    TbfHeaderWriteableFlashRegions = 2,
    TbfHeaderPackageName = 3,
    TbfHeaderPicOption1 = 4,
    TbfHeaderFixedAddresses = 5,
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
    start_process_ram: u32,
    start_process_flash: u32,
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
flash and RAM. While Tock apps are expected to be position-independent, that is
not always possible, and this allows the kernel (and other tools) to check that
the addresses a process expects to be loaded at are being met.

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

## Code

The process code itself has no particular format. It will reside in flash,
but the specific address is determined by the platform. Code in the binary
should be able to execute successfully at any address, e.g. using position
independent code.

