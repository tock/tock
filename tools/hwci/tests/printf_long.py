from tests.console_hello_test import WaitForConsoleMessageTest

test = WaitForConsoleMessageTest(
    ["tests/printf_long"],
    "Hi welcome to Tock. This test makes sure that a greater than 64 byte message can be printed.",
)
