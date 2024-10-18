from tests.console_hello_test import AnalyzeConsoleTest


class WhileOneTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["tests/whileone"])

    def analyze(self, lines):
        # This app does not produce output, but we can check for the absence of crashes
        for line in lines:
            if "Kernel panic" in line or "Fault" in line:
                raise Exception("App crashed with a fault")
        # If no faults, the test passes
        return


test = WhileOneTest()
