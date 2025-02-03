//! Interface definitions used for poststation simulators

/// The `simulator` module is used for `--simulator-devices` usage
pub mod simulator {
    use postcard_rpc::{endpoints, topics, TopicDirection};
    use postcard_schema::Schema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Copy, Clone)]
    pub struct Rgb8 {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    #[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Clone)]
    pub struct Temperature {
        pub temp: f64,
    }


    endpoints! {
        list = ENDPOINT_LIST;
        | EndpointTy            | RequestTy     | ResponseTy    | Path                          |
        | ----------            | ---------     | ----------    | ----                          |
        | GetUniqueIdEndpoint   | ()            | u64           | "poststation/unique_id/get"   |
        | RebootToPicoBoot      | ()            | ()            | "simulator/picoboot/reset"    |
        | SetStatusLed          | Rgb8          | ()            | "simulator/status_led/set"    |
        | GetStatusLed          | ()            | Rgb8          | "simulator/status_led/get"    |
    }

    topics! {
       list = TOPICS_IN_LIST;
       direction = TopicDirection::ToServer;
       | TopicTy        | MessageTy     | Path              |
       | -------        | ---------     | ----              |
    }

    topics! {
       list = TOPICS_OUT_LIST;
       direction = TopicDirection::ToClient;
       | TopicTy        | MessageTy     | Path                      |
       | -------        | ---------     | ----                      |
       | SomeNumber     | Temperature   | "simulator/temperature"   |
    }
}

/// The `interface_tester` module is used for `--interface-testers` usage
pub mod interface_tester {
    use std::collections::HashMap;
    use postcard_rpc::{endpoints, topics, TopicDirection};
    use postcard_schema::Schema;
    use serde::{Deserialize, Serialize};

    // ------
    pub type OptString = Option<String>;
    pub type OptNum = Option<f64>;
    pub type VecString = Vec<String>;
    pub type VecNum = Vec<f64>;
    pub type OneTup = (f64,);
    pub type TwoTup = (bool, String);
    pub type ThreeTup = (i8, u16, i32);
    pub type StringNumMap = HashMap<String, f64>;
    pub type StringStringMap = HashMap<String, String>;

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct MyUnitStruct;

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct MyNtStruct(pub f64);

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct OneTupStruct(f64);

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct TwoTupStruct(bool, String);

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct ThreeTupStruct(i8, u16, i32);

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct AlphaUnsigned {
        pub a: u8,
        pub b: u16,
        pub c: u32,
        pub d: u64,
        pub e: u128,
    }

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct AlphaSigned {
        pub a: i8,
        pub b: i16,
        pub c: i32,
        pub d: i64,
        pub e: i128,
    }

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct Beta {
        pub in_left: AlphaUnsigned,
        pub in_right: AlphaUnsigned,
    }

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub struct Delta {
        pub add_wrapping: AlphaUnsigned,
        pub add_saturating: AlphaUnsigned,
        pub any_add_outrange: bool,
        pub sub_wrapping: AlphaUnsigned,
        pub sub_saturating: AlphaUnsigned,
        pub any_sub_outrange: bool,
    }

    #[derive(Serialize, Deserialize, Debug, Schema)]
    pub enum ExampleEnum {
        UnitVariant,
        NewtypeVariant(f32),
        TupleVariant(u8, bool, String),
        StructVariant { alpha: u8, beta: bool },
    }

    endpoints! {
        list = ENDPOINT_LIST;
        | EndpointTy                | RequestTy         | ResponseTy        | Path                              |
        | ----------                | ---------         | ----------        | ----                              |
        | GetUniqueIdEndpoint       | ()                | u64               | "poststation/unique_id/get"       |
        | RebootToPicoBoot          | ()                | ()                | "simulator/picoboot/reset"        |
        | InvertEndpoint            | bool              | bool              | "simulator/invert"                |
        | U8toI8Endpoint            | u8                | i8                | "simulator/convert/u8i8"          |
        | U16toI16Endpoint          | u16               | i16               | "simulator/convert/u16i16"        |
        | U32toI32Endpoint          | u32               | i32               | "simulator/convert/u32i32"        |
        | U64toI64Endpoint          | u64               | i64               | "simulator/convert/u64i64"        |
        | U128toI128Endpoint        | u128              | i128              | "simulator/convert/u128i128"      |
        | I8toU8Endpoint            | i8                | u8                | "simulator/convert/i8u8"          |
        | I16toU16Endpoint          | i16               | u16               | "simulator/convert/i16u16"        |
        | I32toU32Endpoint          | i32               | u32               | "simulator/convert/i32u32"        |
        | I64toU64Endpoint          | i64               | u64               | "simulator/convert/i64u64"        |
        | I128toU128Endpoint        | i128              | u128              | "simulator/convert/i128u128"      |
        | SineF32Endpoint           | f32               | f32               | "simulator/sine/f32"              |
        | CosF64Endpoint            | f64               | f64               | "simulator/sine/f64"              |
        | StringToLowerEndpoint     | String            | String            | "simulator/convert/lowercase"     |
        | OptionStrNumEndpoint      | OptString         | OptNum            | "simulator/convert/optstrnum"     |
        | UnitEndpoint              | ()                | ()                | "simulator/echo/unit"             |
        | UnitStructEndpoint        | MyUnitStruct      | MyUnitStruct      | "simulator/echo/unitstruct"       |
        | NTStructSineEndpoint      | MyNtStruct        | MyNtStruct        | "simulator/sine/nt"               |
        | SeqStrNumEndpoint         | VecString         | VecNum            | "simulator/convert/seqstrnum"     |
        | OneTupEchoEndpoint        | OneTup            | OneTup            | "simulator/echo/onetup"           |
        | TwoTupEchoEndpoint        | TwoTup            | TwoTup            | "simulator/echo/twotup"           |
        | ThreeTupEchoEndpoint      | ThreeTup          | ThreeTup          | "simulator/echo/threetup"         |
        | OneTupSEchoEndpoint       | OneTupStruct      | OneTupStruct      | "simulator/echo/onetupstruct"     |
        | TwoTupSEchoEndpoint       | TwoTupStruct      | TwoTupStruct      | "simulator/echo/twotupstruct"     |
        | ThreeTupSEchoEndpoint     | ThreeTupStruct    | ThreeTupStruct    | "simulator/echo/threetupstruct"   |
        | MapStrNumEndpoint         | StringNumMap      | StringStringMap   | "simulator/convert/mapstrnum"     |
        | StructToSignedEndpoint    | AlphaUnsigned     | AlphaSigned       | "simulator/convert/structutoi"    |
        | StructMathEndpoint        | Beta              | Delta             | "simulator/math/struct"           |
        | EnumEchoEndpoint          | ExampleEnum       | ExampleEnum       | "simulator/echo/enum"             |
    }

    topics! {
       list = TOPICS_IN_LIST;
       direction = TopicDirection::ToServer;
       | TopicTy        | MessageTy     | Path              |
       | -------        | ---------     | ----              |
    }

    topics! {
       list = TOPICS_OUT_LIST;
       direction = TopicDirection::ToClient;
       | TopicTy        | MessageTy     | Path                      |
       | -------        | ---------     | ----                      |
    }
}
