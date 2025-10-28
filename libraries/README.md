Tock Libraries
==============

This folder contains crates that are effectively libraries developed for and
used by Tock. The libraries are standalone, have become reasonably stable, and
are likely useful outside of Tock. Therefore they have moved to the libraries
folder as separate crates so that external projects can leverage them.

Someday these libraries could become their own repositories if the need arises.

Libraries in Separate Repositories
----------------------------------

The following libraries used to reside in this folder but have graduated to live
in their own repositories:

- [tock-registers](https://github.com/tock/tock-registers)

Using in an External Project
----------------------------

To use one of these libraries, simply add them as a dependency in a Cargo.toml
file. For example:

```toml
[dependencies]
tock-tbf = { git = "https://github.com/tock/tock" }
```

Cargo will handle finding the correct folder inside of the tock repository.
