mod capture;
mod config;
mod forwarder;

use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;

use capture::PacketCapture;
use config::Config;
use forwarder::PacketForwarder;

#[derive(Parser, Debug)]
#[command(name = "linux-dhcp-forwarder")]
#[command(author, version, about = "A DHCP packet forwarder that captures DHCP requests and sends copies to remote servers for informational purposes", long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long, default_value = "/etc/dhcp-forwarder/config.json")]
    config: PathBuf,

    /// Network interface to listen on (overrides config file)
    #[arg(short, long)]
    interface: Option<String>,

    /// Remote IP address to forward to (overrides config file)
    #[arg(short, long)]
    remote_ip: Option<String>,

    /// Remote port to forward to (overrides config file)
    #[arg(short = 'p', long)]
    remote_port: Option<u16>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    info!("Starting DHCP forwarder service...");

    // Load configuration
    let mut config = Config::from_file(&args.config)
        .with_context(|| format!("Failed to load config from {}", args.config.display()))?;

    // Override config with command-line arguments if provided
    if let Some(interface) = args.interface {
        config.interface = interface;
    }
    if let Some(remote_ip) = args.remote_ip {
        config.remote_ip = remote_ip.parse().context("Invalid remote IP address")?;
    }
    if let Some(remote_port) = args.remote_port {
        config.remote_port = remote_port;
    }

    info!("Configuration loaded:");
    info!("  Interface: {}", config.interface);
    info!("  Remote IP: {}", config.remote_ip);
    info!("  Remote Port: {}", config.remote_port);

    // Create packet forwarder
    let config_arc = Arc::new(config.clone());
    let forwarder =
        PacketForwarder::new(config_arc).context("Failed to create packet forwarder")?;

    // Create packet capture
    let capture = PacketCapture::new(config.interface.clone());

    info!("Starting packet capture and forwarding...");
    info!("Press Ctrl+C to stop");

    // Start capturing and forwarding packets
    capture.start_capture(|packet| forwarder.forward_packet(packet))?;

    Ok(())
}
