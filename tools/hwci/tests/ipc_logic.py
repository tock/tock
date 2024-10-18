from tests.console_hello_test import AnalyzeConsoleTest
import logging


class IpcLogicTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(
            apps=[
                "tutorials/05_ipc/led",
                "tutorials/05_ipc/rng",
                "tutorials/05_ipc/logic",
            ]
        )

    def analyze(self, lines):
        expected_output = "Number of LEDs:"
        for line in lines:
            if expected_output in line:
                logging.info("IPC Logic Test passed.")
                return
        raise Exception("IPC Logic Test failed: Expected output not found.")


test = IpcLogicTest()
