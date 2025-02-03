# Relationship to `postcard`

Poststation is part of the greater `postcard` family, and builds on top of it. Namely:

## `postcard` is the ENCODING format

[`postcard`] defines how we turn data types into bytes and back again. You can use `postcard` anywhere you want
to serialize/deserialize data. It serves as the "base layer" of the stack.

`postcard` is an Open Source library (MIT and Apache 2.0 licensed), and has a stable and public [specification].

[specification]: https://postcard.jamesmunns.com/


## `postcard-rpc` is the WIRE PROTOCOL

[`postcard-rpc`] defines how two devices communicate with each other. You can use `postcard-rpc` anywhere you
want two devices to communicate in a client/server role. `postcard-rpc` requires the use of `postcard` as the
encoding format.

`postcard-rpc` is an Open Source library (MIT and Apache 2.0 licensed), and will have a stable and public
specification in the near future (planned for 2025).

## Poststation is a DEVELOPER TOOL

[Poststation] manages connections to multiple devices, historical data, and connectors to other APIs and services.
Poststation uses `postcard` as the encoding format, and `postcard-rpc` as the wire protocol.

Poststation is a paid, closed source tool. It aims to be a handy toolkit, to save developer time from building
integrations and tooling on top of `postcard` or `postcard-rpc` from scratch.

[`postcard`]: https://github.com/jamesmunns/postcard/
[`postcard-rpc`]: https://github.com/jamesmunns/postcard-rpc/
[Poststation]: https://onevariable.com/poststation/


