# Installation

Currently, the only way to obtain the Poststation executables is to
[Contact OneVariable](mailto:contact@onevariable.com) for early access.

## MacOS

You should have received a file named `poststation-aarch64-apple-darwin-signed.zip`.

You can unzip this file in the Finder, or run:

```sh
unzip poststation-aarch64-apple-darwin-signed.zip
```

This contains an application package called `Poststation.app`. This folder contains
the following contents:

```
Poststation.app/
Poststation.app/Contents/
Poststation.app/Contents/_CodeSignature/
Poststation.app/Contents/MacOS/
Poststation.app/Contents/Info.plist
Poststation.app/Contents/_CodeSignature/CodeResources
Poststation.app/Contents/MacOS/poststation
```

This is a signed package, and is a command line executable. You can launch the application
on the command line:

```sh
$ ./Poststation.app/Contents/MacOS/poststation
```

MacOS does not require any additional steps for permissions.

## Linux

You should have received one of the following files:

* `poststation-aarch64-unknown-linux-musl.zip`
    * This will work on 64-bit ARM linux targets, including the newer Raspberry Pi devices
    * You can extract the contents with `unzip poststation-aarch64-unknown-linux-musl.zip`
* `poststation-x86_64-unknown-linux-gnu.tar.xz`
    * This will work on 64-bit AMD/Intel linux targets, including most desktop/laptop systems
    * You can extract the contents with `tar xf poststation-x86_64-unknown-linux-gnu.tar.xz`

These archives will contain a single binary called `poststation`. This can be executed
on the command line:

```sh
./poststation
```

### `udev` rules

Depending on the devices you are using, you may need to add udev rules for the VID and PID of
connected devices, in order to allow access to connected USB devices. It is **NOT** recommended
to run `poststation` as a root user or with `sudo`.

You may need to repeat this process multiple times for each device!

TODO: Add example for the `examples/` repo, where the VID is `0x16c0` and the PID is `0x27DD`.
The [probe-rs docs on udev](https://probe.rs/docs/getting-started/probe-setup/#linux%3A-udev-rules)
are a good place to refer to.

Rough steps are:

* Write a rules file and place it in `/etc/udev/rules.d`
* Run `udevadm control --reload` to ensure the new rules are used.
* Run `udevadm trigger` to ensure the new rules are applied to already added devices.

## Windows

You should have received a file called `poststation-x86_64-pc-windows-msvc.zip`.

You can unzip this in the file explorer. This contains a single file, `poststation.exe`.

You can run this on the command line:

```sh
poststation.exe
```

### Permissions

TODO: I don't think that windows needs any additional permissions for this? If you have issues
then please [open an issue](https://github.com/OneVariable/poststation-util/issues)!
