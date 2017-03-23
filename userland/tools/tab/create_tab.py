#!/usr/bin/env python

import datetime
import io
import os
import sys
import tarfile

TAB_VERSION = 1

output_filename = sys.argv[1]
name = sys.argv[2]
inputs = sys.argv[3:]

metadata = []


metadata.append('tab-version = {}'.format(TAB_VERSION))
metadata.append('name = "{}"'.format(name))
metadata.append('only-for-boards = ""')
metadata.append('build-date = {}'.format(datetime.datetime.now().isoformat()[:19]+'Z'))


with tarfile.open(output_filename, 'w') as tar:
	for name in inputs:
		arcname = os.path.basename(name)
		tar.add(name, arcname=arcname)

	# Add metadata
	data = '\n'.join(metadata).encode('utf-8')
	file = io.BytesIO(data)
	info = tarfile.TarInfo(name='metadata.toml')
	info.size = len(data)
	tar.addfile(tarinfo=info, fileobj=file)
