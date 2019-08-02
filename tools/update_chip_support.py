#!/usr/bin/env python3

'''
Script to generate a table of which HILs each Tock chip supports. Adds it
to the `chips/README.md` file.

Example:
```
| HIL                    | cc26x2 | nrf51 | nrf52 | sam4l | tm4c129x |
|------------------------|--------|-------|-------|-------|----------|
| AES128                 |        |       | ✓     | ✓     |          |
| AES128CBC              |        |       |       | ✓     |          |
| AES128Ctr              |        |       | ✓     | ✓     |          |
| Adc                    |        |       | ✓     | ✓     |          |
```
'''

import os
import re

# Static info of chip crates that just support other chips.
SUBSUMES = {'nrf52': ['nrf5x'],
            'e310x': ['sifive'],
            'arty_e21': ['sifive']}



hils = {}

# Get the name of all HILs
for subdir, dirs, files in os.walk(os.fsencode('kernel/src/hil/')):
	for file in files:
		filepath = os.fsdecode(os.path.join(subdir, file))

		if filepath.endswith('.rs'):
			with open(filepath) as f:
				mod = os.path.splitext(os.path.basename(filepath))[0]
				for l in f:
					if l.startswith('pub trait'):
						items = re.findall(r"[A-Za-z0-9]+|\S", l)

						hil_name = items[2]
						if not 'Client' in hil_name:
							hils[hil_name] = {'module': mod, 'chips': []}

chips = []

# Get each chip and all HILs that chip implements.
for subdir, dirs, files in os.walk(os.fsencode('chips/')):
	for file in files:
		filepath = os.fsdecode(os.path.join(subdir, file))

		if '/src/' in filepath and filepath.endswith('.rs'):
			chip = filepath.split('/')[1]
			chips.append(chip)

			with open(filepath) as f:
				for l in f:
					# Find any line with `impl`
					if l.startswith('impl') and ' for ' in l:

						# Get the text before " for "
						half = l.split(' for ')[0]
						# Split strings apart from all other symbols
						items = re.findall(r"[A-Za-z0-9]+|\S", half)

						# Check each HIL to see if this `impl` line implements
						# that HIL.
						for hil in hils.keys():
							for item in items:
								if item == hil:
									hils[hil]['chips'].append(chip)
									break

# Calculate chips that should be ignored since they only support other chips.
subsumed = []
for k,v in SUBSUMES.items():
	subsumed += v

# Get only proper chips that are not just crates that support other chips.
chips = set(chips).difference(subsumed)

# Setup table and add the header row.
table = []
table.append(['HIL', *sorted(chips)])

# Add rows to the table, one row for each HIL.
for k,v in sorted(hils.items(), key=lambda x: '{}::{}'.format(x[1]['module'], x[0])):
	row = ['{}::{}'.format(v['module'], k)]

	# Skip any HILs that have no chip support. These are likely not HILs
	# that hardware chips implement.
	at_least_one = False

	for chip in sorted(chips):
		# Check if this chip or if any chip it subsumes implements this HIL.
		if chip in v['chips'] or (chip in SUBSUMES and len(set(SUBSUMES[chip]).intersection(set(v['chips'])))):
			at_least_one = True
			row.append('✓')
		else:
			row.append(' ')

	if at_least_one:
		table.append(row)

# Calculate the max widths of each column.
widths = [0]*len(table[0])
for row in table:
	widths[0] = max(len(row[0]), widths[0])
# The other columns we just need to look at the header
for i,item in enumerate(table[0]):
	widths[i] = max(len(item), widths[i])

# Generate the output table.
out = ''
for i,row in enumerate(table):
	# Use the widths to pad each item in each row.
	for j,item in enumerate(row):
		out += '| '
		out += '{1:<{0}s}'.format(widths[j]+1, item)

	out += '|\n'

	# After the first row add the "----" header marking row.
	if i == 0:
		for width in widths:
			out += '|'
			out += '-'*(width+2)
		out += '|\n'

# Update the chips README with the newly calculate table.
readme_first = ''
readme_second = ''
readme_state = 'start'
with open('chips/README.md') as f:
	for l in f:
		if readme_state == 'end':
			readme_second += l
		elif '<!--END OF HIL SUPPORT-->' in l:
			readme_state = 'end'
			readme_second += '\n'
			readme_second += l
		elif '<!--START OF HIL SUPPORT-->' in l:
			readme_state = 'skip'
			readme_first += l
			readme_first += '\n'
		elif readme_state == 'start':
			readme_first += l

with open('chips/README.md', 'w') as f:
	f.write(readme_first)
	f.write(out)
	f.write(readme_second)
