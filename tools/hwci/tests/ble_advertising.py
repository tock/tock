from tests.console_hello_test import WaitForConsoleMessageTest

test = WaitForConsoleMessageTest(
    ["ble_advertising"], "Now advertising every 300 ms as 'TockOS'"
)
