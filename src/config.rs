use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::IpAddr;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Remote IP address to forward DHCP packets to
    pub remote_ip: IpAddr,

    /// Remote port (default 67 for DHCP server)
    #[serde(default = "default_remote_port")]
    pub remote_port: u16,

    /// Network interface to listen on (e.g., "eth0")
    #[serde(default = "default_interface")]
    pub interface: String,
}

fn default_remote_port() -> u16 {
    67
}

fn default_interface() -> String {
    "eth0".to_string()
}

impl Config {
    /// Load configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;

        Ok(config)
    }

    /// Create a default configuration
    #[allow(dead_code)]
    pub fn default() -> Self {
        Config {
            remote_ip: "192.168.1.1".parse().unwrap(),
            remote_port: 67,
            interface: "eth0".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let json = r#"
        {
            "remote_ip": "10.0.0.1",
            "remote_port": 67,
            "interface": "eth1"
        }
        "#;

        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.remote_ip.to_string(), "10.0.0.1");
        assert_eq!(config.remote_port, 67);
        assert_eq!(config.interface, "eth1");
    }
}
