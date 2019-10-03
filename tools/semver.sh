#!/usr/bin/env bash

# Semantic version comparison. This function compares two
# strings of the form X{.Y}* and returns which is larger or
# whether they are equal. This is not a simple text comparison
# because 4.10.1 > 4.1.
#
# Adapted from
# https://stackoverflow.com/questions/4023830/

vercomp () {
    if [[ $1 == $2 ]]
    then
        return 0
    fi
    local IFS=.
    local i ver1=($1) ver2=($2)
    # fill empty fields in ver1 with zeros
    for ((i=${#ver1[@]}; i<${#ver2[@]}; i++))
    do
        ver1[i]=0
    done
    for ((i=0; i<${#ver1[@]}; i++))
    do
        if [[ -z ${ver2[i]} ]]
        then
            # fill empty fields in ver2 with zeros
            ver2[i]=0
        fi
        if ((10#${ver1[i]} > 10#${ver2[i]}))
        then
            return 1
        fi
        if ((10#${ver1[i]} < 10#${ver2[i]}))
        then
            return 2
        fi
    done
    return 0
}

testvercomp () {
    vercomp $1 $2
    case $? in
        0) op='=';;
        1) op='>';;
        2) op='<';;
    esac
    if [[ $op != $3 ]]
    then
        #echo "FAIL: Expected '$3', Actual '$op', Arg1 '$1', Arg2 '$2'"
        echo false
    else
        echo true
    fi
}

testvercomp $1 $3 $2
