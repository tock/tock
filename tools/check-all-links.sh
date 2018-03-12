#!/usr/bin/env bash

### Scan all markdown files and check for broken links.
###
### Requirements:
###
###     gem install awesome_bot
###
###     Then patch /usr/local/lib/ruby/gems/2.4.0/gems/awesome_bot-1.17.2/lib/awesome_bot/check.rb
###     to set `head = true`.

red=`tput setaf 1`
green=`tput setaf 2`
reset=`tput sgr0`

# Keep track of how many READMEs have broken links in them.
let FAIL=0

# Iterate every directory in the repo.
for D in $(find . -mindepth 1 -type d); do
	pushd $D > /dev/null

	# Iterate every markdown file in the folder
	for MD in $(find . -maxdepth 1 -type f -name "*.md"); do
		# Check that this .md file is actually in the repo. Ignore files
		# that may have come from submodules or npm packages or other sources.
		git ls-files --error-unmatch $MD > /dev/null 2>&1
		if [[ $? -eq 0 ]]; then

			printf "CHECKING ${D:2}/${MD:2}"

			let LAST_FAIL=$FAIL

			# Run the actual check on
			OUT=`awesome_bot --allow-dupe --allow-redirect --skip-save-results --allow 405 --base-url https://github.com/tock/tock/blob/master/${D:2}/ $MD`
			let FAIL=FAIL+$?

			# If non-zero return code print the awesome_bot output and failed links.
			if [[ $FAIL-$LAST_FAIL -ne 0 ]]; then
				printf " ${red}FAIL${reset}\n"
				echo "$OUT"
				echo
			else
				printf " ${green}SUCCESS${reset}\n"
			fi
		fi

	done

	popd > /dev/null
done

exit $FAIL
