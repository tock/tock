Qemu and Tock
=============

Tock is experimenting with QEMU.

**All things QEMU are in very early stages and rough edges are to be expected.**

That said, please do file PRs or issues if you run into trouble or find things
confusing. The long-term goal is to integrate QEMU as a core part of CI for Tock.

<!-- npm i -g markdown-toc; markdown-toc -i Qemu.md -->

<!-- toc -->

- [Supported Boards](#supported-boards)
- [Tock QEMU Fork](#tock-qemu-fork)

<!-- tocstop -->

## Supported Boards

QEMU support for embedded platforms is limited. Please check the the table in
the [`boards/` subdirectory](../boards/README.md) for an up-to-date list of
supported boards.

## Tock QEMU Fork

Some of the Tock developers are working to improve board support, but one
consequences is that not everything is yet upstreamed.

In the short term, we abstract all of this away in the build system.
To experiement with the Tock fork, the easiest path is the run
`make ci-job-qemu`, which will fetch and build everything for you.
While things are in flux, those wanting more details are encouraged to look at
the make recipe.
