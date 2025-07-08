# Tock Core Notes 2021-04-02

## Attending
 * Alistair
 * Amit Levy
 * Brad Campbell
 * Gabe Marcano
 * Hudson Ayers
 * Johnathan Van Why
 * Pat Pannuto
 * Philip Levis
 * Branden Ghena
 * Vadim Sukhomlinov
 * Oskar Senft


## Updates

- Johnathan: New libtock-rs actually working!
- Cargo features interact in strange ways. Depending on how cargo is invoked
  binaries can end up with or without features, like a memory allocator.
- Pat: fundamental or transient?
- Johnathan: Not sure.
- Hudson: RFCs for new cargo features, will that help?
- Johnathan: Maybe.
- Vadim: I've run into this issue as well.

- Phil: Working on TRD for creating a HIL.
- Need to update GPIO TRD.
- Related: https://book.tockos.org/development/hil.html

- Hudson: PR for `ReturnCode` -> `Result<(), ErrorCode>`.
- Some more HIL fixups required to return buffers when errors happen.
- Phil and Amit: we can help.

## TRD 104

- https://github.com/tock/tock/blob/a43f4e804d1eb6079587fc47c6d8b0057fdc4a2a/doc/reference/trd104-syscalls.md
- https://github.com/tock/tock/pull/2431

- Phil: TRD documents new system call for 2.0. Various fixups ongoing to the document.

- Document says:
  - `NODEVICE`: returned if system call driver doesn't exist.
  - `NOSUPPORT`: returned if driver exists, but no system call is invalid.
- Should we change the code to match? Make them the same?

- Amit: how would a process use the different error codes?
- Phil: On error, process can tell if the driver exists at all, or if just
  incompatibility between them.

- Amit: if we are hiding/filtering certain subcommands, what do we do then?
- Phil: initial thought is that should be a separate error code.
- Brad: right now we let the filter-er decide the error.
- Phil: we can leave it that way and re-evaluate later.
- Amit: easy to change in the future.

- Alistair: potential security issue, if malicious process can probe.
- Amit: up to the policy on what to return.
- Phil: non filtering case is different.
- Alistair: the TRD should specify that NODEVICE is returned if the entire
  device is being filtered.
- Amit: I would like to see the security policy documented separately.
- Not clear we have a global policy yet that should be documented here.
- Phil: seems reasonable, but not clear we want to commit to that now.
- Amit: we can say that `NODEVICE` means to a process that you do not have
  access to the device. Does not necessarily mean that the device does not
  exist.
- Alistair: need to update the comment for what `NODEVICE` means.
- Phil: yes, need to make those more precise. Also need to update `NOSUPPORT`.

- Amit: should be ready to merge soon.

## Console

- Johnathan: Oskar working on opentitan, and running into issues with console.

- Oskar: Need to interface with board via serial, want to implement console
  recv. Current API needs a length for receive. What happens if I receive 1
  byte, and another byte comes in immediately after?
- Phil: Needs buffering somewhere. Can also use `ReceiveAdvanced`.
- Amit: Expectation is that if you really care you would use UART HW flow
  control.
