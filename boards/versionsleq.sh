#!/bin/bash
echo $(expr $(echo -e "$1\n$2"|sort -V|head -n 1) \=\= $1)
