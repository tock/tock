# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

import serial
from pexpect import fdpexpect
import logging
import queue
import re
import time
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


class MockSerialPort:
    def __init__(self):
        self.buffer = queue.Queue()
        self.accumulated_data = b""

    def write(self, data):
        logging.debug(f"Writing data: {data}")
        self.buffer.put(data)

    def expect(self, pattern, timeout=10):
        end_time = time.time() + timeout
        compiled_pattern = re.compile(pattern.encode())
        while time.time() < end_time:
            try:
                data = self.buffer.get(timeout=0.1)
                logging.debug(f"Received data chunk: {data}")
                self.accumulated_data += data
                if compiled_pattern.search(self.accumulated_data):
                    logging.debug(f"Matched pattern '{pattern}'")
                    return self.accumulated_data
            except queue.Empty:
                continue
        logging.error(f"Timeout waiting for pattern '{pattern}'")
        return None

    def flush_buffer(self):
        self.accumulated_data = b""
        while not self.buffer.empty():
            self.buffer.get()

    def close(self):
        pass

    def reset_input_buffer(self):
        self.flush_buffer()

    def reset_output_buffer(self):
        pass
