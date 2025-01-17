//! API types and interfaces for interacting with Poststation
//!
//! This crate currently documents two main APIs:
//!
//! * A "postcard-rpc over sockets" API that is use for the Rust SDK and the CLI interface
//! * A "web tech" API, oriented towards REST and WebSockets that uses JSON for data.
//!
//! If you are communicating with Poststation from the Rust language, it is recommended to
//! use the postcard-rpc interface, generally through the `poststation-sdk` crate. If you are
//! using another language, you will probably want to use the REST flavored API.
//!
//! These different flavors of APIs will *generally* have similar interfaces, but the specific
//! format used in either may vary based on the realities of the interfaces. As an example, the
//! REST API may contain some data in the query parameters, rather than in the body of the request,
//! and may prefer encoding serial numbers as hex strings instead of a numerical `u64`, due to
//! the use of floating point numbers in JS itself as well as many JSON libraries.

pub mod postsock;

#[cfg(feature = "rest-api")]
pub mod rest;
