use std::{net::SocketAddr, path::PathBuf, time::Instant};

use anyhow::bail;
use clap::{command, Parser, Subcommand};
use device::{device_cmds, Device};
use directories::ProjectDirs;
use postcard_rpc::host_client::{EndpointReport, TopicReport};
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
    command: Commands,

    /// Print timing information
    #[arg(long)]
    timings: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List devices
    Ls,

    /// Show paths for configuration, database storage, and
    /// the CA certificate for external usage
    Folder,

    /// Interact with a specific device
    Device(Device),
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

    let command = cli.command;
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
                Err(_) => bail!(
                    "No serial provided and no POSTSTATION_SERIAL env var found.\nHELP: Try `poststation-cli device SERIAL COMMAND`"
                ),
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

fn print_topic(tp: &TopicReport) {
    println!("* '{}' => Channel<{}>", tp.path, tp.ty.name);
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
