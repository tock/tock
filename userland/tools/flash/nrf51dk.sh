#!/usr/bin/env bash

APP_BASE_ADDR='0x20000'

TMPCONF=$(mktemp)
TMPAPP=$(mktemp)

for app in "$@"
do
  cat "$app" >> $TMPAPP
done

wc $TMPAPP

printf "\0\0\0\0" >> $TMPAPP

cat << EOF > $TMPCONF
interface jlink
transport select swd
source [find target/nrf51.cfg]
EOF

openocd -f $TMPCONF -c "init; reset halt; flash write_image $TMPAPP $APP_BASE_ADDR bin; reset; shutdown"

rm $TMPCONF
rm $TMPAPP
