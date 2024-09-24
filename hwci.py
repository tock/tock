import os
import subprocess
import logging
import sys
import argparse
import serial.tools.list_ports
import asyncio
import time
import re

### HW CI TEST multi_alarm_simple default test

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s - %(levelname)s - %(message)s",
)


def run_command(command, cwd=None):
    logging.info(f"Running command: {command}")
    process = subprocess.Popen(
        command,
        cwd=cwd,
        shell=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        universal_newlines=True,
    )
    for line in process.stdout:
        print(line, end="")
        sys.stdout.flush()
    process.wait()
    if process.returncode != 0:
        logging.error(f"Command failed with return code {process.returncode}")
        raise subprocess.CalledProcessError(process.returncode, command)


def flash_kernel():
    if not os.path.exists("tock"):
        run_command("git clone https://github.com/tock/tock")

    os.chdir("tock/boards/nordic/nrf52840dk")
    logging.info(f"Changed directory to: {os.getcwd()}")
    run_command(
        "openocd -c 'interface jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit'"
    )
    run_command("make flash-openocd")
    os.chdir("../../../../")


def install_apps(apps, target):
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
        run_command(f"tockloader install --board nrf52dk --openocd build/{app}.tab")
        run_command(f"tockloader enable-app {app}")
        os.chdir("../../")

    run_command("tockloader list")
    os.chdir("../../")


def get_serial_ports():
    return list(serial.tools.list_ports.comports())


async def listen_for_output(port, test_type):
    process = await asyncio.create_subprocess_shell(
        f"tockloader listen --port {port}",
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )

    output_lines = []
    try:
        while True:
            line = await process.stdout.readline()
            if not line:
                break
            line = line.decode().strip()
            print(f"Raw output: {line}")
            logging.debug(f"Received line: {line}")
            output_lines.append(line)
    except asyncio.TimeoutError:
        logging.error("Test timed out")
    finally:
        process.terminate()
        await process.wait()

    return analyze_output(output_lines, test_type)


def analyze_output(output_lines, test_type):
    if test_type == "hello_world":
        return "Hello World!" in output_lines
    elif test_type == "multi_alarm_simple_test":
        return analyze_multi_alarm_output(output_lines)
    else:
        logging.error(f"Unknown test type: {test_type}")
        return False


def analyze_multi_alarm_output(output_lines):
    pattern = re.compile(r"^(\d) (\d+) (\d+)$")
    alarm_1_count = 0
    alarm_2_count = 0
    last_time = 0

    for line in output_lines:
        match = pattern.match(line)
        if match:
            alarm_id, current_time, expiration = map(int, match.groups())
            if alarm_id == 1:
                alarm_1_count += 1
            elif alarm_id == 2:
                alarm_2_count += 1

            if int(current_time) < last_time:
                logging.error("Time went backwards")
                return False
            last_time = int(current_time)

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
        default="multi_alarm_simple_test",
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
        elif args.test == "multi_alarm":
            apps = ["multi_alarm_simple_test"]
        else:
            logging.error(f"Unknown test type: {args.test}")
            return

        listen_task = asyncio.create_task(listen_for_output(port, args.test))

        await asyncio.sleep(1)

        install_apps(apps, args.target)

        try:
            test_result = await asyncio.wait_for(listen_task, timeout=600)
            if test_result:
                logging.info("Test completed successfully")
            else:
                logging.error("Test failed")
        except asyncio.TimeoutError:
            logging.error("Test timed out waiting for result")

    except Exception as e:
        logging.exception("An error occurred during script execution")


if __name__ == "__main__":
    asyncio.run(main())
