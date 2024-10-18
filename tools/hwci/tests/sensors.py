from tests.console_hello_test import AnalyzeConsoleTest
import logging


class SensorsTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["sensors"])

    def analyze(self, lines):
        sensors_detected = False
        for line in lines:
            if "Sampling" in line:
                sensors_detected = True
            if "deg C" in line or "Light Intensity" in line or "Humidity" in line:
                logging.info(f"Sensor reading: {line.strip()}")
                return
        if sensors_detected:
            raise Exception("Sensors Test failed: No sensor readings found.")
        else:
            logging.warning("Sensors Test skipped: No sensors detected.")


test = SensorsTest()
