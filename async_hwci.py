import os
import subprocess
import logging
import sys
import argparse
import serial
import serial.tools.list_ports
import time
import asyncio
import pexpect
import fdpexpect
from contextlib import contextmanager

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s - %(levelname)s - %(message)s",
)


def run_command(command, timeout=None, capture_output=True):
    if isinstance(command, str):
        command = command.split()
    try:
        logging.info(f"Running command: {' '.join(command)}")
        if capture_output:
            result = subprocess.run(
                command, check=True, capture_output=True, text=True, timeout=timeout
            )
            logging.debug(f"Command stdout: {result.stdout}")
            logging.debug(f"Command stderr: {result.stderr}")
            return result.stdout, result.stderr
        else:
            subprocess.run(command, check=True, timeout=timeout)
            return None, None
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed: {e}")
        logging.error(f"Stdout: {e.stdout}")
        logging.error(f"Stderr: {e.stderr}")
        raise
    except subprocess.TimeoutExpired:
        logging.error(f"Command timed out after {timeout} seconds")
        return None, None


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
                "adapter driver jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit",
            ]
        )
        logging.info("Flashing Tock kernel")
        run_command("make flash-openocd")
    logging.info("Finished flash_kernel function")


def install_apps(apps, target, port):
    logging.info(f"Starting install_apps function with apps: {apps}, target: {target}")
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
                    f"tockloader install --board nrf52dk --openocd build/{app}.tab"
                )
                logging.info(f"Enabling app: {app}")
                run_command(f"tockloader enable-app {app}")

        logging.info("Listing installed apps")
        run_command(f"tockloader list")
    logging.info("Finished install_apps function")


def get_serial_ports():
    logging.info("Getting list of serial ports")
    ports = list(serial.tools.list_ports.comports())
    logging.info(f"Found serial ports: {[port.device for port in ports]}")
    return ports


async def listen_serial_port(port_device, analysis_func=None, timeout=500):
    logging.info(
        f"Starting to listen on serial port {port_device} with timeout {timeout} seconds"
    )
    try:
        ser = serial.Serial(port_device, baudrate=115200, timeout=1)
        child = fdpexpect.fdspawn(ser.fileno())
        start_time = time.time()
        output_lines = []
        print(f"Listening on serial port, timeout: {timeout}")

        while time.time() - start_time < timeout:
            try:
                await child.expect(".*\r\n", timeout=1, async_=True)
                line = child.match.group(0).decode("utf-8", errors="replace").strip()
                logging.info(f"SERIAL PORT OUTPUT: {line}")
                output_lines.append(line)
                if analysis_func and analysis_func(output_lines):
                    logging.info("Analysis function returned True, stopping listener")
                    break
            except pexpect.TIMEOUT:
                pass
            except pexpect.EOF:
                break

        ser.close()
        logging.info("Finished listening on serial port")
        return True
    except Exception as e:
        logging.error(f"Error in listen_serial_port: {e}")
        return False


def analyze_hello_world_output(output_lines):
    return any("Hello World!" in line for line in output_lines)


def analyze_multi_alarm_output(output_lines):
    """
    Analyzes the output lines from the multi_alarm_simple_test.
    Checks if both alarms are firing and if alarm 1 fires approximately
    twice as often as alarm 2.
    """
    from collections import defaultdict
    import re

    alarm_times = defaultdict(list)
    logging.debug(f"Analyzing output lines: {output_lines}")

    # Regular expression to match the output lines
    pattern = re.compile(r"^(\d+)\s+(\d+)\s+(\d+)$")

    for line in output_lines:
        match = pattern.match(line)
        if match:
            alarm_index = int(match.group(1))
            now = int(match.group(2))
            expiration = int(match.group(3))
            alarm_times[alarm_index].append(now)
        else:
            logging.debug(f"Ignoring non-matching line: {line}")

    logging.info(f"Alarm times: {alarm_times}")
    # Check if both alarms are present
    if 1 not in alarm_times or 2 not in alarm_times:
        logging.error("Not all alarms are present in the output")
        return False

    # Get the counts
    count_alarm_1 = len(alarm_times[1])
    count_alarm_2 = len(alarm_times[2])

    logging.info(f"Alarm 1 fired {count_alarm_1} times")
    logging.info(f"Alarm 2 fired {count_alarm_2} times")

    # Check if alarm 1 fires approximately twice as often as alarm 2
    ratio = count_alarm_1 / count_alarm_2
    if ratio < 1.5 or ratio > 2.5:
        logging.error(
            f"Alarm 1 did not fire approximately twice as often as Alarm 2. Ratio: {ratio}"
        )
        return False

    logging.info("Alarms are firing as expected")
    return True


async def main():
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

        # get the serial port #TODO choose intelligently based on which says J-Link
        if not args.port:
            ports = get_serial_ports()
            if not ports:
                logging.error("No serial ports found")
                return
            args.port = ports[1].device
            logging.info(f"Automatically selected port: {args.port}")

        # Determine which apps to install and the analysis function
        if args.test == "hello_world":
            apps = ["c_hello"]
            analysis_func = analyze_hello_world_output
        elif args.test == "multi_alarm_simple_test":
            apps = ["multi_alarm_simple_test"]
            analysis_func = analyze_multi_alarm_output
        else:
            logging.error(f"Unknown test type: {args.test}")
            return

        # Start listening in the background
        listener_task = asyncio.create_task(
            listen_serial_port(args.port, analysis_func, 1000)
        )

        # Wait a bit to ensure the listener is ready
        await asyncio.sleep(2)

        # Install apps
        install_apps(apps, args.target, args.port)

        # Wait for the listener task to finish
        await listener_task
        logging.info("Listener task finished successfully")

        logging.info("Main function completed")

    except Exception as e:
        logging.exception("An error occurred during script execution")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
