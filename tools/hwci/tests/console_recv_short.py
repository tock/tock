from tests.console_hello_test import WaitForConsoleMessageTest

test = WaitForConsoleMessageTest(
    ["tests/console/console_recv_short"], "[SHORT] Error doing UART receive: -2"
)
