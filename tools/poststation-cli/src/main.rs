use std::{collections::HashSet, net::SocketAddr, time::Instant};

use anyhow::bail;
use clap::{command, Args, Parser, Subcommand};
use postcard_rpc::host_client::{EndpointReport, SchemaReport};
use poststation_api_icd::postsock::Direction;
use poststation_sdk::{
    connect,
    schema::schema::{
        fmt::{discover_tys, is_prim},
        owned::{OwnedDataModelType, OwnedNamedType},
    },
    SquadClient,
};
use serde_json::json;
use uuid::Uuid;

/// The Poststation CLI
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// A path to the server. Defaults to `127.0.0.1:51837`.
    #[arg(short, long, value_name = "SERVER_ADDR")]
    server: Option<SocketAddr>,

    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    timings: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List devices
    Ls,

    /// Endpoints of a given device
    Endpoints { serial: Option<String> },

    /// Get information about a device
    Device(Device),
    /// Proxy an endpoint request/response through the server
    Proxy {
        #[arg(short, long, value_name = "SERIAL")]
        serial: Option<String>,
        #[arg(short, long, value_name = "PATH")]
        path: String,
        #[arg(short, long, value_name = "MSG_JSON")]
        message: String,
    },
    Publish {
        #[arg(short, long, value_name = "SERIAL")]
        serial: String,
        #[arg(short, long, value_name = "PATH")]
        path: String,
        #[arg(short, long, value_name = "MSG_JSON")]
        message: String,
    },
    /// Listen to a given "topic-out" path from a device
    Listen {
        #[arg(short, long, value_name = "SERIAL")]
        serial: String,
        #[arg(short, long, value_name = "PATH")]
        path: String,
    },
}

#[derive(Args)]
struct Device {
    serial: Option<String>,
    #[command(subcommand)]
    command: DeviceCommands,
}

#[derive(Subcommand)]
enum DeviceCommands {
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
    /// View the most recent logs from a given device
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();
    let timings = cli.timings;
    inner_main(cli).await?;
    if timings {
        println!("{:?}", start.elapsed());
    }
    Ok(())
}

async fn inner_main(cli: Cli) -> anyhow::Result<()> {
    let server = cli
        .server
        .unwrap_or_else(|| "127.0.0.1:51837".parse().unwrap());

    let Some(command) = cli.command else {
        return Ok(());
    };
    let client = connect(server).await;

    match command {
        Commands::Ls => {
            let devices = client
                .get_devices()
                .await
                .expect("expected to be able to get devices from server");
            println!();
            println!("# Devices");
            println!();
            println!("| serial           | name       | interface | connected |");
            println!("| :--------------- | ---------: | :-------- | :-------- |");
            for dev in devices.iter() {
                let ser = format!("{:016X}", dev.serial);
                let conn = if dev.is_connected { "yes" } else { "no " };
                println!("| {ser} | {:>10} | {:<9} | {conn:<9} |", dev.name, "usb");
            }
            println!();
            Ok(())
        }
        Commands::Device(d) => device_cmds(client, &d).await,
        Commands::Proxy {
            serial,
            message,
            path,
        } => {
            let serial = guess_serial(serial.as_deref(), &client).await?;
            device_proxy(client, serial, path, message).await
        }
        Commands::Publish {
            serial,
            message,
            path,
        } => device_publish(client, serial, path, message).await,
        Commands::Endpoints { serial } => {
            let serial_num = guess_serial(serial.as_deref(), &client).await?;

            println!("{serial_num:016X}");
            let schema = client
                .get_device_schemas(serial_num)
                .await
                .expect("expected to be able to get schemas for device")
                .expect("expected device to have schemas known by the server");

            println!();
            println!("# Endpoints for {serial_num:016X}");
            println!();
            println!("## By path");
            println!();

            let longest_ep = schema.endpoints.iter().map(|e| e.path.len()).max().unwrap();
            let longest_req = schema
                .endpoints
                .iter()
                .map(|e| e.req_ty.name.len())
                .max()
                .unwrap_or(0)
                .max("Request Type".len());
            let longest_resp = schema
                .endpoints
                .iter()
                .map(|e| e.resp_ty.name.len())
                .max()
                .unwrap_or(0)
                .max("Response Type".len());

            println!(
                "| {:longest_ep$} | {:longest_req$} | {:longest_resp$} |",
                "path", "Request Type", "Response Type"
            );
            println!(
                "| {:-<longest_ep$} | {:-<longest_req$} | {:-<longest_resp$} |",
                "", "", ""
            );

            let mut used_tys = HashSet::new();

            for ep in schema.endpoints {
                println!(
                    "| {:longest_ep$} | {:longest_req$} | {:longest_resp$} |",
                    ep.path, ep.req_ty.name, ep.resp_ty.name
                );
                discover_tys(&ep.req_ty, &mut used_tys);
                discover_tys(&ep.resp_ty, &mut used_tys);
            }
            println!();
            println!("## Type Definitions");
            println!();
            println!("Non-primitive types used by endpoints");

            let mut tys: Vec<OwnedNamedType> = used_tys
                .into_iter()
                .filter(|ont| !is_prim(&ont.ty))
                .collect();
            tys.sort_by_key(|o| o.name.clone());

            for ty in tys {
                println!();
                println!("### `{}`", ty.name);
                println!();
                println!("{}", ty.to_pseudocode());
            }
            println!();

            Ok(())
        }
        Commands::Listen { serial, path } => {
            let serial_num = guess_serial(Some(&serial), &client).await?;
            let mut sub = match client.stream_topic_json(serial_num, &path).await {
                Ok(s) => s,
                Err(e) => bail!("{e}"),
            };

            while let Some(m) = sub.recv().await {
                println!("{serial_num:016X}:'{path}':{m}");
            }
            println!("Closed");
            Ok(())
        }
    }
}

async fn device_proxy(
    client: SquadClient,
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
    client: SquadClient,
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

async fn device_cmds(client: SquadClient, device: &Device) -> anyhow::Result<()> {
    let serial = guess_serial(device.serial.as_deref(), &client).await?;
    let schema = client
        .get_device_schemas(serial)
        .await
        .expect("expected to get schemas for device")
        .expect("expected device to have known schemas");
    match &device.command {
        DeviceCommands::Types => {
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
        DeviceCommands::Endpoints => {
            println!();
            println!("Endpoints offered by device {}", serial);
            println!();

            for ep in schema.endpoints {
                print_endpoint(&ep);
            }
            println!();
            Ok(())
        }
        DeviceCommands::TopicsOut => {
            println!();
            println!("Topics offered by device {}", serial);
            println!();

            for tp in schema.topics_out {
                println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
            }
            println!();
            Ok(())
        }
        DeviceCommands::TopicsIn => {
            println!();
            println!("Topics handled by device {}", serial);
            println!();

            for tp in schema.topics_in {
                println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
            }
            println!();
            Ok(())
        }
        DeviceCommands::Logs { count } => {
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
        DeviceCommands::LogsRange {
            count,
            start,
            direction,
        } => {
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
        DeviceCommands::SmartProxy { command, message } => {
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
                    device_proxy(client, serial, ep.path.clone(), message.to_owned()).await?;
                } else {
                    bail!("Endpoint '{}' requires a message to be sent of the type: async fn({}) -> {}", ep.path, ep.req_ty.name, ep.resp_ty.name);
                }
            }
            Ok(())
        }
    }
}

async fn guess_serial(serial: Option<&str>, client: &SquadClient) -> anyhow::Result<u64> {
    let serial = match serial {
        Some(serial) => serial.to_uppercase(),
        None => {
            let serial_from_env: Result<String, std::env::VarError> =
                std::env::var("POSTSTATION_SERIAL");
            match serial_from_env {
                Ok(serial) => serial.to_uppercase(),
                Err(_) => bail!("No serial provided and no POSTSTATION_SERIAL env var found"),
            }
        }
    };

    let mut serial_num = None;
    let mut serial_fragment = false;

    if let Ok(ser) = u64::from_str_radix(&serial, 16) {
        if serial.len() == 16 {
            serial_num = Some(ser);
        } else {
            serial_fragment = true;
        }
    }

    if serial_num.is_none() {
        let devices = client
            .get_devices()
            .await
            .expect("expected to be able to get devices");
        let uppy = serial.to_uppercase();
        let matches = devices
            .iter()
            .filter(|d| {
                d.name.contains(&uppy)
                    || (serial_fragment && {
                        let this_ser = format!("{:016X}", d.serial);
                        this_ser.contains(&serial)
                    })
            })
            .collect::<Vec<_>>();

        if matches.is_empty() {
            bail!("Failed to find device matching '{serial}'");
        } else if matches.len() > 1 {
            println!("Given '{serial}', found:");
            println!();
            for m in matches {
                println!("* name: '{}' serial: {:016X}", m.name, m.serial);
            }
            println!();
            bail!("Too many matches, be more specific!");
        } else {
            serial_num = Some(matches[0].serial);
        }
    };

    let Some(serial_num) = serial_num else {
        bail!("Couldn't figure a serial number out!");
    };
    Ok(serial_num)
}

fn print_endpoint(ep: &EndpointReport) {
    if ep.resp_ty.ty == OwnedDataModelType::Unit {
        println!("* '{}' => async fn({})", ep.path, ep.req_ty.name);
    } else {
        println!(
            "* '{}' => async fn({}) -> {}",
            ep.path, ep.req_ty.name, ep.resp_ty.name
        );
    }
}
