use anyhow::{Context, Result};
use log::{debug, info, warn};
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::capture::DhcpPacket;
use crate::config::Config;

pub struct PacketForwarder {
    config: Arc<Config>,
    socket: Socket,
}

impl PacketForwarder {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        info!(
            "Creating packet forwarder to {}:{}",
            config.remote_ip, config.remote_port
        );

        // Create a UDP socket
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
            .context("Failed to create UDP socket")?;

        // Set socket to non-blocking mode
        socket.set_nonblocking(false)?;

        Ok(PacketForwarder { config, socket })
    }

    /// Forward a DHCP packet to the remote server
    pub fn forward_packet(&self, packet: DhcpPacket) -> Result<()> {
        let remote_addr = SocketAddr::new(self.config.remote_ip, self.config.remote_port);

        debug!(
            "Forwarding {} byte packet to {}",
            packet.data.len(),
            remote_addr
        );

        // Extract the UDP payload (the actual DHCP data) from the captured packet
        let dhcp_payload = self.extract_dhcp_payload(&packet.data)?;

        // Send the DHCP payload to the remote server
        match self.socket.send_to(&dhcp_payload, &remote_addr.into()) {
            Ok(bytes_sent) => {
                info!(
                    "Successfully forwarded {} bytes to {} at {:?}",
                    bytes_sent, remote_addr, packet.timestamp
                );
                Ok(())
            }
            Err(e) => {
                warn!("Failed to forward packet to {}: {}", remote_addr, e);
                Err(e.into())
            }
        }
    }

    /// Extract the DHCP payload from the captured Ethernet frame
    fn extract_dhcp_payload(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Minimum Ethernet + IP + UDP headers = 14 + 20 + 8 = 42 bytes
        if data.len() < 42 {
            anyhow::bail!("Packet too short to contain DHCP data");
        }

        // Get IP header length (IHL field is in the lower 4 bits of byte 14)
        let ip_header_len = ((data[14] & 0x0F) * 4) as usize;

        // Calculate UDP header position
        let udp_start = 14 + ip_header_len;

        if data.len() < udp_start + 8 {
            anyhow::bail!("Packet too short to contain UDP header");
        }

        // UDP payload starts after the 8-byte UDP header
        let payload_start = udp_start + 8;

        if data.len() <= payload_start {
            anyhow::bail!("No UDP payload found");
        }

        // Extract the DHCP payload
        let payload = data[payload_start..].to_vec();
        debug!("Extracted {} bytes of DHCP payload", payload.len());

        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forwarder_creation() {
        let config = Arc::new(Config {
            remote_ip: "127.0.0.1".parse().unwrap(),
            remote_port: 67,
            interface: "eth0".to_string(),
        });

        let forwarder = PacketForwarder::new(config);
        assert!(forwarder.is_ok());
    }

    #[test]
    fn test_extract_dhcp_payload() {
        let config = Arc::new(Config {
            remote_ip: "127.0.0.1".parse().unwrap(),
            remote_port: 67,
            interface: "eth0".to_string(),
        });

        let forwarder = PacketForwarder::new(config).unwrap();

        // Create a minimal valid packet (Ethernet + IP + UDP + DHCP)
        // This is a simplified test packet
        let mut packet = vec![0u8; 300];

        // Ethernet header (14 bytes)
        packet[12] = 0x08; // EtherType: IPv4
        packet[13] = 0x00;

        // IP header (20 bytes, starting at offset 14)
        packet[14] = 0x45; // Version 4, IHL 5 (20 bytes)
        packet[23] = 17; // Protocol: UDP

        // UDP header (8 bytes, starting at offset 34)
        // Source port (68 - DHCP client)
        packet[34] = 0x00;
        packet[35] = 0x44;
        // Destination port (67 - DHCP server)
        packet[36] = 0x00;
        packet[37] = 0x43;

        // DHCP payload starts at offset 42
        let dhcp_data = b"DHCP_TEST_DATA";
        packet[42..42 + dhcp_data.len()].copy_from_slice(dhcp_data);

        let result = forwarder.extract_dhcp_payload(&packet);
        assert!(result.is_ok());

        let payload = result.unwrap();
        assert!(payload.starts_with(b"DHCP_TEST_DATA"));
    }
}
