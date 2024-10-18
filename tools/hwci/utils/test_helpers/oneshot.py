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
