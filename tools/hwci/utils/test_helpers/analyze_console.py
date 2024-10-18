import logging
from utils.test_helpers import OneshotTest

class AnalyzeConsoleTest(OneshotTest):
    def oneshot_test(self, board):
        logging.info("Starting AnalyzeConsoleTest")
        lines = []
        serial = board.serial
        try:
            while True:
                output = serial.expect(".*\r\n", timeout=5)
                if output:
                    line = output.decode("utf-8", errors="replace").strip()
                    logging.info(f"SERIAL OUTPUT: {line}")
                    lines.append(line)
                else:
                    break
            self.analyze(lines)
        except Exception as e:
            logging.error(f"Error during serial communication: {e}")
        logging.info("Finished AnalyzeConsoleTest")

    def analyze(self, lines):
        pass  # To be implemented by subclasses
