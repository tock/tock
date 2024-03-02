#!/usr/bin/env python3

# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2023.

'''
Script to test basic functionalities of the process console.

In order to run this script on your board, first find the
serial port name of the board.

Make sure you have flashed your board and pressed several times
the "RESET"/"REBOOT" button from the board.

If you want to re-run the script make sure you again
press the "RESET"/"REBOOT" button, in order to clear
the command history and the current output

If you have multiple serial ports to the same board
run the script for every serial port
'''

from serial import Serial, SerialException
from time import sleep


test_output_line_size = 40
tests = {
    "open_serial_port":     (-1, "not_executed"),
    "fill_command_history": (-1, "not_executed"),
    "command_history_api":  (-1, "not_executed"),
    "inserting_at_end":     (-1, "not_executed"),
    "inserting_at_start":   (-1, "not_executed"),
    "inserting_in_middle":  (-1, "not_executed"),
    "inserting":            (-1, "not_executed"),
    "deleting":             (-1, "not_executed"),
    "cariage_return":	    (-1, "not_executed"),
    "newline_return":       (-1, "not_executed"),
    "command_history_edit": (-1, "not_executed"),
}

colors = ["\033[92m", "\033[91m", "\033[00m"]
commands = {
    "left": 		"\x1B[D",
    "right": 		"\x1B[C",
    "home": 		"\x1B[H",
    "end": 			"\x1B[F",
    "delete": 		"\x1B[3~",
    "delete-ascii": "\x7F",
    "backspace": 	"\x08 \x08",
    "up": 			"\x1B[A",
    "down": 		"\x1B[B",
}


class SerialPort:
    ''' Serial Port class utils for sending and receiving data to the board '''

    def __init__(self, serportname="/dev/tock"):
        self.serport = Serial(serportname, 115200, timeout=0.050)

    def send_input(self, payload):
        ''' Send input to the board encoded using ascii '''
        sleep(1)
        if self.serport.is_open:
            self.serport.write(payload.encode("ascii"))

    def recv_output(self):
        ''' Receiving output from the board which is return as a normal string '''
        sleep(1)
        if self.serport.is_open and self.serport.in_waiting:
            return self.__evaluate_encoded(self.serport.readline().decode("ascii"))
        else:
            return ""

    def is_alive(self):
        ''' Check if the serial port is still open '''
        return self.serport.is_open

    def clear_input(self):
        ''' Clears the receiving buffer from the board '''
        if self.serport.is_open:
            sleep(1)
            while self.serport.in_waiting:
                self.serport.read()

    def finish(self):
        ''' Closes the serial port '''
        if self.serport.is_open:
            self.serport.close()

    def __evaluate_encoded(self, encoded):
        '''
        Evaluates a string encoded using ascii,
        this is a simple parser for escape sequences.

        The function will evaluate just Backspace and Left
        escape sequences
        '''

        # delete previous character
        encoded = encoded.replace("\x08 \x08", "@")
        
        # deletes leftover character from previous message.
        # As such we can ignore it
        # ! This check must go after the '@' replacement !
        encoded = encoded.replace(" \x08", "")

        decoded = ""
        count = 0

        for ch in encoded:
            if ch == '\x08':
                count += 1
            elif count > 0:
                decoded = decoded[:-count]
                count = 0
                decoded += ch
            elif ch == '@':
                if decoded != "":
                    decoded = decoded[:-1]
                    count = 0
            else:
                decoded += ch

        return decoded


def pass_test(test):
    ''' Marks a test as passed '''
    tests[test] = (0, "passed")


def fail_test(test):
    ''' Marks a test as failed '''
    tests[test] = (1, "failed")


def exit_if_condition(condition, message):
    ''' Exit the main thread if condition is true and print checker results '''
    if condition:
        print(message)
        print_test_results()
        quit()


def print_title(title):
    ''' Prints the title of the test '''
    print(colors[0], "\n" + title, colors[-1])


def print_test_results():
    ''' Prints all the checker results '''
    print("")
    print("RESULTS")
    print("-------")
    for test in tests:
        test_color, test_status = tests.get(test)
        dots = test_output_line_size - len(test) - len(test_status)
        print(test, end=" ")
        print('.' * dots, end=" ")
        print(colors[test_color] + test_status + colors[-1])


def test_open_serial_port(serport):
    ''' Opens a serial port to the board and checks if it is opened '''
    print_title("Openning the serial port to the board")
    fail_test("open_serial_port")

    try:
        port = SerialPort(serport)
        pass_test("open_serial_port")

        port.send_input("\r\n")
        port.clear_input()

        return port
    except SerialException:
        print("[ERROR] Could not open the serial port")
        print_test_results()
        quit()


def test_fill_command_history(port: SerialPort):
    '''
    Fills the command history with dummy commands in order to test
    it's functionality.

    For this test case the size of the command history must be greater or
    equal to the default size.
    '''
    print_title("Filling command history with dummy commands:")
    fail_test("fill_command_history")

    for i in range(0, 9):
        dummy_text = "Dummy Text" + str(i)
        print("Inserting '" + dummy_text + "'")
        port.send_input(dummy_text + "\r\n")

    port.clear_input()
    pass_test("fill_command_history")


def test_command_history_api(port: SerialPort):
    ''' Tests basic functionality of command history like scrolling up and down '''
    print_title("Testing basic command history functionalities:")
    fail_test("command_history_api")

    print("Moving up in history 4 times")
    port.send_input(commands["up"] * 4)

    out = port.recv_output()
    exit_if_condition(out != "Dummy Text5",
                      "[ERROR] Command does not match")

    print("Moving up in history 4 times")
    port.send_input(commands["up"] * 4)

    out = port.recv_output()
    exit_if_condition(out != "Dummy Text1",
                      "[ERROR] Command does not match")

    print("Moving down in history 2 times")
    port.send_input(commands["down"] * 2)

    out = port.recv_output()
    exit_if_condition(out != "Dummy Text3",
                      "[ERROR] Command does not match")

    print("Moving all the way up")
    port.send_input(commands["up"] * 10)

    out = port.recv_output()
    exit_if_condition(out != "Dummy Text0",
                      "[ERROR] Command does not match")

    print("Moving all the way down")
    port.send_input(commands["down"] * 13)

    print("Moving up in history 1 time")
    port.send_input(commands["up"])

    out = port.recv_output()
    exit_if_condition(out != "Dummy Text8",
                      "[ERROR] Command does not match")

    out = port.send_input("\r\n")
    port.clear_input()
    pass_test("command_history_api")


def test_inserting_at_end(port: SerialPort):
    ''' Inserts a text and then tries to add more text to the end of the command '''
    print_title("Inserting additional text to the end of a command:")
    fail_test("inserting_at_end")

    print("Typing 'FooBar'")
    port.send_input("FooBar")

    print("Insert to the end 'Dummy'")
    port.send_input("Dummy")

    out = port.recv_output()
    port.send_input("\r\n")
    exit_if_condition(out != "FooBarDummy",
                      "[ERROR] Command does not match")

    port.clear_input()
    pass_test("inserting_at_end")


def test_inserting_at_start(port: SerialPort):
    ''' Inserts a text and then tries to add more text to the beginning of the command '''
    print_title("Inserting additional text to the start of a command:")
    fail_test("inserting_at_start")

    print("Typing 'FooBar'")
    port.send_input("FooBar")

    print("Insert to the start 'Dummy'")
    port.send_input(commands["home"])
    port.send_input("Dummy")

    out = port.recv_output()
    port.send_input("\r\n")
    exit_if_condition(out != "DummyFooBar",
                      "[ERROR] Command does not match")

    port.clear_input()
    pass_test("inserting_at_start")


def test_inserting_in_middle(port: SerialPort):
    ''' Inserts a text and then tries to add more text in the middle of the command '''
    print_title("Inserting additional text in the middle of a command:")
    fail_test("inserting_in_middle")

    print("Typing 'FooBar'")
    port.send_input("FooBar")

    print("Insert in the middle 'Dummy'")
    port.send_input(commands["left"] * 3)

    port.send_input("Dummy")

    out = port.recv_output()
    port.send_input("\r\n")
    exit_if_condition(out != "FooDummyBar",
                      "[ERROR] Command does not match")

    port.clear_input()
    pass_test("inserting_in_middle")


def test_inserting(port: SerialPort):
    ''' Performs a series of insertions in different places of the command '''
    print_title("Inserting additional text in the command:")
    fail_test("inserting")

    print("Typing 'FooBar'")
    port.send_input("FooBar")

    print("Typing 'Dummy'")
    port.send_input("Dummy")

    print("Insert to the start 'Dummy'")
    port.send_input(commands["home"])
    port.send_input("Dummy")

    print("Insert in the middle 'Dummy'")
    port.send_input(commands["right"] * 3)
    port.send_input("Dummy")

    out = port.recv_output()
    port.send_input("\r\n")
    exit_if_condition(out != "DummyFooDummyBarDummy",
                      "[ERROR] Command does not match")

    port.clear_input()
    pass_test("inserting")


def test_deleting(port: SerialPort):
    ''' Performs a series of deletions from different places of the command '''
    print_title("Deleting from a command using backspace and delete:")
    fail_test("deleting")
    
    def test_deleting_with(delete_char: str):
        print("Typing 'DummyFooDummyBarDummy'")
        port.send_input("DummyFooDummyBarDummy")

        print("Moving to the start of the command")
        port.send_input(commands["home"])

        print("Delete first 'Dummy'")
        port.send_input(delete_char * 5)

        print("Moving to the end of the command")
        port.send_input(commands["end"])

        print("Delete last 'Dummy'")
        port.send_input(commands["backspace"] * 5)

        print("Moving to the begining of the middle 'Dummy'")
        port.send_input(commands["left"] * 8)

        print("Delete middle 'Dummy'")
        port.send_input(delete_char * 5)

        out = port.recv_output()
        port.send_input("\r\n")
        exit_if_condition(out != "FooBar",
                        "[ERROR] Command does not match")

    print("Testing with ANSI Escpae Sequence...")
    test_deleting_with(commands["delete"])
    port.clear_input()

    print("Testing with ASCII character...")
    test_deleting_with(commands["delete-ascii"])
    port.clear_input()

    pass_test("deleting")


def test_cariage_return(port: SerialPort):
    '''
    Checks the command history api test with cariage return
    and performs basic actions like typing a command and
    deleting it
    '''
    print_title("Sending commands terminated by \\r")
    fail_test("cariage_return")

    for i in range(0, 9):
        dummy_text = "Dummy Text" + str(i)
        print("Inserting '" + dummy_text + "'")
        port.send_input(dummy_text + "\r")

    port.clear_input()
    test_command_history_api(port)

    print("Test if cursor is reseting")

    print("Insert command 'help' and return")
    port.send_input("help\r")

    print("Move up in the command history")
    port.send_input(commands["up"])

    for _ in range(0, 4):
        print("Send 25 backspaces")
        port.send_input(commands["backspace"] * 25)

    print("Move down in the command history")
    port.send_input(commands["down"])

    for _ in range(0, 4):
        print("Send 25 backspaces")
        port.send_input(commands["backspace"] * 25)

    port.clear_input()
    port.send_input("FooBar")

    out = port.recv_output()
    exit_if_condition(out != "FooBar",
                      "[ERROR] Command does not match")

    port.send_input("\r\n")

    pass_test("cariage_return")


def test_newline_return(port: SerialPort):
    '''
    Checks the command history api test with newline return
    and performs basic actions like typing a command and
    deleting it
    '''
    print_title("Sending commands terminated by \\n")
    fail_test("newline_return")

    for i in range(0, 9):
        dummy_text = "Dummy Text" + str(i)
        print("Inserting '" + dummy_text + "'")
        port.send_input(dummy_text + "\r")

    port.clear_input()

    test_command_history_api(port)

    print("Test if cursor is reseting")

    print("Insert command 'help' and return")
    port.send_input("help\r")

    print("Move up in the command history")
    port.send_input(commands["up"])

    for _ in range(0, 4):
        print("Send 25 backspaces")
        port.send_input(commands["backspace"] * 25)

    print("Move down in the command history")
    port.send_input(commands["down"])

    for _ in range(0, 4):
        print("Send 25 backspaces")
        port.send_input(commands["backspace"] * 25)

    port.clear_input()
    port.send_input("FooBar")

    out = port.recv_output()
    exit_if_condition(out != "FooBar",
                      "[ERROR] Command does not match")

    port.send_input("\r\n")

    pass_test("newline_return")

def test_command_history_edit(port: SerialPort):
    ''' Tests basic editing of commands in history '''
    print_title("Testing basic command history editing:")
    fail_test("command_history_edit")

    def test_inserting():
        print("Inserting 'FooBar'")
        port.send_input("FooBar\r\n")
        port.clear_input()

        print("Moving up in history 1 time")
        port.send_input(commands["up"])

        print("Moving to the start of the command")
        port.send_input(commands["home"])

        print("Insert to the start 'Dummy'")
        port.send_input("Dummy")

        print("Moving to the end of the command")
        port.send_input(commands["end"])

        print("Insert to the end 'Dummy'")
        port.send_input("Dummy")

        print("Moving to the beginning of the middle 'Dummy'")
        port.send_input(commands["left"] * 8)

        print("Insert in the middle 'Dummy'")
        port.send_input("Dummy")

        print("Moving down in history 1 time")
        port.send_input(commands["down"])

        out = port.recv_output()

        exit_if_condition(out != "DummyFooDummyBarDummy",
                        "[ERROR] Command does not match")


    def test_deleting_with(delete_char: str):
        print("Moving up in history 1 time")
        port.send_input(commands["up"])

        print("Moving to the start of the command")
        port.send_input(commands["home"])

        print("Delete first 'Dummy'")
        port.send_input(delete_char * 5)

        print("Moving to the end of the command")
        port.send_input(commands["end"])

        print("Delete last 'Dummy'")
        port.send_input(commands["backspace"] * 5)

        print("Moving to the begining of the middle 'Dummy'")
        port.send_input(commands["left"] * 8)

        print("Delete middle 'Dummy'")
        port.send_input(delete_char * 5)

        print("Moving down in history 1 time")
        port.send_input(commands["down"])

        out = port.recv_output()

        exit_if_condition(out != "FooBar",
                        "[ERROR] Command does not match")

    test_inserting()
    out = port.send_input("\r\n")
    port.clear_input()

    print("Testing with ANSI Escpae Sequence...")
    test_deleting_with(commands["delete"])
    port.clear_input()

    print("Testing with ASCII character...")
    test_deleting_with(commands["delete-ascii"])

    out = port.send_input("\r\n")
    port.clear_input()
    pass_test("command_history_edit")


def read_serial_port_name():
    ''' Fetches the serial port name from the user '''
    serial_port_name = ""

    while serial_port_name == "":
        serial_port_name = input("Enter the serial port name: ")

    return serial_port_name


def main():
    ''' Main loop to run all test cases '''
    port = test_open_serial_port(read_serial_port_name())

    test_fill_command_history(port)
    test_command_history_api(port)
    test_inserting_at_end(port)
    test_inserting_at_start(port)
    test_inserting_in_middle(port)
    test_inserting(port)
    test_deleting(port)
    test_cariage_return(port)
    test_newline_return(port)
    test_command_history_edit(port)

    port.finish()

    print_test_results()


main()
