import sys

with open(sys.argv[1], "rb") as f:
    cnt = 7
    s = ["00"]*8
    while True:
        data = f.read(1)
        if not data:
            print(''.join(s))
            exit(0)
        s[cnt] = "{:02X}".format(data[0])
        if cnt == 0:
            print(''.join(s))
            s = ["00"]*8
            cnt = 8
        cnt -= 1
