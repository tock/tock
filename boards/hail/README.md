Hail: Platform-Specific Instructions
=====================================

<img src="media/hail_reva_noheaders_1000x536.jpg" width="30%">
<img src="media/hail_breadboard_1000x859.jpg" width="30%">
<img src="media/hail_reva_noheaders_labeled.png" width="30%">

Hail is an embedded IoT module for running Tock.
It is programmable over USB, uses BLE for wireless, includes
temperature, humidity, and light sensors, and has an onboard accelerometer.
Further, it conforms to the Particle Photon form-factor.

Setup
-----
See ../../doc/Getting_Started.md for how to set up Rust and arm-non-eabi.

## Rust

A specific version of rustc is needed.

See ../../doc/Getting_Started.md for details.

```bash
$ curl https://sh.rustup.rs -sSf | sh
$ rustup override set nightly-2016-07-29
```

### `arm-none-eabi` toolchain

Use Brew, not Macports.

Macports `/opt/local/bin/arm-none-eabi-gcc --version` returned `5.1.0` and running `make program` failed in `tock/boards/hail`.  See [Hail: mac port arm-none-eabi-gcc 5.1.0 fails to compile program](https://github.com/helena-project/tock/issues/229)

To install Brew:

```bash
/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
brew tap ARMmbed/homebrew-formulae
brew update
```

However, "brew install arg-non-eabi-gcc" failed:

```bash
bash-3.2$ brew install arm-none-eabi-gcc
==> Installing arm-none-eabi-gcc from armmbed/formulae
==> Downloading https://launchpad.net/gcc-arm-embedded/5.0/5-2016-q3-update/+download/gcc-arm-none-eabi-5_4-2016q3-20160926-ma
Already downloaded: /Users/cxh/Library/Caches/Homebrew/arm-none-eabi-gcc-5-2016-q3-update.tar.bz2
==> cp -r arm-none-eabi bin lib share /usr/local/Cellar/arm-none-eabi-gcc/5-2016-q3-update/
Error: parent directory is world writable but not sticky
Please report this bug:
  https://git.io/brew-troubleshooting
/System/Library/Frameworks/Ruby.framework/Versions/2.0/usr/lib/ruby/2.0.0/tmpdir.rb:92:in `mktmpdir'
/usr/local/Homebrew/Library/Homebrew/utils/fork.rb:6:in `safe_fork'
/usr/local/Homebrew/Library/Homebrew/formula_installer.rb:597:in `build'
/usr/local/Homebrew/Library/Homebrew/formula_installer.rb:260:in `install'
/usr/local/Homebrew/Library/Homebrew/cmd/install.rb:301:in `install_formula'
/usr/local/Homebrew/Library/Homebrew/cmd/install.rb:194:in `block in install'
/usr/local/Homebrew/Library/Homebrew/cmd/install.rb:194:in `each'
/usr/local/Homebrew/Library/Homebrew/cmd/install.rb:194:in `install'
/usr/local/Homebrew/Library/Homebrew/brew.rb:94:in `<main>'
```

The solution:
```
sudo chmod +t /private/tmp/
brew install arm-none-eabi-gcc
```

Plug the Hail in using a USB Micro B cable. (The [Hail hardware repo](https://github.com/lab11/hail/blob/master/hardware/hail/rev_b/hail_bom.txt) lists a [MICRO_USB_B_HIROSE_ZX62R-B-5P](http://www.digikey.com/product-detail/en/hirose-electric-co-ltd/ZX62R-B-5P/H11574CT-ND/1787106T))


Programming Hail over USB requires the `tockloader` utility. To install:

    sudo pip install tockloader

Under Mac OS X

    sudo port install python36 py36-pip
    sudo port select --set pip pip36
    sudo pip-3.6 install tockloader

Under Mac OS X, tockloader was not present? See [Macports: Python 3.6: tockloader executable not created by pip?](https://github.com/helena-project/tockloader/issues/2)

The fix:
```
sudo ln -s /opt/local/Library/Frameworks/Python.framework/Versions/3.6/lib/python3.6/site-packages/tockloader/main.py /opt/local/bin/tockloader
sudo chmod a+x /opt/local/Library/Frameworks/Python.framework/Versions/3.6/lib/python3.6/site-packages/tockloader/main.py
```


Programming the Tock Kernel and Apps
------------------------------------

To program the kernel for Hail:

```bash
cd tock/boards/hail
make program
```

To program an application:

```bash
cd tock/userland/examples/blink
make TOCK_BOARD=hail program
```

You can also specify the serial port to use:

```bash
make program PORT=/dev/ttyUSB0
```

Under Mac OS X, plug in the board and use `ls -ltr /dev/tty*` to find a recently created device:

```
bash-3.2$ ls -ltr /dev/tty* | tail
crw--w----  1 cxh   tty     16,   4 Dec 28 17:29 /dev/ttys004
crw--w----  1 cxh   tty     16,  10 Dec 28 19:59 /dev/ttys010
crw--w----  1 cxh   tty     16,   9 Dec 28 20:03 /dev/ttys009
crw--w----  1 cxh   tty     16,   8 Dec 28 20:03 /dev/ttys008
crw-rw-rw-  1 root  wheel   18,   6 Dec 28 20:04 /dev/tty.usbserial-00002014
crw--w----  1 cxh   tty     16,  11 Dec 28 21:00 /dev/ttys011
crw--w----  1 cxh   tty     16,   1 Dec 28 21:01 /dev/ttys001
crw--w----  1 cxh   tty     16,   6 Dec 28 21:01 /dev/ttys006
crw-rw-rw-  1 root  wheel    2,   0 Dec 28 21:01 /dev/tty
crw--w----  1 cxh   tty     16,   5 Dec 28 21:01 /dev/ttys005
bash-3.2$
```

Given the above, the command to use would be:

```
make PORT=/dev/tty.usbserial-00002014 program
```
