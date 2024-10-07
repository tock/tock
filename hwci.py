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
        logging.info(f"Running command: {' '.join(command)}")
        result = subprocess.run(command, check=True, capture_output=True, text=True)
        logging.debug(f"Command output: {result.stdout}")
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
    logging.info("Starting flash_kernel function")
    if not os.path.exists("tock"):
        logging.info("Cloning Tock repository")
        run_command("git clone https://github.com/tock/tock")
    else:
        logging.info("Tock repository already exists")

    with change_directory("tock/boards/nordic/nrf52840dk"):
        logging.info("Attempting to recover nRF52 board")
        run_command(
            [
                "openocd",
                "-c",
                "interface jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit",
            ]
        )
        logging.info("Flashing Tock kernel")
        run_command("make flash-openocd")
    logging.info("Finished flash_kernel function")


def install_apps(apps, target, port):
    logging.info(
        f"Starting install_apps function with apps: {apps}, target: {target}, port: {port}"
    )
    if not os.path.exists("libtock-c"):
        logging.info("Cloning libtock-c repository")
        run_command("git clone https://github.com/tock/libtock-c")
    else:
        logging.info("libtock-c repository already exists")

    with change_directory("libtock-c"):
        for app in apps:
            app_dir = (
                f"examples/{app}"
                if app != "multi_alarm_simple_test"
                else f"examples/tests/{app}"
            )
            logging.info(f"Processing app: {app} in directory: {app_dir}")
            if not os.path.exists(app_dir):
                logging.error(f"App directory {app_dir} not found")
                continue

            with change_directory(app_dir):
                logging.info(f"Building app: {app}")
                run_command(f"make TOCK_TARGETS={target}")
                logging.info(f"Installing app: {app}")
                run_command(
                    f"tockloader install --port {port} --board nrf52dk --openocd build/{app}.tab"
                )
                logging.info(f"Enabling app: {app}")
                run_command(f"tockloader enable-app {app} --port {port}")

        logging.info("Listing installed apps")
        run_command(f"tockloader list --port {port}")
    logging.info("Finished install_apps function")


def get_serial_ports():
    logging.info("Getting list of serial ports")
    ports = list(serial.tools.list_ports.comports())
    logging.info(f"Found serial ports: {[port.device for port in ports]}")
    return ports


def listen_for_output(port, analysis_func=None, timeout=60):
    logging.info(
        f"Starting to listen for output on port {port} with timeout {timeout} seconds"
    )
    with serial.Serial(port, 115200, timeout=1) as ser:
        start_time = time.time()
        output_lines = []
        while time.time() - start_time < timeout:
            if ser.in_waiting:
                line = ser.readline().decode().strip()
                logging.info(f"SERIAL: {line}")
                output_lines.append(line)
                if analysis_func and analysis_func(output_lines):
                    logging.info(
                        "Analysis function returned True, ending listen_for_output"
                    )
                    return True
        logging.warning(f"Timeout reached after {timeout} seconds")
        return False


def analyze_multi_alarm_output(output_lines):
    logging.info("Starting analyze_multi_alarm_output function")
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

    logging.info(
        f"Alarm 1 fired {alarm_1_count} times, Alarm 2 fired {alarm_2_count} times"
    )

    if alarm_1_count == 0 or alarm_2_count == 0:
        logging.error("One or both alarms did not fire")
        return False

    if abs(alarm_1_count - 2 * alarm_2_count) > 1:
        logging.error(
            f"Alarm 1 ({alarm_1_count}) did not fire approximately twice as often as Alarm 2 ({alarm_2_count})"
        )
        return False

    logging.info("Test passed: Alarms fired as expected")
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

    logging.info(
        f"Starting main function with test: {args.test}, target: {args.target}"
    )

    try:
        flash_kernel()

        if not args.port:
            ports = get_serial_ports()
            if not ports:
                logging.error("No serial ports found")
                return
            args.port = ports[1].device
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

        logging.info("Waiting 10 seconds for app to start")
        time.sleep(10)  # Wait for app to start

        logging.info("Starting to listen for output")
        if listen_for_output(args.port, analysis_func=analysis_func):
            logging.info("Test completed successfully")
        else:
            logging.error("Test failed")

    except Exception as e:
        logging.exception("An error occurred during script execution")
        sys.exit(1)

    logging.info("Main function completed")


if __name__ == "__main__":
    main()
