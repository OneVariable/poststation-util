//! Rest API version of types
//!
//! This uses Schemars instead of postcard-schema, and avoid types like `u64` that
//! will make JSON/JS sad.
//!
//! At some point in the future we will publish an OpenAPI spec for all available requests.
//! For now, here is a listing of all endpoints and an example CURL request for each of them.
//!
//! ## "Get Devices"
//!
//! ```sh
//! curl http://localhost:4444/api/devices -q -H "Accept: application/json"
//! ```
//!
//! ```json
//! [
//!   {
//!     "serial": "3836363937050630",
//!     "name": "XRAY-013",
//!     "is_connected": false,
//!     "manufacturer": "OneVariable",
//!     "product": "poststation-pico"
//!   },
//!   {
//!     "serial": "6E43B25479AC185C",
//!     "name": "YACHTY-312",
//!     "is_connected": true,
//!     "manufacturer": "Simulator",
//!     "product": "Product"
//!   },
//! ]
//! ```
//!
//! ## "Get Schemas"
//!
//! ```sh
//!  curl http://localhost:4444/api/devices/CA9FF06E058FF9A6/schemas -q -H "Accept: application/json"
//! ```
//!
//! Output: <https://gist.github.com/jamesmunns/0a533d8ed8ffbbc34c282da848a162fd>
//!
//! ## "Get Logs"
//!
//! ```sh
//! curl 'http://localhost:4444/api/devices/3836363937050630/logs?serial=3836363937050630&count=2' \
//! -q -H "Accept: application/json"
//! ```
//!
//! ```json
//! [
//!   {
//!     "uuidv7": "01936033-6231-71f2-9200-49361527b270",
//!     "msg": "Uptime: Duration { ticks: 1347000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936033-6de8-7db0-8a7c-00563efac872",
//!     "msg": "Uptime: Duration { ticks: 1350000000 } freq: 125000000"
//!   }
//! ]
//! ```
//!
//! ## "Get Range of logs"
//!
//! This API is used as a paginated version of "Get Logs". You can use either a UTC millisecond timestamp
//! or the UUIDv7 of a log item as the "anchor" of the request, and then request N logs "Before" or "After" the
//! anchor (excluding the anchor itself).
//!
//! ### Using a UUIDv7 of a log entry as the anchor
//!
//! ```sh
//! curl 'http://localhost:4444/api/devices/3836363937050630/logs/range?serial=3836363937050630&count=4&uuid=01936032-e149-7e92-b4ca-f7e8a30e11cb&direction=After' \
//!   -q -H "Accept: application/json" | jq
//! ```
//!
//! ```json
//! [
//!   {
//!     "uuidv7": "01936033-1029-7e32-8b45-dc4595c98ee8",
//!     "msg": "Uptime: Duration { ticks: 1326000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936033-0471-7912-9eaa-f3db32a47387",
//!     "msg": "Uptime: Duration { ticks: 1323000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936032-f8b9-78e1-929e-99051b2bba64",
//!     "msg": "Uptime: Duration { ticks: 1320000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936032-ed01-7ca0-99ad-f9ccac6c7e22",
//!     "msg": "Uptime: Duration { ticks: 1317000000 } freq: 125000000"
//!   }
//! ]
//! ```
//!
//! ### Using a unix millisecond timestamp as the anchor
//!
//! ```sh
//! curl 'http://localhost:4444/api/devices/3836363937050630/logs/range?serial=3836363937050630&count=4&unix_ms_ts=1732485767497&direction=After' \
//! -q -H "Accept: application/json" | jq
//!
//! ```json
//! [
//!   {
//!     "uuidv7": "01936033-0471-7912-9eaa-f3db32a47387",
//!     "msg": "Uptime: Duration { ticks: 1323000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936032-f8b9-78e1-929e-99051b2bba64",
//!     "msg": "Uptime: Duration { ticks: 1320000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936032-ed01-7ca0-99ad-f9ccac6c7e22",
//!     "msg": "Uptime: Duration { ticks: 1317000000 } freq: 125000000"
//!   },
//!   {
//!     "uuidv7": "01936032-e149-7e92-b4ca-f7e8a30e11cb",
//!     "msg": "Uptime: Duration { ticks: 1314000000 } freq: 125000000"
//!   }
//! ]
//! ```
//!
//! ## "Get Topic Messages"
//!
//! ```sh
//! curl 'http://localhost:4444/api/devices/CA9FF06E058FF9A6/topics?path=simulator/temperature&key=583A352440D70716&count=3' \
//!     -H "Accept: application/json"
//! ```
//!
//! ```json
//! [
//!   {
//!     "uuidv7": "01938dff-2bad-7ae1-9e3f-0ce6e2805ec0",
//!     "msg": {
//!       "temp": 3207.5660993151787
//!     }
//!   },
//!   {
//!     "uuidv7": "01938dff-2da1-7301-9654-7e3d338cf1eb",
//!     "msg": {
//!       "temp": 3210.7076919687684
//!     }
//!   },
//!   {
//!     "uuidv7": "01938dff-2f95-7b22-b71b-16dcd0c34f5a",
//!     "msg": {
//!       "temp": 3213.8492846223585
//!     }
//!   }
//! ]
//! ```
//!
//! ## "Proxy an endpoint request"
//!
//! ```sh
//! curl \
//!     -X POST \
//!     -H 'Content-Type: application/json' \
//!     -H "Accept: application/json" \
//!     'http://localhost:4444/api/devices/CA9FF06E058FF9A6/proxy' \
//!     -d '{
//!         "path": "postcard-rpc/ping",
//!         "req_key": "E8EDEF24F26C7C91",
//!         "resp_key": "E8EDEF24F26C7C91",
//!         "seq_no": 0,
//!         "body": 123
//!     }'
//! ```
//!
//! ```json
//! {
//!   "resp_key": "E8EDEF24F26C7C91",
//!   "seq_no": 0,
//!   "body": 123
//! }
//! ```
//!
//! ## "Proxy a topic publish"
//!
//! ```sh
//! curl \
//!     -X POST \
//!     -H 'Content-Type: application/json' \
//!     -H "Accept: application/json" \
//!     'http://localhost:4444/api/devices/CA9FF06E058FF9A6/publish' \
//!     -d '{
//!         "path": "some/topic/into/server",
//!         "topic_key": "E8EDEF24F26C7C91",
//!         "seq_no": 0,
//!         "body": { "some": "payload" }
//!     }'
//! ```
//!
//! ```json
//! {}
//! ```
//!
//! # "Subscribe to a stream of topic_out messages"
//!
//! This is a **WebSocket** endpoint, which gives you a live feed of a specific topic from a
//! specific device.
//!
//! ```sh
//! websocat "ws://localhost:4444/api/devices/CA9FF06E058FF9A6/listen?path=simulator/temperature&key=583A352440D70716" | jq
//! ```
//!
//! ```json
//! {
//!   "msg": {
//!     "temp": 2726.9024233159403
//!   },
//!   "seq_no": 868
//! }
//! {
//!   "msg": {
//!     "temp": 2730.0440159695304
//!   },
//!   "seq_no": 869
//! }
//! {
//!   "msg": {
//!     "temp": 2733.18560862312
//!   },
//!   "seq_no": 870
//! }
//! ```

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
    pub count: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct Log {
    pub uuidv7: Uuid,
    pub msg: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct LogRangeRequest {
    pub uuid: Option<Uuid>,
    pub unix_ms_ts: Option<u64>,
    pub direction: Direction,
    pub count: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub enum Direction {
    Before,
    After,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicRequest {
    pub path: String,
    pub key: foreign::Key,
    pub count: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicMsg {
    pub uuidv7: Uuid,
    pub msg: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicStreamRequest {
    pub path: String,
    pub key: foreign::Key,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct TopicStreamMsg {
    pub stream_id: Uuid,
    pub msg: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub enum TopicStreamResult {
    Started(Uuid),
    NoDeviceKnown,
    DeviceDisconnected,
    NoSuchTopic,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProxyRequest {
    pub path: String,
    pub req_key: foreign::Key,
    pub resp_key: foreign::Key,
    pub seq_no: u32,
    pub body: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProxyResponseOk {
    pub resp_key: foreign::Key,
    pub seq_no: u32,
    pub body: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ProxyResponseError {
    WireErr {
        resp_key: foreign::Key,
        seq_no: u32,
        body: foreign::WireError,
    },
    OtherErr(String),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PublishRequest {
    pub path: String,
    pub topic_key: foreign::Key,
    pub seq_no: u32,
    pub body: serde_json::Value,
}

/// These are types from other crates I'm pasting here just so I can impl JsonSchema on it
pub mod foreign {
    use std::collections::HashSet;

    use schema::OwnedNamedType;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    impl From<postcard_rpc::Key> for Key {
        fn from(value: postcard_rpc::Key) -> Self {
            Self(format!("{:016X}", u64::from_le_bytes(value.to_bytes())))
        }
    }

    impl TryFrom<Key> for postcard_rpc::Key {
        type Error = String;
        fn try_from(value: Key) -> Result<Self, Self::Error> {
            let Ok(val) = u64::from_str_radix(&value.0, 16) else {
                return Err(value.0);
            };
            unsafe { Ok(postcard_rpc::Key::from_bytes(val.to_le_bytes())) }
        }
    }

    #[derive(
        Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Hash, JsonSchema,
    )]
    pub struct Key(String);

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
                types: value.types.iter().map(Into::into).collect(),
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
                ty: (&value.ty).into(),
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
                req_ty: (&value.req_ty).into(),
                resp_key: value.resp_key.into(),
                resp_ty: (&value.resp_ty).into(),
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

    pub mod schema {
        //! Owned + JSON friendly Schema version

        use postcard_schema::schema::owned as real;
        use schemars::JsonSchema;
        use serde::{Deserialize, Serialize};
        use std::{boxed::Box, ops::Deref, string::String, vec::Vec};

        // ---

        /// The owned version of [`NamedType`]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        pub struct OwnedNamedType {
            /// The name of this type
            pub name: String,
            /// The type
            pub ty: OwnedDataModelType,
        }

        impl From<&real::OwnedNamedType> for OwnedNamedType {
            fn from(value: &real::OwnedNamedType) -> Self {
                Self {
                    name: value.name.to_string(),
                    ty: (&value.ty).into(),
                }
            }
        }

        // ---

        /// The owned version of [`DataModelType`]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum OwnedDataModelType {
            /// The `bool` Serde Data Model Type
            Bool,

            /// The `i8` Serde Data Model Type
            I8,

            /// The `u8` Serde Data Model Type
            U8,

            /// A variably encoded i16
            I16,

            /// A variably encoded i32
            I32,

            /// A variably encoded i64
            I64,

            /// A variably encoded i128
            I128,

            /// A variably encoded u16
            U16,

            /// A variably encoded u32
            U32,

            /// A variably encoded u64
            U64,

            /// A variably encoded u128
            U128,

            /// A variably encoded usize
            Usize,

            /// A variably encoded isize
            Isize,

            /// The `f32` Serde Data Model Type
            F32,

            /// The `f64 Serde Data Model Type
            F64,

            /// The `char` Serde Data Model Type
            Char,

            /// The `String` Serde Data Model Type
            String,

            /// The `&[u8]` Serde Data Model Type
            ByteArray,

            /// The `Option<T>` Serde Data Model Type
            Option(Box<OwnedNamedType>),

            /// The `()` Serde Data Model Type
            Unit,

            /// The "unit struct" Serde Data Model Type
            UnitStruct,

            /// The "newtype struct" Serde Data Model Type
            NewtypeStruct(Box<OwnedNamedType>),

            /// The "Sequence" Serde Data Model Type
            Seq(Box<OwnedNamedType>),

            /// The "Tuple" Serde Data Model Type
            Tuple(Vec<OwnedNamedType>),

            /// The "Tuple Struct" Serde Data Model Type
            TupleStruct(Vec<OwnedNamedType>),

            /// The "Map" Serde Data Model Type
            Map {
                /// The map "Key" type
                key: Box<OwnedNamedType>,
                /// The map "Value" type
                val: Box<OwnedNamedType>,
            },

            /// The "Struct" Serde Data Model Type
            Struct(Vec<OwnedNamedValue>),

            /// The "Enum" Serde Data Model Type (which contains any of the "Variant" types)
            Enum(Vec<OwnedNamedVariant>),

            /// A NamedType/OwnedNamedType
            Schema,
        }

        impl From<&real::OwnedDataModelType> for OwnedDataModelType {
            fn from(other: &real::OwnedDataModelType) -> Self {
                match other {
                    real::OwnedDataModelType::Bool => Self::Bool,
                    real::OwnedDataModelType::I8 => Self::I8,
                    real::OwnedDataModelType::U8 => Self::U8,
                    real::OwnedDataModelType::I16 => Self::I16,
                    real::OwnedDataModelType::I32 => Self::I32,
                    real::OwnedDataModelType::I64 => Self::I64,
                    real::OwnedDataModelType::I128 => Self::I128,
                    real::OwnedDataModelType::U16 => Self::U16,
                    real::OwnedDataModelType::U32 => Self::U32,
                    real::OwnedDataModelType::U64 => Self::U64,
                    real::OwnedDataModelType::U128 => Self::U128,
                    real::OwnedDataModelType::Usize => Self::Usize,
                    real::OwnedDataModelType::Isize => Self::Isize,
                    real::OwnedDataModelType::F32 => Self::F32,
                    real::OwnedDataModelType::F64 => Self::F64,
                    real::OwnedDataModelType::Char => Self::Char,
                    real::OwnedDataModelType::String => Self::String,
                    real::OwnedDataModelType::ByteArray => Self::ByteArray,
                    real::OwnedDataModelType::Option(o) => Self::Option(Box::new(o.deref().into())),
                    real::OwnedDataModelType::Unit => Self::Unit,
                    real::OwnedDataModelType::UnitStruct => Self::UnitStruct,
                    real::OwnedDataModelType::NewtypeStruct(nts) => {
                        Self::NewtypeStruct(Box::new(nts.deref().into()))
                    }
                    real::OwnedDataModelType::Seq(s) => Self::Seq(Box::new(s.deref().into())),
                    real::OwnedDataModelType::Tuple(t) => {
                        Self::Tuple(t.iter().map(|i| i.into()).collect())
                    }
                    real::OwnedDataModelType::TupleStruct(ts) => {
                        Self::TupleStruct(ts.iter().map(|i| i.into()).collect())
                    }
                    real::OwnedDataModelType::Map { key, val } => Self::Map {
                        key: Box::new(key.deref().into()),
                        val: Box::new(val.deref().into()),
                    },
                    real::OwnedDataModelType::Struct(s) => {
                        Self::Struct(s.iter().map(|i| i.into()).collect())
                    }
                    real::OwnedDataModelType::Enum(e) => {
                        Self::Enum(e.iter().map(|i| i.into()).collect())
                    }
                    real::OwnedDataModelType::Schema => Self::Schema,
                }
            }
        }

        // ---

        /// The owned version of [`DataModelVariant`]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        pub enum OwnedDataModelVariant {
            /// The "unit variant" Serde Data Model Type
            UnitVariant,
            /// The "newtype variant" Serde Data Model Type
            NewtypeVariant(Box<OwnedNamedType>),
            /// The "Tuple Variant" Serde Data Model Type
            TupleVariant(Vec<OwnedNamedType>),
            /// The "Struct Variant" Serde Data Model Type
            StructVariant(Vec<OwnedNamedValue>),
        }

        impl From<&real::OwnedDataModelVariant> for OwnedDataModelVariant {
            fn from(value: &real::OwnedDataModelVariant) -> Self {
                match value {
                    real::OwnedDataModelVariant::UnitVariant => Self::UnitVariant,
                    real::OwnedDataModelVariant::NewtypeVariant(d) => {
                        Self::NewtypeVariant(Box::new(d.deref().into()))
                    }
                    real::OwnedDataModelVariant::TupleVariant(d) => {
                        Self::TupleVariant(d.iter().map(|i| i.into()).collect())
                    }
                    real::OwnedDataModelVariant::StructVariant(d) => {
                        Self::StructVariant(d.iter().map(|i| i.into()).collect())
                    }
                }
            }
        }

        // ---

        /// The owned version of [`NamedValue`]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        pub struct OwnedNamedValue {
            /// The name of this value
            pub name: String,
            /// The type of this value
            pub ty: OwnedNamedType,
        }

        impl From<&real::OwnedNamedValue> for OwnedNamedValue {
            fn from(value: &real::OwnedNamedValue) -> Self {
                Self {
                    name: value.name.to_string(),
                    ty: (&value.ty).into(),
                }
            }
        }

        // ---

        /// The owned version of [`NamedVariant`]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
        pub struct OwnedNamedVariant {
            /// The name of this variant
            pub name: String,
            /// The type of this variant
            pub ty: OwnedDataModelVariant,
        }

        impl From<&real::OwnedNamedVariant> for OwnedNamedVariant {
            fn from(value: &real::OwnedNamedVariant) -> Self {
                Self {
                    name: value.name.to_string(),
                    ty: (&value.ty).into(),
                }
            }
        }
    }
}
