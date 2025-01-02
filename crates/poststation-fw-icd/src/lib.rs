//! Reserved or "special" types and interfaces within Poststation
//!
//! ## NOTE
//!
//! For now, there is no way to "import" these definitions directly, so they
//! are provided as examples, and the types are expected to be used directly.
//!
//! See [postcard-rpc#71] for more information.
//!
//! [postcard-rpc#71]: https://github.com/jamesmunns/postcard-rpc/issues/71

#![cfg_attr(not(feature = "use-std"), no_std)]


#[allow(unused_imports)]
use postcard_rpc::{endpoint, topic};
use serde::{Serialize, Deserialize};
use postcard_schema::Schema;

/// # Unique ID
///
/// This is an endpoint for identifying the unique ID of the device. It is
/// expected to be unique across all devices connected to a Poststation server.
/// This value is expected to be constant over the entire lifetime of the device.
///
/// Common sources of this unique ID are:
///
/// * Serial number or unique ID of the processor
/// * Serial number or unique ID of the external flash
/// * Factory or programmed ID stored in non-volatile memory
///
/// If your device provides more or less than 64-bits of unique values, it is
/// recommended to use a deterministic way to extend or shorten the value, for
/// example taking the least significant bytes (to shorten) or hashing the available
/// information (to lengthen), for example using fnv64a.
pub mod unique_id {
    use super::*;
    pub type UniqueId = u64;
    endpoint!(GetUniqueIdEndpoint, (), UniqueId, "poststation/unique_id/get");
}

/// # Bridging
///
/// This is a currently experimental interface for transparent bridging, for
/// example over a radio link, where one device directly connected to Poststation
/// will maintain connections to other devices.
///
/// For example: an nRF52840 is used as a bridge, and connected via USB to the
/// PC running Poststation. The bridge then maintains a wireless network to other
/// devices with postcard-rpc, allowing the remote wireless devices to "appear as if"
/// they are directly connected to the PC running Poststation.
pub mod bridging {
    use super::*;
    // | Bridge2HostTopic          | ProxyMessage<'a>  | "bridge/to/host"  | cfg(not(feature = "use-std"))     |
    // | Bridge2HostTopic          | ProxyMessage      | "bridge/to/host"  | cfg(feature = "use-std")          |
    // | BridgeTableTopic          | BridgeTable       | "bridge/table"    |                                   |

    // TODO: The endpoint! macro doesn't support borrowing
    // #[cfg(not(feature = "use-std"))]
    // endpoint!(Host2BridgeEndpoint, ProxyMessage<'a>, ProxyResult, "poststation/host/to/bridge");

    #[cfg(feature = "use-std")]
    endpoint!(Host2BridgeEndpoint, ProxyMessage, ProxyResult, "poststation/host/to/bridge");

    // TODO: The topic! macro doesn't support borrowing
    // #[cfg(not(feature = "use-std"))]
    // topic!(Bridge2HostTopic, ProxyMessage<'a>, "poststation/bridge/to/host");

    #[cfg(feature = "use-std")]
    topic!(Bridge2HostTopic, ProxyMessage, "poststation/bridge/to/host");

    topic!(BridgeTableTopic, BridgeTable, "poststation/bridge/table");


    /// A device Unique ID, stored as a little-endian array of bytes
    pub type UniqueIdBytes = [u8; 8];

    /// A message proxied to or from a remote device
    #[cfg(not(feature = "use-std"))]
    #[derive(Debug, Serialize, Deserialize, Schema)]
    pub struct ProxyMessage<'a> {
        /// The unique id of the remote device
        pub serial: UniqueIdBytes,
        /// The postcard-rpc encoded payload of the message
        pub msg: &'a [u8],
    }

    /// A message proxied to or from a remote device
    #[cfg(feature = "use-std")]
    #[derive(Debug, Serialize, Deserialize, Schema)]
    pub struct ProxyMessage {
        /// The unique id of the remote device
        pub serial: UniqueIdBytes,
        /// The postcard-rpc encoded payload of the message
        pub msg: Vec<u8>,
    }

    /// The table of currently connected remote devices
    // TODO: This is not right, it's hardcoded to my esb bridge stuff
    #[cfg(not(feature = "use-std"))]
    #[derive(Debug, Serialize, Deserialize, Schema)]
    pub struct BridgeTable {
        pub sers: heapless::Vec<UniqueIdBytes, 7>,
    }

    /// The table of currently connected remote devices
    #[cfg(feature = "use-std")]
    #[derive(Debug, Serialize, Deserialize, Schema)]
    pub struct BridgeTable {
        pub sers: Vec<UniqueIdBytes>,
    }

    /// Error when proxying
    #[derive(Debug, Serialize, Deserialize, Schema)]
    pub enum ProxyError {
        UnknownDevice,
    }

    pub type ProxyResult = Result<(), ProxyError>;

}
