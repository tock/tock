#!/usr/bin/env bash

set -e

# Parse a search-index.js file to get the known crates.
function get_known_crates {
	FILE=$1
	FOUND_CRATES=$(grep -o 'searchIndex\["[a-zA-Z0-9_-]*"\]' $FILE | cut -d'"' -f2)
	echo $FOUND_CRATES
}

# Function to add new board.
function add_board {
	BOARD=$1

	echo "Building docs for $BOARD"
	pushd boards/$BOARD > /dev/null
	make doc
	popd > /dev/null

	EXISTING_CRATES=$(get_known_crates doc/rustdoc/search-index.js)
	BUILT_CRATES=$(get_known_crates boards/$BOARD/target/thumb*-none-eabi*/doc/search-index.js)

	# Get any new crates.
	NEW_CRATES=" ${BUILT_CRATES[*]} "
	for item in ${EXISTING_CRATES[@]}; do
		NEW_CRATES=${NEW_CRATES/ ${item} / }
	done

	# Copy those crates over.
	for item in ${NEW_CRATES[@]}; do
		cp -r boards/$BOARD/target/thumb*-none-eabi*/doc/$item doc/rustdoc/

		# Add the line to the search-index.js file.
		grep "searchIndex\[\"$item\"\]" boards/$BOARD/target/thumb*-none-eabi*/doc/search-index.js >> doc/rustdoc/search-index.js

		# Then need to move `initSearch(searchIndex);` to the bottom.
		#
		# Nothing in-place (i.e. `sed -i`) is safely cross-platform, so
		# just use a temporary file.
		#
		# First remove it.
		grep -v 'initSearch(searchIndex);' doc/rustdoc/search-index.js > doc/rustdoc/search-index-temp.js
		# Then add it again.
		echo "initSearch(searchIndex);" >> doc/rustdoc/search-index-temp.js
		mv doc/rustdoc/search-index-temp.js doc/rustdoc/search-index.js
	done
}

# Delete any old docs
rm -rf doc/rustdoc

# Need to build one board to get things started.
echo "Building docs for hail"
pushd boards/hail > /dev/null
make doc
popd > /dev/null
cp -r boards/hail/target/thumbv7em-none-eabi/doc doc/rustdoc

# Now can do all the rest.
add_board imix
add_board nordic/nrf52dk
add_board nordic/nrf52840dk
add_board launchxl

# Temporary redirect rule
# https://www.netlify.com/docs/redirects/
cat > doc/rustdoc/_redirects << EOF
# While we don't have a home page :/
/            /kernel            302
EOF
