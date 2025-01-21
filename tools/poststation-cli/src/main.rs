use std::{net::SocketAddr, path::PathBuf, time::Instant};

use anyhow::bail;
use clap::{command, Args, Parser, Subcommand};
use device::{device_cmds, Device};
use directories::ProjectDirs;
use postcard_rpc::host_client::EndpointReport;
use poststation_sdk::{
    connect, connect_insecure, schema::schema::owned::OwnedDataModelType, PoststationClient,
};

mod device;

/// The Poststation CLI
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// A path to the server. Defaults to `127.0.0.1:51837`.
    #[arg(short, long, value_name = "SERVER_ADDR")]
    server: Option<SocketAddr>,

    /// When set, a plaintext connection will be made with the server
    #[arg(long)]
    insecure: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    timings: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List devices
    Ls,

    // /// Endpoints of a given device
    // Endpoints { serial: Option<String> },
    /// Show the folder used for configuration, database storage, and
    /// the CA certificate for external usage
    Folder,

    /// Interact with a specific device
    Device(Device),
    // /// Proxy an endpoint request/response through the server
    // Proxy {
    //     #[arg(short, long, value_name = "SERIAL")]
    //     serial: Option<String>,
    //     #[arg(short, long, value_name = "PATH")]
    //     path: String,
    //     #[arg(short, long, value_name = "MSG_JSON")]
    //     message: String,
    // },
    // Publish {
    //     #[arg(short, long, value_name = "SERIAL")]
    //     serial: String,
    //     #[arg(short, long, value_name = "PATH")]
    //     path: String,
    //     #[arg(short, long, value_name = "MSG_JSON")]
    //     message: String,
    // },
    // /// Listen to a given "topic-out" path from a device
    // Listen {
    //     #[arg(short, long, value_name = "SERIAL")]
    //     serial: String,
    //     #[arg(short, long, value_name = "PATH")]
    //     path: String,
    // },
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
    let client = if cli.insecure {
        connect_insecure(server.port()).await
    } else {
        connect(server).await
    }
    .unwrap();

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
        // Commands::Proxy {
        //     serial,
        //     message,
        //     path,
        // } => {
        //     let serial = guess_serial(serial.as_deref(), &client).await?;
        //     device_proxy(client, serial, path, message).await
        // }
        // Commands::Publish {
        //     serial,
        //     message,
        //     path,
        // } => device_publish(client, serial, path, message).await,
        // Commands::Endpoints { serial } => {
        //     let serial_num = guess_serial(serial.as_deref(), &client).await?;

        //     println!("{serial_num:016X}");
        //     let schema = client
        //         .get_device_schemas(serial_num)
        //         .await
        //         .expect("expected to be able to get schemas for device")
        //         .expect("expected device to have schemas known by the server");

        //     println!();
        //     println!("# Endpoints for {serial_num:016X}");
        //     println!();
        //     println!("## By path");
        //     println!();

        //     let longest_ep = schema.endpoints.iter().map(|e| e.path.len()).max().unwrap();
        //     let longest_req = schema
        //         .endpoints
        //         .iter()
        //         .map(|e| e.req_ty.name.len())
        //         .max()
        //         .unwrap_or(0)
        //         .max("Request Type".len());
        //     let longest_resp = schema
        //         .endpoints
        //         .iter()
        //         .map(|e| e.resp_ty.name.len())
        //         .max()
        //         .unwrap_or(0)
        //         .max("Response Type".len());

        //     println!(
        //         "| {:longest_ep$} | {:longest_req$} | {:longest_resp$} |",
        //         "path", "Request Type", "Response Type"
        //     );
        //     println!(
        //         "| {:-<longest_ep$} | {:-<longest_req$} | {:-<longest_resp$} |",
        //         "", "", ""
        //     );

        //     let mut used_tys = HashSet::new();

        //     for ep in schema.endpoints {
        //         println!(
        //             "| {:longest_ep$} | {:longest_req$} | {:longest_resp$} |",
        //             ep.path, ep.req_ty.name, ep.resp_ty.name
        //         );
        //         discover_tys(&ep.req_ty, &mut used_tys);
        //         discover_tys(&ep.resp_ty, &mut used_tys);
        //     }
        //     println!();
        //     println!("## Type Definitions");
        //     println!();
        //     println!("Non-primitive types used by endpoints");

        //     let mut tys: Vec<OwnedNamedType> = used_tys
        //         .into_iter()
        //         .filter(|ont| !is_prim(&ont.ty))
        //         .collect();
        //     tys.sort_by_key(|o| o.name.clone());

        //     for ty in tys {
        //         println!();
        //         println!("### `{}`", ty.name);
        //         println!();
        //         println!("{}", ty.to_pseudocode());
        //     }
        //     println!();

        //     Ok(())
        // }
        // Commands::Listen { serial, path } => {
        //     let serial_num = guess_serial(Some(&serial), &client).await?;
        //     let mut sub = match client.stream_topic_json(serial_num, &path).await {
        //         Ok(s) => s,
        //         Err(e) => bail!("{e}"),
        //     };

        //     while let Some(m) = sub.recv().await {
        //         println!("{serial_num:016X}:'{path}':{m}");
        //     }
        //     println!("Closed");
        //     Ok(())
        // }
        Commands::Folder => {
            let Some(dirs) = ProjectDirs::from("com.onevariable", "onevariable", "poststation")
            else {
                bail!("Failed to get working directory!");
            };
            let data_dir = dirs.data_dir();
            let mut cert_path = PathBuf::from(data_dir);
            cert_path.push("ca-cert.pem");
            let mut cfg_path = PathBuf::from(data_dir);
            cfg_path.push("poststation-config.toml");

            println!();
            println!("Poststation Folder Information:");
            println!("===============================");
            println!("Folder:         {data_dir:?}");
            println!("CA Certificate: {cert_path:?}");
            println!("Configuration:  {cfg_path:?}");
            println!();
            Ok(())
        }
    }
}

async fn guess_serial(serial: Option<&str>, client: &PoststationClient) -> anyhow::Result<u64> {
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
