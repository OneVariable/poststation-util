use crate::{guess_serial, print_endpoint};
use anyhow::bail;
use clap::{Args, Subcommand};
use postcard_rpc::host_client::SchemaReport;
use poststation_api_icd::postsock::Direction;
use poststation_sdk::{schema::schema::owned::OwnedDataModelType, PoststationClient};
use serde_json::json;
use uuid::Uuid;

#[derive(Args)]
pub struct Device {
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
        count: Option<u32>,
        start: String,
        direction: String,
    },
    /// Takes a guess at which endpoint you want to proxy and sends a message to it if you provide one
    SmartProxy {
        command: String,
        message: Option<String>,
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
        DeviceCommands::SmartProxy { command, message } => {
            device_smart_proxy(client, serial, &schema, command, &message.as_deref()).await
        }
    }
}

async fn device_topics_in(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Topics handled by device {}", serial);
    println!();

    for tp in &schema.topics_in {
        println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
    }
    println!();
    Ok(())
}

async fn device_topics_out(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Topics offered by device {}", serial);
    println!();

    for tp in &schema.topics_out {
        println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
    }
    println!();
    Ok(())
}

async fn device_endpoints(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Endpoints offered by device {}", serial);
    println!();

    for ep in &schema.endpoints {
        print_endpoint(ep);
    }
    println!();
    Ok(())
}

async fn device_types(serial: u64, schema: &SchemaReport) -> anyhow::Result<()> {
    println!();
    println!("Types used by device {}", serial);
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

async fn device_smart_proxy(
    client: PoststationClient,
    serial: u64,
    schema: &SchemaReport,
    command: &str,
    message: &Option<&str>,
) -> anyhow::Result<()> {
    let matches = schema
        .endpoints
        .iter()
        .filter(|e| e.path.contains(command))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        bail!("No endpoint found matching '{command}'");
    } else if matches.len() > 1 {
        println!("Given '{command}', found:");
        println!();
        for matched_endpoint in matches {
            print_endpoint(matched_endpoint);
        }
        println!();
        bail!("Too many matches, be more specific!");
    } else {
        let ep = matches[0];
        if ep.req_ty.ty == OwnedDataModelType::Unit {
            device_proxy(client, serial, ep.path.clone(), "".to_string()).await?;
            return Ok(());
        }
        if let Some(message) = message {
            device_proxy(client, serial, ep.path.clone(), message.to_string()).await?;
        } else {
            bail!(
                "Endpoint '{}' requires a message to be sent of the type: async fn({}) -> {}",
                ep.path,
                ep.req_ty.name,
                ep.resp_ty.name
            );
        }
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

    let res = client.proxy_endpoint_json(serial, &path, 0, msg).await;

    match res {
        Ok(v) => println!("Response: '{v}'"),
        Err(e) => println!("Error: '{e}'"),
    }

    Ok(())
}

async fn device_publish(
    client: PoststationClient,
    serial: String,
    path: String,
    message: String,
) -> anyhow::Result<()> {
    let serial = u64::from_str_radix(&serial, 16)?;
    let msg = message.parse()?;

    let res = client.publish_topic_json(serial, &path, 0, msg).await;

    match res {
        Ok(()) => println!("Published."),
        Err(e) => println!("Error: '{e}'"),
    }

    Ok(())
}
