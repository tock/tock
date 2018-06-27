---
location: Shanghai, China
date: October 28th
---

# Tock OS Training @ SOSP 2017

This course introduces you to Tock, a secure embedded operating system for sensor
networks and the Internet of Things. Tock is the first operating system to
allow multiple untrusted applications to run concurrently on a microcontroller-based
computer. The Tock kernel is written in Rust, a memory-safe systems language that
does not rely on a garbage collector. Userspace applications are run in
single-threaded processes that can be written in any language. A paper
describing Tock's goals, design, and implementation will be presented at the
conference on Monday and is available [here](https://www.amitlevy.com/papers/tock-sosp2017.pdf).

This course is based on TockOS as it was in October, 2017. It serves as a useful
introduction to Tock, but it is not updated as Tock evolves. To view this
tutorial in the context it was originally designed for, please checkout the
correct commit hash:

```bash
$ git checkout 3c12437d23b83db896a5a8e218a3dc14468ce2df
```

This commit includes a working version of this tutorial.
