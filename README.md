# Poststation Utilities

Public libraries and utilities for using `poststation`. Repo highlights:

## `crates/`

These are crates that are depended on by other parts of this repository,
and are not typically intended to be used directly by users of Poststation.
Currently this is just `crates/poststation-api-icd`, which contains types
used to communicate with the Poststation server on the host side.

## `examples/`

This folder contains end-to-end examples usable to demonstrate functionalty
of Poststation. See [`examples/README.md`](./examples/README.md) for more
details.

## `templates/`

This folder is intended to contain up-to-date templates that can be used to
start a project. You should generally be able to copy the directory, rename
the projects, and use that as the beginning of your Poststation compatible
project.

## `tools/`

This folder contains tools that end-users are expected to be used by developers
working with Poststation.

## License

The `poststation` server is proprietary. Crates in this repository are licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

