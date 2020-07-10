Qemu and Tock
=============

Tock is experimenting with QEMU.

**All things QEMU are in early stages and rough edges are to be expected.**

That said, please do file PRs or issues if you run into trouble or find things
confusing. The long-term goal is to integrate QEMU as a core part of CI for Tock.

<!-- npm i -g markdown-toc; markdown-toc -i Qemu.md -->

<!-- toc -->

- [Supported Boards](#supported-boards)
- [Building QEMU](#building-qemu)

<!-- tocstop -->

## Supported Boards

QEMU support for embedded platforms is limited. Please check the the table in
the [`boards/` subdirectory](../boards/README.md) for an up-to-date list of
supported boards.

## Building QEMU

Tock requires the master branch of QEMU. The `make ci-setup-qemu` Make target
will build this for you. If you would prefer to build it yourself or you need
more help look at the QEMU wiki: https://wiki.qemu.org/Hosts

Although both Tock and QEMU have automated testing it's possible that the version
of QEMU and Tock will become out of sync and will no longer work. If you are having
problems try older versions of QEMU and/or Tock.
