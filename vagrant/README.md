Vagrant
=======

Vagrant is tool for building and sharing development environments. To get
started first [install Vagrant](https://www.vagrantup.com/downloads.html) if
you have not already.


Vagrant Quick Start
-------------------

Vagrant is essentially a management layer over a virtual machine. To get started:

 1. `vagrant up` - (create and) start the virtual machine
 2. `vagrant ssh` - log in to the virtual machine
 3. `cd /tock` - access the repository; this is a shared folder with the tock
                 repository already checked out on your machine, so you can
                 use local and familiar editing tools
 4. `make` - build tock

If you won't be developing Tock for a while, you can stop the Tock VM via

 - `vagrant suspend` - Pauses the VM (more disk space, but faster)
 - `vagrant halt` - Shuts down the VM

In either case, simply `vagrant up` to begin working again.


Getting code onto hardware
--------------------------

To flash or program boards, you will need to enable USB passthrough using the
virtual machine manager of your choice for the board-specific programmer for
your device. As the `tock/` folder is shared, you may also simply use this
vagrant image for building and the host machine directly to flash images.
