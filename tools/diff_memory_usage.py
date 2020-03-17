#!/usr/bin/env python3

import argparse

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('prev_bench', help='Memory benchmark of base branch of PR')
    parser.add_argument('cur_bench', help='Memory benchmark of PR post-merge')
    parser.add_argument('outfile', help='Filename to output diffs into')
    parser.add_argument('board', help='Board these measurements are derived from')
    args = parser.parse_args()

    board = args.board
    prev_flash = -1
    prev_RAM = -1
    cur_flash = -1
    cur_RAM = -1
    with open(args.prev_bench, 'r') as f:
        for line in f:
            if 'Kernel occupies' in line:
                if 'flash' in line:
                    prev_flash = int(line.split()[2])
                elif 'RAM' in line:
                    prev_RAM = int(line.split()[2])
                    break
    if prev_flash == -1 or prev_RAM == -1:
        sys.exit('Failed to parse prev_bench for board: {}'.format(board))

    with open(args.cur_bench, 'r') as f:
        for line in f:
            if 'Kernel occupies' in line:
                if 'flash' in line:
                    cur_flash = int(line.split()[2])
                elif 'RAM' in line:
                    cur_RAM = int(line.split()[2])
                    break
    if cur_flash == -1 or cur_RAM == -1:
        sys.exit('Failed to parse cur_bench for board: {}'.format(board))

    diff_flash = cur_flash - prev_flash
    diff_RAM = cur_RAM - prev_RAM

    if diff_flash == 0 and diff_RAM == 0:
        print("No diff for board: {}".format(board))
        return; #Don't write to file for boards with no change in size

    f = open(args.outfile, 'a+')
    f.write('{}: flash increase = {} bytes, RAM increase = {} bytes\n'.format(board, diff_flash, diff_RAM))

if __name__ == "__main__":
    main()
