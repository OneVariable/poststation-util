//! Rest API version of types
//!
//! This uses Schemars instead of postcard-schema, and avoid types like `u64` that
//! will make JSON/JS sad

use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct DeviceData {
    pub serial: String,
    pub name: String,
    pub is_connected: bool,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct LogRequest {
    pub serial: String,
    pub count: u32,
}

// TODO: now that postcard-schema has a Schema impl for Uuid we might
// not actually need this anymore
#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct Uuidv7(pub [u8; 16]);

impl From<Uuid> for Uuidv7 {
    fn from(value: Uuid) -> Self {
        Self(value.into_bytes())
    }
}

impl From<Uuidv7> for Uuid {
    fn from(val: Uuidv7) -> Self {
        Uuid::from_bytes(val.0)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct Log {
    pub uuidv7: Uuidv7,
    pub msg: String,
}

impl Uuidv7 {
    pub fn id_to_time(&self) -> DateTime<Local> {
        let uuid = Uuid::from_bytes(self.0);
        let ts = uuid.get_timestamp().unwrap();
        let (a, b) = ts.to_unix();
        DateTime::from_timestamp(a as i64, b).unwrap().into()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicRequest {
    pub serial: u64,
    pub path: String,
    pub key: foreign::Key,
    pub count: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicMsg {
    pub uuidv7: Uuidv7,
    pub msg: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicStreamRequest {
    pub serial: u64,
    pub path: String,
    pub key: foreign::Key,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicStreamMsg {
    pub stream_id: Uuidv7,
    pub msg: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub enum TopicStreamResult {
    Started(Uuidv7),
    NoDeviceKnown,
    DeviceDisconnected,
    NoSuchTopic,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProxyRequest {
    pub serial: u64,
    pub path: String,
    pub req_key: foreign::Key,
    pub resp_key: foreign::Key,
    pub seq_no: u32,
    pub req_body: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ProxyResponse {
    Ok {
        resp_key: foreign::Key,
        seq_no: u32,
        body: Vec<u8>,
    },
    WireErr {
        resp_key: foreign::Key,
        seq_no: u32,
        body: foreign::WireError,
    },
    OtherErr(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PublishRequest {
    pub serial: u64,
    pub path: String,
    pub topic_key: foreign::Key,
    pub seq_no: u32,
    pub topic_body: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum PublishResponse {
    Sent,
    OtherErr(String),
}

/// These are types from other crates I'm pasting here just so I can impl JsonSchema on it
pub mod foreign {
    use std::collections::HashSet;

    use postcard_schema::schema::owned::OwnedNamedType;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};


    impl From<postcard_rpc::Key> for Key {
        fn from(value: postcard_rpc::Key) -> Self {
            Self(value.to_bytes())
        }
    }

    impl From<Key> for postcard_rpc::Key {
        fn from(value: Key) -> Self {
            unsafe {
                postcard_rpc::Key::from_bytes(value.0)
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, Hash, JsonSchema)]
    pub struct Key([u8; 8]);

    /// The given frame was too long
    #[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
    pub struct FrameTooLong {
        /// The length of the too-long frame
        pub len: u32,
        /// The maximum frame length supported
        pub max: u32,
    }

    /// The given frame was too short
    #[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
    pub struct FrameTooShort {
        /// The length of the too-short frame
        pub len: u32,
    }

    /// A protocol error that is handled outside of the normal request type, usually
    /// indicating a protocol-level error
    #[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
    pub enum WireError {
        /// The frame exceeded the buffering capabilities of the server
        FrameTooLong(FrameTooLong),
        /// The frame was shorter than the minimum frame size and was rejected
        FrameTooShort(FrameTooShort),
        /// Deserialization of a message failed
        DeserFailed,
        /// Serialization of a message failed, usually due to a lack of space to
        /// buffer the serialized form
        SerFailed,
        /// The key associated with this request was unknown
        UnknownKey,
        /// The server was unable to spawn the associated handler, typically due
        /// to an exhaustion of resources
        FailedToSpawn,
        /// The provided key is below the minimum key size calculated to avoid hash
        /// collisions, and was rejected to avoid potential misunderstanding
        KeyTooSmall,
    }


    /// A report describing the schema spoken by the connected device
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
    pub struct SchemaReport {
        /// All custom types spoken by the device (on any endpoint or topic),
        /// as well as all primitive types. In the future, primitive types may
        /// be removed.
        pub types: HashSet<OwnedNamedType>,
        /// All incoming (client to server) topics reported by the device
        pub topics_in: Vec<TopicReport>,
        /// All outgoing (server to client) topics reported by the device
        pub topics_out: Vec<TopicReport>,
        /// All endpoints reported by the device
        pub endpoints: Vec<EndpointReport>,
    }

    impl From<postcard_rpc::host_client::SchemaReport> for SchemaReport {
        fn from(value: postcard_rpc::host_client::SchemaReport) -> Self {
            Self {
                types: value.types,
                topics_in: value.topics_in.into_iter().map(Into::into).collect(),
                topics_out: value.topics_out.into_iter().map(Into::into).collect(),
                endpoints: value.endpoints.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<SchemaReport> for postcard_rpc::host_client::SchemaReport {
        fn from(value: SchemaReport) -> Self {
            Self {
                types: value.types,
                topics_in: value.topics_in.into_iter().map(Into::into).collect(),
                topics_out: value.topics_out.into_iter().map(Into::into).collect(),
                endpoints: value.endpoints.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<postcard_rpc::host_client::TopicReport> for TopicReport {
        fn from(value: postcard_rpc::host_client::TopicReport) -> Self {
            Self {
                path: value.path,
                key: value.key.into(),
                ty: value.ty,
            }
        }
    }

    impl From<TopicReport> for postcard_rpc::host_client::TopicReport {
        fn from(value: TopicReport) -> Self {
            Self {
                path: value.path,
                key: value.key.into(),
                ty: value.ty,
            }
        }
    }

    /// A description of a single Topic
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
    pub struct TopicReport {
        /// The human readable path of the topic
        pub path: String,
        /// The Key of the topic (which hashes the path and type)
        pub key: Key,
        /// The schema of the type of the message
        pub ty: OwnedNamedType,
    }

    impl From<postcard_rpc::host_client::EndpointReport> for EndpointReport {
        fn from(value: postcard_rpc::host_client::EndpointReport) -> Self {
            Self {
                path: value.path,
                req_key: value.req_key.into(),
                req_ty: value.req_ty,
                resp_key: value.resp_key.into(),
                resp_ty: value.resp_ty,
            }
        }
    }

    impl From<EndpointReport> for postcard_rpc::host_client::EndpointReport {
        fn from(value: EndpointReport) -> Self {
            Self {
                path: value.path,
                req_key: value.req_key.into(),
                req_ty: value.req_ty,
                resp_key: value.resp_key.into(),
                resp_ty: value.resp_ty,
            }
        }
    }

    /// A description of a single Endpoint
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
    pub struct EndpointReport {
        /// The human readable path of the endpoint
        pub path: String,
        /// The Key of the request (which hashes the path and type)
        pub req_key: Key,
        /// The schema of the request type
        pub req_ty: OwnedNamedType,
        /// The Key of the response (which hashes the path and type)
        pub resp_key: Key,
        /// The schema of the response type
        pub resp_ty: OwnedNamedType,
    }
}
