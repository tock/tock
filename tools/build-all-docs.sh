#!/usr/bin/env bash

# Parse a search-index.js file to get the known crates.
function get_known_crates {
	FILE=$1

	# This sed seems to be okay x-platform bsd/gnu
	FOUND_CRATES=`sed -nE "s/.*searchIndex\[\"([a-z0-9_-]*)\"\].*/\1/gp" $FILE`
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
	BUILT_CRATES=$(get_known_crates boards/$BOARD/target/thumb*-none-eabi/doc/search-index.js)

	# Get any new crates.
	NEW_CRATES=" ${BUILT_CRATES[*]} "
	for item in ${EXISTING_CRATES[@]}; do
		NEW_CRATES=${NEW_CRATES/ ${item} / }
	done

	# Copy those crates over.
	for item in ${NEW_CRATES[@]}; do
		cp -r boards/$BOARD/target/thumb*-none-eabi/doc/$item doc/rustdoc/

		# Add the line to the search-index.js file.
		SEARCHINDEX=`grep "searchIndex\[\"$item\"\]" boards/$BOARD/target/thumb*-none-eabi/doc/search-index.js`

		# nothing in-place is x-platform bsd/gnu (os x defaults...)
		/usr/bin/awk -v var="$SEARCHINDEX" "/initSearch/{print var}1" doc/rustdoc/search-index.js > doc/rustdoc/search-index-new.js
		mv doc/rustdoc/search-index-new.js doc/rustdoc/search-index.js
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
add_board nrf51dk
add_board nrf52dk

# Temporary redirect rule
# https://www.netlify.com/docs/redirects/
cat > doc/rustdoc/_redirects << EOF
# While we don't have a home page :/
/            /kernel            302
EOF
