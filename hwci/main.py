# main.py

import argparse
import logging
import importlib.util
import sys
from serial_port import SerialPort


def main():
    parser = argparse.ArgumentParser(description="Run tests on Tock OS")
    parser.add_argument("--board", required=True, help="Path to the board module")
    parser.add_argument("--test", required=True, help="Path to the test module")
    args = parser.parse_args()

    # Set up logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s - %(levelname)s - %(message)s",
    )

    # 1. Load board module
    board_spec = importlib.util.spec_from_file_location("board_module", args.board)
    board_module = importlib.util.module_from_spec(board_spec)
    board_spec.loader.exec_module(board_module)
    # Instantiate the board class (assuming it's named Nrf52dk)
    if hasattr(board_module, "board"):
        board = board_module.board
    else:
        logging.error("No board class found in the specified board module")
        sys.exit(1)

    # 2. Get serial port information
    serial_port_name = board.get_uart_port()
    baudrate = board.get_uart_baudrate()
    logging.info(f"Using serial port: {serial_port_name} at baudrate {baudrate}")

    # 3. Open serial port, instantiate pexpect
    serial = SerialPort(serial_port_name, baudrate)

    # 4. TBD: GPIO (set to None for now)
    gpio = None

    # 5. Load test module, run test function
    test_spec = importlib.util.spec_from_file_location("test_module", args.test)
    test_module = importlib.util.module_from_spec(test_spec)
    test_spec.loader.exec_module(test_module)
    if hasattr(test_module, "test"):
        test = test_module.test
    else:
        logging.error("No test variable found in the specified test module")
        sys.exit(1)

    # Run the test
    try:
        test.test(serial, gpio, board)
        logging.info("Test completed successfully")
    except Exception as e:
        logging.exception("An error occurred during test execution")
        sys.exit(1)
    finally:
        serial.close()


if __name__ == "__main__":
    main()
