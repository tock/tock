Spi Master Transfer Test
========================
This test passes one of two buffers over the Spi bus; either an all-zero
buffer, or a buffer with sequentially increasing integers. This test assumes
that the receiver (the attached Spi Slave) echoes the buffers back. That is,
on the first transfer, the Spi Master sends a buffer with sequentially
increasing integers, and expects a buffer of all zeroes. On the next call to
'spi_read_write', the Master sends a buffer of all zeroes, and expects a buffer
of sequentially increasing integers.

Note that this test is intended to be used in tandem with the Spi Slave Transfer
Test. To use this test, connect two boards together, using one as an Spi Master
and one as an Spi Slave, and load the Spi Master/Slave Test code. On the master
board, every time the user button is pressed, a buffer is sent. On failure (when
an unexpected buffer is received), the LED is turned on.
