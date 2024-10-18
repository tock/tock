from tests.console_hello_test import AnalyzeConsoleTest
import logging


class AdcTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["tests/adc/adc"])

    def analyze(self, lines):
        adc_driver_found = False
        adc_readings_found = False
        for line in lines:
            if "ADC driver exists" in line:
                adc_driver_found = True
            if "ADC Reading:" in line:
                adc_readings_found = True
                break
            if "No ADC driver!" in line:
                logging.warning("No ADC driver available.")
                return  # Test passes if ADC is not available
        if adc_driver_found and adc_readings_found:
            logging.info("ADC Test passed with readings.")
        elif adc_driver_found:
            raise Exception("ADC Test failed: Driver found but no readings.")
        else:
            logging.warning("ADC Test skipped: No ADC driver.")
            return  # Test passes if ADC is not available


test = AdcTest()
