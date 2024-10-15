# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2024.


class BoardHarness:
    arch = None
    kernel_board_path = None

    def __init__(self):
        self.serial = None
        self.gpio = None

    def get_uart_port(self):
        pass

    def get_uart_baudrate(self):
        pass

    def get_serial_port(self):
        pass

    def get_gpio_interface(self):
        pass

    def erase_board(self):
        pass

    def flash_kernel(self):
        pass

    def flash_app(self, app):
        pass
