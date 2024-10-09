import serial
from pexpect import fdpexpect
import logging


class SerialPort:
    def __init__(self, port, baudrate=115200):
        self.port = port
        self.baudrate = baudrate
        try:
            self.ser = serial.Serial(port, baudrate=baudrate, timeout=1)
            self.child = fdpexpect.fdspawn(self.ser.fileno())
            logging.info(f"Opened serial port {port} at baudrate {baudrate}")
        except serial.SerialException as e:
            logging.error(f"Failed to open serial port {port}: {e}")
            raise

    def flush_buffer(self):
        self.ser.reset_input_buffer()
        self.ser.reset_output_buffer()
        logging.info("Flushed serial buffers")

    def expect(self, pattern, timeout=10):
        try:
            index = self.child.expect(pattern, timeout=timeout)
            logging.debug(f"Matched pattern '{pattern}'")
            return self.child.after
        except fdpexpect.TIMEOUT:
            logging.error(f"Timeout waiting for pattern '{pattern}'")
            return None
        except fdpexpect.EOF:
            logging.error("EOF reached while waiting for pattern")
            return None

    def close(self):
        self.ser.close()
        logging.info(f"Closed serial port {self.port}")
