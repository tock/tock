# nrf52dk.py

from tockloader_board import TockloaderBoard
import os
import subprocess
import logging
from contextlib import contextmanager
import serial.tools.list_ports


class Nrf52dk(TockloaderBoard):
    def __init__(self):
        super().__init__()
        self.arch = "cortex-m4"
        self.kernel_board_path = "tock/boards/nordic/nrf52840dk"
        self.uart_port = self.get_uart_port()
        self.uart_baudrate = self.get_uart_baudrate()
        self.openocd_board = "nrf52dk"
        self.board = "nrf52dk"

    def get_uart_port(self):
        logging.info("Getting list of serial ports")
        ports = list(serial.tools.list_ports.comports())
        for port in ports:
            if "J-Link" in port.description:
                logging.info(f"Found J-Link port: {port.device}")
                return port.device
        if ports:
            logging.info(f"Automatically selected port: {ports[0].device}")
            return ports[0].device
        else:
            logging.error("No serial ports found")
            raise Exception("No serial ports found")

    def get_uart_baudrate(self):
        return 115200  # Default baudrate for the board

    def erase_board(self):
        logging.info("Erasing the board")
        command = [
            "openocd",
            "-c",
            "adapter driver jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit",
        ]
        subprocess.run(command, check=True)

    def flash_kernel(self):
        logging.info("Flashing the Tock OS kernel")
        if not os.path.exists("tock"):
            logging.info("Cloning Tock repository")
            subprocess.run(["git", "clone", "https://github.com/tock/tock"], check=True)
        with self.change_directory(self.kernel_board_path):
            subprocess.run(["make", "flash-openocd"], check=True)

    # The flash_app method is inherited from TockloaderBoard

    @contextmanager
    def change_directory(self, new_dir):
        previous_dir = os.getcwd()
        os.chdir(new_dir)
        logging.info(f"Changed directory to: {os.getcwd()}")
        try:
            yield
        finally:
            os.chdir(previous_dir)
            logging.info(f"Reverted to directory: {os.getcwd()}")


board = Nrf52dk()
