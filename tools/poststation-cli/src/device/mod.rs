use crate::{guess_serial, print_endpoint, print_topic};
use anyhow::bail;
use clap::{Args, Subcommand};
use postcard_rpc::host_client::{EndpointReport, SchemaReport, TopicReport};
use poststation_api_icd::postsock::Direction;
use poststation_sdk::{schema::schema::owned::OwnedDataModelType, PoststationClient};
use rand::{thread_rng, Rng};
use serde_json::json;
use uuid::Uuid;

#[derive(Args)]
pub struct Device {
    /// Device Serial Number or Name. Can be set via POSTSTATION_SERIAL env var
    pub serial: Option<String>,
    #[command(subcommand)]
    pub command: DeviceCommands,
}

#[derive(Subcommand)]
pub enum DeviceCommands {
    /// View all types used for communicating with a given device
    Types,
    /// View all endpoints available for communicating with a given device
    Endpoints,
    /// View all topics published by a given device
    TopicsOut,
    /// View all topics handled by a given device
    TopicsIn,
    /// View the most recent logs from a given device
    Logs { count: Option<u32> },
    /// View the selected range of logs from a given device
    LogsRange {
        /// Number of logs to print
        count: Option<u32>,
        /// The UUID of the log to start from
        start: String,
        /// Direction to print from ('before' or 'after')
        direction: String,
    },
    /// Proxy message to device endpoint
    Proxy {
        path: String,
        message: Option<String>,
    },
    /// Publish a topic message to a device
    Publish {
        path: String,
        message: Option<String>,
    },
    /// Listen to a given "topic-out" path from a device
    Listen {
        #[arg(value_name = "PATH")]
        path: String,
    },
}

pub async fn device_cmds(client: PoststationClient, device: &Device) -> anyhow::Result<()> {
    let serial = guess_serial(device.serial.as_deref(), &client).await?;
    let schema = client
        .get_device_schemas(serial)
        .await
        .expect("expected to get schemas for device")
        .expect("expected device to have known schemas");
    match &device.command {
        DeviceCommands::Types => device_types(serial, &schema).await,
        DeviceCommands::Endpoints => device_endpoints(serial, &schema).await,
        DeviceCommands::TopicsOut => device_topics_out(serial, &schema).await,
        DeviceCommands::TopicsIn => device_topics_in(serial, &schema).await,
        DeviceCommands::Logs { count } => device_logs(client, serial, count).await,
        DeviceCommands::LogsRange {
            count,
            start,
            direction,
        } => device_logs_range(client, serial, count, start, direction).await,
        DeviceCommands::Proxy { path, message } => {
            device_smart_proxy(client, serial, &schema, path, message.as_deref()).await
        }
        DeviceCommands::Publish { path, message } => {
            device_smart_publish(client, &schema, serial, path, message.as_deref()).await
        }
        DeviceCommands::Listen { path } => device_smart_listen(client, &schema, serial, path).await,
    }
}

async fn device_smart_listen(
    client: PoststationClient,
    schema: &SchemaReport,
    serial: u64,
    path: &str,
) -> anyhow::Result<()> {
    let path = &fuzzy_topic_out_match(path, schema)?.path;

    let mut sub = match client.stream_topic_json(serial, path).await {
        Ok(s) => s,
        Err(e) => bail!("{e}"),
    };

    while let Some(m) = sub.recv().await {
        println!("{serial:016X}:'{path}':{m}");
    }
    println!("Closed");
    Ok(())
}

async fn device_topics_in(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Topics handled by device {:016X}", serial);
    println!();

    for tp in &schema.topics_in {
        println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
    }
    println!();
    Ok(())
}

async fn device_topics_out(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Topics offered by device {:016X}", serial);
    println!();

    for tp in &schema.topics_out {
        println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
    }
    println!();
    Ok(())
}

async fn device_endpoints(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Endpoints offered by device {:016X}", serial);
    println!();

    for ep in &schema.endpoints {
        print_endpoint(ep);
    }
    println!();
    Ok(())
}

async fn device_types(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Types used by device {:016X}", serial);
    println!();

    let base = SchemaReport::default();
    let uniq_tys = schema.types.difference(&base.types);

    for ty in uniq_tys {
        println!("* {ty}");
    }
    println!();
    Ok(())
}

async fn device_logs(
    client: PoststationClient,
    serial: u64,
    count: &Option<u32>,
) -> anyhow::Result<()> {
    let count = count.unwrap_or(8);
    let logs = client
        .get_device_logs(serial, count)
        .await
        .expect("expected to be able to get logs for device")
        .expect("expected device to have known logs");

    println!();
    println!("Logs (last {} messages):", count.min(logs.len() as u32));
    println!();
    for log in logs {
        // println!("* {} => {}", log.uuidv7.id_to_time().time(), log.msg);
        let time = log.uuidv7.id_to_time().time();
        println!(
            "* {} ({}) => {}",
            uuid::Uuid::from(log.uuidv7),
            time,
            log.msg
        );
    }
    println!();
    Ok(())
}

async fn device_logs_range(
    client: PoststationClient,
    serial: u64,
    count: &Option<u32>,
    start: &str,
    direction: &str,
) -> anyhow::Result<()> {
    let count = count.unwrap_or(8);
    let start = start.parse::<Uuid>()?;
    let dir = match direction.to_lowercase().as_str() {
        "after" => Direction::After,
        "before" => Direction::Before,
        _ => bail!("Should provide 'after' or 'before' for direction"),
    };

    let logs = client
        .get_device_logs_range(
            serial,
            count,
            dir,
            poststation_api_icd::postsock::Anchor::Uuid(start.into()),
        )
        .await
        .expect("expected to be able to get log range for device")
        .expect("expected device to have known logs");

    println!();
    println!("Logs (last {} messages):", count.min(logs.len() as u32));
    println!();
    for log in logs {
        // println!("* {} => {}", log.uuidv7.id_to_time().time(), log.msg);
        let time = log.uuidv7.id_to_time().time();
        println!(
            "* {} ({}) => {}",
            uuid::Uuid::from(log.uuidv7),
            time,
            log.msg
        );
    }
    println!();
    Ok(())
}

fn fuzzy_endpoint_match<'a>(
    path: &str,
    schema: &'a SchemaReport,
) -> anyhow::Result<&'a EndpointReport> {
    let matches = schema
        .endpoints
        .iter()
        .filter(|e| e.path.contains(path))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => {
            bail!("No endpoint found matching '{path}'");
        }
        [ep] => Ok(ep),
        more @ [..] => {
            println!("Given '{path}', found:");
            println!();
            for matched_endpoint in more {
                print_endpoint(matched_endpoint);
            }
            println!();
            bail!("Too many matches, be more specific!");
        }
    }
}

fn fuzzy_topic_out_match<'a>(
    path: &str,
    schema: &'a SchemaReport,
) -> anyhow::Result<&'a TopicReport> {
    let matches = schema
        .topics_out
        .iter()
        .filter(|to| to.path.contains(path))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => {
            bail!("No topic-out found matching '{path}'");
        }
        [tp] => Ok(tp),
        more @ [..] => {
            println!("Given '{path}', found:");
            println!();
            for matched_topic in more {
                print_topic(matched_topic);
            }
            println!();
            bail!("Too many matches, be more specific!");
        }
    }
}

fn fuzzy_topic_in_match<'a>(
    path: &str,
    schema: &'a SchemaReport,
) -> anyhow::Result<&'a TopicReport> {
    let matches = schema
        .topics_in
        .iter()
        .filter(|to| to.path.contains(path))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => {
            bail!("No topic-in found matching '{path}'");
        }
        [tp] => Ok(tp),
        more @ [..] => {
            println!("Given '{path}', found:");
            println!();
            for matched_topic in more {
                print_topic(matched_topic);
            }
            println!();
            bail!("Too many matches, be more specific!");
        }
    }
}

async fn device_smart_proxy(
    client: PoststationClient,
    serial: u64,
    schema: &SchemaReport,
    command: &str,
    message: Option<&str>,
) -> anyhow::Result<()> {
    let ep = fuzzy_endpoint_match(command, schema)?;

    match (&ep.req_ty.ty, message) {
        (OwnedDataModelType::Unit, None) => {
            device_proxy(client, serial, ep.path.clone(), "".to_string()).await?;
        }
        (_, None) => {
            bail!(
                "Endpoint '{}' requires a message to be sent of the type: async fn({}) -> {}",
                ep.path,
                ep.req_ty.name,
                ep.resp_ty.name
            );
        }
        (_, Some(message)) => {
            device_proxy(client, serial, ep.path.clone(), message.to_string()).await?;
        }
    }

    Ok(())
}

async fn device_smart_publish(
    client: PoststationClient,
    schema: &SchemaReport,
    serial: u64,
    path: &str,
    message: Option<&str>,
) -> anyhow::Result<()> {
    let topic_in = fuzzy_topic_in_match(path, schema)?;

    let msg = match (&topic_in.ty.ty, message) {
        (OwnedDataModelType::Unit, None) => serde_json::Value::Null,
        (_, None) => {
            bail!(
                "Topic '{}' requires a message to be sent of the type: {}",
                topic_in.path,
                topic_in.ty.name,
            );
        }
        (_, Some(message)) => message.parse()?,
    };

    let seq = thread_rng().gen();
    let res = client
        .publish_topic_json(serial, &topic_in.path, seq, msg)
        .await;

    match res {
        Ok(()) => println!("Published."),
        Err(e) => println!("Error: '{e}'"),
    }

    Ok(())
}

async fn device_proxy(
    client: PoststationClient,
    serial: u64,
    path: String,
    message: String,
) -> anyhow::Result<()> {
    let msg = match message.parse() {
        Ok(m) => m,
        Err(_) => {
            //Attempting to just parse value as a string if it fails
            json!(message)
        }
    };
    let seq = thread_rng().gen();
    let res = client.proxy_endpoint_json(serial, &path, seq, msg).await;

    match res {
        Ok(v) => println!("Response: '{v}'"),
        Err(e) => println!("Error: '{e}'"),
    }

    Ok(())
}
