import logging
from utils.test_helpers import OneshotTest

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
