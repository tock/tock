---
location: Delft, Netherlands
date: November 5, 2017
---

# Tock OS Training @ SenSys 2017

This course introduces you to Tock, a secure embedded operating system for
sensor networks and the Internet of Things. Tock is the first operating system
to allow multiple untrusted applications to run concurrently on a
microcontroller-based computer. The Tock kernel is written in Rust, a
memory-safe systems language that does not rely on a garbage collector.
Userspace applications are run in single-threaded processes that can be written
in any language. A paper describing Tock's goals, design, and implementation was
published at the SOSP'17 conference and is available
[here](https://www.amitlevy.com/papers/tock-sosp2017.pdf).

In this course, you will learn the basic Tock system architecture, how to write
a userspace process in C, Tock's system call interface, and fill in code for a
small kernel extension written in Rust. The course assumes experience
programming embedded devices and fluency in C. It assumes no knowledge of Rust,
although knowing Rust will allow you to be more creative in the Rust programming
part of the course.

This course was primarily designed in November of 2017, but has been updated
through May 2018. It is a useful introduction to Tock but is no longer updated
as Tock evolves in preparation for new courses that will be used in November
2018. To follow this course, please checkout the commit below to use Tock at a
point where the tutorial is known to be working.

```bash
$ git checkout 1203437bff05667bb0636dc9ab69e1daca13c2a2
```
