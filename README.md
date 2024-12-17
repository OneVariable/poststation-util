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
