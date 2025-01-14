use core::fmt::Debug;
use std::{
    error::Error, fmt::Display, future::Future, marker::PhantomData, net::{IpAddr, Ipv4Addr, SocketAddr}, path::{Path, PathBuf}, sync::Arc
};

use directories::ProjectDirs;
use postcard_dyn::Value;
use postcard_rpc::{
    host_client::{
        HostClient, MultiSubRxError, MultiSubscription, SchemaReport, TopicReport, WireRx,
        WireSpawn, WireTx,
    },
    standard_icd::{PingEndpoint, WireError, ERROR_PATH},
    Endpoint, Topic,
};
use poststation_api_icd::postsock::{
    Anchor, DeviceData, Direction, GetDevicesEndpoint, GetLogsEndpoint, GetLogsRangeEndpoint,
    GetSchemasEndpoint, GetTopicsEndpoint, Log, LogRangeRequest, LogRequest, ProxyEndpoint,
    ProxyRequest, ProxyResponse, PublishEndpoint, PublishRequest, PublishResponse,
    StartStreamEndpoint, SubscribeTopic, TopicMsg, TopicRequest, TopicStreamMsg,
    TopicStreamRequest, TopicStreamResult, Uuidv7,
};
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, ServerName},
    RootCertStore,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

pub use postcard_schema as schema;
pub use poststation_api_icd as icd;
use tokio_rustls::TlsConnector;

// ---

#[derive(Clone)]
pub struct PoststationClient {
    client: HostClient<WireError>,
}

impl PoststationClient {
    pub fn raw_client(&self) -> &HostClient<WireError> {
        &self.client
    }

    pub async fn get_devices(&self) -> Result<Vec<DeviceData>, ()> {
        self.client
            .send_resp::<GetDevicesEndpoint>(&())
            .await
            .map_err(drop)
    }

    pub async fn get_device_schemas(&self, serial: u64) -> Result<Option<SchemaReport>, ()> {
        let res = self
            .client
            .send_resp::<GetSchemasEndpoint>(&serial)
            .await
            .map_err(drop)?;
        let Some(res) = res else {
            return Ok(None);
        };
        Ok(Some(res))
    }

    pub async fn get_device_logs(&self, serial: u64, count: u32) -> Result<Option<Vec<Log>>, ()> {
        self.client
            .send_resp::<GetLogsEndpoint>(&LogRequest { serial, count })
            .await
            .map_err(drop)
    }

    pub async fn get_device_logs_range(
        &self,
        serial: u64,
        count: u32,
        dir: Direction,
        anchor: Anchor,
    ) -> Result<Option<Vec<Log>>, ()> {
        self.client
            .send_resp::<GetLogsRangeEndpoint>(&LogRangeRequest {
                serial,
                count,
                anchor,
                direction: dir,
            })
            .await
            .map_err(drop)
    }

    pub async fn get_device_topics_out_by_path_raw(
        &self,
        serial: u64,
        path: &str,
        count: u32,
    ) -> Result<Option<Vec<TopicMsg>>, ()> {
        let schemas = self.get_device_schemas(serial).await?;
        let Some(schemas) = schemas else {
            return Ok(None);
        };

        // find key
        let res = schemas
            .topics_out
            .iter()
            .find(|t| t.path.as_str() == path)
            .map(|t| t.key);
        let Some(key) = res else { return Ok(None) };

        self.client
            .send_resp::<GetTopicsEndpoint>(&TopicRequest {
                serial,
                count,
                path: path.to_string(),
                key,
            })
            .await
            .map_err(drop)
    }

    pub async fn get_device_topics_out_by_path_json(
        &self,
        serial: u64,
        path: &str,
        count: u32,
    ) -> Result<Option<Vec<(Uuidv7, Value)>>, ()> {
        let schemas = self.get_device_schemas(serial).await?;
        let Some(schemas) = schemas else {
            return Ok(None);
        };

        // find key
        let res = schemas.topics_out.iter().find(|t| t.path.as_str() == path);
        let Some(schema) = res else { return Ok(None) };

        let raws = self
            .client
            .send_resp::<GetTopicsEndpoint>(&TopicRequest {
                serial,
                count,
                path: path.to_string(),
                key: schema.key,
            })
            .await
            .map_err(drop)?;
        let Some(raws) = raws else {
            return Ok(None);
        };

        let res = raws
            .into_iter()
            .map(|tm| {
                let msg = postcard_dyn::from_slice_dyn(&schema.ty, &tm.msg).map_err(drop)?;
                Ok((tm.uuidv7, msg))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(res))
    }

    pub async fn proxy_endpoint<E>(
        &self,
        serial: u64,
        seq_no: u32,
        body: &E::Request,
    ) -> Result<E::Response, String>
    where
        E: Endpoint,
        E::Request: Serialize,
        E::Response: DeserializeOwned,
    {
        let Ok(Some(schemas)) = self.get_device_schemas(serial).await else {
            return Err("endpoint not found".into());
        };

        // find key
        // TODO: Don't compare the types because the names don't match even though we've
        // type-punned
        let res = schemas.endpoints.iter().find(|e| {
            e.path.as_str() == E::PATH && e.req_key == E::REQ_KEY && e.resp_key == E::RESP_KEY
        });
        let Some(schema) = res else {
            return Err("endpoint not found".into());
        };
        let Ok(body) = postcard::to_stdvec(body) else {
            return Err("Serialization Failed".into());
        };

        let req = ProxyRequest {
            serial,
            path: schema.path.clone(),
            req_key: schema.req_key,
            resp_key: schema.resp_key,
            seq_no,
            req_body: body,
        };

        let resp = self.client.send_resp::<ProxyEndpoint>(&req).await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Server error: {e:?}"));
            }
        };

        let resp = match resp {
            ProxyResponse::Ok { body, .. } => body,
            ProxyResponse::WireErr { body, .. } => return Err(format!("WireErr: {body:?}")),
            ProxyResponse::OtherErr(e) => return Err(format!("Other Server Err: '{e}'")),
        };

        let resp = postcard::from_bytes::<E::Response>(&resp);

        match resp {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Decode error: '{e:?}'")),
        }
    }

    pub async fn proxy_endpoint_json(
        &self,
        serial: u64,
        path: &str,
        seq_no: u32,
        body: Value,
    ) -> Result<Value, String> {
        let Ok(Some(schemas)) = self.get_device_schemas(serial).await else {
            return Err("endpoint not found".into());
        };

        // find key
        let res = schemas.endpoints.iter().find(|e| e.path.as_str() == path);
        let Some(schema) = res else {
            return Err("endpoint not found".into());
        };

        let Ok(body) = postcard_dyn::to_stdvec_dyn(&schema.req_ty, &body) else {
            return Err(
                "provided JSON does not match the expected schema for this endpoint".into(),
            );
        };
        let req = ProxyRequest {
            serial,
            path: schema.path.clone(),
            req_key: schema.req_key,
            resp_key: schema.resp_key,
            seq_no,
            req_body: body,
        };

        let resp = self.client.send_resp::<ProxyEndpoint>(&req).await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Server error: {e:?}"));
            }
        };

        let resp = match resp {
            ProxyResponse::Ok { body, .. } => body,
            ProxyResponse::WireErr { body, .. } => return Err(format!("WireErr: {body:?}")),
            ProxyResponse::OtherErr(e) => return Err(format!("Other Server Err: '{e}'")),
        };

        let resp = postcard_dyn::from_slice_dyn(&schema.resp_ty, &resp);

        match resp {
            Ok(v) => Ok(v),
            Err(e) => Err(format!("Decode error: '{e:?}'")),
        }
    }

    pub async fn publish_topic_json(
        &self,
        serial: u64,
        path: &str,
        seq_no: u32,
        body: Value,
    ) -> Result<(), String> {
        let Ok(Some(schemas)) = self.get_device_schemas(serial).await else {
            return Err("topic not found".into());
        };

        // find key
        let res = schemas.topics_in.iter().find(|e| e.path.as_str() == path);
        let Some(schema) = res else {
            return Err("topic not found".into());
        };

        let Ok(body) = postcard_dyn::to_stdvec_dyn(&schema.ty, &body) else {
            return Err("provided JSON does not match the schema for this topic".into());
        };
        let req = PublishRequest {
            serial,
            path: schema.path.clone(),
            topic_key: schema.key,
            seq_no,
            topic_body: body,
        };

        let resp = self.client.send_resp::<PublishEndpoint>(&req).await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Server error: {e:?}"));
            }
        };

        match resp {
            PublishResponse::Sent => Ok(()),
            PublishResponse::OtherErr(e) => Err(format!("Other Server Err: '{e}'")),
        }
    }

    pub async fn publish_topic<T>(
        &self,
        serial: u64,
        seq_no: u32,
        body: &T::Message,
    ) -> Result<(), String>
    where
        T: Topic,
        T::Message: Serialize,
    {
        let Ok(Some(schemas)) = self.get_device_schemas(serial).await else {
            return Err("topic not found".into());
        };

        // find key
        // TODO: Don't compare the types because the names don't match even though we've
        // type-punned
        let res = schemas
            .topics_in
            .iter()
            .find(|t| t.path.as_str() == T::PATH && t.key == T::TOPIC_KEY);
        let Some(schema) = res else {
            return Err("topic not found".into());
        };

        let Ok(body) = postcard::to_stdvec(body) else {
            return Err("serialization failed".into());
        };
        let req = PublishRequest {
            serial,
            path: schema.path.clone(),
            topic_key: schema.key,
            seq_no,
            topic_body: body,
        };

        let resp = self.client.send_resp::<PublishEndpoint>(&req).await;

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                return Err(format!("Server error: {e:?}"));
            }
        };

        match resp {
            PublishResponse::Sent => Ok(()),
            PublishResponse::OtherErr(e) => Err(format!("Other Server Err: '{e}'")),
        }
    }

    /// Listen to a given topic path, receiving a subscription that yields live messages
    pub async fn stream_topic_json(
        &self,
        serial: u64,
        path: &str,
    ) -> Result<JsonStreamListener, String> {
        let Ok(Some(schemas)) = self.get_device_schemas(serial).await else {
            return Err("topic not found".into());
        };

        // find key
        let res = schemas
            .topics_out
            .iter()
            .find(|e| e.path.as_str() == path)
            .cloned();
        let Some(schema) = res else {
            return Err("topic not found".into());
        };

        let sub = self
            .client
            .subscribe_multi::<SubscribeTopic>(64)
            .await
            .map_err(|e| format!("Error: {e:?}"))?;

        let res = self
            .client
            .send_resp::<StartStreamEndpoint>(&TopicStreamRequest {
                serial,
                path: path.to_string(),
                key: schema.key,
            })
            .await;

        let stream_id = match res {
            Ok(TopicStreamResult::Started(id)) => id,
            Ok(TopicStreamResult::DeviceDisconnected) => return Err("Device Disconnected".into()),
            Ok(TopicStreamResult::NoDeviceKnown) => return Err("No Device Known".into()),
            Ok(TopicStreamResult::NoSuchTopic) => return Err("No Such Topic".into()),
            Err(e) => return Err(format!("Error: {e:?}")),
        };

        Ok(JsonStreamListener {
            schema,
            sub,
            stream_id,
        })
    }

    /// Listen to a given topic path, receiving a subscription that yields live messages
    pub async fn stream_topic<T>(&self, serial: u64) -> Result<StreamListener<T>, String>
    where
        T: Topic,
        T::Message: DeserializeOwned,
    {
        let Ok(Some(schemas)) = self.get_device_schemas(serial).await else {
            return Err("topic not found".into());
        };

        // find key
        let res = schemas
            .topics_out
            .iter()
            .find(|e| e.path.as_str() == T::PATH && e.key == T::TOPIC_KEY)
            .cloned();
        let Some(schema) = res else {
            return Err("topic not found".into());
        };

        let sub = self
            .client
            .subscribe_multi::<SubscribeTopic>(64)
            .await
            .map_err(|e| format!("Error: {e:?}"))?;

        let res = self
            .client
            .send_resp::<StartStreamEndpoint>(&TopicStreamRequest {
                serial,
                path: T::PATH.to_string(),
                key: schema.key,
            })
            .await;

        let stream_id = match res {
            Ok(TopicStreamResult::Started(id)) => id,
            Ok(TopicStreamResult::DeviceDisconnected) => return Err("Device Disconnected".into()),
            Ok(TopicStreamResult::NoDeviceKnown) => return Err("No Device Known".into()),
            Ok(TopicStreamResult::NoSuchTopic) => return Err("No Such Topic".into()),
            Err(e) => return Err(format!("Error: {e:?}")),
        };

        Ok(StreamListener {
            sub,
            stream_id,
            _pd: PhantomData,
        })
    }
}

pub struct JsonStreamListener {
    stream_id: Uuidv7,
    schema: TopicReport,
    sub: MultiSubscription<TopicStreamMsg>,
}

impl JsonStreamListener {
    /// Receive a single message from this subscription
    ///
    /// Returns None if the connection has been closed
    pub async fn recv(&mut self) -> Option<Value> {
        loop {
            let msg = match self.sub.recv().await {
                Ok(m) => m,
                Err(MultiSubRxError::IoClosed) => return None,
                Err(MultiSubRxError::Lagged(n)) => {
                    tracing::warn!(stream_id = ?self.stream_id, lags = n, "Stream lagged");
                    continue;
                }
            };

            let TopicStreamMsg { stream_id, msg } = msg;
            if stream_id != self.stream_id {
                continue;
            }

            let Ok(msg) = postcard_dyn::from_slice_dyn(&self.schema.ty, &msg) else {
                continue;
            };
            return Some(msg);
        }
    }
}

pub struct StreamListener<T>
where
    T: Topic,
    T::Message: DeserializeOwned,
{
    stream_id: Uuidv7,
    sub: MultiSubscription<TopicStreamMsg>,
    _pd: PhantomData<fn() -> T>,
}

impl<T> StreamListener<T>
where
    T: Topic,
    T::Message: DeserializeOwned,
{
    /// Receive a single message from this subscription
    ///
    /// Returns None if the connection has been closed
    pub async fn recv(&mut self) -> Option<T::Message> {
        loop {
            let msg = match self.sub.recv().await {
                Ok(m) => m,
                Err(MultiSubRxError::IoClosed) => return None,
                Err(MultiSubRxError::Lagged(n)) => {
                    tracing::warn!(stream_id = ?self.stream_id, lags = n, "Stream lagged");
                    continue;
                }
            };

            let TopicStreamMsg { stream_id, msg } = msg;
            if stream_id != self.stream_id {
                continue;
            }

            let Ok(msg) = postcard::from_bytes(&msg) else {
                continue;
            };
            return Some(msg);
        }
    }
}

/// Connect to a server configured in "insecure" mode
pub async fn connect_insecure(port: u16) -> PoststationClient {
    // Insecure can only be located on localhost
    let socket = TcpStream::connect(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .await
        .expect("expected to be able to connect to server");
    let addr = socket
        .peer_addr()
        .expect("expected to have peer address for server");
    socket
        .set_nodelay(true)
        .expect("expected to be able to set nodelay on socket");
    let (rx, tx) = split(socket);

    let client = HostClient::<WireError>::new_with_wire(
        TcpCommsTx { tx },
        TcpCommsRx {
            rx,
            addr,
            buf: vec![],
        },
        TcpSpawn,
        postcard_rpc::header::VarSeqKind::Seq4,
        ERROR_PATH,
        64,
    );

    let res = client
        .send_resp::<PingEndpoint>(&42)
        .await
        .expect("expected to be able to ping poststation server socket, ping failed");
    assert_eq!(res, 42, "sanity check failed: ping mismatch");

    PoststationClient { client }
}

/// Connect to a server configured with Self Signed TLS certificates (default)
pub async fn connect<T: tokio::net::ToSocketAddrs>(addr: T) -> PoststationClient {
    let dirs = ProjectDirs::from("com.onevariable", "onevariable", "poststation").unwrap();
    let data_dir = dirs.data_dir();
    let mut pem_path = PathBuf::from(data_dir);
    pem_path.push("ca-cert.pem");
    connect_with_ca_pem(addr, &pem_path).await
}

/// Connect to a server with the given TLS CA certificate
pub async fn connect_with_ca_pem<T: tokio::net::ToSocketAddrs>(addr: T, ca_path: &Path) -> PoststationClient {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store
        .add(CertificateDer::from_pem_file(ca_path).unwrap())
        .unwrap();
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(addr).await.unwrap();
    stream.set_nodelay(false).unwrap();
    let addr = stream.peer_addr().unwrap();
    let stream = connector
        .connect(ServerName::IpAddress(addr.ip().into()), stream)
        .await
        .unwrap();
    let (rx, tx) = split(stream);

    let client = HostClient::<WireError>::new_with_wire(
        TcpCommsTx { tx },
        TcpCommsRx {
            rx,
            addr,
            buf: vec![],
        },
        TcpSpawn,
        postcard_rpc::header::VarSeqKind::Seq4,
        ERROR_PATH,
        64,
    );

    let res = client
        .send_resp::<PingEndpoint>(&42)
        .await
        .expect("expected to be able to ping poststation server socket, ping failed");
    assert_eq!(res, 42, "sanity check failed: ping mismatch");

    PoststationClient { client }
}

pub enum TcpCommsRxError {
    RxOverflow,
    ConnError,
}

impl Debug for TcpCommsRxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("oops")
    }
}

impl Display for TcpCommsRxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("oops")
    }
}

impl Error for TcpCommsRxError {}

struct TcpCommsRx<T: AsyncRead + Send + 'static> {
    addr: SocketAddr,
    buf: Vec<u8>,
    rx: ReadHalf<T>,
}

impl<T: AsyncRead + Send + 'static> TcpCommsRx<T> {
    async fn receive_inner(&mut self) -> Result<Vec<u8>, TcpCommsRxError> {
        let mut rx_buf = [0u8; 1024];
        'frame: loop {
            if self.buf.len() > (1024 * 1024) {
                tracing::warn!(?self.addr, "Refusing to collect >1MiB, terminating");
                self.buf.clear();
                return Err(TcpCommsRxError::RxOverflow);
            }

            // Do we have a message already?
            if let Some(pos) = self.buf.iter().position(|b| *b == 0) {
                // we found the end of a message, attempt to decode it
                let mut split = self.buf.split_off(pos + 1);
                core::mem::swap(&mut self.buf, &mut split);

                // Can we decode the cobs?
                let res = cobs::decode_vec(&split);
                let Ok(msg) = res else {
                    tracing::warn!(?self.addr, discarded = split.len(), "Discarding bad message (cobs)");
                    continue 'frame;
                };

                return Ok(msg);
            }

            // No message yet, let's try and receive some data
            let Ok(used) = self.rx.read(&mut rx_buf).await else {
                tracing::warn!(?self.addr, "Closing");
                return Err(TcpCommsRxError::ConnError);
            };
            if used == 0 {
                tracing::warn!(?self.addr, "Closing");
                return Err(TcpCommsRxError::ConnError);
            }
            self.buf.extend_from_slice(&rx_buf[..used]);
        }
    }
}

impl<T: AsyncRead + Send + 'static> WireRx for TcpCommsRx<T> {
    type Error = TcpCommsRxError;

    fn receive(&mut self) -> impl Future<Output = Result<Vec<u8>, Self::Error>> + Send {
        self.receive_inner()
    }
}

// ---

pub enum TcpCommsTxError {
    CommsError,
}

impl Debug for TcpCommsTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("oops")
    }
}

impl Display for TcpCommsTxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("oops")
    }
}

impl Error for TcpCommsTxError {}

struct TcpCommsTx<T: AsyncWrite + Send + 'static> {
    tx: WriteHalf<T>,
}

impl<T: AsyncWrite + Send + 'static> TcpCommsTx<T> {
    async fn send_inner(&mut self, data: Vec<u8>) -> Result<(), TcpCommsTxError> {
        let mut data = cobs::encode_vec(&data);
        data.push(0);
        self.tx
            .write_all(&data)
            .await
            .map_err(|_| TcpCommsTxError::CommsError)
    }
}

impl<T: AsyncWrite + Send + 'static> WireTx for TcpCommsTx<T> {
    type Error = TcpCommsTxError;

    fn send(&mut self, data: Vec<u8>) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.send_inner(data)
    }
}

// ---

struct TcpSpawn;

impl WireSpawn for TcpSpawn {
    fn spawn(&mut self, fut: impl Future<Output = ()> + Send + 'static) {
        tokio::spawn(fut);
    }
}
