from tests.console_hello_test import WaitForConsoleMessageTest

test = WaitForConsoleMessageTest(
    ["tests/console/console_recv_long"], "[SHORT] Error doing UART receive: -2"
)
