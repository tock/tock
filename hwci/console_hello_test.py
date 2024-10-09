import logging
import re
from collections import defaultdict
from test_harness import TestHarness


class OneshotTest(TestHarness):
    def __init__(self, apps=[]):
        self.apps = apps

    def test(self, serial, gpio, board):
        logging.info("Starting OneshotTest")
        board.erase_board()
        serial.flush_buffer()
        board.flash_kernel()
        for app in self.apps:
            board.flash_app(app)
        self.oneshot_test(serial, gpio, board)
        logging.info("Finished OneshotTest")

    def oneshot_test(self, serial, gpio, board):
        pass  # To be implemented by subclasses


class AnalyzeConsoleTest(OneshotTest):
    def oneshot_test(self, serial, gpio, board):
        logging.info("Starting AnalyzeConsoleTest")
        lines = []
        try:
            while True:
                output = serial.expect(".*\r\n", timeout=5)
                if output:
                    line = output.decode("utf-8", errors="replace").strip()
                    logging.info(f"SERIAL OUTPUT: {line}")
                    lines.append(line)
                else:
                    break
        except Exception as e:
            logging.error(f"Error during serial communication: {e}")
        self.analyze(lines)
        logging.info("Finished AnalyzeConsoleTest")

    def analyze(self, lines):
        pass  # To be implemented by subclasses


class MultiAlarmTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["multi_alarm_simple_test"])

    def analyze(self, lines):
        """
        Analyzes the output lines from the multi_alarm_simple_test.
        Checks if both alarms are firing and if alarm 1 fires approximately
        twice as often as alarm 2.
        """
        alarm_times = defaultdict(list)
        logging.debug(f"Analyzing output lines: {lines}")

        # Regular expression to match the output lines
        pattern = re.compile(r"^(\d+)\s+(\d+)\s+(\d+)$")

        for line in lines:
            match = pattern.match(line)
            if match:
                alarm_index = int(match.group(1))
                now = int(match.group(2))
                expiration = int(match.group(3))
                alarm_times[alarm_index].append(now)
            else:
                logging.debug(f"Ignoring non-matching line: {line}")

        logging.info(f"Alarm times: {dict(alarm_times)}")
        # Check if both alarms are present
        if 1 not in alarm_times or 2 not in alarm_times:
            logging.error("Not all alarms are present in the output")
            return False

        # Get the counts
        count_alarm_1 = len(alarm_times[1])
        count_alarm_2 = len(alarm_times[2])

        logging.info(f"Alarm 1 fired {count_alarm_1} times")
        logging.info(f"Alarm 2 fired {count_alarm_2} times")

        # Check if alarm 1 fires approximately twice as often as alarm 2
        ratio = count_alarm_1 / count_alarm_2
        if ratio < 1.5 or ratio > 2.5:
            logging.error(
                f"Alarm 1 did not fire approximately twice as often as Alarm 2. Ratio: {ratio}"
            )
            return False

        logging.info("Alarms are firing as expected")
        return True


class WaitForConsoleMessageTest(OneshotTest):
    def __init__(self, apps, message):
        super().__init__(apps)
        self.message = message

    def oneshot_test(self, serial, gpio, board):
        logging.info(f"Waiting for message: '{self.message}'")
        output = serial.expect(self.message, timeout=10)
        if output:
            logging.info(f"Received expected message: '{self.message}'")
        else:
            logging.error(f"Did not receive expected message: '{self.message}'")


# Example usage:
# test = WaitForConsoleMessageTest(["c_hello"], "Hello World")
