from tests.console_hello_test import AnalyzeConsoleTest
import logging


class CHelloAndPrintfLongTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["c_hello", "tests/printf_long"])

    def analyze(self, lines):
        expected_messages = [
            "Hello World!",
            "Hi welcome to Tock. This test makes sure that a greater than 64 byte message can be printed.",
            "And a short message.",
        ]
        messages_found = {msg: False for msg in expected_messages}

        for line in lines:
            for msg in expected_messages:
                if msg in line:
                    messages_found[msg] = True

        for msg, found in messages_found.items():
            if not found:
                raise Exception(f"Did not find expected message: '{msg}'")
        logging.info("C Hello and Printf Long Test passed.")


test = CHelloAndPrintfLongTest()
