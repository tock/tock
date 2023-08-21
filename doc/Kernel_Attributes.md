Kernel Attributes
=================

Kernel attributes are stored in a data structure at the end of the kernel's
allocated flash region. These attributes describe properties of the flashed
kernel on a particular hardware board. External tools can read these attributes
to learn about the kernel installed on the board.

<!-- npm i -g markdown-toc; markdown-toc -i Kernel_Attributes.md -->

<!-- toc -->

- [Format](#format)
  * [Header Format](#header-format)
  * [TLV Format](#tlv-format)
- [TLVs](#tlvs)
  * [App Memory (0x0101)](#app-memory-0x0101)
  * [Kernel Binary (0x0102)](#kernel-binary-0x0102)
- [Kernel Attributes Location](#kernel-attributes-location)

<!-- tocstop -->

## Format

Kernel attributes are stored in a descending TLV (type-length-value) structure.
That means they start at the highest address in flash, and are appended in
descending flash addresses.

The first four bytes are a sentinel that spells "TOCK" (in ASCII). This sentinel
allows external tools to check if kernel attributes are present. Note, "first"
in this context means the four bytes with the largest address since this
structure is stored at the _end_ of flash.

The next byte is a version byte. This allows for future changes to the
structure.

The next three bytes are reserved.

After the header are zero or more TLV structures that hold the kernel
attributes.

### Header Format

```text
0          1          2          3          4 (bytes)
+----------+----------+----------+----------+
|                            TLVs...        |
+----------+----------+----------+----------+
| Reserved | Reserved | Reserved | Version  |
+----------+----------+----------+----------+
| T (0x54) | O (0x4F) | C (0x43) | K (0x4B) |
+----------+----------+----------+----------+
                                            ^
                        end of flash region─┘
```

### TLV Format

```text
0          1          2          3          4 (bytes)
+----------+----------+----------+----------+
|                           Value...        |
+----------+----------+----------+----------+
| Type                | Length              |
+----------+----------+----------+----------+
```

- Type: Indicates which TLV this is. Little endian.
- Length: The length of the value. Little endian.
- Value: Length bytes corresponding to the TLV.

## TLVs

The TLV types used for kernel attributes are unrelated to the TLV types used for
the [Tock Binary Format](./doc/TockBinaryFormat.md#tlv-types). However, to
minimize possible confusion, type values for each should not use the same
numbers.

### App Memory (0x0101)

Specifies the region of memory the kernel will use for applications.

```text
0          1          2          3          4 (bytes)
+----------+----------+----------+----------+
| Start Address                             |
+----------+----------+----------+----------+
| App Memory Length                         |
+----------+----------+----------+----------+
| Type = 0x0101       | Length = 8          |
+----------+----------+----------+----------+
```

- Start Address: The address in RAM the kernel will use to start allocation
  memory for apps. Little endian.
- App Memory Length: The number of bytes in the region of memory for apps.
  Little endian.


### Kernel Binary (0x0102)

Specifies where the kernel binary is and its size.

```text
0          1          2          3          4 (bytes)
+----------+----------+----------+----------+
| Start Address                             |
+----------+----------+----------+----------+
| Binary Length                             |
+----------+----------+----------+----------+
| Type = 0x0102       | Length = 8          |
+----------+----------+----------+----------+
```

- Start Address: The address in flash the kernel binary starts at. Little
  endian.
- Binary Length: The number of bytes in the kernel binary. Little endian.

## Kernel Attributes Location

Kernel attributes are stored at the end of the kernel's flash region and
immediately before the start of flash for TBFs.
