//! Rest API version of types
//!
//! This uses Schemars instead of postcard-schema, and avoid types like `u64` that
//! will make JSON/JS sad

use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Debug, PartialEq, Serialize, Deserialize, Hash, JsonSchema)]
pub struct DeviceData {
    pub serial: String,
    pub name: String,
    pub is_connected: bool,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}
