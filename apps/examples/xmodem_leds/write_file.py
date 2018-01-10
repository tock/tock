#!env python

f = open("dark.bin", 'wb');
f.write(b'\x00')
f.write(b'\x00')
f.write(b'\x00')
f.write(b'\x00')
for i in range(0,144):
    f.write(b'\xff')
    f.write(b'\x00')
    f.write(b'\x00')
    f.write(b'\x00')
f.close()

