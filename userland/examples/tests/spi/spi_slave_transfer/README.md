Spi Slave Transfer Test
========================
This test passes one of two buffers over the Spi bus; either an all-zero
buffer, or a buffer with sequentially increasing integers. This test assumes
that the transmitter (the attached Spi Master) first sends a buffer with
sequentially increasing integers, before alternating with the zero buffer.

Note that this test is intended to be used in tandem with the Spi Master
Transfer Test. To use this test, connect two boards together, using one as an
Spi Master and one as an Spi Slave, and load the Spi Master/Slave Test code. On
the slave board, every time a transfer is received, the received buffer is
compared with the expected buffer, and a new buffer is queued. On failure (when
an unexpected buffer is received), the LED is turned on.
