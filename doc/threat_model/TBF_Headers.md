TBF Headers
===========

TBF is the [Tock Binary Format](../TockBinaryFormat.md). It is the format of
application binaries in a Tock system's flash storage.

TBF headers are considered part of an application, and are mostly untrusted.
As such, TBF header parsing must be robust against malicious inputs (e.g.
pointers must be checked to confirm they are in-bounds for the binary).

However, because the kernel relies on the TBF's `total_size` field to load the
binaries, the application loader is responsible for verifying the `total_size`
field at install time. The kernel trusts the `total_size` field for
confidentiality and integrity.

When possible, [TLV types](../TockBinaryFormat.md#tlv-types) should be designed
so that the kernel does not need to trust their correctness. When a TLV type is
defined that the kernel must trust, then the threat model must be updated to
indicate that application loaders are responsible for verifying the value of
that TLV type.
