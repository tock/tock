import argparse
import sys
from Runner import Runner

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        '-b', '--build',
        help='Compile Tock OS', action='store_true', dest='build')
    parser.add_argument(
        '-i', '--install',
        help='Flash Tock OS onto target board', action='store_true',
        dest='install')
    parser.add_argument(
        '-t', '--test',
        help='Run test workflow', action='store_true', dest='test')

    # Get args as dict
    args = vars(parser.parse_args())

    if not any(args.values()):
        parser.print_help()
        sys.exit(1)

    return Runner(**args).run()

if __name__ == '__main__':
    sys.exit(main())
