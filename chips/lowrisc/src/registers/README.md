# lowRISC Register Definitions

The files in this directory are auto-generated register definitions for
lowRISC peripherals.  These files were auto-generated from the OpenTitan
codebase as follows:

```bash
$ cd $OPENTITAN_TREE
$ git checkout earlgrey_es
$ bazel build //sw/device/tock:tock_lowrisc_registers
$ tar -C $TOCK_TREE -xvf bazel-bin/sw/device/tock/tock_lowrisc_registers.tar
```

Note: the existence of a file in this directory does not necessarily mean
that a tock peripheral implementation exists.  These files are _only_
the register definitions.
