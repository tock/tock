import os
import serial
import time

INSTRUMENTED_TOCK_DIR = "../../tock/boards/nordic/nrf52840dk"
INSTRUMENTED_VTOCK_DIR = "../boards/nordic/nrf52840dk"
LIBTOCKC_DIR = "../../libtock-c"
TEST_LENGTH = 5

def install_and_run_apps(serial_port, test_app_paths):
    output = []
    package_names = []
    libtockc_dir_abs = os.path.abspath(LIBTOCKC_DIR)

    # First, install and run all apps
    for i, test_app_path in enumerate(test_app_paths):
        # Get folder name for fallback package name
        folder_name = os.path.basename(os.path.normpath(test_app_path))
        
        # Extract package name from Makefile
        path = os.path.join(libtockc_dir_abs, test_app_path)

        # Parse the Makefile to get PACKAGE_NAME
        package_name = None
        makefile_path = os.path.join(path, "Makefile")
        with open(makefile_path, 'r') as makefile:
            for line in makefile:
                if line.strip().startswith("PACKAGE_NAME"):
                    package_name = line.split("=")[1].strip()
                    break
        
        # If no PACKAGE_NAME found, use folder name
        if not package_name:
            package_name = folder_name
            
        package_names.append(package_name)

        # Read the output from the serial port. Stop after TEST_LENGTH seconds. Only do this if this is the last app to install
        if i == len(test_app_paths) - 1:
            # Open serial port
            ser = serial.Serial(serial_port, 115200, timeout=15)

            # Install the app
            os.system(f"cd {path} && make install")

            start_time = time.time()
            while True:
                # Read a line from the serial port
                line = ser.readline().decode('utf-8').strip()
                # Add the line to the output
                output.append(line)
                # Check if TEST_LENGTH seconds have passed
                if time.time() - start_time > TEST_LENGTH:
                    break
        
            # Close the serial port
            ser.close()
        else:
            # only install, don't record anything
            # Install the app
            os.system(f"cd {path} && make install")
         
    # After all apps have run, uninstall them all at once
    if package_names:
        os.system("tockloader uninstall " + " ".join(package_names))
    
    return output


def run_tests(kernel_path, serial_port, tests_to_run):
    output = {}
    abs_dir = os.path.abspath(kernel_path)
    os.system(f"cd {abs_dir} && make install")

    for test in tests_to_run:
        name = ",".join(test)
        output[name] = install_and_run_apps(serial_port, test)
        
    return output


def compare_outputs(vtock_output, tock_output):
    print("Comparing Outputs!")
    print()
    for test in vtock_output:
        vtock_test_output = "\n".join(vtock_output[test]).strip()
        tock_test_output = "\n".join(tock_output[test]).strip()
        if vtock_test_output != tock_test_output:
            print(f"Diff in {test}") 
            print(f"VTock output:\n{vtock_test_output}")
            print(f"Tock output:\n{tock_test_output}")
            print()
            print()


TESTS_TO_RUN = [
    ["examples/sensors"], 
    ["examples/c_hello"],
    ["examples/tests/printf_long"],
    ["examples/tests/console/console_recv_short"],
    ["examples/tests/console/console_recv_long"],
    ["examples/tests/console/console_timeout"],
    ["examples/blink"],
    ["examples/rot13_client", "examples/rot13_service"],
    ["examples/tests/malloc_test01"],
    ["examples/tests/malloc_test02"],
    ["examples/tests/stack_size_test01"],
    ["examples/tests/stack_size_test02"],
    ["examples/tests/mpu/mpu_stack_growth"],
    ["examples/tests/mpu/mpu_walk_region"],
    ["examples/tests/adc/adc"],
    ["examples/tutorials/05_ipc/led", "examples/tutorials/05_ipc/rng", "examples/tutorials/05_ipc/logic"],
    ["examples/ble_advertising"],
    ["vtock_brk_bench"]
]


if __name__ == "__main__":
    serial_port = "/dev/cu.usbmodem0010502398691"
    vtock_output = run_tests(INSTRUMENTED_VTOCK_DIR, serial_port, TESTS_TO_RUN)
    tock_output = run_tests(INSTRUMENTED_TOCK_DIR, serial_port, TESTS_TO_RUN)
    compare_outputs(vtock_output, tock_output)
