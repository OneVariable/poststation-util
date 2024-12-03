//! Rest API version of types
//!
//! This uses Schemars instead of postcard-schema, and avoid types like `u64` that
//! will make JSON/JS sad

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
    pub count: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct Log {
    pub uuidv7: Uuid,
    pub msg: String,
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
pub struct ProxyResponseOk{
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
            unsafe {
                Ok(postcard_rpc::Key::from_bytes(val.to_le_bytes()))
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize, Hash, JsonSchema)]
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

        use schemars::JsonSchema;
        use serde::{Deserialize, Serialize};
        use std::{boxed::Box, ops::Deref, string::String, vec::Vec};
        use postcard_schema::schema::owned as real;

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
                    real::OwnedDataModelType::NewtypeStruct(nts) => Self::NewtypeStruct(Box::new(nts.deref().into())),
                    real::OwnedDataModelType::Seq(s) => Self::Seq(Box::new(s.deref().into())),
                    real::OwnedDataModelType::Tuple(t) => Self::Tuple(t.iter().map(|i| i.into()).collect()),
                    real::OwnedDataModelType::TupleStruct(ts) => {
                        Self::TupleStruct(ts.iter().map(|i| i.into()).collect())
                    }
                    real::OwnedDataModelType::Map { key, val } => Self::Map {
                        key: Box::new(key.deref().into()),
                        val: Box::new(val.deref().into()),
                    },
                    real::OwnedDataModelType::Struct(s) => Self::Struct(s.iter().map(|i| i.into()).collect()),
                    real::OwnedDataModelType::Enum(e) => Self::Enum(e.iter().map(|i| i.into()).collect()),
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
                    real::OwnedDataModelVariant::NewtypeVariant(d) => Self::NewtypeVariant(Box::new(d.deref().into())),
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
