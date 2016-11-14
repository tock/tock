#!/usr/bin/env bash

APP_BASE_ADDR='0x30000'

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
source [find target/at91sam4lXX.cfg]
EOF

/opt/gnuarmeclipse/openocd/0.10.0-201610281609-dev/bin/openocd -f $TMPCONF -c "init; reset halt; flash write_image $TMPAPP $APP_BASE_ADDR bin; reset; shutdown"

rm $TMPCONF
rm $TMPAPP
