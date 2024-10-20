# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.

from utils.test_helpers import WaitForConsoleMessageTest

test = WaitForConsoleMessageTest(["c_hello"], "Hello World!")
