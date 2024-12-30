# Poststation User Guide

This is a user guide for the Poststation tool.

## What is Poststation?

Poststation is a tool for connecting embedded devices and applications running on a PC,
laptop, or embedded linux system. It works with any device speaking the `postcard-rpc` protocol.

Poststation is used to:

* Discover and connect to externally connected embedded devices over USB, I2C, SPI, etc.
* Retrieve information about each of the connected devices, e.g. "Service Discovery"
* Provide API access for locally running user programs to interact with connected devices
* Maintain historical information about devices, including a history of messages sent to/from each device

A typical setup looks something like this:

```
┌───────────────────────────────────────────────────────────────────────────────┐
│ ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐                               │
│   ┌───────────────┬───┐       ┌─────────────┐                                 │
│ │ │ User Program  │SDK│◀──┐   │             │ │                               │
│   └───────────────┴───┘   │   │             │                                 │
│ │ ┌───────────────┬───┐   │   │             │ │                USB            │
│   │ User Program  │SDK│◀──┼──▶│ Poststation │────────┬──────┬──────┬──────┐   │
│ │ └───────────────┴───┘   │   │             │ │      ▼      ▼      ▼      ▼   │
│   ┌───────────────┬───┐   │   │             │     ┌────┐ ┌────┐ ┌────┐ ┌────┐ │
│ │ │ User Program  │SDK│◀──┘   │             │ │   │SDK │ │SDK │ │SDK │ │SDK │ │
│   └───────────────┴───┘       └─────────────┘     ├────┤ ├────┤ ├────┤ ├────┤ │
│ │                                             │   │MCU │ │MCU │ │MCU │ │MCU │ │
│                                                   └────┘ └────┘ └────┘ └────┘ │
│ │  ┌ ─ ─ ─                                    │                               │
│  ─ ─  PC  │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                │
│    └ ─ ─ ─                                                                    │
│ ┌───────────────┐                                                             │
└─┤ Typical Setup ├─────────────────────────────────────────────────────────────┘
  └───────────────┘
```

## Poststation for prototyping

If you are prototyping or building experimental projects such as demos, proof of concepts, or one-offs,
Poststation is intended to make your life easier by:

* Handling the communications, enumeration, and service discovery for you automatically
* Providing tools, like `poststation-cli`, that make it quick to work with
  devices interactively or via scripts on your PC
* Providing PC-side libraries, REST APIs, and socket interfaces for writing applications
  that speak with your devices
* Providing MCU-side libraries and templates, to make starting a project painless

Poststation is offered as a single user license, intended for prototyping and development.

## Poststation for production

Poststation is also intended to be a component you can use all the way through production.

It is lightweight, and can run on small Embedded Linux devices, as a background service managing
connections to microcontrollers on your product. It can serve as a single interface for monitoring
logs, sending commands, or triggering firmware updates, which you can orchestrate using your
existing application stack, whether that is in Rust, Python, Node, or whatever else.

If you are looking to ship Poststation as part of your product, or use it internally on more than
one machine, please [Contact OneVariable](mailto:contact@onevariable.com) for pricing.
