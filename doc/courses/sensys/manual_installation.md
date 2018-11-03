## Manual Installation

If you choose to install manually, you will need the following software:

1. Command line utilities: curl, make, git

1. Python 3 and pip3

1. A local clone of the Tock repository

        $ git clone https://github.com/tock/tock.git

1. A local clone of the Tock applications repository (for apps written in C)

        $ git clone https://github.com/tock/libtock-c.git

1. [rustup](http://rustup.rs/). This tool helps manage installations of the
   Rust compiler and related tools.

        $ curl https://sh.rustup.rs -sSf | sh

1. [arm-none-eabi toolchain](https://developer.arm.com/open-source/gnu-toolchain/gnu-rm/downloads) (version >= 5.2)

   OS-specific installation instructions can be found
   [here](https://github.com/tock/tock/blob/master/doc/Getting_Started.md#arm-none-eabi-toolchain)

1. [tockloader](https://github.com/tock/tockloader)

        $ pip3 install -U --user tockloader

    > Note: On MacOS, you may need to add `tockloader` to your path. If you
    > cannot run it after installation, run the following:

        $ export PATH=$HOME/Library/Python/3.6/bin/:$PATH

    > Similarly, on Linux distributions, this will typically install to
    > `$HOME/.local/bin`, and you may need to add that to your `$PATH` if not
    > already present:

        $ PATH=$HOME/.local/bin:$PATH


### Testing

To verify you have everything installed correctly,
[hop back over to the testing directions in the main README](README.md#testing).
