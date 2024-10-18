from tests.console_hello_test import AnalyzeConsoleTest
import logging


class BlePassiveScanningTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["ble_passive_scanning"])

    def analyze(self, lines):
        found_advertisements = False
        for line in lines:
            if "PDU Type:" in line:
                found_advertisements = True
                break
        if found_advertisements:
            logging.info("BLE Passive Scanning Test passed.")
        else:
            logging.warning(
                "BLE Passive Scanning Test could not detect advertisements. Ensure BLE devices are nearby."
            )
            raise Exception(
                "BLE Passive Scanning Test failed: No advertisements detected."
            )


test = BlePassiveScanningTest()
