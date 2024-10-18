from tests.console_hello_test import AnalyzeConsoleTest
import logging


class GpioInterruptTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["tests/gpio/gpio_interrupt"])

    def analyze(self, lines):
        interrupt_detected = False
        for line in lines:
            if "GPIO Interrupt!" in line:
                interrupt_detected = True
                break
        if interrupt_detected:
            logging.info("GPIO Interrupt Test passed.")
        else:
            raise Exception("GPIO Interrupt Test failed: No interrupt detected.")


test = GpioInterruptTest()
