import os
import subprocess
import logging
import sys
import argparse
import serial.tools.list_ports
import asyncio
import time
import re
from contextlib import contextmanager


### HW CI TEST multi_alarm_simple default test wip

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s - %(levelname)s - %(message)s",
)


def run_command(command, cwd=None):
    logging.info(f"Running command: {' '.join(command)}")
    process = subprocess.Popen(
        command,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    for line in process.stdout:
        print(line, end="")
        sys.stdout.flush()
    process.wait()
    if process.returncode != 0:
        logging.error(f"Command failed with return code {process.returncode}")
        raise subprocess.CalledProcessError(process.returncode, command)


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
        run_command(["git", "clone", "https://github.com/tock/tock"])

    with change_directory("tock/boards/nordic/nrf52840dk"):
        run_command(
            [
                "openocd",
                "-c",
                "interface jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit",
            ]
        )
        run_command(["make", "flash-openocd"])


def install_apps(apps, target, port):
    if not os.path.exists("libtock-c"):
        run_command("git clone https://github.com/tock/libtock-c")

    os.chdir("libtock-c")
    for app in apps:
        app_dir = f"examples/{app}"
        if app == "multi_alarm_simple_test":
            app_dir = f"examples/tests/{app}"
        if not os.path.exists(app_dir):
            logging.error(f"App directory {app_dir} not found")
            continue

        os.chdir(app_dir)
        logging.info(f"Changed directory to: {os.getcwd()}")
        run_command(f"make TOCK_TARGETS={target}")
        run_command(
            f"tockloader install --port {port} --board nrf52dk --openocd build/{app}.tab"
        )
        run_command(f"tockloader enable-app {app} --port {port}")
        os.chdir("../../")

    run_command(f"tockloader list --port {port}")
    os.chdir("../../")


def get_serial_ports():
    return list(serial.tools.list_ports.comports())


async def listen_for_output(command, analysis_func=None, timeout=60):
    process = await asyncio.create_subprocess_shell(
        command,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )

    logging.info(f"Listening for output from command: {command}")

    output_lines = []

    try:

        async def read_stream(stream, name):
            while True:
                line = await stream.readline()
                if not line:
                    break  # EOF
                decoded_line = line.decode().strip()
                logging.info(f"{name}: {decoded_line}")
                output_lines.append(decoded_line)

        stdout_task = asyncio.create_task(read_stream(process.stdout, "STDOUT"))
        stderr_task = asyncio.create_task(read_stream(process.stderr, "STDERR"))

        await asyncio.wait(
            [stdout_task, stderr_task],
            timeout=timeout,
            return_when=asyncio.ALL_COMPLETED,
        )

        if analysis_func:
            return analysis_func(output_lines)
        else:
            return output_lines

    except asyncio.TimeoutError:
        logging.error(f"Timeout expired after {timeout} seconds")
        return None
    finally:
        process.kill()
        await process.wait()


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


async def main():
    parser = argparse.ArgumentParser(description="Run tests on Tock OS")
    parser.add_argument("--port", help="Serial port to use (e.g., /dev/ttyACM0)")
    parser.add_argument(
        "--test",
        choices=["hello_world", "multi_alarm_simple_test"],
        default="hello_world",
        help="Test to run (hello_world or multi_alarm_simple_test)",
    )
    parser.add_argument(
        "--target", default="cortex-m4", help="Target architecture (e.g., cortex-m4)"
    )
    args = parser.parse_args()

    logging.info(f"Running test: {args.test}")

    try:
        flash_kernel()

        if args.port:
            port = args.port
        else:
            ports = get_serial_ports()
            if not ports:
                logging.error("No serial ports found")
                return
            port = ports[0].device
            logging.info(f"Automatically selected port: {port}")

        if args.test == "hello_world":
            apps = ["c_hello"]
            search_text = "Hello World!"
            analysis_func = lambda output: search_text in "\n".join(output)
        elif args.test == "multi_alarm_simple_test":
            apps = ["multi_alarm_simple_test"]
            analysis_func = analyze_multi_alarm_output
        else:
            logging.error(f"Unknown test type: {args.test}")
            return

        install_apps(apps, args.target, port)

        await asyncio.sleep(1)

        output = await listen_for_output(
            f"tockloader listen --port {port} --no-terminal",
            analysis_func=analysis_func,
            timeout=60,
        )

        if output is True:
            logging.info("Test completed successfully")
        else:
            logging.error("Test failed")

    except Exception as e:
        logging.exception("An error occurred during script execution")


if __name__ == "__main__":
    try:
        asyncio.run(main())
        sys.exit(0)
    except Exception as e:
        logging.exception("An error occurred during script execution")
        sys.exit(1)
