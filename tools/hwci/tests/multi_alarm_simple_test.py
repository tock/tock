from tests.console_hello_test import AnalyzeConsoleTest
import logging


class MultiAlarmSimpleTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["tests/multi_alarm_simple_test"])

    def analyze(self, lines):
        # Initialize counts for each alarm
        alarm_counts = {1: 0, 2: 0}

        for line in lines:
            tokens = line.strip().split()
            if len(tokens) >= 3:
                try:
                    alarm_index = int(tokens[0])
                    # Optional: parse timestamps if needed
                    # now = int(tokens[1])
                    # expiration = int(tokens[2])

                    # Record counts
                    if alarm_index in [1, 2]:
                        alarm_counts[alarm_index] += 1
                except ValueError:
                    continue  # Skip lines that don't parse correctly

        logging.info(f"Alarm counts: {alarm_counts}")
        count1 = alarm_counts.get(1, 0)
        count2 = alarm_counts.get(2, 0)
        if count1 < 2 or count2 < 1:
            raise Exception("MultiAlarmSimpleTest failed: Not enough alarms fired")
        if count1 < 2 * count2:
            raise Exception(
                "MultiAlarmSimpleTest failed: Alarm 1 did not fire at least twice as often as Alarm 2"
            )

        logging.info("MultiAlarmSimpleTest passed")


test = MultiAlarmSimpleTest()
