from tests.console_hello_test import AnalyzeConsoleTest


class MpuStackGrowthTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["tests/mpu/mpu_stack_growth"])

    def analyze(self, lines):
        for line in lines:
            if "[TEST] MPU Stack Growth" in line:
                # Test started successfully
                return
        raise Exception("MPU Stack Growth test did not start")


test = MpuStackGrowthTest()
