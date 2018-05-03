# ![TockOS](http://www.tockos.org/assets/img/tock.svg "TockOS Logo")

[![Build Status](https://travis-ci.org/tock/tock.svg?branch=master)](https://travis-ci.org/tock/tock)
[![irc](https://img.shields.io/badge/irc-%23tock-lightgrey.svg)](https://kiwiirc.com/client/irc.freenode.net/tock)

Tock is an embedded operating system designed for running multiple concurrent, mutually
distrustful applications on Cortex-M based embedded platforms. Tock's design
centers around protection, both from potentially malicious applications and
from device drivers. Tock uses two mechanisms to protect different components
of the operating system. First, the kernel and device drivers are written in
Rust, a systems programming language that provides compile-time memory safety,
type safety and strict aliasing. Tock uses Rust to protect the kernel (e.g. the
scheduler and hardware abstraction layer) from platform specific device drivers
as well as isolate device drivers from each other. Second, Tock uses memory
protection units to isolate applications from each other and the kernel.


Learn More
----------

How would you like to get started?

### Use Tock

First, follow our [getting started guide](doc/Getting_Started.md) to setup
your system to compile Tock and Tock applications.

Then head to the [hardware page](https://www.tockos.org/hardware/)
to learn about the hardware platforms Tock supports. Also check out the
[tutorials](doc/tutorials) to get started running apps with TockOS.


### Develop Tock

Read our [getting started guide](doc/Getting_Started.md) to get the correct
version of the Rust compiler, then look through the `/kernel`, `/capsules`,
`/chips`, and `/boards` directories.

We're happy to accept pull requests and look forward to seeing how Tock grows.


### Learn How Tock Works

Both the design and implementation of Tock are documented in the
[docs](doc) folder. Read through the guides there to learn about the kernel,
Tock's use of Rust, the build system, and applications.


### Keep Up To Date

Check out the [blog](https://www.tockos.org/blog/) where the **Talking Tock**
post series highlights what's new in Tock. Also, follow
[@talkingtock](https://twitter.com/talkingtock) on Twitter.

You can also browse our
[email group](https://groups.google.com/forum/#!forum/tock-dev) to see
discussions on Tock development.
