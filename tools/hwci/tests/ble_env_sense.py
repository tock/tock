from tests.console_hello_test import WaitForConsoleMessageTest

test = WaitForConsoleMessageTest(
    ["services/ble-env-sense", "services/ble-env-sense/test-with-sensors"],
    "BLE ERROR: Code = 16385",
)
