# Command Line Options

The Poststation server contains a couple of options:

```sh
$ poststation --help

A tool for making it easy to talk to devices

Usage: poststation [OPTIONS]

Options:
      --simulator-devices <SIM_DEVICES>
          How many virtual simulator devices to spawn. Defaults to zero
      --interface-testers <INTERFACE_TEST_DEVICES>
          How many virtual simulator devices to spawn. Defaults to zero
      --headless
          Run the device in headless mode, instead of launching the TUI
      --config-path <CONFIG_PATH>

  -h, --help
          Print help
  -V, --version
          Print version
```

## `--simulator-devices <SIM_DEVICES>`

This can be used to spawn a few simple simulated devices that can be used
for testing interactions with the SDK or REST APIs.

## `--interface-testers <INTERFACE_TEST_DEVICES>`

This can be used to spawn simulated devices that exercise a larger portion
of possible [`postcard-schema`] types. This can be useful to ensure that
your application can correctly handle a variety of different kinds of
messages.

[`postcard-schema`]: https://docs.rs/postcard-schema

## `--headless`

This option disables the TUI interface, and instead prints logs to stdout.
This is intended to be used when installing poststation as a background
service.

When run in `--headless` mode, the `RUST_LOG` environment variable can be
used to control the log level of the system. See the [`EnvFilter` docs]
for examples of how to fine tune the logging.

[`EnvFilter` docs]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#example-syntax

## `--config-path`

This controls the path used for the configuration file used by the poststation
server. This overrides the default, and can be used for testing.
