import os
import subprocess
import logging
import sys
import argparse
import serial.tools.list_ports
import time
import re
from contextlib import contextmanager

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s - %(levelname)s - %(message)s",
)


def run_command(command):
    if isinstance(command, str):
        command = command.split()
    try:
        result = subprocess.run(command, check=True, capture_output=True, text=True)
        return result.stdout
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed: {e}")
        logging.error(f"Stderr: {e.stderr}")
        raise


@contextmanager
def change_directory(new_dir):
    previous_dir = os.getcwd()
    os.chdir(new_dir)
    logging.info(f"Changed directory to: {os.getcwd()}")
    try:
        yield
    finally:
        os.chdir(previous_dir)
        logging.info(f"Reverted to directory: {os.getcwd()}")


def flash_kernel():
    if not os.path.exists("tock"):
        run_command("git clone https://github.com/tock/tock")

    with change_directory("tock/boards/nordic/nrf52840dk"):
        # Fix: Use a list of arguments instead of a single string
        run_command(
            [
                "openocd",
                "-c",
                "interface jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit",
            ]
        )
        run_command("make flash-openocd")


def install_apps(apps, target, port):
    if not os.path.exists("libtock-c"):
        run_command("git clone https://github.com/tock/libtock-c")

    with change_directory("libtock-c"):
        for app in apps:
            app_dir = (
                f"examples/{app}"
                if app != "multi_alarm_simple_test"
                else f"examples/tests/{app}"
            )
            if not os.path.exists(app_dir):
                logging.error(f"App directory {app_dir} not found")
                continue

            with change_directory(app_dir):
                run_command(f"make TOCK_TARGETS={target}")
                run_command(
                    f"tockloader install --port {port} --board nrf52dk --openocd build/{app}.tab"
                )
                run_command(f"tockloader enable-app {app} --port {port}")

        run_command(f"tockloader list --port {port}")


def get_serial_ports():
    return list(serial.tools.list_ports.comports())


def listen_for_output(port, analysis_func=None, timeout=60):
    with serial.Serial(port, 115200, timeout=1) as ser:
        start_time = time.time()
        output_lines = []
        while time.time() - start_time < timeout:
            if ser.in_waiting:
                line = ser.readline().decode().strip()
                logging.info(f"SERIAL: {line}")
                output_lines.append(line)
                if analysis_func and analysis_func(output_lines):
                    return True
        return False


def analyze_multi_alarm_output(output_lines):
    pattern = re.compile(r"^Alarm (\d): Time (\d+), Expiration (\d+)$")
    alarm_counts = {1: 0, 2: 0}
    last_time = -1

    for line in output_lines:
        match = pattern.match(line)
        if match:
            alarm_id, current_time, expiration = map(int, match.groups())
            alarm_counts[alarm_id] += 1

            if current_time < last_time:
                logging.error("Time went backwards")
                return False
            last_time = current_time

    alarm_1_count = alarm_counts[1]
    alarm_2_count = alarm_counts[2]

    if alarm_1_count == 0 or alarm_2_count == 0:
        logging.error("One or both alarms did not fire")
        return False

    if abs(alarm_1_count - 2 * alarm_2_count) > 1:
        logging.error(
            f"Alarm 1 ({alarm_1_count}) did not fire approximately twice as often as Alarm 2 ({alarm_2_count})"
        )
        return False

    logging.info(
        f"Test passed: Alarm 1 fired {alarm_1_count} times, Alarm 2 fired {alarm_2_count} times"
    )
    return True


def main():
    parser = argparse.ArgumentParser(description="Run tests on Tock OS")
    parser.add_argument("--port", help="Serial port to use (e.g., /dev/ttyACM0)")
    parser.add_argument(
        "--test",
        choices=["hello_world", "multi_alarm_simple_test"],
        default="hello_world",
        help="Test to run",
    )
    parser.add_argument("--target", default="cortex-m4", help="Target architecture")
    args = parser.parse_args()

    logging.info(f"Running test: {args.test}")

    try:
        flash_kernel()

        if not args.port:
            ports = get_serial_ports()
            if not ports:
                logging.error("No serial ports found")
                return
            args.port = ports[0].device
            logging.info(f"Automatically selected port: {args.port}")

        if args.test == "hello_world":
            apps = ["c_hello"]
            analysis_func = lambda output: "Hello World!" in "\n".join(output)
        elif args.test == "multi_alarm_simple_test":
            apps = ["multi_alarm_simple_test"]
            analysis_func = analyze_multi_alarm_output
        else:
            logging.error(f"Unknown test type: {args.test}")
            return

        install_apps(apps, args.target, args.port)

        time.sleep(10)  # Wait for app to start

        if listen_for_output(args.port, analysis_func=analysis_func):
            logging.info("Test completed successfully")
        else:
            logging.error("Test failed")

    except Exception as e:
        logging.exception("An error occurred during script execution")
        sys.exit(1)


if __name__ == "__main__":
    main()
