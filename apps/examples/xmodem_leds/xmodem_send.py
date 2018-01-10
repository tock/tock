#!/usr/bin/python

"""Julie Zelenski 2017.
Based on earlier rpi-install.py by Pat Hanrahan.
Edited by Omar Rizwan 2017-04-23.

This program was designed to be a bootloader client to upload binary 
images to execute on a bare metal Raspberry Pi. However, it can also
be used to just perform basic xmodem transfers, and so you can use it
to upload new images for an embedded system controlling a bicycle light.

Communicates over serial port using xmodem protocol.

Should work with:
- Python 2.7+ and Python 3
- any version of the on-Pi bootloader
- macOS and Linux

Maybe Cygwin and Ubuntu on Windows as well.

Dependencies:

    # pip install {pyserial,xmodem}

"""
from __future__ import print_function
import argparse, logging, os, serial, sys, time
from serial.tools import list_ports
from xmodem import XMODEM

# This version updated May 30, 2017 with timeout flag.
VERSION = 0.7

# From https://stackoverflow.com/questions/287871/print-in-terminal-with-colors-using-python
# Plus Julie's suggestion to push bold and color together.
class bcolors:
    RED = '\033[31m'
    BLUE = '\033[34m'
    GREEN = '\033[32m'
    BOLD = '\033[1m'
    OKBLUE = BOLD + BLUE
    OKGREEN = BOLD + GREEN
    FAIL = BOLD + RED
    ENDC = '\033[0m'

def error(shortmsg, msg=""):
    sys.stderr.write("\n%s: %s\n" % (
        sys.argv[0],
        bcolors.FAIL + shortmsg + bcolors.ENDC + "\n" + msg
    ))
    sys.exit(1)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="This script sends a binary file over a serial port using xmodem. Version %s." % VERSION)

    parser.add_argument("port", help="serial port", nargs="?")
    parser.add_argument("file", help="binary file to upload",
                        type=argparse.FileType('rb'))
    parser.add_argument('-v', help="verbose logging of serial activity",
                        action="store_true")
    parser.add_argument('-q', help="do not print while uploading",
                        action="store_true")
    parser.add_argument('-t', help="timeout for -p",
                        action="store", type=int, default=100)

    after = parser.add_mutually_exclusive_group()
    after.add_argument('-p', help="print output after uploading",
                       action="store_true")
    after.add_argument('-s', help="open `screen` on the serial port after uploading",
                       action="store_true")


    args = parser.parse_args()

    def printq(*pos_args, **kwargs):
        if not args.q:
            print(*pos_args, **kwargs)

    logging.getLogger().addHandler(logging.StreamHandler())
    if args.v: logging.getLogger().setLevel(logging.DEBUG)

    if args.port:
        portname = args.port
    else:
        # The CP2102 units from winter 2014-15 and spring 2016-17 both have
        # vendor ID 0x10C4 and product ID 0xEA60.
        try:
            # pyserial 2.6 in the VM has a bug where grep is case-sensitive.
            # It also requires us to use [0] instead of .device on the result
            # to get the serial device path.
            portname = next(list_ports.grep(r'(?i)VID:PID=10C4:EA60'))[0]
            printq('Found serial port:', bcolors.OKBLUE + portname + bcolors.ENDC)

            # We used to just have a preset list --
            # /dev/tty.SLAB_USBtoUART for macOS + Silicon Labs driver,
            # /dev/cu.usbserial for Prolific,
            # /dev/ttyUSB0 for Linux, etc.
            # Hopefully the device ID-based finder is more reliable.

        except StopIteration:
            error("Couldn't find serial port", """
I looked through the serial ports on your computer, and couldn't
find any port associated with a CP2102 USB-to-serial adapter. Is
your Pi plugged in?
""")

    try:
        # timeout set at creation of Serial will be used as default for both read/write
        port = serial.Serial(port=portname, baudrate=115200, timeout=2)

    except (OSError, serial.serialutil.SerialException):
        error("The serial port `%s` is not available" % portname, """
Do you have a `screen` or `rpi-install.py` currently running that's
hanging onto that port?
""")

    stream = args.file
    printq("Sending `%s` (%d bytes): " % (stream.name, os.stat(stream.name).st_size), end='')

    success = False
    def getc(size, timeout=1):
        ch = port.read(size)
        # echo 'x' to report failure if read timed out/failed
        if ch == b'':
            if not success: printq('x', end='')
            sys.stdout.flush()
        return ch

    def putc(data, timeout=1):
        n = port.write(data)
        # echo '.' to report full packet successfully sent
        if n >= 128:
            printq('.', end='')
            sys.stdout.flush()

    try:
        xmodem = XMODEM(getc, putc)
        success = xmodem.send(stream, retry=5)
        if not success:
            error("Send failed (bootloader not listening?)", """
I waited a few seconds for an acknowledgement from the bootloader
and didn't hear anything. Do you need to reset your Pi?

Further help at http://cs107e.github.io/guides/bootloader/#troubleshooting
""")
    except serial.serialutil.SerialException as ex:
        error(str(ex))
    except KeyboardInterrupt:
        error("Canceled by user pressing Ctrl-C.", """
You should probably restart the Pi, since you interrupted it mid-load.
""")

    printq(bcolors.OKGREEN + "\nSuccessfully sent!" + bcolors.ENDC)
    stream.close()

    start = time.time()
    if args.p:  # Print after sending.
        try:
            while True:
                if time.time() - start > args.t:
                    break

                c = getc(1)
                if c == chr(4): break # End of transmission.
                if c is None: continue

                print(c,end='')
        except:
            pass
    elif args.s:  # Run `screen` after sending.
        sys.exit(os.system('screen %s 115200' % portname))

    sys.exit(0)
