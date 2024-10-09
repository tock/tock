import os
import subprocess
import logging
import sys
import argparse
import serial.tools.list_ports
import time
import re
import threading
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
        run_command(f"tockloader list")
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

    command = ["tockloader", "listen", "--port", port]
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            universal_newlines=True,
        )

        start_time = time.time()
        output_lines = []

        def read_stream(stream, prefix):
            for line in iter(stream.readline, ""):
                line = line.strip()
                logging.info(f"{prefix}: {line}")
                output_lines.append(line)
                if analysis_func and analysis_func(output_lines):
                    return True
            return False

        while time.time() - start_time < timeout:
            if read_stream(process.stdout, "TOCKLOADER STDOUT") or read_stream(
                process.stderr, "TOCKLOADER STDERR"
            ):
                logging.info(
                    "Analysis function returned True, ending listen_for_output"
                )
                process.terminate()
                return True

            if process.poll() is not None:
                break

        process.terminate()
        logging.warning(
            f"Tockloader listen ended or timed out after {time.time() - start_time:.2f} seconds"
        )
        return False

    except Exception as e:
        logging.error(f"Error in listen_for_output: {e}")
        return False


def analyze_hello_world_output(output_lines):
    return "Hello World!" in "\n".join(output_lines)


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
        # Flash the kernel first
        flash_kernel()

        # Now get the serial port
        if not args.port:
            ports = get_serial_ports()
            if not ports:
                logging.error("No serial ports found")
                return
            args.port = ports[0].device
            logging.info(f"Automatically selected port: {args.port}")

        # Determine which apps to install and the analysis function
        if args.test == "hello_world":
            apps = ["c_hello"]
            analysis_func = analyze_hello_world_output
        elif args.test == "multi_alarm_simple_test":
            apps = ["multi_alarm_simple_test"]
            analysis_func = None  # Implement analysis function if needed
        else:
            logging.error(f"Unknown test type: {args.test}")
            return

        # Start listening in a separate thread before installing apps
        listener_thread = threading.Thread(
            target=listen_for_output, args=(args.port, analysis_func, 60)
        )
        listener_thread.start()

        # Install apps
        install_apps(apps, args.target, args.port)

        # Wait for the listener thread to finish
        listener_thread.join()

        logging.info("Main function completed")

    except Exception as e:
        logging.exception("An error occurred during script execution")
        sys.exit(1)


if __name__ == "__main__":
    main()
