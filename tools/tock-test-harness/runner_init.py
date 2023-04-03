import os
import toml
from pathlib import Path
from git import Repo

# KNOWN_BOARDS is variable listing all known board model extracted from 
# tockloader, which can be found in tockloader repository 
# https://github.com/tock/tockloader/blob/master/tockloader/board_interface.py
KNOWN_BOARDS = {
    'hail': {'description': 'Hail development module.',
                'arch': 'cortex-m4',
                'jlink_device': 'ATSAM4LC8C',
                'page_size': 512},
    'imix': {'description': 'Low-power IoT research platform',
                'arch': 'cortex-m4',
                'jlink_device': 'ATSAM4LC8C',
                'page_size': 512},
    'nrf51dk': {'description': 'Nordic nRF51-based development kit',
                'arch': 'cortex-m0',
                'jlink_device': 'nrf51422',
                'page_size': 1024,
                'openocd': 'nordic_nrf51_dk.cfg',
                'openocd_options': ['workareazero']},
    'nrf52dk': {'description': 'Nordic nRF52-based development kit',
                'arch': 'cortex-m4',
                'jlink_device': 'nrf52',
                'page_size': 4096,
                'openocd': 'nordic_nrf52_dk.cfg'},
    'nano33ble': {'description': 'Arduino Nano 33 BLE board',
                    'arch': 'cortex-m4'},
    'launchxl-cc26x2r1': {'description': 'TI CC26x2-based launchpad',
                            'arch': 'cortex-m4',
                            'page_size': 512,
                            'jlink_device': 'cc2652r1f',
                            'jlink_speed': 4000,
                            'jlink_if': 'jtag',
                            'openocd': 'ti_cc26x2_launchpad.cfg',
                            'openocd_options': ['noreset', 'resume'],
                            'openocd_commands': {'program': 'flash write_image erase {{binary}} {address:#x};\
                                                            verify_image {{binary}} {address:#x};'}},
    'ek-tm4c1294xl': {'description': 'TI TM4C1294-based launchpad',
                        'arch': 'cortex-m4',
                        'page_size': 512,
                        'openocd': 'ek-tm4c1294xl.cfg'},
    'arty': {'description': 'Arty FPGA running SiFive RISC-V core',
                'arch': 'rv32imac',
                'apps_start_address': 0x40430000,
                # arty exposes just the flash to openocd, this does the mapping
                # from the address map to what openocd must use.
                'address_translator': lambda addr: addr - 0x40000000,
                'page_size': 512,
                'openocd': 'external', # No supported board in openocd proper
                'openocd_options': ['nocmdprefix'],
                'openocd_prefix': 'source [find interface/ftdi/digilent-hs1.cfg];\
                                ftdi_device_desc \\"Digilent USB Device\\";\
                                adapter_khz 10000;\
                                transport select jtag;\
                                source [find cpld/xilinx-xc7.cfg];\
                                source [find cpld/jtagspi.cfg];\
                                proc jtagspi_read {{fname offset len}} {{\
                                    global _FLASHNAME;\
                                    flash read_bank $_FLASHNAME $fname $offset $len;\
                                }};\
                                init;\
                                jtagspi_init 0 {bitfile};'
                                .format(bitfile=os.path.join( # Need path to bscan_spi_xc7a100t.bit
                                    os.path.dirname(os.path.realpath(__file__)),
                                    '..', 'bitfiles', 'bscan_spi_xc7a100t.bit')),
                'openocd_commands': {'program': 'jtagspi_program {{binary}} {address:#x};',
                                    'read': 'jtagspi_read {{binary}} {address:#x} {length};',
                                    'erase': 'flash fillb {address:#x} 0x00 512;'}},
    'stm32f3discovery': {'description': 'STM32F3-based Discovery Boards',
                            'arch': 'cortex-m4',
                            'apps_start_address': 0x08020000,
                            'page_size': 2048,
                            'openocd': 'external',
                            'openocd_prefix': 'interface hla; \
                                            hla_layout stlink; \
                                            hla_device_desc "ST-LINK/V2-1"; \
                                            hla_vid_pid 0x0483 0x374b; \
                                            set WORKAREASIZE 0xC000; \
                                            source [find target/stm32f3x.cfg];'},
    'stm32f4discovery': {'description': 'STM32F4-based Discovery Boards',
                            'arch': 'cortex-m4',
                            'apps_start_address': 0x08040000,
                            'page_size': 2048,
                            'openocd': 'external',
                            'openocd_prefix': 'interface hla; \
                                                hla_layout stlink; \
                                                hla_device_desc "ST-LINK/V2-1"; \
                                                hla_vid_pid 0x0483 0x374b; \
                                                set WORKAREASIZE 0x40000; \
                                                source [find target/stm32f4x.cfg];'},
    'nucleof4': {'description': 'STM32f4-based Nucleo development boards',
                    'arch': 'cortex-m4',
                    'apps_start_address': 0x08040000,
                    'page_size': 2048,
                    'openocd': 'st_nucleo_f4.cfg'},
    'hifive1': {'description': 'SiFive HiFive1 development board',
                'arch': 'rv32imac',
                'apps_start_address': 0x20430000,
                'page_size': 512,
                'openocd': 'sifive-hifive1.cfg'},
    'hifive1b': {'description': 'SiFive HiFive1b development board',
                    'arch': 'rv32imac',
                    'apps_start_address': 0x20040000,
                    'page_size': 512,
                    'jlink_device': 'FE310',
                    'jlink_if': 'jtag',
                    'openocd': 'sifive-hifive1-revb.cfg'},
    'edu-ciaa': {'description': 'Educational NXP board, from the CIAA project',
                    'arch': 'cortex-m4',
                    'page_size': 512,
                    'apps_start_address': 0x1a040000,
                    'openocd': 'ftdi_lpc4337.cfg',
                    'openocd_options': ['noreset'],
                    'openocd_commands': {'program': 'flash write_image erase {{binary}} {address:#x};verify_image {{binary}} {address:#x};',
                    'erase': 'flash fillb {address:#x} 0x00 512;'}},
    'microbit_v2': {'description': 'BBC Micro:bit v2',
                    'arch': 'cortex-m4',
                    'apps_start_address': 0x00040000,
                    'page_size': 4096,
                    'openocd': 'external',
                    'openocd_prefix': 'source [find interface/cmsis-dap.cfg]; \
                                        transport select swd; \
                                        source [find target/nrf52.cfg]; \
                                        set WORKAREASIZE 0x40000; \
                                        $_TARGETNAME configure -work-area-phys 0x20000000 -work-area-size $WORKAREASIZE -work-area-backup 0; \
                                        flash bank $_CHIPNAME.flash nrf51 0x00000000 0 1 1 $_TARGETNAME;'},
}

KNOWN_CI_BOARDS = {
    'hail': {'description': 'Hail development module.',
                'arch': 'cortex-m4',
                'jlink_device': 'ATSAM4LC8C',
                'page_size': 512},
    'imix': {'description': 'Low-power IoT research platform',
                'arch': 'cortex-m4',
                'jlink_device': 'ATSAM4LC8C',
                'page_size': 512},
    'nrf51dk': {'description': 'Nordic nRF51-based development kit',
                'arch': 'cortex-m0',
                'jlink_device': 'nrf51422',
                'page_size': 1024,
                'openocd': 'nordic_nrf51_dk.cfg',
                'openocd_options': ['workareazero']},
    'nrf52dk': {'description': 'Nordic nRF52-based development kit',
                'arch': 'cortex-m4',
                'jlink_device': 'nrf52',
                'page_size': 4096,
                'openocd': 'nordic_nrf52_dk.cfg'},
    'nrf52840dk': {'description': 'Nordic nRF52-based development kit',
                'arch': 'cortex-m4',
                'jlink_device': 'nrf52',
                'page_size': 4096,
                'openocd': 'nordic_nrf52_dk.cfg'},
    'nano33ble': {'description': 'Arduino Nano 33 BLE board',
                    'arch': 'cortex-m4'},
    'launchxl-cc26x2r1': {'description': 'TI CC26x2-based launchpad',
                            'arch': 'cortex-m4',
                            'page_size': 512,
                            'jlink_device': 'cc2652r1f',
                            'jlink_speed': 4000,
                            'jlink_if': 'jtag',
                            'openocd': 'ti_cc26x2_launchpad.cfg',
                            'openocd_options': ['noreset', 'resume'],
                            'openocd_commands': {'program': 'flash write_image erase {{binary}} {address:#x};\
                                                            verify_image {{binary}} {address:#x};'}},
    'ek-tm4c1294xl': {'description': 'TI TM4C1294-based launchpad',
                        'arch': 'cortex-m4',
                        'page_size': 512,
                        'openocd': 'ek-tm4c1294xl.cfg'},
    'arty': {'description': 'Arty FPGA running SiFive RISC-V core',
                'arch': 'rv32imac',
                'apps_start_address': 0x40430000,
                # arty exposes just the flash to openocd, this does the mapping
                # from the address map to what openocd must use.
                'address_translator': lambda addr: addr - 0x40000000,
                'page_size': 512,
                'openocd': 'external', # No supported board in openocd proper
                'openocd_options': ['nocmdprefix'],
                'openocd_prefix': 'source [find interface/ftdi/digilent-hs1.cfg];\
                                ftdi_device_desc \\"Digilent USB Device\\";\
                                adapter_khz 10000;\
                                transport select jtag;\
                                source [find cpld/xilinx-xc7.cfg];\
                                source [find cpld/jtagspi.cfg];\
                                proc jtagspi_read {{fname offset len}} {{\
                                    global _FLASHNAME;\
                                    flash read_bank $_FLASHNAME $fname $offset $len;\
                                }};\
                                init;\
                                jtagspi_init 0 {bitfile};'
                                .format(bitfile=os.path.join( # Need path to bscan_spi_xc7a100t.bit
                                    os.path.dirname(os.path.realpath(__file__)),
                                    '..', 'bitfiles', 'bscan_spi_xc7a100t.bit')),
                'openocd_commands': {'program': 'jtagspi_program {{binary}} {address:#x};',
                                    'read': 'jtagspi_read {{binary}} {address:#x} {length};',
                                    'erase': 'flash fillb {address:#x} 0x00 512;'}},
    'stm32f3discovery': {'description': 'STM32F3-based Discovery Boards',
                            'arch': 'cortex-m4',
                            'apps_start_address': 0x08020000,
                            'page_size': 2048,
                            'openocd': 'external',
                            'openocd_prefix': 'interface hla; \
                                            hla_layout stlink; \
                                            hla_device_desc "ST-LINK/V2-1"; \
                                            hla_vid_pid 0x0483 0x374b; \
                                            set WORKAREASIZE 0xC000; \
                                            source [find target/stm32f3x.cfg];'},
    'stm32f4discovery': {'description': 'STM32F4-based Discovery Boards',
                            'arch': 'cortex-m4',
                            'apps_start_address': 0x08040000,
                            'page_size': 2048,
                            'openocd': 'external',
                            'openocd_prefix': 'interface hla; \
                                                hla_layout stlink; \
                                                hla_device_desc "ST-LINK/V2-1"; \
                                                hla_vid_pid 0x0483 0x374b; \
                                                set WORKAREASIZE 0x40000; \
                                                source [find target/stm32f4x.cfg];'},
    'nucleof4': {'description': 'STM32f4-based Nucleo development boards',
                    'arch': 'cortex-m4',
                    'apps_start_address': 0x08040000,
                    'page_size': 2048,
                    'openocd': 'st_nucleo_f4.cfg'},
    'hifive1': {'description': 'SiFive HiFive1 development board',
                'arch': 'rv32imac',
                'apps_start_address': 0x20430000,
                'page_size': 512,
                'openocd': 'sifive-hifive1.cfg'},
    'hifive1b': {'description': 'SiFive HiFive1b development board',
                    'arch': 'rv32imac',
                    'apps_start_address': 0x20040000,
                    'page_size': 512,
                    'jlink_device': 'FE310',
                    'jlink_if': 'jtag',
                    'openocd': 'sifive-hifive1-revb.cfg'},
    'edu-ciaa': {'description': 'Educational NXP board, from the CIAA project',
                    'arch': 'cortex-m4',
                    'page_size': 512,
                    'apps_start_address': 0x1a040000,
                    'openocd': 'ftdi_lpc4337.cfg',
                    'openocd_options': ['noreset'],
                    'openocd_commands': {'program': 'flash write_image erase {{binary}} {address:#x};verify_image {{binary}} {address:#x};',
                    'erase': 'flash fillb {address:#x} 0x00 512;'}},
    'microbit_v2': {'description': 'BBC Micro:bit v2',
                    'arch': 'cortex-m4',
                    'apps_start_address': 0x00040000,
                    'page_size': 4096,
                    'openocd': 'external',
                    'openocd_prefix': 'source [find interface/cmsis-dap.cfg]; \
                                        transport select swd; \
                                        source [find target/nrf52.cfg]; \
                                        set WORKAREASIZE 0x40000; \
                                        $_TARGETNAME configure -work-area-phys 0x20000000 -work-area-size $WORKAREASIZE -work-area-backup 0; \
                                        flash bank $_CHIPNAME.flash nrf51 0x00000000 0 1 1 $_TARGETNAME;'},
}
COMMUNICATION_PROCTOCOLS = ['Other', 'jlink', 'openocd']
TOCK_BOARD_PATH = f'{Path.home()}/actions-runner/_work/tock/tock/boards/'
TOCK_HARNESS_PATH = f'{Path.home()}/tock/tools/tock-test-harness/'
I2C_BOOT_CONFIG = ['yes', 'no']
title = ''
board = ''
board_to_test ''
board_path = ''
harness_no = ''
comm_proc = ''

# Print description and function of the setup
DESCRIPTION = ("Initialized configuration setup guide to create "
               "test.config.toml\n"
               "For more information, please visit "
               "https://github.com/tock/tock/tools/tock-test-harness\n")
print(DESCRIPTION)

# Pre setup: clone tock
if not os.path.isdir(f'{Path.home()}/actions-runner/_work'):
    print("Cloning Tock OS...")
    os.makedirs(f'{Path.home()}/actions-runner/_work/tock/tock')

    # Checkout Tock Repo
    Repo.clone_from('https://github.com/tock/tock.git', f'{Path.home()}/actions-runner/_work/tock/tock')

# Input configuration name
title = input('Name the configuration: ')

if title == '':
    title = 'untitled'

# Separate next prompt with the previous prompt
print('\n')

# Input board
while True:
    # Other board option
    print('[0] Other board')

    for idx, board_name in enumerate(KNOWN_BOARDS):
        print(f'[{idx + 1}] {board_name}')

    # Print disclaimer
    print('\nNote: if the board is not listed here, it is not supported.')

    board = input('Board Model (default to [0]/Other board): ')

    # Check board validity
    if board.isdigit() and (0 <= int(board) <= len(KNOWN_BOARDS)):
        if board == '0':
            board = input('Input board name: ')

            # Break before the KNOWN_BOARD check, if the string is not empty
            if board != '':
                break
        else:
            board = list(KNOWN_BOARDS.keys())[int(board) - 1]
    
    if board in KNOWN_BOARDS:
        print(f'\n{board} has been selected.')
        break
    else:
        print('Board ', board, ' is invalid')


# Input board
while True:
    # Other board option
    print('[0] Other board')

    for idx, board_name in enumerate(KNOWN_CI_BOARDS):
        print(f'[{idx + 1}] {board_name}')

    # Print disclaimer
    print('\nNote: if the board is not listed here, it is not supported.')

    board_to_test = input('Tested Board Model (default to [0]/Other board - BE SPECIFIC): ')

    # Check board validity
    if board_to_test.isdigit() and (0 <= int(board_to_test) <= len(KNOWN_CI_BOARDS)):
        if board_to_test == '0':
            board_to_test = input('Input board name: ')

            # Break before the KNOWN_BOARD check, if the string is not empty
            if board_to_test != '':
                break
        else:
            board_to_test = list(KNOWN_CI_BOARDS.keys())[int(board_to_test) - 1]
    
    if board_to_test in KNOWN_CI_BOARDS:
        print(f'\n{board_to_test} has been selected.')
        break
    else:
        print('Board ', board_to_test, ' is invalid')
        

# Separate next prompt with the previous prompt
print('\n')

# Enter communication protocol
print('Input communication protocol for the board.\n')

for idx, comm_proc_name in enumerate(COMMUNICATION_PROCTOCOLS):
    print(f'[{idx}] {comm_proc_name}')

print() # Line break from options

while True:
    comm_proc = input('Enter communication protocol: ')

    if comm_proc.isdigit() and (0 <= int(comm_proc) <= len(COMMUNICATION_PROCTOCOLS)):
        if comm_proc == '0':
            comm_proc = '' # Placeholder, not sure what to do with this
            break
        else:
            comm_proc = COMMUNICATION_PROCTOCOLS[int(comm_proc)]
    
    if comm_proc in COMMUNICATION_PROCTOCOLS:
        print(f'\n{comm_proc} has been selected.')
        break
    else:
        print(f'\nCommunication protocol {comm_proc} is invalid.')


while True:
    i2c_board = input('Enter if board has i2c test (yes/no/y/n): ')

    if i2c_board.lower() == 'yes' or i2c_board.lower() == 'y':
        i2c_board = I2C_BOOT_CONFIG[0]
        break
    elif i2c_board.lower() == 'no' or i2c_board.lower() == 'n':
        i2c_board = I2C_BOOT_CONFIG[1]
        break
    else:
        print(f'\nInvalid input - {i2c_board} - for I2C boot configuration ')
# Separate next prompt with the previous prompt
print('\n')

# Enter path to the board
print("Input board directory relative to Tock boards directory, ",
      "(e.g. [tock/boards]/nordic/nrf52840dk) ")

while True:
    board_path = input("Enter path, or 'f' to list directories: ")

    if board_path == 'f':
        fds = os.listdir(TOCK_BOARD_PATH)
        for fd in fds:
            if os.path.isdir(TOCK_BOARD_PATH + fd):
                print(f'/{fd}')
        print() # Line break
    elif board_path != '':
        if os.path.exists(TOCK_BOARD_PATH + board_path):
            print(f"\nPath '{board_path}' has been selected.")
            break

# Separate next prompt with the previous prompt
print('\n')

# Enter harness ID
print('Input harness ID to specify the runner action, (default 0)')

harness_id = input('Enter harness ID: ')

if harness_id == '':
    harness_id = '0'

print(f'Selected harness ID {harness_id}')

# Deserialize and dump to toml file
print('\nCreating Toml Configuration File...\n')

with open(TOCK_HARNESS_PATH + 'config.toml', 'w') as output_toml_file:
    final_dict = {
        'title': title,

        'env': {
            'board': board,
            'board_to_test': board_to_test,
            'path': board_path,
            'harness_id': harness_id,
            'communication_protocol': comm_proc,
            'i2c_on_boot' : i2c_board
        }
    }

    toml.dump(final_dict, output_toml_file)