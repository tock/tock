import os
import subprocess
import logging
import time
# import serial
import sys
# import glob

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
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


# def wait_for_console_output(port, expected_string, timeout=60):
#     logging.info(f"Waiting for '{expected_string}' on console...")
#     ser = serial.Serial(port, 115200, timeout=1)
#     start_time = time.time()
# 
#     while time.time() - start_time < timeout:
#         if ser.in_waiting:
#             line = ser.readline().decode("utf-8").strip()
#             print(f"Console output: {line}")  # Print in real-time
#             sys.stdout.flush()
#             if expected_string in line:
#                 logging.info(f"Found expected string: '{expected_string}'")
#                 ser.close()
#                 return True
# 
#     logging.error(
#         f"Timeout: Did not receive '{expected_string}' within {timeout} seconds"
#     )
#     ser.close()
#     return False


#def find_serial_ports():
#    if sys.platform.startswith("win"):
#        ports = ["COM%s" % (i + 1) for i in range(256)]
#    elif sys.platform.startswith("linux") or sys.platform.startswith("cygwin"):
#        ports = glob.glob("/dev/tty[A-Za-z]*")
#    elif sys.platform.startswith("darwin"):
#        ports = glob.glob("/dev/tty.*")
#    else:
#        raise EnvironmentError("Unsupported platform")
#
#    result = []
#    for port in ports:
#        try:
#            s = serial.Serial(port)
#            s.close()
#            result.append(port)
#        except (OSError, serial.SerialException):
#            pass
#    return result


def main():
    # run_command("git clone https://github.com/tock/tock.git")

    os.chdir("tock/boards/nordic/nrf52840dk")
    logging.info(f"Changed directory to: {os.getcwd()}")

    run_command(
        "openocd -c 'interface jlink; transport select swd; source [find target/nrf52.cfg]; init; nrf52_recover; exit'"
    )

    run_command("make flash-openocd")

    os.chdir("../../../../")
    logging.info(f"Changed directory to: {os.getcwd()}")

    run_command("git clone https://github.com/tock/libtock-c")

    os.chdir("libtock-c/examples/c_hello")
    logging.info(f"Changed directory to: {os.getcwd()}")

    run_command("make TOCK_TARGETS=cortex-m4")

    run_command("tockloader install --board nrf52dk --openocd build/c_hello.tab")

    run_command("tockloader enable-app c_hello")

    run_command("tockloader list")

    # available_ports = find_serial_ports()
    # if not available_ports:
    #     logging.error("No serial ports found. Make sure your board is connected.")
    #     return

    # logging.info(f"Available serial ports: {', '.join(available_ports)}")

    # for port in available_ports:
    #     logging.info(f"Attempting to communicate on port: {port}")
    #     if wait_for_console_output(port, "Hello World!"):
    #         logging.info(
    #             f"Successfully received 'Hello World!' from the board on port {port}"
    #         )
    #         break
    # else:
    #     logging.error("Failed to receive 'Hello World!' on any available port")

    logging.info("Script completed successfully")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        logging.exception("An error occurred during script execution")
