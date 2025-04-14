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
        print(test_app_path)
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
                print(line)
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
    output = []
    abs_dir = os.path.abspath(kernel_path)
    os.system(f"cd {abs_dir} && git fetch && git checkout memory-benchmarks && make install")

    for test in tests_to_run:
        output += install_and_run_apps(serial_port, test)
        
    return output


def outputs_into_avgs(output_list):
    average_hash_map = {}
    for item in output_list:
        # split by spaces
        parts = item.split(" ")
        if parts[0] == "[EVAL]":
            try: 
                fn_name = parts[1]
                mem_diff = int(parts[2])
            except:
                # if the value is not an int, skip it
                continue

            # check if the function name is in the hash map
            if fn_name not in average_hash_map:
                average_hash_map[fn_name] = (1, mem_diff)
            else:
                # get the current count and sum of cycles
                count, sum_mem_diff = average_hash_map[fn_name]

                # increment the count
                count += 1

                # add the cycles to the sum
                sum_mem_diff += mem_diff

                # update the hash map
                average_hash_map[fn_name] = (count, sum_mem_diff)

    for key, (count, sum_mem_diff) in average_hash_map.items():
        average_hash_map[key] = sum_mem_diff / count 

    return average_hash_map


TESTS_TO_RUN = [
    ["vtock_brk_bench"]
]


# Example usage
if __name__ == "__main__":
    os.system(f"cd {LIBTOCKC_DIR} && git fetch && git checkout vtock-benchmarks")
    serial_port = "/dev/cu.usbmodem0010502398691"
    vtock_output = run_tests(INSTRUMENTED_VTOCK_DIR, serial_port, TESTS_TO_RUN)
    tock_output = run_tests(INSTRUMENTED_TOCK_DIR, serial_port, TESTS_TO_RUN)
    vtock_avgs = outputs_into_avgs(vtock_output)
    tock_avgs = outputs_into_avgs(tock_output)
    for fn_name, vtock_avg in vtock_avgs.items():
        tock_avg = tock_avgs[fn_name]
        print("[{}] VTock: {} | Baseline: {} | Difference: {} Bytes | Percent Difference {:.2f}%".format(fn_name, vtock_avg, tock_avg, vtock_avg - tock_avg, (vtock_avg - tock_avg) / tock_avg * 100)) 
