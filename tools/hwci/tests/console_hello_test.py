# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

import logging
from core.test_harness import TestHarness


class OneshotTest(TestHarness):
    def __init__(self, apps=[]):
        self.apps = apps

    def test(self, board):
        logging.info("Starting OneshotTest")
        board.erase_board()
        board.serial.flush_buffer()
        board.flash_kernel()
        for app in self.apps:
            board.flash_app(app)
        self.oneshot_test(board)
        logging.info("Finished OneshotTest")

    def oneshot_test(self, board):
        pass  # To be implemented by subclasses


class AnalyzeConsoleTest(OneshotTest):
    def oneshot_test(self, board):
        logging.info("Starting AnalyzeConsoleTest")
        lines = []
        serial = board.serial
        try:
            while True:
                output = serial.expect(".*\r\n", timeout=5)
                if output:
                    line = output.decode("utf-8", errors="replace").strip()
                    logging.info(f"SERIAL OUTPUT: {line}")
                    lines.append(line)
                else:
                    break
            self.analyze(lines)
        except Exception as e:
            logging.error(f"Error during serial communication: {e}")
        logging.info("Finished AnalyzeConsoleTest")

    def analyze(self, lines):
        pass  # To be implemented by subclasses


class WaitForConsoleMessageTest(OneshotTest):
    def __init__(self, apps, message):
        super().__init__(apps)
        self.message = message

    def oneshot_test(self, board):
        logging.info(f"Waiting for message: '{self.message}'")
        output = board.serial.expect(self.message, timeout=10)
        if output:
            logging.info(f"Received expected message: '{self.message}'")
        else:
            raise Exception(f"Did not receive expected message: '{self.message}'")
