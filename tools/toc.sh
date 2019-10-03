#!/usr/bin/env bash

# Run `markdown-toc` on all Markdown files with tables of contents
# to see if any of them need an updated TOC.
#
# As a side effect, if you run this locally it will generate
# all needed TOC and update the markdown documents.
#
# Author: Brad Campbell <bradjc5@gmail.com>

let ERROR=0

# Find all markdown files
for f in $(find * -name "*.md"); do

	# Only use ones that include a table of contents
	grep '<!-- toc -->' $f > /dev/null
	let rc=$?

	if [[ $rc == 0 ]]; then
		# Try running the TOC tool and see if anything changes
		before=`cat $f`
		markdown-toc -i $f
		after=`cat $f`

		if [[ "$before" != "$after" ]]; then
			echo "$f has an outdated table of contents"
			ERROR=1
		fi
	fi

done

# Make sure to return with an error if anything changes
# so that Travis will fail.
if [[ $ERROR == 1 ]]; then
	exit -1
fi
