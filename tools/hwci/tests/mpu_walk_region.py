from tests.console_hello_test import AnalyzeConsoleTest


class MpuWalkRegionTest(AnalyzeConsoleTest):
    def __init__(self):
        super().__init__(apps=["tests/mpu/mpu_walk_region"])

    def analyze(self, lines):
        for line in lines:
            if "[TEST] MPU Walk Regions" in line:
                # Test started successfully
                return
        raise Exception("MPU Walk Region test did not start")


test = MpuWalkRegionTest()
