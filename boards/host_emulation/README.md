Host Side Testing Framework
===========================

For details on what this framework is see the [docs](../../doc/HostEmulation.md).

### Building

An example, executable platform implementation can be built via cargo.

```shell
cargo build
```

This will produce an executable at the project top level in `target/debug`.

This executable requires one and exactly one Tock app to be supplied as a
command line argument. This is temporary as abstractions are worked out for
extending the platform definition.
