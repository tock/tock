#!/usr/bin/env python3

'''
Check if all of the available capsules are documented in the README.
'''

import os
import re

SKIP = ['/mod.rs',
        'src/lib.rs',
        '/test',
        'src/driver.rs',
        'src/rf233_const.rs']


documented_capsules = []
implemented_capsules = []

# Find all documented capsules
with open('capsules/README.md') as f:
	for l in f:
		items = re.findall(r".*\((src/.*?)\).*", l)
		if len(items) > 0:
			for item in items:
				documented_capsules.append('capsules/{}'.format(item))


# Find all capsule source files.
for subdir, dirs, files in os.walk(os.fsencode('capsules/src/')):
	for file in files:
		filepath = os.fsdecode(os.path.join(subdir, file))

		# Get just the part after /src, effectively.
		folders = filepath.split('/')
		filepath = '/'.join(folders[0:3])

		# Skip some noise.
		for skip in SKIP:
			if skip in filepath:
				break
		else:
			implemented_capsules.append(filepath)


# Calculate what doesn't seem to be documented.
missing = list(set(implemented_capsules) - set(documented_capsules))


print('The following capsules do not seem to be documented:')
for m in sorted(missing):
	print(' - {}'.format(m))

