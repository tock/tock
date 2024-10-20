# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

import logging
import threading
import time
from core.board_harness import BoardHarness
from utils.serial_port import MockSerialPort


class MockBoard(BoardHarness):
    def __init__(self):
        super().__init__()
        self.arch = "cortex-m4"
        self.kernel_board_path = "tock/boards/nordic/nrf52840dk"
        self.uart_port = self.get_uart_port()
        self.uart_baudrate = self.get_uart_baudrate()
        self.serial = self.get_serial_port()
        self.serial_output_thread = None
        self.running = False

    def get_uart_port(self):
        # Return a mock serial port identifier
        return "MOCK_SERIAL_PORT"

    def get_uart_baudrate(self):
        return 115200  # Same as the actual board

    def get_serial_port(self):
        return MockSerialPort()  # Initialize the mock serial port

    def erase_board(self):
        logging.info("Mock erase of the board")

    def flash_kernel(self):
        logging.info("Mock flashing of the Tock OS kernel")

    def flash_app(self, app):
        logging.info(f"Mock flashing of app: {app}")
        # Depending on the app, set up simulated output
        if app == "c_hello":
            self.simulate_output("Hello World!\r\n")
        else:
            logging.warning(f"No mock output configured for app: {app}")

    def simulate_output(self, message):
        # Start a thread to simulate serial output
        def output_thread():
            self.running = True
            logging.info("Starting mock serial output thread")
            time.sleep(1)
            self.serial.write(message.encode())
            self.running = False
            logging.info("Mock serial output thread finished")

        self.serial_output_thread = threading.Thread(target=output_thread)
        self.serial_output_thread.start()
        time.sleep(1)

    def simulate_multi_alarm_output(self):
        def output_thread():
            self.running = True
            logging.info("Starting mock multi-alarm serial output thread")
            start_time = int(time.time())
            while self.running:
                current_time = int(time.time()) - start_time
                # Simulate alarm 1 firing every 2 seconds
                if current_time % 2 == 0:
                    line = f"1 {current_time} {current_time + 2}\r\n"
                    self.serial.write(line.encode())
                # Simulate alarm 2 firing every 4 seconds
                if current_time % 4 == 0:
                    line = f"2 {current_time} {current_time + 4}\r\n"
                    self.serial.write(line.encode())
                time.sleep(1)
                if current_time > 10:  # Timeout after 10 seconds
                    self.running = False
            logging.info("Mock multi-alarm serial output thread finished")

        self.serial_output_thread = threading.Thread(target=output_thread)
        self.serial_output_thread.start()

    def stop(self):
        self.running = False
        if self.serial_output_thread and self.serial_output_thread.is_alive():
            self.serial_output_thread.join()


board = MockBoard()
