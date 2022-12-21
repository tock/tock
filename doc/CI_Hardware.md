# Tock Hardware-in-the-Loop CI
In addition to software tests, Tock allows platform maintainers to deploy CI runners for hardware platforms, to ensure continuous on-metal performance correctness. The hardware CI is federated, and maintainers for each hardware platform are responsible for ensuring that their hardware CI system is set up, accessible, and runs correctly.

The Tock core team maintains the following hardware CI instances:
- [Nrf52840dk](../boards/nordic/nrf52840dk)	UC San Diego; contact: Pat Pannuto <ppannuto@ucsd.edu>, Anthony Quiroga <aquiroga@ucsd.edu> 
- [Hail (WIP)](https://github.com/tock/tock/tree/master/boards/hail)	UC San Diego; contact: Pat Pannuto <ppannuto@ucsd.edu>, Anthony Quiroga <aquiroga@usd.edu>

<!-- toc -->

- [How To Set Up a New Instance](#how-to-set-up-a-new-instance)
  * [Getting Started](#getting-started)
  * [Troubleshoot](#troubleshoot)
    + [To run on your local-host (instead of Github)](#to-run-on-your-local-host-instead-of-github)
    + [WiFi not connected after first boot](#wifi-not-connected-after-first-boot)
    + [Action runner: **command not found**](#action-runner-command-not-found)
  * [Creating the Configuration File](#creating-the-configuration-file)
- [How Instances Work](#how-instances-work)
  * [Process of Instances](#process-of-instances)
    + [What are actions/runners?](#what-are-actionsrunners)
  * [Configuration Files](#configuration-files)
  * [Configuring the Raspberry Pi](#configuring-the-raspberry-pi)
    + [Uart Setup](#uart-setup)
    + [I2C Setup](#i2c-setup)
    + [Downloading Pi-GPIO open source Python Library](#downloading-pi-gpio-open-source-python-library)
  * [Looking in the Workflow](#looking-in-the-workflow)
    + [Creating the Workflow File](#creating-the-workflow-file)
- [Where Tests are Located and How They Work](#where-tests-are-located-and-how-they-work)
  * [Location](#location)
  * [How Tests Work](#how-tests-work)
- [How Tests are Chosen](#how-tests-are-chosen)
- [What to do if Something Goes Wrong](#what-to-do-if-something-goes-wrong)
  * [If a test fails](#if-a-test-fails)
  * [If there’s boot up issues of the Raspberry Pi](#if-theres-boot-up-issues-of-the-raspberry-pi)

<!-- tocstop -->

![Workflow](images/ci-hardware/workflow.png)

# How To Set Up a New Instance

You will need a local machine to run the unit tests that is attached to the Tock development board under test.

This section explains the software setup needed for this box. These directions assume you are using a Raspberry Pi (RPi), which will likely be easiest as there are already pins and drivers for GPIO, SPI, and I2C testing available for RPis, but in principle, any Linux-like platform should work.

**Note:** This machine should be on a stable, reliable network (i.e. a campus or office setup) to ensure it is always available to run tests. Also of note, runners may run arbitrary code from community-generated Pull Requests (PRs), which may create a security risk. Once initial configuration is complete, the only network access the runner should need is to Github servers, so this box can be firewalled accordingly.

## Getting Started 
[Source](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/doc/CI_Hardware.md) of the **Getting Started** guide

**Note:** if you are having trouble connecting Raspberry Pi to wifi (in step 1), either see [Troubleshoot](#troubleshoot) or connect to ethernet if possible. Ethernet connection is reliable and seamless for the following process. To find your Pi's IP address, you can check your network devices for an easier method (if possible) rather than using command line arguments. 

1. Install Ubuntu Server on Raspberry Pi.
    * Follow the guide [here](https://ubuntu.com/tutorials/how-to-install-ubuntu-on-your-raspberry-pi#1-overview) until step 5. We do not need desktop environment.

**Note:** At this point, you should be on the Raspberry Pi through terminal or a linux subsystem (Ubuntu is a nice subsystem to use here for Windows Operating Systems).
For example, when you continue onto step 2, you should be working on a system like this (This is on Ubuntu):

![UbuntuExample](images/ci-hardware/ubuntuexample.png)

2. Install the [Requirements](https://github.com/tock/tock/blob/master/doc/Getting_Started.md#requirements) here.
3. Download and install JLinkExe
```bash
$ wget --post-data 'accept_license_agreement=accepted&non_emb_ctr=confirmed&submit=Download+software' https://www.segger.com/downloads/jlink/JLink_Linux_arm64.tgz
$ tar xvf JLink_Linux_arm64.tgz
$ sudo cp JLink_Linux_V700a_arm64/99-jlink.rules /etc/udev/rules.d/ # Depends on JLink version
 
# Add the Jlink directory to the path in .profile or .bashrc
# Example command below (XXX = version number)
# For example, with our commands above, XXX = 700a
$ echo 'export PATH=$HOME/JLink_Linux_VXXX_arm64:$PATH' >> ~/.profile

# Update .profile to include new path
$ source .profile
```
4. Install pip3, if not yet installed
```bash
sudo apt install python3-pip
```

**Note:** if python 3 is not installed, follow these commands and go back to step 4.

```bash
$ sudo apt-get update
$ sudo apt-get install python3.6 #python version can change
```

5. Checkout test harness in home directory 
```bash
cd ~; git clone https://github.com/goodoomoodoo/tock-test-harness.git
cd ~; git clone -b ci-test https://github.com/goodoomoodoo/libtock-c.git # link will vary for user specifications
```
6. Install gpiozero if not yet installed. (It should come with the image)
```bash
$ sudo apt install python3-gpiozero
$ sudo chown root:$USER /dev/gpiomem
$ sudo chmod g+rw /dev/gpiomem
```
7. Install linux library
```bash
$ sudo apt-get install -y gcc-arm-none-eabi
# Install bluetooth library
$ sudo apt-get install bluez bluez-tools pi-bluetooth 
# !IMPORTANT! Remember to reboot after installation
$ sudo reboot
```
8. Install python3 libraries
```bash
$ cd ~/tock-test-harness; pip3 install -r requirements.txt
```
**Note:** this step will have to be done by a member of @tock/core-wg

9. Hook up Action Runner of the Tock Repo to the Raspberry Pi.
    * Go to \'Settings\' and go to \'Actions\' down the list on the left
    * Under \'Self-hosted runners\' click **Add runner**
    * Follow the steps to set up action runner on Raspberry Pi

10. Add path to JLink to ```.path``` in Action Runner
```bash
$ echo "JLink_Linux_VXXX_arm64" > ~/actions-runner/.path # Path will vary to your version of JLINK
```

11. Configure the harness
    * Go to tock test harness directory
    ```bash
    $ cd ~/tock-test-harness
    ```
    * Run configuration wizard, which will create config.toml in this directory. For more information, go to [Creating the configuration File](#creating-the-configuration-file)
    * If script doesn't properly create "_work" directory in your "actions-runner" directory, create the directory.
    ```bash
    $ mkdir ~/actions-runner/_work
    ```
    * Then copy official Tock directory onto your Raspberry Pi in this directory or your forked version of the Tock Repository. 
    ```bash
    $ cd ~/actions-runner/_work
    $ git clone  https://github.com/tock/tock.git # Depends on user needs
    ```

## Troubleshoot
### To run on your local-host (instead of Github)

Make sure to add "self-hosted" as a label in runs-on for the job:

```
jobs:
  "job name":
    runs-on: self-hosted
 ```

### WiFi not connected after first boot

1. Configure network plan ```sudo vi /etc/netplan/50-cloud-init.yaml```

```yaml
# This file is generated from information provided by the datasource. Changes
# to it will not persist across an instance reboot. To disable cloud-init's
# network configuration capabilities, write a file
# /etc/cloud/cloud.cfg.d/99-disable-network-config.cfg with the following:
# network: {config: disabled}
network:
    ethernets:
        eth0:
            dhcp4: true
            optional: true
    version: 2
    wifis:
        wlan0:
            dhcp4: true
            optional: true
            access-points:
                "SSID_name":
                    password: "WiFi_password"
```
2. Generate and apply new network plan
```sudo netplan generate; sudo netplan apply```

### Action runner: **command not found**

If you experience issue with the command not being found, but you are very sure that you have installed the corresponding software. Try to uninstall the runner and reinstall it.

```bash
# 1. Uninstall Action Runner Server
$ cd ~/actions-runner
$ sudo ./svc.sh stop
$ sudo ./svc.sh uninstall
$ ./config.sh remove

# 2. Reinstall Action Runner Server
# Follow the guide in Getting Started.
```
**Note**: Possible reason to this issue is that the server environment and path varaible cannot be update after it has started. Thus, it requires a reconfiguration to included the updated path and environment variables.

## Creating the Configuration File
After all is set up, the first step in creating an instance is to run the [runner_init](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/tools/tock-test-harness/runner_init.py) script to create a configuration file for the raspberry pi you just set up.
Be sure to change permission to allow execution of the script:
```bash
$ sudo chmod u+x runner_init.py
```

**Note:** runner_init.py is not a self executable script as shebang was not added to it. To properly run it, use this form of command:
```bash
$ python3 runner_init.py
```

This configuration file will be used in the workflow of actions for github whenever an update is pushed to be accessed by the runners, which will be denoted as config.toml for simplicity sake here.
This file is important for the workflow of hardware continuous integration, as this file denotes what board the Raspberry Pi is confiugred to test on, where the board files are located in the Raspberry Pi, and when the Raspberry Pi should be running based on the harness ID. 

What this **configuration file** contains is:
- Title
    - Name of the configuration. 
    - This can be anything you want but make sure to title it appropriately.
- Board name
    - This will be provided when the executable is called to provide which boards are supported. Choose which board you are setting this Raspberry Pi to test that is supported. You can NOT use a board that is not supported by the Tock OS, or else testing scripts will **NOT** work. For example, to use the specific nrf52480dk board, make sure the path is to this board, but the board name should be listed as "nrf52dk".
    - Note, a single Raspberry Pi can only be configured to run tests in one board at a time. To change what board is ran on the Raspberry Pi, re-run this executable and delete the previous configuration file.
- Board Path
    - This is the path of the Tock supported board in the official tock repository. The path must be provided when setting up the configuration. This is essentially the directory relative to the Tock boards directory, and should be the speicifc board you are intending to test/use on this Raspberry Pi.
    - Note, a single Raspberry Pi can only be configured to run tests in one board at a time. To change what board is ran on the Raspberry Pi, re-run this executable and delete the previous configuration file.
- Harness ID
    - This is the specific Identity Number of the Raspberry Pi, which can be found with the command:
    ```bash
    $ cat /proc/cpuinfo
    ```
    - It is denoted by the tag "Serial" when printed on the terminal. This will be used in the ci_test.config.toml which is kept for each board type to run tests on in the runner and the configuration file can be set to run on specific Raspberry Pis with this Harness ID.
- Communication Protocol 
    - Used to run tockloader
    - The list of communication protocols are provided, and one chosen must be in the list. 
        - Typically Jlink is the one chosen here
- I2C On Boot
    - Used to run an i2c testing script on boot
    - I2C Master Tests (When RPi runs as Slave) run into an issue when the Raspberry Pi boots up, the test initially fails for the first run due to an empty buffer transfer, but following executions of the testing script pass. It only fails in the first execution of the testing script. More info here [i2cboot](https://github.com/AnthonyQ619/libtock-c/tree/ci-test/examples/ci-tests/boot-up)
    - Answers to this prompt is Yes/No, which is based on whether the board the Raspberry Pi is being configured to has an I2C test or not
        - Yes: Runs the script and commits a i2c Master Reception transaction on the board. More info on script in `i2cboot` link above
        - No: Ignores the I2C test and instantly returns.


When runner_init script is executed, there will be prompts to fill in this information and the configuration file will be created based on the responses, filling in the file with the information listed above.

**Example Configuration File:**

```toml
title = "pi_test"

[env]
board = "nrf52dk"
path = "{home}/actions-runner/_work/tock/tock/boards/nordic/nrf52dk/"
harness_id = "{Pi ID}"
communication_protocol = "jlink"
i2c_on_boot = "yes"
```

**Note:** {home} and {Pi ID} would depend on your configuration and Pi, where {home} will be your home directory to start the path, and {Pi ID} will be your specific Raspberry Pi ID.

## Add I2C Script to bashrc File

After configuration file is made, head into home directory with this command:

```bash
$ cd ~
```
Then, once in home directory, open your bashrc file to add the proper i2c on boot testing script to run whenever the RPi is ssh into from terminal
```bash
$ sudo nano /home/ubuntu/.bashrc
```
Once in the bashrc file, add the following two lines
```bash
echo Running at boot 
sudo python /home/ubuntu/libtock-c/examples/ci-tests/boot-up/i2cboot.py
```
This will then run the script whenever the RPi is ssh into from terminal. Thus, it's important to ssh into the RPi initially on boot before left to run as a test harness, or i2c test will fail in the initial run.

**Note:** For more info how bashrc file works, check this [link](https://www.dexterindustries.com/howto/run-a-program-on-your-raspberry-pi-at-startup/) on look at method 2

**Note:** For more info on the testing script, check this github [link](https://github.com/AnthonyQ619/libtock-c/tree/ci-test/examples/ci-tests/boot-up)
# How Instances Work
Instances in this case are workflows with Github actions that set runners to complete said actions. This is how testing is conducted in Hardware Continuous Integration. The entire process is taken within the workflow through the yml files, which are another type of configuration file. Currently using the tock-hw-ci.yml as the workflow to run tests, in which we will use as an example throughout this document. (This file is located [here](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/.github/workflows/tock-hw-ci.yml))

## Process of Instances
As stated instances work through GitHub actions that set up runners. These are called workflows, and are set to call whenever a specific action occurs.

### What are actions/runners?
Github actions are automated tasks that the user sets up for software development. These actions are event-driven, meaning that the tasks, or set of commands, are run after a specified event has occurred. In our case, this would be everytime someone pushes changes to the repository to update software to certain Tock supported boards, and this would automatically run tasks that execute tests to verify such updates work as intended. 

Github actions can be used to set up runners, which are [servers that have the GitHub Actions runner application installed.](https://docs.github.com/en/actions/learn-github-actions/introduction-to-github-actions) Runners can be run through the github server, or be hosted by the user locally, which we typically do the latter. A runner listens for jobs, does one job at a time, and reports progress and results back to github, keeping all information at one place for convenience. 

## Configuration Files
Before looking at the workflow, there are two configuration files that are important to the whole process of hardware CI and how it’s currently working through the main executable file that is called to run on runners. The two configuration files are:
- **Config.toml**
    - This file is created when runner_init, the script used when setting up the Raspberry Pi, is run, which holds information about the target board and the harness id of the Raspberry Pi. Information that is held in this file was explained in the [previous section](#creating-the-configuration-file).
- **CI_test.config.toml** (located [here - nrf52840dk board](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/boards/nordic/nrf52840dk/ci_test.config.toml)) Note: Each specific board will have this file when Hardware CI is implemented for said board.
    - This configuration file is made for each board that is supported by the Tock OS that has Hardware CI set. Currently, the only board that has this file is the nrf52840dk board. This toml file contains board-specific test information, such as what tests are run for all Raspberry Pi’s, and what tests are to be performed on specific Raspberry Pi's harness ID. Note that the current setting for run is the default install script which can be changed to other scripts. Each test configuration file for each board will have a file in this format, and it’s necessary to be able to run tests.
    - **The file contains**
        - Script Object:
            - Script.install specifier:
                - Script for installing Tock OS into Raspberry Pi. “run” variable holds the install script, and “make install” is the default.
            - Script.test.all specifier:
                - Script for pre/post run installations for testing on the raspberry pi. Default runs if script.test is not specified. The all in the specifier denotes this test script to run on all Raspberry Pi’s with this board.
        - Test Object:
            - Test.all specifier:
                - Tests to be built, installed, and tested on the raspberry pi. The “app” variable holds the list of tests to be conducted. The “all” in the specifier can be changed to a Raspberry Pi ID to conduct specific tests on specific Raspberry Pi’s.

## Configuring the Raspberry Pi

The Raspberry Pi does not have Uart, I2C, and SPI configured by default,so there are several things that must be done to enable these interfaces to properly test Tock hardware. 

### Uart Setup
[Source](https://askubuntu.com/questions/1254376/enable-uart-communication-on-pi4-ubuntu-20-04)

Follow these commands to setup Uart on raspberry to transmit, and receive messages. This specifically allows the user to send data through port ttys0.


1. Add `enable_uart=1` to `/boot/firmware/config.txt`
2. Remove `console=serial0,115200` from `/boot/firmware/cmdline.txt` on Ubuntu 
3. Make sure you have `pyserial` installed if you're using the python serial library, not `python-serial` from `apt`. **NOTE THIS IS OPTIONAL**
4. Setup udev rules
    - Create file with this command.
    ```bash
        sudo nano /etc/udev/rules.d/10-local.rules
    ```
    - put below content in new file
    ```bash
        KERNEL=="ttyS0", SYMLINK+="serial0" GROUP="tty" MODE="0660"
        KERNEL=="ttyAMA0", SYMLINK+="serial1" GROUP="tty" MODE="0660"
    ```
    - Reload udev rules: 
        ```bash
        sudo udevadm control --reload-rules && sudo udevadm trigger
        ```
5. Disable the Serial Service which used the miniUART
    ```bash
        sudo systemctl stop serial-getty@ttyS0.service
        sudo systemctl disable serial-getty@ttyS0.service
        sudo systemctl mask serial-getty@ttyS0.service
    ```
6. Add the user which will use the miniUART to `tty` and `dialout` group
    ```bash
        sudo adduser ${USER} tty
        sudo adduser ${USER} dialout
    ```
7. Update the permissions for group read on the devices
    ```bash
        sudo chmod g+r /dev/ttyS0
        sudo chmod g+r /dev/ttyAMA0
    ```
6. Finally, reboot Ubuntu 20.04, then both `hci0` and `/dev/ttyS0` can work at the same time 

**Sanity Check**
If UART is properly setup, using the command `ls -l /dev` you should see the image below after scrolling up slightly.
```bash
    ls -l /dev
```
![UartSanity](images/ci-hardware/uartsanity.png)
[Source](https://www.electronicwings.com/raspberry-pi/raspberry-pi-uart-communication-using-python-and-c)

### I2C Setup
[Source of Guide](https://askubuntu.com/questions/1273700/enable-spi-and-i2c-on-ubuntu-20-04-raspberry-pi)

Again, I2C is disabled by default on the Raspberry Pi, and since we are using ubuntu server, we have to manually setup the config file to enable this communication protocal. Follow the steps below to enable I2C. There's also a sanity check I have provided for you to double check if the set up is correct.

1. Access the config file
    ```bash
        sudo nano /boot/firmware/config.txt
    ```
    - Then, add the following two lines at the bottom of the text:
    `dtparam=i2c1=on` and `dtparam=i2c_arm=on` respectively.
2. Install relevant packages. You'll want `i2c-tools` and `python3-smbus` at least, but I'd recommend getting `libi2c-dev` if that's your library of choice.
3. Try probing the bus as user and root:
    ```bash
        sudo i2cdetect -y 1 # (or 0, I2C bus number is hardware-dependent)
    ```
    - If this gives you an address matrix, I2C is on and working.
    ```bash
        i2cdetect -y 1 (or 0)
    ```
    - If this gives you a permission error, perform the next step. Otherwise, skip it.
4. Create an I2C usergroup, apply it to the bus and add your user to this group:
    ```bash
        sudo groupadd i2c (group may exist already)

        sudo chown :i2c /dev/i2c-1 (or i2c-0)

        sudo chmod g+rw /dev/i2c-1

        sudo usermod -aG i2c *INSERT YOUR USERNAME*
    ```
    - Reboot:
    ```bash
        sudo reboot
        # Log back in
        i2cdetect -y 1
    ```

**Sanity Check**
Using the command `i2cdetect -y 1`, if you see the matrix below then I2C is properly set up.
```bash
    ubuntu@ubuntu:~/$ i2cdetect -y 1
     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
00:          -- -- -- -- -- -- -- -- -- -- -- -- --
10: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
20: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
30: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
40: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
50: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
60: -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
70: -- -- -- -- -- -- -- --

```

### Downloading Pi-GPIO open source Python Library
For more details or troubleshooting, [see the complete pigpio documentation](https://abyz.me.uk/rpi/pigpio/download.html).

The Raspberry Pi does not have I2C slave enabled by default on traditional I2C pins. In order to have the Raspberry Pi act as a slave, we must use Broadcom Pins 10 & 11 (Or GPIO Pins 19 & 23). Then, we also to have to utilize the open source Python Library known as "pigpio". To install this Python lirbary onto the Raspberry Pi, follow the steps below.

1. First, run an update to get current version:
    ```bash
        sudo apt-get update
    ```

2. For Installation, follow these lines:
    ```bash
        wget https://github.com/joan2937/pigpio/archive/master.zip
        unzip master.zip
        cd pigpio-master
        make
        sudo make install
    ```

3. If python section did not download properly, run the following command:
    ```bash
        sudo apt install python-setuptools python3-setuptools
    ```

4. To run, run the follow commands:
    - To run pigpio daemon 
        ```bash
            sudo pigpiod
        ```
    - To shut down pigpio daemon
        ```bash
            sudo killall pigpiod
        ```

If commands on **Step 4** do not output anything, pigpio is properly setup. Slave access is now available on Raspberry Pi.

## Looking in the Workflow 

**Note:** File we are using as example is the ([tock-hw-ci.yml](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/.github/workflows/tock-hw-ci.yml))

These files are the workflow of how instances work, setting up actions that create runners to run certain tasks or files.
These configuration files call the python executables located at each directory listed, known as “main.py”, through the workflow/runners.
The main.py file is the central part of how instances work, and it takes an argument for each job of the workflow, which are build, install, and test.
For example: 
- The workflow listed below is set to run when pushes or pull requests are called, then the listed jobs are executed.
- There are three jobs when the workflow is called to be executed. These jobs are: Build, Install, and test. Each job requires the previous job to be executed. Remember, each job is performed one at a time. Thus, the chronological order of each job is important as it’s necessary to build, install, and test in this exact order.
- **Build**
    - This job goes to the directory path to execute Main.py with the “-b” flag. The Main (located [here](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/tools/tock-test-harness/lib/main.py)) file then calls the runner file (located [here](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/tools/tock-test-harness/lib/Runner.py)), which contains the functions to actually build, install, and test the Raspberry Pi with the specific board setup. In this case, the build flag, “-b”, is the argument when calling main, which calls the Tock_build function in Runner.py, building the Tock OS for the specified board from the configuration settings in config.toml file for the Raspberry Pi. (The configuration file made from `runner_init`) 
- **Install**
    - When this job runs, it executes Main.py with the “-i” flag to run the install functions in Runner.py. This flag calls the Tock_install function which flashes the Tock OS bin to the board using information from the board-specific configuration from the Raspberry Pi, or config.toml. For install, the Tock_install function uses the run variable in the script object to flash the board and uses the postrun variable in the script to conduct a post-run installation script. A pre-install script can be created for the test configuration to be conducted as well, but this current board’s configuration file does not have this implemented.
- **Test**
    - When this job runs, it executes Main.py with the “-t” flag, calling the tock_test function in Runner.py. This function builds the tests by compiling them, installs the tests to the board, then runs the tests on the board. The tests to be installed are designated by the ci_test.config.toml file for each board. Currently, for the board we have set up for hardware CI, nrf52840dk, we have the test object have an “all” specifier in the test.config.toml file that  runs a set of tests for all boards of this type, regardless of harness ID of the RPi. We can set, later, for the test object to have certain Harness ID’s and Raspberry Pi’s run certain tests.  The runner.py, then, runs all the tests specified in the “test” object of the test configuration file for the specific Raspberry Pi and respective board. Through this process, this is how the tests are conducted on the board. 

### Creating the Workflow File
Each workflow file will be designed for eachboard, placed in `tock/.github/workflows` and be titled with whatever title you want it to be. Following the previous workflow example file, it will have three jobs that are build, install, and test. But, to create a proper workflow file, we must specify the board and the branch to listen to. As such, follow these steps to create a proper workflow for CI-testing.
1. Name the File
    - First, name the workflow file you want it to be titled. Then, on the file, we specify the name as such
    `name: {name of file}`
    In the example file, we used the name "tock-hw-ci"
2. Then, create the key `on` and label keys for what git actions you want the workflow to activate on. In the example, we have the git actions as keys `pull_request` and `push`. Thus with these git actions, the workflow will activate. We can also specify what branch these git actions will activate the workflow. In the example file, we label it as the main branch. An example would be:
    ```yaml
    on:
        push:
            branches: [ master ]
    ```
3. Now, add the following line:
    ```yaml
    env:
        CARGO_TERM_COLOR: always
    ```
4. Now, add the key `job` and the following keys `build` `install` and `test`.
    - the Build key will contain:
        ```yaml

        runs-on: [nrf52dk]
        steps:
        - uses: actions/checkout@v2
        - name: Build
            run: python3 ~/tock-test-harness/lib/main.py -b
        ```
    `runs-on` specifies that this workflow will only run on the runner with the `nrf52dk` label. the key `steps:` define what build will do, which uses the `actions/checkout@v2` and name for itself, then calls the `main.py` file with the "-b" flag to run the build functions. 
    - the Install key will contain:
        ```yaml

            runs-on: [nrf52dk]
            needs: build
            steps:
            - name: Install
                run: python3 ~/tock-test-harness/lib/main.py -i
        ```
    same with build, `runs-on` specifies that this portion will only run on the runner with the `nrf52dk` label. `needs:` specifies that the jobs associated must be achieved, which is a key, in order to be ran, that being `build` from previously. `steps` only contains the key `name:` and the file to run, being the `main.py` with the "-i" flag to indicate the installation functions. 
    - the Test key will contain:
        ```yaml

        runs-on: [self-hosted]
        needs: install
        steps:
        - name: HW Test
            run: python3 ~/tock-test-harness/lib/main.py -t
        ```
        Now, `runs-on` specifies that it will run with runners containing the label `self-hosted` only. Then, just like `install` test needs the job `install` to run before test can occur. `steps` contain the same as `install` with `name` and `run` which calls `main.py` with the flag "-t" to run the test functions. 


For a **sanity check**, here is what the example file looks like
```yaml
name: tock-hw-ci

on:
  push:
    branches: [ master ]
  pull_request:
    branches: [ master ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:

    runs-on: [nrf52dk]
    steps:
    - uses: actions/checkout@v2
    - name: Build
      run: python3 ~/tock-test-harness/lib/main.py -b
      
  install:
    
    runs-on: [nrf52dk]
    needs: build
    steps:
    - name: Install
      run: python3 ~/tock-test-harness/lib/main.py -i
      
  test:
    
    runs-on: [self-hosted]
    needs: install
    steps:
    - name: HW Test
      run: python3 ~/tock-test-harness/lib/main.py -t
      
```

# Where Tests are Located and How They Work

## Location
The tests are located in the “libtock-c” repository, and there is a list of tests held in the directory [libtock-c/examples/ci-tests](https://github.com/AnthonyQ619/libtock-c/tree/ci-test/examples/ci-tests). Tests are held in each folder, and the labels of each folder are the kind of tests programmed to run. For example, ble folder represents the ble test. To explain how the tests work with the multiple files for each test folder, I’ll use the gpio test as reference.

## How Tests Work 

**Note:** Example test we are working with is in the ([gpio folder](https://github.com/AnthonyQ619/libtock-c/tree/ci-test/examples/ci-tests/gpio))

There are two files for each test that are used for the actual process of testing, one makefile for the installation of the application onto the board/Raspberry Pi, and readme to explain the test. 
- Main.c (located [here](https://github.com/AnthonyQ619/libtock-c/blob/ci-test/examples/ci-tests/gpio/main.c)):
    - This file is what’s used to install the test, or application, onto the target board. This goes for all main files for the test folders. What this application does is toggle the gpio state for a certain period of time.
- Gpio_test.py (located [here](https://github.com/AnthonyQ619/libtock-c/blob/ci-test/examples/ci-tests/gpio/test.py)):
    - What this file does is monitor the state, the message, or the action on the target board. This goes for all python files for the test folders. What this specific python program does is that it is monitoring the gpio state, and reading into it, on the target board and gathering any meaningful information to dump. 
    - In sum, the main file will install the application onto the target board, and the test file, the python file, will be used to monitor the target board, reading for any messages, state, or action that is meaningful. The test is the process of running both of these files onto the target board, and compiling any messages from the python file to determine whether it succeeds or fails the test. 

This is the workflow of how instances work through Hardware CI. Ideally, every board will have this workflow to be called whenever updates occur and follow this form processing. 
![Initialization/Testing](images/ci-hardware/processdiagram.png)

# How Tests are Chosen
Tests are chosen when initially running the program through the setup/configuration of the Raspberry Pi, where you build a configuration file that is run through the runners and when creating the file. Essentially, when setting up the Raspberry Pi, you create a configuration file on boot up with the get started guide listed that sets the specified board type, and that board type will have a test configuration file containing the tests for all boards of that type and/or the specific Raspberry Pi through the harness Identity number. Then, tests are chosen for the specified board that is supported by the Tock OS.
As noted in the previous section we have a test.config.toml file that contains information of the test to run for each board in the Tock/Boards directory , which contains each specific Tock supported board as well. Thus to Add tests, there are two primary steps:
- **Adding Tests**
    1. To add tests, you’ll need to create two files: a main.c (c) file and a test.py (python) file.
        - The c file will be titled “main” which is the application to be used to test on the target board. For more information on this file and what it does, look at the section How instances work - How tests work.
        - The python file will be named “test” which is the file that reads into any information from the application, and outputs any messages that are meaningful. This information can be the state, action, or messages from the application onto the target board.
        - Then, you can create a readme and makefile corresponding to the test to explain what functions are being tested on the board and to compile/build the application (test) for the board.
        - To add the test, you’ll push the files of the test with the title of the function being tested to the directory path libtock-c/examples/ci-tests in github. The full path will then be libtock-c/examples/ci-tests/{app} with “{app}” being the title to the test you made as the folder. This is necessary because the “main.py” file uses the path to build, install, and run the test.
    2. To add the tests to the boards you want to test, you’ll need to go to the specific board in Tock/Boards directory ([Official Tock Repository](https://github.com/tock/tock/tree/master/boards/nordic) or [Forked Repository  w/ file](https://github.com/AnthonyQ619/tock/tree/aq-config-updated/boards/nordic/nrf52840dk)), and in the boards directory, you’ll either edit, or create, the test.config.toml file for that board. There, you’ll go to the test object, or create one, and either add it to the “all” specifier for all raspberry Pi’s to run the test, or add the test to specific Raspberry Pi’s by making the specifier for the test object be the identity number of said raspberry pi. To see what’s included currently in the test configuration file for each board, visit the [nrf52840dk board](https://github.com/AnthonyQ619/tock/blob/aq-config-updated/boards/nordic/nrf52840dk/ci_test.config.toml) to see the contents and format. Also revisit the previous section discussing the test [configuration file](#configuration-files).

![Diagram for Adding/Choosing Tests](images/ci-hardware/testsdiagram.png)

# What to do if Something Goes Wrong

## If a test fails
Take a look at what type of test failed (in the output on github) and see what is the issue of the build of the board accordingly. The output in github currently prints what test failed to give the user a better understanding of what’s wrong in the installation.

## If there’s boot up issues of the Raspberry Pi
Go to the [Troubleshooting section](#troubleshoot)

**Further updates here will be made accordingly to new issues that arise.**
