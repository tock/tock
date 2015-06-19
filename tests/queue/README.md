Queue test
Philip Levis <pal@cs.stanford.edu>
Jun 19 2015

This tests the a lock-free fixed-size queue (a ring buffer). It randomly enqueues and dequeues,
to test edge conditions. Since a random arrival and departure process will in theory lead to
an unbounded queue, this test (if run long enough) can also test overflow.

Limitations

This test currently uses a copy of the Queue in hil and sam4l; future versions
should figure out how to extricate the code from its dependencies and platform
assumptions.

