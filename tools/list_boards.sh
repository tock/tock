#!/usr/bin/env bash

# http://stackoverflow.com/questions/192249/how-do-i-parse-command-line-arguments-in-bash
oneline=0
while getopts "h?1" opt; do
    case "$opt" in
    h|\?)
        echo Prints supported tock boards
        echo ""
        echo "    -1  One entry per line, for scripting"
        exit 0
        ;;
    1)  oneline=1
        ;;
    esac
done

boards=`ls -l boards | egrep '^d' | awk '{print $9}'`

if [ $oneline -eq 1 ]; then
    for board in $boards; do
        echo $board
    done
    exit 0
fi

echo Supported Tock boards: $boards
echo ""
echo To build the kernel for a particular board, change to that direcotry
echo "    cd boards/hail"
echo "    make"

